//! Index completeness: `docs/adr/README.md` (XTASK-R025) and `docs/specs/README.md` (R027).

use std::collections::BTreeMap;

use super::resolve;
use super::rules::{Adr, Spec};
use crate::finding::Finding;
use crate::markdown;
use crate::repo::RepoModel;

/// How to find and compare one index.
struct Index {
    rule: &'static str,
    /// The index file.
    path: &'static str,
    /// First header cell of an index table.
    first_header: &'static str,
    /// Header of the compared column.
    column: &'static str,
    /// What the indexed files are called in messages.
    kind: &'static str,
    /// Whether the compared cell may be wrapped in backticks.
    backticks_allowed: bool,
}

/// One indexed file and the cell value its frontmatter implies (`None`: not comparable).
struct Entry<'a> {
    path: &'a str,
    name: &'a str,
    expected: Option<&'a str>,
}

/// XTASK-R025: the ADR index lists every ADR once, with its frontmatter status.
pub(super) fn adr_index(model: &RepoModel, adrs: &[Adr], findings: &mut Vec<Finding>) {
    let index = Index {
        rule: "XTASK-R025",
        path: "docs/adr/README.md",
        first_header: "ADR",
        column: "Status",
        kind: "ADR",
        backticks_allowed: false,
    };
    let entries: Vec<Entry> = adrs
        .iter()
        .map(|adr| Entry {
            path: &adr.path,
            name: &adr.id,
            expected: adr.index_status.as_deref(),
        })
        .collect();
    check(model, &index, &entries, findings);
}

/// XTASK-R027: the spec index lists every spec once, with its frontmatter maturity.
pub(super) fn spec_index(model: &RepoModel, specs: &[Spec], findings: &mut Vec<Finding>) {
    let index = Index {
        rule: "XTASK-R027",
        path: "docs/specs/README.md",
        first_header: "Spec",
        column: "Maturity",
        kind: "spec",
        backticks_allowed: true,
    };
    let entries: Vec<Entry> = specs
        .iter()
        .map(|spec| Entry {
            path: &spec.path,
            name: crate::repo::file_name(&spec.path),
            expected: spec.maturity.as_deref(),
        })
        .collect();
    check(model, &index, &entries, findings);
}

fn check(model: &RepoModel, index: &Index, entries: &[Entry], findings: &mut Vec<Finding>) {
    let Some(tables) = index_tables(model, index, findings) else {
        return;
    };
    let listed = check_rows(index, &tables, entries, findings);
    check_listed(index, entries, &listed, findings);
}

/// The index tables of `index`, or `None` after reporting that there are none.
fn index_tables(
    model: &RepoModel,
    index: &Index,
    findings: &mut Vec<Finding>,
) -> Option<Vec<markdown::Table>> {
    let Some(text) = model.markdown.get(index.path) else {
        findings.push(Finding::error(
            index.rule,
            index.path,
            format!("the {} index {} does not exist", index.kind, index.path),
            "Create it with a table whose header starts with the index's first column",
        ));
        return None;
    };
    let tables: Vec<markdown::Table> = markdown::tables(text)
        .into_iter()
        .filter(|table| {
            table.header.first().map(String::as_str) == Some(index.first_header)
                && table.header.iter().any(|cell| cell == index.column)
        })
        .collect();
    if tables.is_empty() {
        findings.push(Finding::error(
            index.rule,
            index.path,
            format!(
                "no index table: expected a table with first header cell `{}` and a `{}` column",
                index.first_header, index.column
            ),
            "Restore the index table",
        ));
        return None;
    }
    Some(tables)
}

/// Checks every row: it links to an indexed file and shows the expected cell. Returns the rows'
/// line numbers per indexed file.
fn check_rows<'a>(
    index: &Index,
    tables: &[markdown::Table],
    entries: &'a [Entry],
    findings: &mut Vec<Finding>,
) -> BTreeMap<&'a str, Vec<usize>> {
    let mut listed: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for table in tables {
        let column = table
            .header
            .iter()
            .position(|cell| cell == index.column)
            .expect("invariant: tables were filtered to those with the column");
        for row in &table.rows {
            let location = format!("{}:{}", index.path, row.line);
            let entry = row
                .cells
                .first()
                .and_then(|cell| markdown::first_link_target(cell))
                .and_then(|target| resolve(index.path, &target))
                .and_then(|target| entries.iter().find(|entry| entry.path == target));
            let Some(entry) = entry else {
                findings.push(Finding::error(
                    index.rule,
                    location,
                    format!("the first cell does not link to a {} file", index.kind),
                    "Link the first cell to the indexed file, relative to the index",
                ));
                continue;
            };
            listed.entry(entry.path).or_default().push(row.line);
            let cell = row.cells.get(column).map_or("", String::as_str);
            let shown = if index.backticks_allowed {
                cell.strip_prefix('`')
                    .and_then(|inner| inner.strip_suffix('`'))
                    .unwrap_or(cell)
            } else {
                cell
            };
            if let Some(expected) = entry.expected
                && shown != expected
            {
                findings.push(Finding::error(
                    index.rule,
                    location,
                    format!(
                        "{} is listed with {} `{cell}`, but its frontmatter says `{expected}`",
                        entry.name,
                        index.column.to_lowercase()
                    ),
                    "Make the index agree with the frontmatter",
                ));
            }
        }
    }
    listed
}

/// Checks that every indexed file is listed exactly once.
fn check_listed(
    index: &Index,
    entries: &[Entry],
    listed: &BTreeMap<&str, Vec<usize>>,
    findings: &mut Vec<Finding>,
) {
    for entry in entries {
        match listed.get(entry.path).map_or(&[][..], Vec::as_slice) {
            [] => findings.push(Finding::error(
                index.rule,
                index.path,
                format!("{} ({}) is not listed", entry.name, entry.path),
                "Add a row for it to the index table",
            )),
            [_] => {}
            lines => {
                let lines: Vec<String> = lines.iter().map(ToString::to_string).collect();
                findings.push(Finding::error(
                    index.rule,
                    format!("{}:{}", index.path, lines[1]),
                    format!(
                        "{} is listed {} times (lines {})",
                        entry.name,
                        lines.len(),
                        lines.join(", ")
                    ),
                    "Keep exactly one row per file",
                ));
            }
        }
    }
}
