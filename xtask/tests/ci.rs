//! Runs the real `xtask` binary for `ci` (XTASK-R003 … R008) and checks exit code and output.
//!
//! The full `ci` run is not started from tests: its `test` and `coverage` steps run `cargo test`,
//! which would run this test again. The tests here select only the native check steps (which run
//! inside xtask) or the `deny` step with `$CARGO` set to a program that does not exist, so nothing
//! spawns the test suite. The real cargo steps run when `cargo xtask ci` runs locally and in every
//! job of the CI workflow (`docs/specs/xtask.md` §10).
//!
//! Every `ci` run writes `target/xtask/ci-summary.json` (XTASK-R005, allowed by R010). Runs that
//! read it back are in one test so that no parallel test overwrites it in between.

use std::path::PathBuf;
use std::process::{Command, Output};

use serde_json::Value;

/// The native check steps, in §5 order.
const NATIVE_STEPS: &[&str] = &[
    "check-arch",
    "check-docs",
    "trace",
    "hygiene",
    "status-check",
];

fn xtask_with(args: &[&str], cargo: Option<&PathBuf>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_xtask"));
    command.args(args);
    if let Some(cargo) = cargo {
        command.env("CARGO", cargo);
    }
    command
        .output()
        .expect("test setup: the xtask binary must start")
}

fn xtask(args: &[&str]) -> Output {
    xtask_with(args, None)
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("output is UTF-8")
}

fn summary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("test setup: xtask/ has a parent")
        .join("target/xtask/ci-summary.json")
}

/// Verifies: XTASK-R007, XTASK-R008
#[test]
fn unknown_step_names_exit_2_without_running_anything() {
    for args in [
        &["ci", "--only", "nonsense"][..],
        &["ci", "--skip", "fmt,nonsense"][..],
        &["ci", "--allow-skip=nonsense"][..],
        &["ci", "--bogus"][..],
    ] {
        let output = xtask(args);
        assert_eq!(output.status.code(), Some(2), "args: {args:?}");
        assert!(output.stdout.is_empty(), "args: {args:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.starts_with("error: "), "stderr: {stderr}");
        assert!(stderr.contains("usage: cargo xtask ci"), "stderr: {stderr}");
    }
}

/// Verifies: XTASK-R003, XTASK-R005, XTASK-R006, XTASK-R007
#[test]
fn ci_reports_passing_and_skipped_steps_in_json_human_output_and_the_summary_file() {
    // Passing native steps, named out of order: they run in §5 order, exit 0.
    let json = xtask(&[
        "ci",
        "--json",
        "--only",
        "status-check,hygiene,trace",
        "--only=check-docs,check-arch",
    ]);
    let stdout = text(&json.stdout);
    assert_eq!(json.status.code(), Some(0), "stdout: {stdout}");
    let summary: Value = serde_json::from_str(&stdout).expect("stdout is the JSON summary");
    assert_eq!(summary["version"], 1);
    assert_eq!(summary["ok"], true);
    let steps = summary["steps"].as_array().expect("`steps` is a list");
    let names: Vec<&str> = steps.iter().filter_map(|s| s["name"].as_str()).collect();
    assert_eq!(names, NATIVE_STEPS);
    assert!(steps.iter().all(|s| s["status"] == "pass"), "{stdout}");
    let file = std::fs::read_to_string(summary_path()).expect("the summary file is written");
    assert_eq!(file, stdout);

    // Human mode: one line per step, the result and the summary path.
    let human = xtask(&["ci", "--only", "check-docs,status-check"]);
    let stdout = text(&human.stdout);
    assert_eq!(human.status.code(), Some(0), "stdout: {stdout}");
    assert!(stdout.contains("result: ok\n"), "stdout: {stdout}");
    assert!(
        stdout.ends_with("summary written to target/xtask/ci-summary.json\n"),
        "stdout: {stdout}"
    );
    let file: Value = serde_json::from_str(&std::fs::read_to_string(summary_path()).unwrap())
        .expect("the summary file is JSON");
    let names: Vec<&str> = file["steps"]
        .as_array()
        .expect("`steps` is a list")
        .iter()
        .filter_map(|s| s["name"].as_str())
        .collect();
    assert_eq!(names, ["check-docs", "status-check"]);

    // A step whose tool cannot start is skipped, which fails the run (exit 1) ...
    let missing = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("xtask-test-no-such-cargo");
    let skipped = xtask_with(
        &["ci", "--json", "--only", "check-docs,deny"],
        Some(&missing),
    );
    let stdout = text(&skipped.stdout);
    assert_eq!(skipped.status.code(), Some(1), "stdout: {stdout}");
    let summary: Value = serde_json::from_str(&stdout).expect("stdout is the JSON summary");
    assert_eq!(summary["ok"], false);
    assert_eq!(summary["steps"][0]["status"], "pass");
    let deny = &summary["steps"][1];
    assert_eq!(deny["name"], "deny");
    assert_eq!(deny["status"], "skipped");
    assert!(
        deny["reason"].is_string() && deny["help"].is_string(),
        "{stdout}"
    );

    // ... unless it is named in `--allow-skip`; it is still reported as skipped (exit 0).
    let allowed = xtask_with(
        &["ci", "--json", "--only", "deny", "--allow-skip", "deny"],
        Some(&missing),
    );
    let stdout = text(&allowed.stdout);
    assert_eq!(allowed.status.code(), Some(0), "stdout: {stdout}");
    let summary: Value = serde_json::from_str(&stdout).expect("stdout is the JSON summary");
    assert_eq!(summary["ok"], true);
    assert_eq!(summary["steps"][0]["status"], "skipped");
}
