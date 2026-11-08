//! What every check command shares: the `--json` option, the report format and the exit code
//! (XTASK-R009, `docs/specs/xtask.md` "Check command output").

use std::fmt::Write as _;
use std::io::Write;
use std::path::Path;

use serde::Serialize;

use crate::cli::{self, CommandError, Exit};
use crate::finding::{Finding, Severity};
use crate::repo::{self, LoadError};

/// A check: loads the repository at the given root and returns its findings, sorted.
pub(crate) type CheckFn = fn(&Path) -> Result<Vec<Finding>, LoadError>;

/// Summary schema version of a check command's `--json` output.
const REPORT_VERSION: u32 = 1;

impl From<LoadError> for CommandError {
    fn from(error: LoadError) -> Self {
        CommandError::Internal {
            code: error.code,
            message: format!("{}: {} (help: {})", error.path, error.detail, error.help),
        }
    }
}

#[derive(Serialize)]
struct Report<'a> {
    version: u32,
    ok: bool,
    findings: &'a [Finding],
}

/// Runs the check command `name` on the real repository.
pub(crate) fn command(
    name: &str,
    args: &[String],
    out: &mut dyn Write,
    check: CheckFn,
) -> Result<Exit, CommandError> {
    execute(name, args, out, &repo::root(), check)
}

/// Runs the check command `name` on the repository at `root`.
pub(crate) fn execute(
    name: &str,
    args: &[String],
    out: &mut dyn Write,
    root: &Path,
    check: CheckFn,
) -> Result<Exit, CommandError> {
    let mut json = false;
    for arg in args {
        match arg.as_str() {
            "--json" => json = true,
            _ => return Err(cli::unexpected_argument(name, arg)),
        }
    }
    let findings = check(root)?;
    let ok = !findings
        .iter()
        .any(|finding| finding.severity == Severity::Error);
    if json {
        let report = Report {
            version: REPORT_VERSION,
            ok,
            findings: &findings,
        };
        let mut text = serde_json::to_string_pretty(&report)
            .expect("invariant: the report contains only strings, numbers, booleans and lists");
        text.push('\n');
        out.write_all(text.as_bytes())?;
    } else {
        out.write_all(human(name, &findings).as_bytes())?;
    }
    Ok(if ok { Exit::Success } else { Exit::Failure })
}

/// One block per finding: `<severity> <rule> <location>: <message>` and an indented `help:` line.
pub(crate) fn finding_lines(findings: &[Finding]) -> String {
    let mut text = String::new();
    for finding in findings {
        // Writing to a `String` cannot fail.
        let _ = writeln!(
            text,
            "{} {} {}: {}\n  help: {}",
            finding.severity.as_str(),
            finding.rule,
            finding.location,
            finding.message,
            finding.help
        );
    }
    text
}

/// The human-readable report: [`finding_lines`], then a result line.
fn human(name: &str, findings: &[Finding]) -> String {
    let mut text = finding_lines(findings);
    let _ = match findings.len() {
        0 => writeln!(text, "{name}: ok"),
        1 => writeln!(text, "{name}: 1 finding"),
        n => writeln!(text, "{name}: {n} findings"),
    };
    text
}

/// Sorts findings by location (path, then line number), then rule, then message (the spec's
/// fixed order).
pub(crate) fn sort(findings: &mut [Finding]) {
    findings.sort_by(|a, b| {
        (location_key(&a.location), &a.rule, &a.message).cmp(&(
            location_key(&b.location),
            &b.rule,
            &b.message,
        ))
    });
}

/// `path:line` → (`path`, Some(line)); `path` → (`path`, None), which sorts first.
fn location_key(location: &str) -> (&str, Option<u64>) {
    match location.rsplit_once(':') {
        Some((path, line)) => match line.parse() {
            Ok(line) => (path, Some(line)),
            Err(_) => (location, None),
        },
        None => (location, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[expect(
        clippy::unnecessary_wraps,
        reason = "must have the signature of check::CheckFn"
    )]
    fn two_findings(_: &Path) -> Result<Vec<Finding>, LoadError> {
        Ok(vec![
            Finding::error("XTASK-R024", "a.md:3", "broken", "fix it"),
            Finding::error("XTASK-R021", "b.md", "bad status", "use accepted"),
        ])
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "must have the signature of check::CheckFn"
    )]
    fn no_findings(_: &Path) -> Result<Vec<Finding>, LoadError> {
        Ok(Vec::new())
    }

    fn cannot_load(_: &Path) -> Result<Vec<Finding>, LoadError> {
        Err(LoadError::read("docs", &std::io::Error::other("denied")))
    }

    fn run(args: &[&str], check: CheckFn) -> (Result<Exit, CommandError>, String) {
        let args: Vec<String> = args.iter().map(ToString::to_string).collect();
        let mut out = Vec::new();
        let result = execute("check-x", &args, &mut out, Path::new("."), check);
        (result, String::from_utf8(out).unwrap())
    }

    /// Verifies: XTASK-R009
    #[test]
    fn json_reports_findings_in_the_summary_format() {
        let (result, out) = run(&["--json"], two_findings);
        assert!(matches!(result, Ok(Exit::Failure)));
        let value: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(value["version"], 1);
        assert_eq!(value["ok"], false);
        assert_eq!(
            value["findings"][0],
            serde_json::json!({ "rule": "XTASK-R024", "severity": "error",
                "location": "a.md:3", "message": "broken", "help": "fix it" })
        );
        let (result, out) = run(&["--json"], no_findings);
        assert!(matches!(result, Ok(Exit::Success)));
        assert_eq!(
            out,
            "{\n  \"version\": 1,\n  \"ok\": true,\n  \"findings\": []\n}\n"
        );
    }

    #[test]
    fn human_mode_lists_findings_and_a_result_line() {
        let (result, out) = run(&[], two_findings);
        assert!(matches!(result, Ok(Exit::Failure)));
        assert_eq!(
            out,
            "error XTASK-R024 a.md:3: broken\n  help: fix it\n\
             error XTASK-R021 b.md: bad status\n  help: use accepted\ncheck-x: 2 findings\n"
        );
        assert_eq!(run(&[], no_findings).1, "check-x: ok\n");
    }

    #[test]
    fn unknown_arguments_are_usage_errors_and_load_errors_are_internal() {
        for args in [&["--verbose"][..], &["extra"][..]] {
            assert!(matches!(
                run(args, no_findings).0,
                Err(CommandError::Usage(_))
            ));
        }
        let (result, out) = run(&[], cannot_load);
        let Err(CommandError::Internal { code, message }) = result else {
            panic!("expected an internal error, got {result:?}");
        };
        assert_eq!(code, "LIM-XTASK-0001");
        assert!(
            message.starts_with("docs: could not be read: denied"),
            "{message}"
        );
        assert_eq!(out, "");
    }

    #[test]
    fn sort_orders_by_location_then_rule_then_message() {
        let mut findings = vec![
            Finding::error("XTASK-R022", "b.md", "m", "h"),
            Finding::error("XTASK-R021", "b.md", "z", "h"),
            Finding::error("XTASK-R021", "b.md", "a", "h"),
            Finding::error("XTASK-R025", "a.md:10", "m", "h"),
            Finding::error("XTASK-R025", "a.md:9", "m", "h"),
            Finding::error("XTASK-R025", "a.md", "m", "h"),
        ];
        sort(&mut findings);
        let keys: Vec<(&str, &str, &str)> = findings
            .iter()
            .map(|f| (f.location.as_str(), f.rule.as_str(), f.message.as_str()))
            .collect();
        assert_eq!(
            keys,
            [
                ("a.md", "XTASK-R025", "m"),
                ("a.md:9", "XTASK-R025", "m"),
                ("a.md:10", "XTASK-R025", "m"),
                ("b.md", "XTASK-R021", "a"),
                ("b.md", "XTASK-R021", "z"),
                ("b.md", "XTASK-R022", "m")
            ]
        );
    }
}
