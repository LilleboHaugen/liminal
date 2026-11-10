//! Fixture repositories for `check-arch` (spec §10: "integration on fixture `cargo metadata` JSON
//! saved in fixtures"). Layout: `xtask/tests/fixtures/check_arch/README.md`. No test here runs
//! cargo: the model is built from the saved `metadata.json`.

use std::path::{Path, PathBuf};

use super::*;
use crate::test_support::{TempRoot, copy_dir};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/check_arch")
}

/// Runs the `check-arch` rules on `valid/` with the overlay `case` (if any) copied on top.
fn run_fixture(case: Option<&str>) -> Vec<Finding> {
    let root = TempRoot::new(&format!("check-arch-{}", case.unwrap_or("valid")));
    copy_dir(&fixtures().join("valid"), root.path());
    if let Some(case) = case {
        copy_dir(&fixtures().join(case), root.path());
    }
    let json = std::fs::read_to_string(root.path().join("metadata.json")).unwrap();
    check(&repo::load_arch(root.path(), &json).unwrap())
}

/// Asserts that `case` fails with findings of `rule` only, each with a help text (XTASK-R037),
/// and returns them.
fn fails_with_exactly(case: &str, rule: &str) -> Vec<Finding> {
    let findings = run_fixture(Some(case));
    assert!(!findings.is_empty(), "{case}: no findings");
    assert!(
        findings
            .iter()
            .all(|f| f.rule == rule && !f.help.is_empty()),
        "{case}: expected only {rule}, got:\n{}",
        check::finding_lines(&findings)
    );
    findings
}

/// `(location, message)` of each finding.
fn located(findings: &[Finding]) -> Vec<(&str, &str)> {
    findings
        .iter()
        .map(|f| (f.location.as_str(), f.message.as_str()))
        .collect()
}

/// Verifies: XTASK-R030, XTASK-R031, XTASK-R032, XTASK-R033, XTASK-R034, XTASK-R035, XTASK-R036, XTASK-R039
#[test]
fn valid_fixture_passes() {
    let findings = run_fixture(None);
    assert!(findings.is_empty(), "{}", check::finding_lines(&findings));
}

#[test]
fn valid_fixture_metadata_is_what_the_readme_says() {
    // Guards against a stale or hand-edited metadata.json: the packages, their roots and the
    // dependency kinds the fixture is meant to cover.
    let json = std::fs::read_to_string(fixtures().join("valid/metadata.json")).unwrap();
    let packages = repo::metadata::parse(&json).unwrap();
    let names: Vec<&str> = packages.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["base", "cli", "macros", "peer", "runtime"]);
    for package in &packages {
        let manifest = fixtures()
            .join("valid")
            .join(&package.manifest_dir)
            .join("Cargo.toml");
        assert!(manifest.is_file(), "{}", manifest.display());
        for target in &package.targets {
            let file = fixtures().join("valid").join(&target.src_path);
            assert!(file.is_file(), "{}", file.display());
        }
    }
    let peer = &packages[3];
    let kinds: Vec<&str> = peer
        .targets
        .iter()
        .flat_map(|t| t.kinds.iter().map(String::as_str))
        .collect();
    assert_eq!(kinds, ["lib", "test", "custom-build"]);
    let cli = &packages[1];
    assert_eq!(rules::actual_kind(cli), "bin");
    assert_eq!(rules::actual_kind(&packages[2]), "proc-macro");
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
            "r030_registry",
            "r031_unregistered_member",
            "r032_layer_violation",
            "r033_external_scope",
            "r034_dev_dependency",
            "r035_missing_forbid",
            "r036_tool_dependency",
            "r039_incomplete_clippy_config",
            "r039_missing_clippy_config",
        ]
    );
}

/// Verifies: XTASK-R030
#[test]
fn r030_registry_fixture() {
    let findings = fails_with_exactly("r030_registry", "XTASK-R030");
    assert_eq!(
        located(&findings),
        [(
            "architecture.toml:56",
            "crate `planned` has kind `library`; kind is one of \"lib\", \"proc-macro\", \"bin\""
        )]
    );
}

/// Verifies: XTASK-R031
#[test]
fn r031_unregistered_member_fixture() {
    let findings = fails_with_exactly("r031_unregistered_member", "XTASK-R031");
    assert_eq!(
        located(&findings),
        [(
            "crates/extra/Cargo.toml",
            "workspace crate `extra` is not registered in architecture.toml"
        )]
    );
}

/// Verifies: XTASK-R032
#[test]
fn r032_layer_violation_fixture() {
    let findings = fails_with_exactly("r032_layer_violation", "XTASK-R032");
    assert_eq!(
        located(&findings),
        [(
            "crates/base/Cargo.toml",
            "crate `base` (layer 0) has a dev dependency on `runtime` (layer 1); allowed are \
             lower layers and the crate's same_layer_deps"
        )]
    );
}

/// Verifies: XTASK-R033
#[test]
fn r033_external_scope_fixture() {
    let findings = fails_with_exactly("r033_external_scope", "XTASK-R033");
    assert_eq!(
        located(&findings),
        [(
            "crates/runtime/Cargo.toml",
            "crate `runtime` (layer 1, kind lib) has a normal dependency on `serde` with scope \
             `tool`: `tool` scope is for layer 5 crates only"
        )]
    );
}

/// Verifies: XTASK-R034
#[test]
fn r034_dev_dependency_fixture() {
    let findings = fails_with_exactly("r034_dev_dependency", "XTASK-R034");
    assert_eq!(
        located(&findings),
        [(
            "crates/base/Cargo.toml",
            "crate `base` has an external dev dependency on `loom`, which is not in its \
             `dev_external` list"
        )]
    );
}

/// Verifies: XTASK-R035
#[test]
fn r035_missing_forbid_fixture() {
    let findings = fails_with_exactly("r035_missing_forbid", "XTASK-R035");
    assert_eq!(
        located(&findings),
        [(
            "crates/peer/src/lib.rs",
            "crate `peer` has unsafe = \"forbid\", but its crate root `crates/peer/src/lib.rs` \
             does not start with `#![forbid(unsafe_code)]`"
        )]
    );
}

/// Verifies: XTASK-R036
#[test]
fn r036_tool_dependency_fixture() {
    let findings = fails_with_exactly("r036_tool_dependency", "XTASK-R036");
    assert_eq!(
        located(&findings),
        [(
            "crates/runtime/Cargo.toml",
            "crate `runtime` (layer 1) has a normal dependency on `cli`, a layer 5 tool crate"
        )]
    );
}

/// Verifies: XTASK-R039
#[test]
fn r039_missing_clippy_config_fixture() {
    // `peer` became deterministic and has no clippy.toml of its own: the root file applies, and it
    // bans nothing. `runtime` (deterministic, complete file) and the other crates (not
    // deterministic, no file) give no finding.
    let findings = fails_with_exactly("r039_missing_clippy_config", "XTASK-R039");
    assert_eq!(
        located(&findings),
        [(
            "clippy.toml",
            "deterministic crate `peer`: its clippy configuration `clippy.toml` does not ban \
             `f32::sin`, `f32::cos`, `f32::tan`, `f32::powf`, `f32::exp`, `f32::ln`, `f64::sin`, \
             `f64::cos`, `f64::tan`, `f64::powf`, `f64::exp`, `f64::ln`, \
             `std::collections::HashMap::new`, `std::time::Instant::now` in `disallowed-methods`"
        )]
    );
    assert!(findings[0].help.contains("`crates/peer/clippy.toml`"));
}

/// Verifies: XTASK-R039
#[test]
fn r039_missing_clippy_config_without_a_root_file() {
    let root = TempRoot::new("check-arch-r039-no-file");
    copy_dir(&fixtures().join("valid"), root.path());
    copy_dir(&fixtures().join("r039_missing_clippy_config"), root.path());
    std::fs::remove_file(root.path().join("clippy.toml")).unwrap();
    let json = std::fs::read_to_string(root.path().join("metadata.json")).unwrap();
    let findings = check(&repo::load_arch(root.path(), &json).unwrap());
    assert_eq!(
        located(&findings),
        [(
            "crates/peer/Cargo.toml",
            "deterministic crate `peer` has no clippy configuration (no `.clippy.toml` or \
             `clippy.toml` in `crates/peer` or a parent directory), so clippy bans none of the \
             methods CODE_STYLE.md lists"
        )]
    );
}

/// Verifies: XTASK-R039
#[test]
fn r039_incomplete_clippy_config_fixture() {
    let findings = fails_with_exactly("r039_incomplete_clippy_config", "XTASK-R039");
    assert_eq!(
        located(&findings),
        [(
            "crates/runtime/clippy.toml",
            "deterministic crate `runtime`: its clippy configuration `crates/runtime/clippy.toml` \
             does not ban `f64::ln` in `disallowed-methods`"
        )]
    );
}

#[test]
fn an_invalid_clippy_config_of_a_deterministic_crate_is_lim_xtask_0004() {
    let root = TempRoot::new("check-arch-r039-invalid");
    copy_dir(&fixtures().join("valid"), root.path());
    std::fs::write(
        root.path().join("crates/runtime/clippy.toml"),
        "disallowed-methods = 1\n",
    )
    .unwrap();
    let json = std::fs::read_to_string(root.path().join("metadata.json")).unwrap();
    let error = repo::load_arch(root.path(), &json).unwrap_err();
    assert_eq!(
        (error.code, error.path.as_str()),
        ("LIM-XTASK-0004", "crates/runtime/clippy.toml")
    );
}

#[test]
fn a_missing_crate_root_file_is_lim_xtask_0001() {
    let root = TempRoot::new("check-arch-missing-root");
    copy_dir(&fixtures().join("valid"), root.path());
    std::fs::remove_file(root.path().join("crates/base/src/lib.rs")).unwrap();
    let json = std::fs::read_to_string(root.path().join("metadata.json")).unwrap();
    let error = repo::load_arch(root.path(), &json).unwrap_err();
    assert_eq!(
        (error.code, error.path.as_str()),
        ("LIM-XTASK-0001", "crates/base/src/lib.rs")
    );
}

/// The real repository passes (acceptance criterion 1). This test runs `cargo metadata`, through
/// the same function the command uses.
#[test]
fn the_real_repository_passes() {
    let findings = run_check(&repo::root()).unwrap();
    assert!(findings.is_empty(), "{}", check::finding_lines(&findings));
}
