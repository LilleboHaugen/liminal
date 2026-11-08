//! The Markdown that `check-docs` needs: link targets and tables (`docs/specs/xtask.md`
//! "`check-docs` details"). Not a general Markdown parser. It covers the constructs used in this
//! repository's docs, and those constructs are listed in the spec.

/// A link target found in a Markdown file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Link {
    /// 1-based line number.
    pub line: usize,
    /// The target as written, without the `<>` of the angle-bracket form and without a title.
    pub target: String,
}

/// A Markdown table: a header row, a delimiter row, and body rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Table {
    /// Header cells, trimmed.
    pub header: Vec<String>,
    /// Body rows.
    pub rows: Vec<Row>,
}

/// One body row of a [`Table`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Row {
    /// 1-based line number.
    pub line: usize,
    /// Cells, trimmed.
    pub cells: Vec<String>,
}

/// The lines of `text` that are not inside fenced code blocks (fence lines excluded), paired with
/// their 1-based line numbers.
fn prose_lines(text: &str) -> Vec<(usize, &str)> {
    let mut fence: Option<(u8, usize)> = None;
    let mut lines = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if let Some(marker) = fence_marker(line) {
            match fence {
                None => {
                    fence = Some(marker);
                    continue;
                }
                Some((byte, length)) if marker.0 == byte && marker.1 >= length => {
                    // A closing fence carries no info string.
                    if line.trim().bytes().all(|b| b == byte) {
                        fence = None;
                        continue;
                    }
                }
                Some(_) => {}
            }
        }
        if fence.is_none() {
            lines.push((index + 1, line));
        }
    }
    lines
}

/// The fence character and run length if `line` starts a fence (at most 3 spaces of indentation,
/// then 3 or more backticks or tildes).
fn fence_marker(line: &str) -> Option<(u8, usize)> {
    let trimmed = line.trim_start_matches(' ');
    if line.len() - trimmed.len() > 3 {
        return None;
    }
    let byte = *trimmed.as_bytes().first()?;
    if byte != b'`' && byte != b'~' {
        return None;
    }
    let length = trimmed.bytes().take_while(|b| *b == byte).count();
    (length >= 3).then_some((byte, length))
}

/// Replaces inline code spans (a run of N backticks up to the next run of exactly N backticks on
/// the same line) with spaces. An unmatched run stays as it is.
fn blank_code_spans(line: &str) -> String {
    let bytes = line.as_bytes();
    let mut out = String::with_capacity(line.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'`' {
            let next = line[index..]
                .find('`')
                .map_or(line.len(), |offset| index + offset);
            out.push_str(&line[index..next]);
            index = next;
            continue;
        }
        let run = bytes[index..].iter().take_while(|b| **b == b'`').count();
        let after = index + run;
        if let Some(close) = find_run(bytes, after, run) {
            out.extend(std::iter::repeat_n(' ', close + run - index));
            index = close + run;
        } else {
            out.push_str(&line[index..after]);
            index = after;
        }
    }
    out
}

/// The start of the next run of exactly `run` backticks at or after `from`.
fn find_run(bytes: &[u8], from: usize, run: usize) -> Option<usize> {
    let mut index = from;
    while index < bytes.len() {
        if bytes[index] == b'`' {
            let length = bytes[index..].iter().take_while(|b| **b == b'`').count();
            if length == run {
                return Some(index);
            }
            index += length;
        } else {
            index += 1;
        }
    }
    None
}

/// Every inline link, image and reference definition target in `text`, outside code.
pub(crate) fn links(text: &str) -> Vec<Link> {
    let mut links = Vec::new();
    for (line_number, line) in prose_lines(text) {
        let line = blank_code_spans(line);
        if let Some(target) = reference_definition(&line) {
            links.push(Link {
                line: line_number,
                target,
            });
            continue;
        }
        let mut rest = line.as_str();
        while let Some(start) = rest.find("](") {
            let after = &rest[start + 2..];
            if let Some(target) = destination(after) {
                links.push(Link {
                    line: line_number,
                    target,
                });
            }
            rest = after;
        }
    }
    links
}

/// The target of a reference definition line: `[label]: target`, indented at most 3 spaces.
fn reference_definition(line: &str) -> Option<String> {
    let trimmed = line.trim_start_matches(' ');
    if line.len() - trimmed.len() > 3 || !trimmed.starts_with('[') || trimmed.starts_with("[^") {
        return None;
    }
    let close = trimmed.find("]:")?;
    if trimmed[1..close].contains(['[', ']']) || close == 1 {
        return None;
    }
    let target = trimmed[close + 2..].split_whitespace().next()?;
    Some(
        target
            .trim_start_matches('<')
            .trim_end_matches('>')
            .to_string(),
    )
}

/// Parses an inline link destination starting right after `](`.
fn destination(after: &str) -> Option<String> {
    let after = after.trim_start();
    if let Some(inner) = after.strip_prefix('<') {
        return inner.find('>').map(|end| inner[..end].to_string());
    }
    let mut depth = 0usize;
    let mut end = after.len();
    for (index, c) in after.char_indices() {
        match c {
            '(' => depth += 1,
            ')' if depth == 0 => {
                end = index;
                break;
            }
            ')' => depth -= 1,
            c if c.is_whitespace() => {
                end = index;
                break;
            }
            _ => {}
        }
    }
    let target = &after[..end];
    (!target.is_empty()).then(|| target.to_string())
}

/// Every table in `text`, outside fenced code.
pub(crate) fn tables(text: &str) -> Vec<Table> {
    let lines = prose_lines(text);
    let mut tables = Vec::new();
    let mut index = 0;
    while index + 1 < lines.len() {
        let (_, header) = lines[index];
        let (delimiter_line, delimiter) = lines[index + 1];
        let consecutive = delimiter_line == lines[index].0 + 1;
        if !(consecutive && is_table_line(header) && is_delimiter(delimiter)) {
            index += 1;
            continue;
        }
        let mut table = Table {
            header: cells(header),
            rows: Vec::new(),
        };
        let mut previous = delimiter_line;
        index += 2;
        while let Some(&(line_number, line)) = lines.get(index) {
            if line_number != previous + 1 || !is_table_line(line) {
                break;
            }
            table.rows.push(Row {
                line: line_number,
                cells: cells(line),
            });
            previous = line_number;
            index += 1;
        }
        tables.push(table);
    }
    tables
}

fn is_table_line(line: &str) -> bool {
    line.trim_start().starts_with('|')
}

fn is_delimiter(line: &str) -> bool {
    is_table_line(line)
        && cells(line).iter().all(|cell| {
            let inner = cell.trim_start_matches(':').trim_end_matches(':');
            !inner.is_empty() && inner.bytes().all(|b| b == b'-')
        })
}

/// Cells of a table line: split on `|` not preceded by `\\`, outer pipes dropped, trimmed. An
/// escaped `\\|` stays in the cell as written.
fn cells(line: &str) -> Vec<String> {
    let line = line.trim();
    let line = line.strip_prefix('|').unwrap_or(line);
    let line = line
        .strip_suffix('|')
        .filter(|rest| !rest.ends_with('\\'))
        .unwrap_or(line);
    let mut cells = Vec::new();
    let mut start = 0;
    let bytes = line.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte == b'|' && (index == 0 || bytes[index - 1] != b'\\') {
            cells.push(line[start..index].trim().to_string());
            start = index + 1;
        }
    }
    cells.push(line[start..].trim().to_string());
    cells
}

/// The target of the first inline link in a table cell or other fragment.
pub(crate) fn first_link_target(fragment: &str) -> Option<String> {
    links(fragment).into_iter().next().map(|link| link.target)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn targets(text: &str) -> Vec<(usize, String)> {
        links(text)
            .into_iter()
            .map(|link| (link.line, link.target))
            .collect()
    }

    /// Verifies: XTASK-R024
    #[test]
    fn finds_inline_links_images_and_reference_definitions() {
        let text = "See [a](a.md) and [b](dir/b.md#anchor \"title\").\n\
                    ![img](img/x.png) [c](<with space.md>) [d]( spaced.md )\n\
                    [f](https://example.com) [g](#local) [h](p(1).md)\n\
                    [ref]: ref/target.md\n   [ref2]: <ref2.md> \"t\"\n";
        assert_eq!(
            targets(text),
            [
                (1, "a.md".to_string()),
                (1, "dir/b.md#anchor".to_string()),
                (2, "img/x.png".to_string()),
                (2, "with space.md".to_string()),
                (2, "spaced.md".to_string()),
                (3, "https://example.com".to_string()),
                (3, "#local".to_string()),
                (3, "p(1).md".to_string()),
                (4, "ref/target.md".to_string()),
                (5, "ref2.md".to_string()),
            ]
        );
    }

    /// Verifies: XTASK-R024
    #[test]
    fn ignores_links_in_fenced_code_and_inline_code() {
        let text = "```rust\n[x](fenced.md)\n```\n~~~~\n[y](tilde.md)\n```\nstill code\n~~~~\n\
                    `[z](span.md)` ``a ` [w](double.md) `` [v](real.md) `unclosed [u](open.md)\n\
                    ````md\n```\n[t](inner.md)\n````\n[s](after.md)\n";
        assert_eq!(
            targets(text),
            [
                (9, "real.md".to_string()),
                (9, "open.md".to_string()),
                (14, "after.md".to_string()),
            ]
        );
    }

    /// Verifies: XTASK-R024
    #[test]
    fn footnote_definitions_are_not_reference_definitions() {
        let text = "Text[^1].\n\n[^1]: footnote.md is not a target, but [a](a.md) is.\n";
        assert_eq!(targets(text), [(3, "a.md".to_string())]);
    }

    #[test]
    fn indented_lines_with_four_spaces_are_not_fences_or_definitions() {
        let text = "    ```\n[a](a.md)\n    [b]: b.md\n";
        assert_eq!(targets(text), [(2, "a.md".to_string())]);
    }

    #[test]
    fn parses_tables_with_line_numbers() {
        let text = "Intro\n\n| ADR | Title | Status |\n|---|:---:|---|\n| [0001](0001-a.md) | A | accepted |\n\
                    | [0002](0002-b.md) | B \\| C | superseded by ADR-0003 |\nafter\n\n|not|a table|\n\
                    ```\n| H |\n|---|\n| in code |\n```\n| `Spec` | `x` |\n|---|---|\n| `a` | b |\n";
        let tables = tables(text);
        assert_eq!(tables.len(), 2);
        assert_eq!(tables[1].header, ["`Spec`", "`x`"]);
        assert_eq!(tables[1].rows[0].cells, ["`a`", "b"]);
        assert_eq!(tables[0].header, ["ADR", "Title", "Status"]);
        assert_eq!(tables[0].rows.len(), 2);
        assert_eq!(tables[0].rows[0].line, 5);
        assert_eq!(
            tables[0].rows[0].cells,
            ["[0001](0001-a.md)", "A", "accepted"]
        );
        assert_eq!(tables[0].rows[1].cells[1], "B \\| C");
        assert_eq!(tables[0].rows[1].cells[2], "superseded by ADR-0003");
        assert_eq!(
            first_link_target(&tables[0].rows[1].cells[0]),
            Some("0002-b.md".to_string())
        );
        assert_eq!(first_link_target("no link"), None);
    }
}
