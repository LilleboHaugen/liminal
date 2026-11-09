//! Typed access to a document's frontmatter fields, reporting problems as findings of one rule.

use crate::finding::Finding;
use crate::frontmatter::{Frontmatter, Value};

/// Reads the fields of one file for one rule (R021, R022 or R023). Missing and unknown fields are
/// reported when the reader is created. Wrong value forms are reported when a field is read.
pub(super) struct Fields<'a> {
    rule: &'static str,
    path: &'a str,
    frontmatter: &'a Frontmatter,
    findings: &'a mut Vec<Finding>,
}

impl<'a> Fields<'a> {
    /// A reader for `path`. Reports every missing field of `required` and every field not in it.
    pub(super) fn new(
        rule: &'static str,
        path: &'a str,
        frontmatter: &'a Frontmatter,
        required: &[&str],
        findings: &'a mut Vec<Finding>,
    ) -> Self {
        let mut fields = Self {
            rule,
            path,
            frontmatter,
            findings,
        };
        for key in required {
            if frontmatter.get(key).is_none() {
                fields.report(
                    None,
                    format!("required field `{key}` is missing"),
                    format!("Add `{key}:`; required fields: {}", required.join(", ")),
                );
            }
        }
        for field in &frontmatter.fields {
            if !required.contains(&field.key.as_str()) {
                fields.report(
                    Some(field.line),
                    format!("unknown field `{}`", field.key),
                    format!(
                        "Remove it or fix its name; allowed fields: {}",
                        required.join(", ")
                    ),
                );
            }
        }
        fields
    }

    /// Records a finding of this reader's rule at `line` (or the file, if `None`).
    pub(super) fn report(
        &mut self,
        line: Option<usize>,
        message: impl Into<String>,
        help: impl Into<String>,
    ) {
        let location = match line {
            Some(line) => format!("{}:{line}", self.path),
            None => self.path.to_string(),
        };
        self.findings
            .push(Finding::error(self.rule, location, message, help));
    }

    /// The string value of `key` and its line. `None` if missing (already reported) or a list
    /// (reported here).
    pub(super) fn scalar(&mut self, key: &str) -> Option<(&'a str, usize)> {
        let field = self.frontmatter.get(key)?;
        match &field.value {
            Value::Scalar(text) => Some((text.as_str(), field.line)),
            Value::List(_) => {
                self.report(
                    Some(field.line),
                    format!("`{key}` must be a single value, not a list"),
                    format!("Write `{key}: <value>`"),
                );
                None
            }
        }
    }

    /// The list value of `key` and its line. `None` if missing (already reported) or a single
    /// value (reported here).
    pub(super) fn list(&mut self, key: &str) -> Option<(&'a [String], usize)> {
        let field = self.frontmatter.get(key)?;
        match &field.value {
            Value::List(items) => Some((items.as_slice(), field.line)),
            Value::Scalar(_) => {
                self.report(
                    Some(field.line),
                    format!("`{key}` must be a list"),
                    format!("Write `{key}: [a, b]`, or `{key}: []` for none"),
                );
                None
            }
        }
    }

    /// The value of `key` if it passes `valid`; otherwise reports `expected` and returns `None`.
    pub(super) fn scalar_where(
        &mut self,
        key: &str,
        valid: impl Fn(&str) -> bool,
        expected: &str,
    ) -> Option<(&'a str, usize)> {
        let (value, line) = self.scalar(key)?;
        if valid(value) {
            Some((value, line))
        } else {
            self.report(
                Some(line),
                format!("`{key}` is `{value}`; expected {expected}"),
                format!("Set `{key}` to {expected}"),
            );
            None
        }
    }
}

/// Whether `text` is one of `allowed`; for use with [`Fields::scalar_where`].
pub(super) fn one_of(allowed: &'static [&'static str]) -> impl Fn(&str) -> bool {
    move |text| allowed.contains(&text)
}

/// `^M\d+$`.
pub(crate) fn is_milestone(text: &str) -> bool {
    text.strip_prefix('M')
        .is_some_and(crate::frontmatter::is_integer)
}

/// `^M\d+-T\d{3}$`.
pub(crate) fn is_task_id(text: &str) -> bool {
    text.split_once("-T").is_some_and(|(milestone, number)| {
        is_milestone(milestone) && number.len() == 3 && number.bytes().all(|b| b.is_ascii_digit())
    })
}

/// `^[A-Z]+$`.
pub(super) fn is_req_prefix(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|b| b.is_ascii_uppercase())
}
