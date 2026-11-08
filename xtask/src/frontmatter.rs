//! Parser for the restricted YAML subset used in spec, task and ADR frontmatter
//! (`docs/process/DOCUMENTATION.md` §Frontmatter, `docs/specs/xtask.md` XTASK-R020 and the
//! "`check-docs` details").
//!
//! The parser is strict: every line it cannot read exactly as the subset defines is an error with
//! its line number. Anything YAML would read differently (quotes, block scalars, anchors, nested
//! mappings) is rejected rather than guessed at.

use std::fmt;

/// A frontmatter value: a string or an inline list of strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Value {
    /// `key: text`. Integers and dates are strings at this level.
    Scalar(String),
    /// `key: [a, b]`. `[]` is the empty list.
    List(Vec<String>),
}

/// One `key: value` line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Field {
    /// The key (`snake_case`).
    pub key: String,
    /// The parsed value.
    pub value: Value,
    /// 1-based line number in the file.
    pub line: usize,
}

/// The parsed frontmatter of one file, fields in file order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Frontmatter {
    /// Every field, in the order of the file. Keys are unique.
    pub fields: Vec<Field>,
}

impl Frontmatter {
    /// The field with `key`, if present.
    pub(crate) fn get(&self, key: &str) -> Option<&Field> {
        self.fields.iter().find(|field| field.key == key)
    }
}

/// A syntax error: where and what.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParseError {
    /// 1-based line number in the file.
    pub line: usize,
    /// What is wrong, phrased for the author of the file.
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

/// First characters a value or list item must not have: YAML gives them a meaning, and the
/// subset has no quoting to escape them.
const FORBIDDEN_START: &[char] = &['"', '\'', '|', '>', '&', '*', '!', '{', '}', '%', '@', '`'];

/// First characters a value or list item must not have when whitespace or the end of the text
/// follows: YAML reads them as a sequence entry (`- `), a complex key (`? `) or a mapping value
/// (`: `). Followed by other text (`-1`, `?x`), YAML reads them as plain text.
const INDICATORS_BEFORE_SPACE: &[char] = &['-', '?', ':'];

/// Parses the frontmatter at the start of `text`.
///
/// # Errors
///
/// A [`ParseError`] for the first line that is not in the subset, for a missing opening `---` on
/// line 1, or for a missing closing `---`.
pub(crate) fn parse(text: &str) -> Result<Frontmatter, ParseError> {
    let mut lines = text.lines().map(normalize).enumerate();
    match lines.next() {
        Some((_, "---")) => {}
        _ => {
            return Err(error(
                1,
                "line 1 must be `---`, which opens the frontmatter",
            ));
        }
    }
    let mut fields: Vec<Field> = Vec::new();
    for (index, line) in lines {
        let number = index + 1;
        if line == "---" {
            return Ok(Frontmatter { fields });
        }
        let field = parse_line(line, number)?;
        if let Some(first) = fields.iter().find(|other| other.key == field.key) {
            return Err(error(
                number,
                format!(
                    "duplicate key `{}` (first on line {})",
                    field.key, first.line
                ),
            ));
        }
        fields.push(field);
    }
    Err(error(
        1,
        "the frontmatter opened here is never closed by `---`",
    ))
}

/// Drops trailing whitespace, including the `\r` of CRLF line endings.
fn normalize(line: &str) -> &str {
    line.trim_end()
}

fn error(line: usize, message: impl Into<String>) -> ParseError {
    ParseError {
        line,
        message: message.into(),
    }
}

fn parse_line(line: &str, number: usize) -> Result<Field, ParseError> {
    if line.is_empty() {
        return Err(error(number, "blank lines are not allowed in frontmatter"));
    }
    if line.starts_with(char::is_whitespace) {
        return Err(error(number, "indented line: nesting is not allowed"));
    }
    if line.starts_with('#') {
        return Err(error(
            number,
            "comment lines are not allowed; a comment may only follow a value",
        ));
    }
    let Some((key, rest)) = line.split_once(':') else {
        return Err(error(number, "expected `key: value`"));
    };
    if !is_key(key) {
        return Err(error(
            number,
            format!("key `{key}` is not snake_case (`^[a-z][a-z0-9_]*$`)"),
        ));
    }
    if !rest.is_empty() && !rest.starts_with([' ', '\t']) {
        return Err(error(number, format!("expected a space after `{key}:`")));
    }
    let value = strip_comment(rest).trim();
    if value.is_empty() {
        return Err(error(
            number,
            format!("`{key}` has no value; an empty list is `[]`"),
        ));
    }
    let value = if let Some(inner) = value.strip_prefix('[') {
        let Some(inner) = inner.strip_suffix(']') else {
            return Err(error(number, "a list must end with `]` on the same line"));
        };
        Value::List(parse_list(inner, number)?)
    } else {
        check_text(value, "value", number)?;
        Value::Scalar(value.to_string())
    };
    Ok(Field {
        key: key.to_string(),
        value,
        line: number,
    })
}

fn is_key(key: &str) -> bool {
    let mut chars = key.chars();
    chars.next().is_some_and(|first| first.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Cuts a comment: `#` preceded by a space or tab, to the end of the line.
fn strip_comment(rest: &str) -> &str {
    let bytes = rest.as_bytes();
    for (index, &byte) in bytes.iter().enumerate() {
        if byte == b'#' && index > 0 && matches!(bytes[index - 1], b' ' | b'\t') {
            return &rest[..index];
        }
    }
    rest
}

fn parse_list(inner: &str, number: usize) -> Result<Vec<String>, ParseError> {
    if inner.contains(['[', ']']) {
        return Err(error(
            number,
            "list items must not contain `[` or `]`; nested lists are not allowed",
        ));
    }
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }
    inner
        .split(',')
        .map(|item| {
            let item = item.trim();
            if item.is_empty() {
                return Err(error(number, "empty list item"));
            }
            check_text(item, "list item", number)?;
            Ok(item.to_string())
        })
        .collect()
}

/// Rejects text that YAML would not read as the same plain string.
fn check_text(text: &str, what: &str, number: usize) -> Result<(), ParseError> {
    if let Some(first) = text.chars().next().filter(|c| FORBIDDEN_START.contains(c)) {
        return Err(error(
            number,
            format!(
                "{what} `{text}` starts with `{first}`, which YAML interprets; the subset has no quoting"
            ),
        ));
    }
    let mut chars = text.chars();
    if let Some(first) = chars.next().filter(|c| INDICATORS_BEFORE_SPACE.contains(c))
        && chars.next().is_none_or(char::is_whitespace)
    {
        return Err(error(
            number,
            format!(
                "{what} `{text}` starts with `{first} `, which YAML reads as a sequence, key or mapping indicator"
            ),
        ));
    }
    if text.ends_with(':') {
        return Err(error(
            number,
            format!("{what} `{text}` ends with `:`, which YAML reads as a mapping key"),
        ));
    }
    if text.contains(": ") || text.contains(":\t") {
        return Err(error(
            number,
            format!("{what} `{text}` contains `: `, which YAML reads as a nested mapping"),
        ));
    }
    Ok(())
}

/// `^\d+$`.
pub(crate) fn is_integer(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}

/// `^\d{4}-\d{2}-\d{2}$`. The calendar date is not validated.
pub(crate) fn is_date(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 10
        && bytes.iter().enumerate().all(|(index, byte)| match index {
            4 | 7 => *byte == b'-',
            _ => byte.is_ascii_digit(),
        })
}

#[cfg(test)]
mod tests;
