//! Fixture repositories for `trace` (spec §10: "one passing fixture plus one fixture per rule that
//! must fail with exactly that rule ID"). Layout: `xtask/tests/fixtures/trace/README.md`.

use std::path::{Path, PathBuf};

use super::*;
use crate::test_support::{TempRoot, copy_dir};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/trace")
}

/// Runs the `trace` step on `valid/` with the overlay `case` (if any) copied on top.
fn run_fixture(case: Option<&str>) -> Vec<Finding> {
    let root = TempRoot::new(&format!("trace-{}", case.unwrap_or("valid")));
    copy_dir(&fixtures().join("valid"), root.path());
    if let Some(case) = case {
        copy_dir(&fixtures().join(case), root.path());
    }
    run_check(root.path()).unwrap()
}

/// `(rule, location, message)` of every finding of `case`.
fn found(case: &str) -> Vec<(String, String, String)> {
    run_fixture(Some(case))
        .into_iter()
        .map(|f| (f.rule, f.location, f.message))
        .collect()
}

/// Asserts that `case` has exactly one finding, of `rule` at `location`, with `message`.
fn one(case: &str, rule: &str, location: &str, message: &str) {
    assert_eq!(
        found(case),
        [(rule.to_string(), location.to_string(), message.to_string())],
        "{case}"
    );
}

/// Verifies: XTASK-R040, XTASK-R041, XTASK-R042, XTASK-R043, XTASK-R044, XTASK-R045
#[test]
fn valid_fixture_passes() {
    // A `usable` spec whose MUSTs are tested, except a withdrawn one; an untested SHOULD; an
    // excluded file with an annotation naming an unknown ID.
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
            "r040_bad_row",
            "r041_unattached",
            "r043_unknown_id",
            "r043_withdrawn_id",
            "r044_untested_must",
            "r045_duplicate_id",
        ]
    );
}

/// Verifies: XTASK-R040
#[test]
fn r040_bad_row_fixture() {
    one(
        "r040_bad_row",
        "XTASK-R040",
        "docs/specs/demo.md:21",
        "requirement row `DEMO-R005` needs a level (MUST, SHOULD or MAY) in the second cell and \
         the requirement text in the third",
    );
}

/// Verifies: XTASK-R041
#[test]
fn r041_unattached_fixture() {
    one(
        "r041_unattached",
        "XTASK-R041",
        "crates/demo/src/extra.rs:1",
        "annotation is not attached to a function: the next item after it is not a `fn`",
    );
}

/// Verifies: XTASK-R043
#[test]
fn r043_unknown_id_fixture() {
    one(
        "r043_unknown_id",
        "XTASK-R043",
        "crates/demo/src/extra.rs:1",
        "annotation names `DEMO-R009`, which is not a requirement in any spec",
    );
}

/// Verifies: XTASK-R043
#[test]
fn r043_withdrawn_id_fixture() {
    one(
        "r043_withdrawn_id",
        "XTASK-R043",
        "crates/demo/src/extra.rs:1",
        "annotation names `DEMO-R003`, which is withdrawn",
    );
}

/// Verifies: XTASK-R044
#[test]
fn r044_untested_must_fixture() {
    one(
        "r044_untested_must",
        "XTASK-R044",
        "docs/specs/demo.md:21",
        "MUST requirement `DEMO-R005` has no verifying test, and the spec's maturity is `usable`",
    );
}

/// Verifies: XTASK-R045
#[test]
fn r045_duplicate_id_fixture() {
    one(
        "r045_duplicate_id",
        "XTASK-R045",
        "docs/specs/demo.md:21",
        "requirement ID `DEMO-R001` is already defined at docs/specs/demo.md:17",
    );
}
