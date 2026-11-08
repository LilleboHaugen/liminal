//! Requirements extracted from spec section 6 tables (`docs/specs/xtask.md` XTASK-R040, R045 and
//! the "`trace` and `hygiene` details"). Used by `trace` and by `check-docs` (XTASK-R028).
//!
//! Extraction is a pure function of the [`RepoModel`]. It does not depend on a spec's frontmatter
//! being valid: the crate is the spec's file name (XTASK-R022 requires `crate` to equal it), and
//! `req_prefix` and `maturity` are used only when the frontmatter can be read. `check-docs`
//! reports broken frontmatter, so a broken spec does not also turn every annotation of its
//! requirements into an unknown-ID finding.

use std::collections::BTreeMap;

use crate::finding::Finding;
use crate::frontmatter::{self, Value};
use crate::markdown;
use crate::repo::{RepoModel, file_name};

/// Requirement level (RFC 2119 keyword in the table's second cell).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Level {
    /// `MUST`.
    Must,
    /// `SHOULD`.
    Should,
    /// `MAY`.
    May,
}

impl Level {
    /// Every level, in report order.
    pub(crate) const ALL: [Level; 3] = [Level::Must, Level::Should, Level::May];

    /// The keyword as written in specs.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Level::Must => "MUST",
            Level::Should => "SHOULD",
            Level::May => "MAY",
        }
    }

    fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|level| level.as_str() == text)
    }
}

/// One spec file, as far as `trace` needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Spec {
    /// Repository-relative path, e.g. `docs/specs/xtask.md`.
    pub path: String,
    /// The crate: the file name without `.md`.
    pub crate_name: String,
    /// `maturity` from the frontmatter, if the frontmatter is readable and has it.
    pub maturity: Option<String>,
}

impl Spec {
    /// Whether XTASK-R044 makes untested MUST requirements errors for this spec.
    pub(crate) fn gated(&self) -> bool {
        matches!(
            self.maturity.as_deref(),
            Some("usable" | "stable" | "optimized")
        )
    }
}

/// One requirement row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Requirement {
    /// `PREFIX-Rnnn`.
    pub id: String,
    /// MUST, SHOULD or MAY.
    pub level: Level,
    /// Whether the text contains `(withdrawn`.
    pub withdrawn: bool,
    /// Index into [`Extracted::specs`].
    pub spec: usize,
    /// 1-based line of the row in the spec.
    pub line: usize,
}

/// Everything extracted from the specs, and the XTASK-R040 and R045 findings.
#[derive(Debug, Clone, Default)]
pub(crate) struct Extracted {
    /// Every spec file, sorted by path.
    pub specs: Vec<Spec>,
    /// Every valid requirement, first occurrence of each ID, in spec and row order.
    pub requirements: Vec<Requirement>,
    /// IDs of rows that look like requirements but are malformed (an XTASK-R040 finding each).
    /// They are known IDs for XTASK-R028 and R043, so one mistake gives one finding.
    pub malformed: Vec<String>,
    /// XTASK-R040 and R045 findings, unsorted.
    pub findings: Vec<Finding>,
}

impl Extracted {
    /// The requirement with `id`, if valid.
    pub(crate) fn get(&self, id: &str) -> Option<&Requirement> {
        self.requirements.iter().find(|r| r.id == id)
    }

    /// Whether `id` is a requirement row of some spec, valid or malformed.
    pub(crate) fn is_known(&self, id: &str) -> bool {
        self.get(id).is_some() || self.malformed.iter().any(|m| m == id)
    }
}

/// Whether `text` matches `^[A-Z]+-R\d{3}$`.
pub(crate) fn is_requirement_id(text: &str) -> bool {
    let Some((prefix, number)) = text.split_once("-R") else {
        return false;
    };
    !prefix.is_empty()
        && prefix.bytes().all(|b| b.is_ascii_uppercase())
        && number.len() == 3
        && number.bytes().all(|b| b.is_ascii_digit())
}

/// `text` without its inline code spans (`` `…` ``), so that a requirement can name the marker
/// `(withdrawn` in code without being withdrawn (XTASK-R040 does). An unmatched backtick is kept
/// as text.
fn outside_code_spans(text: &str) -> String {
    let mut kept = String::new();
    let mut rest = text;
    while let Some(open) = rest.find('`') {
        let run = rest[open..].bytes().take_while(|b| *b == b'`').count();
        let fence = &rest[open..open + run];
        let after = &rest[open + run..];
        let Some(close) = after.find(fence) else {
            break;
        };
        kept.push_str(&rest[..open]);
        rest = &after[close + run..];
    }
    kept.push_str(rest);
    kept
}

/// The 1-based line range of section 6 in `text`: from the level-2 heading whose text starts
/// with `6.` or `6 ` up to the next level-1 or level-2 heading. Headings in fenced code do not
/// count.
fn section_6(text: &str) -> Option<(usize, usize)> {
    let mut start = None;
    for (line, content) in markdown::prose_lines(text) {
        let heading = content
            .strip_prefix("## ")
            .or_else(|| content.strip_prefix("# "));
        let Some(heading) = heading else { continue };
        match start {
            None if content.starts_with("## ")
                && (heading.starts_with("6.") || heading.starts_with("6 ")) =>
            {
                start = Some(line);
            }
            Some(start) => return Some((start, line)),
            None => {}
        }
    }
    start.map(|start| (start, usize::MAX))
}

/// Extracts every requirement of every spec in `model` (XTASK-R040), reporting malformed rows
/// (R040) and duplicate IDs (R045).
pub(crate) fn extract(model: &RepoModel) -> Extracted {
    let mut extracted = Extracted::default();
    let mut first_seen: BTreeMap<String, String> = BTreeMap::new();
    for (path, text) in model.spec_files() {
        let fields = frontmatter::parse(text).ok();
        let scalar = |key: &str| match fields.as_ref()?.get(key).map(|field| &field.value) {
            Some(Value::Scalar(value)) => Some(value.clone()),
            _ => None,
        };
        let prefix = scalar("req_prefix");
        let spec = extracted.specs.len();
        extracted.specs.push(Spec {
            path: path.to_string(),
            crate_name: file_name(path).trim_end_matches(".md").to_string(),
            maturity: scalar("maturity"),
        });
        let Some((start, end)) = section_6(text) else {
            continue;
        };
        for table in markdown::tables(text) {
            for row in &table.rows {
                if row.line <= start || row.line >= end {
                    continue;
                }
                let Some(id) = row.cells.first().filter(|id| is_requirement_id(id)) else {
                    continue;
                };
                let location = format!("{path}:{}", row.line);
                let mut report = |message: String, help: &str| {
                    extracted
                        .findings
                        .push(Finding::error("XTASK-R040", &location, message, help));
                };
                if let Some(first) = first_seen.get(id) {
                    extracted.findings.push(Finding::error(
                        "XTASK-R045",
                        &location,
                        format!("requirement ID `{id}` is already defined at {first}"),
                        "Give the requirement the next free ID of its spec; IDs are never reused",
                    ));
                    continue;
                }
                first_seen.insert(id.clone(), location.clone());
                let level = row.cells.get(1).and_then(|level| Level::parse(level));
                let text = row.cells.get(2).map_or("", String::as_str);
                let Some(level) = level.filter(|_| !text.is_empty()) else {
                    report(
                        format!(
                            "requirement row `{id}` needs a level (MUST, SHOULD or MAY) in the \
                             second cell and the requirement text in the third"
                        ),
                        "Write the row as `| ID | MUST | text |`",
                    );
                    extracted.malformed.push(id.clone());
                    continue;
                };
                if let Some(prefix) = &prefix {
                    let id_prefix = id.split_once("-R").map_or("", |(p, _)| p);
                    if id_prefix != prefix {
                        report(
                            format!(
                                "requirement `{id}` does not use the spec's prefix \
                                 `req_prefix: {prefix}`"
                            ),
                            "Use the spec's `req_prefix` for every requirement ID in it",
                        );
                    }
                }
                extracted.requirements.push(Requirement {
                    id: id.clone(),
                    level,
                    withdrawn: outside_code_spans(text).contains("(withdrawn"),
                    spec,
                    line: row.line,
                });
            }
        }
    }
    extracted
}

#[cfg(test)]
mod tests;
