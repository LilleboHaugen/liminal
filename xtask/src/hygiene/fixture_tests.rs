//! Fixture repositories for `hygiene` (spec §10). Layout:
//! `xtask/tests/fixtures/hygiene/README.md`.

use std::path::{Path, PathBuf};

use super::*;
use crate::test_support::{TempRoot, copy_dir};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hygiene")
}

/// Runs `hygiene` on `valid/` with the overlay `case` (if any) copied on top.
fn run_fixture(case: Option<&str>) -> Vec<Finding> {
    let root = TempRoot::new(&format!("hygiene-{}", case.unwrap_or("valid")));
    copy_dir(&fixtures().join("valid"), root.path());
    if let Some(case) = case {
        copy_dir(&fixtures().join(case), root.path());
    }
    run_check(root.path()).unwrap()
}

/// `(rule, location)` of every finding of `case`.
fn found(case: &str) -> Vec<(String, String)> {
    run_fixture(Some(case))
        .into_iter()
        .map(|f| (f.rule, f.location))
        .collect()
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(a, b)| ((*a).to_string(), (*b).to_string()))
        .collect()
}

/// Verifies: XTASK-R050, XTASK-R051, XTASK-R052
#[test]
fn valid_fixture_passes() {
    // It contains tracked markers, reasoned ignores and patterns in strings; violations in an
    // excluded directory and in a `.rs` file outside every workspace member.
    let findings = run_fixture(None);
    assert!(findings.is_empty(), "{}", check::finding_lines(&findings));
}

#[test]
fn every_case_directory_is_tested() {
    let mut cases: Vec<String> = std::fs::read_dir(fixtures())
        .unwrap()
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name != "valid" && name != "cases")
        .collect();
    cases.sort();
    assert_eq!(
        cases,
        [
            "r050_bare_ignore",
            "r050_forbidden_macro",
            "r050_untracked_marker",
            "r051_done_task",
            "r051_unknown_task",
            "r052_exclusion",
            "r085_advisory_ignore",
        ]
    );
}

/// Verifies: XTASK-R050
#[test]
fn r050_untracked_marker_fixture() {
    assert_eq!(
        found("r050_untracked_marker"),
        pairs(&[("XTASK-R050", "crates/demo/src/extra.rs:1")])
    );
}

/// Verifies: XTASK-R050
#[test]
fn r050_forbidden_macro_fixture() {
    assert_eq!(
        found("r050_forbidden_macro"),
        pairs(&[("XTASK-R050", "crates/demo/src/extra.rs:2")])
    );
}

/// Verifies: XTASK-R050
#[test]
fn r050_bare_ignore_fixture() {
    assert_eq!(
        found("r050_bare_ignore"),
        pairs(&[("XTASK-R050", "crates/demo/src/extra.rs:2")])
    );
}

/// Verifies: XTASK-R051
#[test]
fn r051_unknown_task_fixture() {
    assert_eq!(
        found("r051_unknown_task"),
        pairs(&[("XTASK-R051", "crates/demo/src/extra.rs:1")])
    );
}

/// Verifies: XTASK-R051
#[test]
fn r051_done_task_fixture() {
    assert_eq!(
        found("r051_done_task"),
        pairs(&[("XTASK-R051", "crates/demo/src/extra.rs:1")])
    );
}

/// Verifies: XTASK-R052
#[test]
fn r052_exclusion_fixture() {
    // The `hygiene.exclude` entry has a blank reason (and still excludes its path, so the excluded
    // file gives no R050 finding); the `trace.exclude` path does not exist.
    let findings = run_fixture(Some("r052_exclusion"));
    let summary: Vec<(&str, &str, &str)> = findings
        .iter()
        .map(|f| (f.rule.as_str(), f.location.as_str(), f.message.as_str()))
        .collect();
    assert_eq!(
        summary,
        [
            (
                "XTASK-R052",
                "ci/hygiene.toml:1",
                "`hygiene.exclude` entry `crates/demo/tests/data` has no reason"
            ),
            (
                "XTASK-R052",
                "ci/hygiene.toml:5",
                "`trace.exclude` path `crates/demo/tests/missing` does not exist"
            ),
        ]
    );
}

/// Verifies: XTASK-R052
#[test]
fn r052_exclusion_paths_must_be_repository_relative() {
    let root = fixtures().join("valid");
    let mut model = sources::load(&root).unwrap();
    for path in [
        "/abs",
        "./crates",
        "crates/../ci",
        "crates\\demo",
        "",
        "crates//demo",
    ] {
        model.config.trace = vec![Exclusion {
            path: path.to_string(),
            reason: "r".to_string(),
            line: 9,
        }];
        let findings = check(&model);
        let summary: Vec<(&str, &str)> = findings
            .iter()
            .map(|f| (f.rule.as_str(), f.location.as_str()))
            .collect();
        assert_eq!(summary, [("XTASK-R052", "ci/hygiene.toml:9")], "{path:?}");
        assert!(
            findings[0]
                .message
                .contains("is not a repository-relative path"),
            "{path:?}"
        );
    }
    model.config.trace[0].path = "crates/demo/".to_string();
    assert_eq!(check(&model), Vec::new());
}

/// Verifies: XTASK-R085
#[test]
fn r085_advisory_ignore_fixture() {
    // `valid/deny.toml` has one entry that expires in 9999 and names an open task; the overlay
    // replaces it with an expired entry, one without an expiry, a bare ID and one naming a `done`
    // task.
    let findings = run_fixture(Some("r085_advisory_ignore"));
    let summary: Vec<(&str, &str, &str)> = findings
        .iter()
        .map(|f| (f.rule.as_str(), f.location.as_str(), f.message.as_str()))
        .collect();
    let expired = format!(
        "ignored advisory `RUSTSEC-2026-0001` expired on 2000-01-01 (today is {})",
        advisories::today()
    );
    assert_eq!(
        summary,
        [
            ("XTASK-R085", "deny.toml:3", expired.as_str()),
            (
                "XTASK-R085",
                "deny.toml:4",
                "the reason of ignored advisory `RUSTSEC-2026-0002` does not end with an expiry"
            ),
            (
                "XTASK-R085",
                "deny.toml:5",
                "ignored advisory `RUSTSEC-2026-0003` has no `reason`"
            ),
            (
                "XTASK-R085",
                "deny.toml:6",
                "ignored advisory `RUSTSEC-2026-0004` names task M0-T001, which is done"
            ),
        ]
    );
}

#[test]
fn the_real_repository_passes() {
    let findings = run_check(&crate::repo::root()).unwrap();
    assert!(findings.is_empty(), "{}", check::finding_lines(&findings));
}
