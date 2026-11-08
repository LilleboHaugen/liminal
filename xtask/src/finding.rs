//! The finding record shared by every check (`docs/specs/xtask.md` §4 "Finding", §5 schema).
//!
//! Checks added in M0-T003 onward return `Vec<Finding>`; `ci` attaches them to the step that
//! produced them, and each check's `--json` mode (XTASK-R009) serializes the same structure.

use serde::Serialize;

/// How serious a finding is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Severity {
    /// Makes the check fail. (`warning` is added by the first check that reports one.)
    // Constructed only in tests until the first check lands; `expect` turns into a warning (and a
    // CI failure) the moment non-test code constructs it, which forces removing this attribute.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "first constructed by check-docs (M0-T003)")
    )]
    Error,
}

impl Severity {
    /// The name used in JSON and human output: `error`.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
        }
    }
}

/// One violation of a repository rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Finding {
    /// The rule violated, e.g. `XTASK-R044` (INV-2: always a rule ID from the xtask spec).
    pub rule: String,
    /// Whether the finding fails the check.
    pub severity: Severity,
    /// `path` or `path:line`, relative to the repository root, `/`-separated on every OS.
    pub location: String,
    /// What is wrong.
    pub message: String,
    /// How to fix it.
    pub help: String,
}
