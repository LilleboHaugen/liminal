//! Fixture repositories for `check-docs` (spec §10: "one passing fixture plus one fixture per rule
//! that must fail with exactly that rule ID"). Layout: `xtask/tests/fixtures/check_docs/README.md`.

use std::path::{Path, PathBuf};

use super::*;
use crate::test_support::{TempRoot, copy_dir};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/check_docs")
}

/// Runs `check-docs` on `valid/` with the overlay `case` (if any) copied on top.
fn run_fixture(case: Option<&str>) -> Vec<Finding> {
    let root = TempRoot::new(&format!("check-docs-{}", case.unwrap_or("valid")));
    copy_dir(&fixtures().join("valid"), root.path());
    if let Some(case) = case {
        copy_dir(&fixtures().join(case), root.path());
    }
    run_check(root.path()).unwrap()
}

/// Asserts that `case` fails with findings of `rule` only, and returns them.
fn fails_with_exactly(case: &str, rule: &str) -> Vec<Finding> {
    let findings = run_fixture(Some(case));
    assert!(!findings.is_empty(), "{case}: no findings");
    let rules: Vec<&str> = findings.iter().map(|f| f.rule.as_str()).collect();
    assert!(
        rules.iter().all(|r| *r == rule),
        "{case}: expected only {rule}, got:\n{}",
        check::finding_lines(&findings)
    );
    findings
}

fn locations(findings: &[Finding]) -> Vec<&str> {
    findings.iter().map(|f| f.location.as_str()).collect()
}

/// Verifies: XTASK-R020, XTASK-R021, XTASK-R022, XTASK-R023, XTASK-R024, XTASK-R025, XTASK-R026, XTASK-R027, XTASK-R028
#[test]
fn valid_fixture_passes() {
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
        .filter(|name| name != "valid")
        .collect();
    cases.sort();
    assert_eq!(
        cases,
        [
            "r020_frontmatter_syntax",
            "r021_adr_fields",
            "r022_spec_fields",
            "r023_task_fields",
            "r024_broken_link",
            "r025_adr_index",
            "r026_member_spec",
            "r027_spec_index",
            "r028_task_requirements",
        ]
    );
}

/// Verifies: XTASK-R020
#[test]
fn r020_frontmatter_syntax_fixture() {
    let findings = fails_with_exactly("r020_frontmatter_syntax", "XTASK-R020");
    assert_eq!(locations(&findings), ["docs/tasks/M0-T002-second.md:5"]);
}

/// Verifies: XTASK-R021
#[test]
fn r021_adr_fields_fixture() {
    let findings = fails_with_exactly("r021_adr_fields", "XTASK-R021");
    assert_eq!(locations(&findings), ["docs/adr/0002-second.md:4"]);
    assert!(findings[0].message.contains("`status` is `approved`"));
}

/// Verifies: XTASK-R022
#[test]
fn r022_spec_fields_fixture() {
    let findings = fails_with_exactly("r022_spec_fields", "XTASK-R022");
    assert_eq!(locations(&findings), ["docs/specs/demo.md:4"]);
}

/// Verifies: XTASK-R023
#[test]
fn r023_task_fields_fixture() {
    let findings = fails_with_exactly("r023_task_fields", "XTASK-R023");
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].message,
        "dependency cycle: M0-T001 → M0-T002 → M0-T001"
    );
}

/// Verifies: XTASK-R024
#[test]
fn r024_broken_link_fixture() {
    let findings = fails_with_exactly("r024_broken_link", "XTASK-R024");
    assert_eq!(locations(&findings), ["README.md:16"]);
}

/// Verifies: XTASK-R025
#[test]
fn r025_adr_index_fixture() {
    let findings = fails_with_exactly("r025_adr_index", "XTASK-R025");
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].message,
        "ADR-0002 (docs/adr/0002-second.md) is not listed"
    );
}

/// Verifies: XTASK-R026
#[test]
fn r026_member_spec_fixture() {
    let findings = fails_with_exactly("r026_member_spec", "XTASK-R026");
    assert_eq!(locations(&findings), ["tools/unspecified/Cargo.toml"]);
}

/// Verifies: XTASK-R027
#[test]
fn r027_spec_index_fixture() {
    let findings = fails_with_exactly("r027_spec_index", "XTASK-R027");
    assert_eq!(locations(&findings), ["docs/specs/README.md:7"]);
}

/// Verifies: XTASK-R009
#[test]
fn check_docs_json_on_a_failing_fixture() {
    let root = TempRoot::new("check-docs-json");
    copy_dir(&fixtures().join("valid"), root.path());
    copy_dir(&fixtures().join("r024_broken_link"), root.path());
    let mut out = Vec::new();
    let exit = check::execute(
        NAME,
        &["--json".to_string()],
        &mut out,
        root.path(),
        run_check,
    );
    assert!(matches!(exit, Ok(Exit::Failure)));
    let value: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(value["ok"], false);
    assert_eq!(value["findings"][0]["rule"], "XTASK-R024");
    assert_eq!(value["findings"][0]["location"], "README.md:16");
}

#[test]
fn invalid_architecture_toml_is_an_internal_error() {
    let root = TempRoot::new("check-docs-bad-arch");
    copy_dir(&fixtures().join("valid"), root.path());
    std::fs::write(root.path().join("architecture.toml"), "schema_version = [").unwrap();
    let error = run_check(root.path()).unwrap_err();
    assert_eq!(error.code, "LIM-XTASK-0003");
}

/// Verifies: XTASK-R028
#[test]
fn r028_task_requirements_fixture() {
    let findings = fails_with_exactly("r028_task_requirements", "XTASK-R028");
    assert_eq!(locations(&findings), ["docs/tasks/M0-T002-second.md:8"]);
}
