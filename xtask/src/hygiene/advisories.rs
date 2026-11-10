//! XTASK-R085: every ignored advisory in `deny.toml` carries an expiry date and a task, and is
//! removed by that date (`docs/specs/xtask.md` "`doctor` and `deny` details").
//!
//! [`check`] is a pure function of the parsed entries, the current date and the task statuses.
//! Only [`load`] reads a file and [`today`] reads the clock.

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use serde::Deserialize;
use toml::Spanned;

use crate::finding::Finding;
use crate::repo::{self, LoadError};

/// Repository-relative path of the cargo-deny policy.
pub(crate) const DENY_PATH: &str = "deny.toml";

/// The form every `reason` must end with.
const REASON_FORM: &str = "`<why it does not affect us>; expires YYYY-MM-DD (<task ID>)`";

/// A calendar date (proleptic Gregorian), ordered by time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Date {
    /// Year.
    pub year: i64,
    /// Month, 1–12.
    pub month: u32,
    /// Day of the month, 1–31.
    pub day: u32,
}

impl Date {
    /// The date `days` days after 1970-01-01 (negative: before). Howard Hinnant's
    /// `civil_from_days` algorithm.
    pub(crate) fn from_unix_days(days: i64) -> Self {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097);
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = doy - (153 * mp + 2) / 5 + 1;
        let month = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = yoe + era * 400 + i64::from(month <= 2);
        Self {
            year,
            // Both are in range by construction (1..=12 and 1..=31).
            month: u32::try_from(month).unwrap_or(1),
            day: u32::try_from(day).unwrap_or(1),
        }
    }

    /// Parses `YYYY-MM-DD` (exactly four, two and two digits) that names an existing day.
    pub(crate) fn parse(text: &str) -> Option<Self> {
        let bytes = text.as_bytes();
        let digits = |range: std::ops::Range<usize>| {
            bytes[range.clone()]
                .iter()
                .all(u8::is_ascii_digit)
                .then(|| text[range].parse::<u32>().ok())
                .flatten()
        };
        if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
            return None;
        }
        let date = Self {
            year: i64::from(digits(0..4)?),
            month: digits(5..7)?,
            day: digits(8..10)?,
        };
        let leap = date.year % 4 == 0 && (date.year % 100 != 0 || date.year % 400 == 0);
        let days_in_month = match date.month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if leap => 29,
            2 => 28,
            _ => return None,
        };
        (1..=days_in_month).contains(&date.day).then_some(date)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// Today's date in UTC, from the system clock. A clock before 1970 counts as 1970-01-01.
pub(crate) fn today() -> Date {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    Date::from_unix_days(i64::try_from(seconds / 86_400).unwrap_or(0))
}

/// One `[advisories] ignore` entry as written, with its line.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct IgnoreEntry {
    /// 1-based line of the entry in `deny.toml`.
    pub line: usize,
    /// The entry: a string (advisory ID) or a table (`id` or `crate`, and `reason`).
    pub value: toml::Value,
}

#[derive(Deserialize, Default)]
struct RawPolicy {
    #[serde(default)]
    advisories: RawAdvisories,
}

#[derive(Deserialize, Default)]
struct RawAdvisories {
    #[serde(default)]
    ignore: Vec<Spanned<toml::Value>>,
}

/// Reads the `[advisories] ignore` entries of `deny.toml` in `root`. A repository without
/// `deny.toml` has none.
///
/// # Errors
///
/// [`LoadError`] if the file cannot be read, and LIM-XTASK-0004 if it is not valid TOML or
/// `advisories.ignore` is not an array.
pub(crate) fn load(root: &Path) -> Result<Vec<IgnoreEntry>, LoadError> {
    if !root.join(DENY_PATH).is_file() {
        return Ok(Vec::new());
    }
    parse(&repo::read(root, DENY_PATH)?)
}

/// Parses the `[advisories] ignore` entries of the text of `deny.toml`. Other keys are not
/// read; cargo-deny validates them.
///
/// # Errors
///
/// LIM-XTASK-0004 if `text` is not valid TOML or `advisories.ignore` is not an array.
pub(crate) fn parse(text: &str) -> Result<Vec<IgnoreEntry>, LoadError> {
    let raw: RawPolicy = toml::from_str(text).map_err(|error| LoadError {
        code: "LIM-XTASK-0004",
        path: DENY_PATH.to_string(),
        detail: format!("is not valid: {}", error.to_string().trim_end()),
        help: "Fix the file at the reported location",
    })?;
    Ok(raw
        .advisories
        .ignore
        .into_iter()
        .map(|entry| IgnoreEntry {
            line: text[..entry.span().start].matches('\n').count() + 1,
            value: entry.into_inner(),
        })
        .collect())
}

/// Every XTASK-R085 finding for `entries` on `today`. `tasks` maps task ID → status (`None` if
/// unreadable), as for XTASK-R051.
pub(crate) fn check(
    entries: &[IgnoreEntry],
    today: Date,
    tasks: &BTreeMap<String, Option<String>>,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for entry in entries {
        let location = format!("{DENY_PATH}:{}", entry.line);
        let name = entry_name(&entry.value);
        let mut report = |message: String, help: String| {
            findings.push(Finding::error("XTASK-R085", &location, message, help));
        };
        let Some(reason) = entry
            .value
            .get("reason")
            .and_then(toml::Value::as_str)
            .filter(|_| entry.value.is_table())
        else {
            report(
                format!("ignored advisory {name} has no `reason`"),
                format!(
                    "Write the entry as `{{ id = \"RUSTSEC-YYYY-NNNN\", reason = \"…\" }}` with a \
                     reason of the form {REASON_FORM}"
                ),
            );
            continue;
        };
        let Some((date, task)) = expiry(reason) else {
            report(
                format!("the reason of ignored advisory {name} does not end with an expiry"),
                format!("End the reason with {REASON_FORM}"),
            );
            continue;
        };
        match Date::parse(date) {
            None => report(
                format!("ignored advisory {name} expires on `{date}`, which is not a date"),
                "Write the expiry date as YYYY-MM-DD".to_string(),
            ),
            Some(date) if date <= today => report(
                format!("ignored advisory {name} expired on {date} (today is {today})"),
                format!(
                    "Remove the entry if the advisory no longer applies, or decide again in task \
                     {task} and set a new expiry date"
                ),
            ),
            Some(_) => {}
        }
        match tasks.get(task) {
            None => report(
                format!("ignored advisory {name} names task {task}, which does not exist"),
                "Name the task that removes the entry".to_string(),
            ),
            Some(Some(status)) if status == "done" => report(
                format!("ignored advisory {name} names task {task}, which is done"),
                "Remove the entry, or name an open task that removes it".to_string(),
            ),
            Some(_) => {}
        }
    }
    findings
}

/// `` `RUSTSEC-…` `` (the `id`, or `crate`, or the string itself), or `entry` if it has none.
fn entry_name(value: &toml::Value) -> String {
    let name = value.as_str().or_else(|| {
        ["id", "crate"]
            .iter()
            .find_map(|key| value.get(key).and_then(toml::Value::as_str))
    });
    name.map_or_else(|| "entry".to_string(), |name| format!("`{name}`"))
}

/// The date and task ID of a reason that ends with `expires <date> (<task ID>)` (trailing
/// whitespace allowed). The date is returned unparsed.
fn expiry(reason: &str) -> Option<(&str, &str)> {
    let rest = reason.trim_end().strip_suffix(')')?;
    let (head, task) = rest.rsplit_once(" (")?;
    let (before, date) = head.rsplit_once("expires ")?;
    let separated = before.is_empty() || before.ends_with(|c: char| c.is_whitespace());
    let is_task = super::task_ids(task).first().map(String::as_str) == Some(task);
    (separated && is_task && !date.contains(char::is_whitespace)).then_some((date, task))
}

#[cfg(test)]
mod tests;
