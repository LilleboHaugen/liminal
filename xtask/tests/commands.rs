//! Runs the real `xtask` binary for every command except `help` (`tests/cli.rs`) and `ci`
//! (`tests/ci.rs`), and checks exit code and output (MATURITY.md, tool-crate `usable` gate).
//!
//! The binary always works on this repository: its root is fixed at compile time
//! (`CARGO_MANIFEST_DIR/..`, XTASK-R001), and there is no option to point it elsewhere. So:
//!
//! - `ok` runs (exit 0) expect the repository to pass the check, as the library tests
//!   `the_real_repository_passes` already do.
//! - Failing runs use usage errors (exit 2), or `$CARGO` set to a program that does not exist
//!   (spec: every command spawns `$CARGO`, else `cargo` from `PATH`). That gives a missing tool
//!   (`doctor`, `coverage`) or LIM-XTASK-0005 (`check-arch`) without changing the repository.
//! - No test writes to the repository (XTASK-R010). `status` without `--check` writes
//!   `docs/STATUS.md` when it is stale, so its test runs it only after `status --check` passed.
//! - `doctor` assertions hold whether or not `cargo-llvm-cov`, `cargo-deny` and Miri are installed
//!   (the CI `checks` jobs have none of them).
//! - A real `coverage` measurement runs `cargo llvm-cov`, which runs this test suite again. The
//!   test doing it is ignored, so the nested run skips it, and the `coverage` job of the CI
//!   workflow runs it with `--ignored`. `ci` runs of the `coverage` step are covered by the same
//!   job (`cargo xtask ci --only coverage`).

use std::path::PathBuf;
use std::process::{Command, Output};

use serde_json::Value;

/// The result of one run: exit code, stdout, stderr.
struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Run {
    /// stdout parsed as JSON.
    fn json(&self) -> Value {
        serde_json::from_str(&self.stdout)
            .unwrap_or_else(|error| panic!("stdout is not JSON ({error}): {}", self.stdout))
    }
}

/// Runs the xtask binary with `args`, with `$CARGO` replaced by `cargo` if given.
fn xtask_with(args: &[&str], cargo: Option<&PathBuf>) -> Run {
    let mut command = Command::new(env!("CARGO_BIN_EXE_xtask"));
    command.args(args);
    if let Some(cargo) = cargo {
        command.env("CARGO", cargo);
    }
    let Output {
        status,
        stdout,
        stderr,
    } = command
        .output()
        .expect("test setup: the xtask binary must start");
    Run {
        code: status.code(),
        stdout: String::from_utf8(stdout).expect("stdout is UTF-8"),
        stderr: String::from_utf8(stderr).expect("stderr is UTF-8"),
    }
}

fn xtask(args: &[&str]) -> Run {
    xtask_with(args, None)
}

/// A `$CARGO` value naming a program that does not exist.
fn missing_cargo() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("xtask-test-no-such-cargo")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("test setup: xtask/ has a parent")
        .to_path_buf()
}

/// A check command's `--json` report: `version` 1, `ok` matching the exit code, `findings` a list
/// (spec "Check command output").
fn assert_check_report(run: &Run) -> Value {
    let report = run.json();
    assert_eq!(report["version"], 1, "stdout: {}", run.stdout);
    let ok = report["ok"].as_bool().expect("`ok` is a boolean");
    assert_eq!(run.code, Some(i32::from(!ok)), "stdout: {}", run.stdout);
    assert!(report["findings"].is_array(), "stdout: {}", run.stdout);
    report
}

/// A passing check command: exit 0, result line in human mode, `ok: true` and no findings in JSON.
fn assert_check_passes(command: &str) {
    let human = xtask(&[command]);
    assert_eq!(
        human.code,
        Some(0),
        "stdout: {}\nstderr: {}",
        human.stdout,
        human.stderr
    );
    assert!(
        human.stdout.ends_with(&format!("{command}: ok\n")),
        "stdout: {}",
        human.stdout
    );
    assert_eq!(human.stderr, "");

    let json = xtask(&[command, "--json"]);
    let report = assert_check_report(&json);
    assert_eq!(report["ok"], true);
    assert_eq!(report["findings"], Value::Array(Vec::new()));
    assert_eq!(json.stderr, "");
}

/// An option the command does not accept: usage error on stderr, nothing on stdout, exit 2.
fn assert_usage_error(args: &[&str], expected: &str) {
    let run = xtask(args);
    assert_eq!(run.code, Some(2), "args: {args:?}");
    assert_eq!(run.stdout, "", "args: {args:?}");
    assert!(
        run.stderr.starts_with(&format!("error: {expected}")),
        "stderr: {}",
        run.stderr
    );
    assert!(
        run.stderr.contains("Usage: cargo xtask"),
        "stderr: {}",
        run.stderr
    );
}

/// Verifies: XTASK-R009
#[test]
fn check_arch_passes_on_the_repository() {
    assert_check_passes("check-arch");
}

/// Verifies: XTASK-R002
#[test]
fn check_arch_rejects_unknown_options() {
    assert_usage_error(
        &["check-arch", "--bogus"],
        "unknown option `--bogus` for `check-arch`",
    );
}

#[test]
fn check_arch_reports_a_cargo_that_cannot_start_as_internal_error_on_stderr() {
    let run = xtask_with(&["check-arch", "--json"], Some(&missing_cargo()));
    assert_eq!(run.code, Some(1));
    assert_eq!(run.stdout, "");
    assert!(
        run.stderr.starts_with("error: LIM-XTASK-0005: "),
        "stderr: {}",
        run.stderr
    );
    assert!(
        run.stderr.contains("could not start"),
        "stderr: {}",
        run.stderr
    );
}

/// Verifies: XTASK-R009
#[test]
fn check_docs_passes_on_the_repository() {
    assert_check_passes("check-docs");
}

/// Verifies: XTASK-R002
#[test]
fn check_docs_rejects_unexpected_arguments() {
    assert_usage_error(
        &["check-docs", "docs"],
        "unexpected argument `docs` for `check-docs`",
    );
}

/// Verifies: XTASK-R009
#[test]
fn hygiene_passes_on_the_repository() {
    assert_check_passes("hygiene");
}

/// Verifies: XTASK-R002
#[test]
fn hygiene_rejects_unknown_options() {
    assert_usage_error(
        &["hygiene", "--fix"],
        "unknown option `--fix` for `hygiene`",
    );
}

/// Verifies: XTASK-R009
#[test]
fn trace_passes_and_prints_the_matrix() {
    assert_check_passes("trace");
    let human = xtask(&["trace", "--crate", "xtask"]);
    assert_eq!(human.code, Some(0));
    assert!(
        human.stdout.contains("XTASK-R001"),
        "stdout: {}",
        human.stdout
    );
}

/// Verifies: XTASK-R009
#[test]
fn trace_json_with_crate_lists_only_that_crate() {
    let run = xtask(&["trace", "--json", "--crate=xtask"]);
    let report = assert_check_report(&run);
    let crates = report["crates"].as_array().expect("`crates` is a list");
    assert_eq!(crates.len(), 1);
    assert_eq!(crates[0]["crate"], "xtask");
    let requirements = report["requirements"]
        .as_array()
        .expect("`requirements` is a list");
    assert!(requirements.iter().any(|row| row["id"] == "XTASK-R001"));
    assert!(requirements.iter().all(|row| row["crate"] == "xtask"));
}

/// Verifies: XTASK-R002
#[test]
fn trace_rejects_a_crate_without_spec_and_a_missing_crate_name() {
    assert_usage_error(
        &["trace", "--crate", "xtask-test-no-such-crate"],
        "`--crate xtask-test-no-such-crate`: no spec",
    );
    assert_usage_error(&["trace", "--crate"], "`--crate` needs a crate name");
}

/// Runs only the passing side of `status --check`: the failing side needs a stale
/// `docs/STATUS.md`, and the binary cannot be pointed at another root or made to see one without
/// changing the repository (XTASK-R010). The library tests in `src/status/tests.rs` verify
/// XTASK-R072 on both sides.
#[test]
fn status_check_passes_and_status_leaves_an_up_to_date_file_unchanged() {
    let path = repo_root().join("docs/STATUS.md");
    let check = xtask(&["status", "--check"]);
    assert_eq!(
        check.code,
        Some(0),
        "docs/STATUS.md is stale; run `cargo xtask status` (this test does not, to keep the \
         repository unchanged). stdout: {}",
        check.stdout
    );
    assert_eq!(check.stdout, "status: ok\n");
    assert_eq!(check.stderr, "");

    let before = std::fs::read(&path).expect("docs/STATUS.md is readable");
    let write = xtask(&["status"]);
    assert_eq!(write.code, Some(0));
    assert_eq!(write.stdout, "status: docs/STATUS.md is up to date\n");
    assert_eq!(write.stderr, "");
    assert_eq!(
        std::fs::read(&path).expect("docs/STATUS.md is readable"),
        before
    );
}

/// Verifies: XTASK-R002
#[test]
fn status_has_no_json_option() {
    assert_usage_error(
        &["status", "--json"],
        "unknown option `--json` for `status`",
    );
}

/// Tool names in `doctor` output order (spec "`doctor` and `deny` details").
const TOOLS: &[&str] = &[
    "rustc",
    "cargo",
    "rustfmt",
    "clippy",
    "cargo-llvm-cov",
    "cargo-deny",
    "miri",
];

/// Checks a `doctor --json` report against the spec and returns its tools and its number of
/// findings. Holds whatever is installed.
fn assert_doctor_report(run: &Run) -> (Vec<Value>, usize) {
    let report = assert_check_report(run);
    let tools = report["tools"]
        .as_array()
        .expect("`tools` is a list")
        .clone();
    let names: Vec<&str> = tools.iter().filter_map(|t| t["tool"].as_str()).collect();
    assert_eq!(names, TOOLS);
    let findings = report["findings"].as_array().expect("`findings` is a list");
    for tool in &tools {
        let name = tool["tool"].as_str().expect("`tool` is a string");
        let finding = findings.iter().find(|f| f["location"] == name);
        if tool["present"] == true {
            assert_eq!(tool["help"], Value::Null, "tool: {tool}");
            assert!(finding.is_none(), "present tool {name} has a finding");
        } else {
            assert_eq!(tool["present"], false, "tool: {tool}");
            assert_eq!(tool["version"], Value::Null, "tool: {tool}");
            assert!(tool["help"].is_string(), "tool: {tool}");
            let finding = finding.unwrap_or_else(|| panic!("missing tool {name} has no finding"));
            assert_eq!(finding["rule"], "XTASK-R080");
            let needed = !tool["needed_by"].as_array().expect("list").is_empty();
            let severity = if needed { "error" } else { "warning" };
            assert_eq!(finding["severity"], severity, "finding: {finding}");
            assert!(
                finding["help"].as_str().is_some_and(
                    |help| help.ends_with("ask the project owner before installing (ADR-0022)")
                ),
                "finding: {finding}"
            );
        }
    }
    let errors = findings.iter().any(|f| f["severity"] == "error");
    assert_eq!(report["ok"], !errors);
    (tools, findings.len())
}

/// Verifies: XTASK-R080, XTASK-R009
#[test]
fn doctor_reports_every_tool_whatever_is_installed() {
    let json = xtask(&["doctor", "--json"]);
    assert_eq!(json.stderr, "");
    let (tools, findings) = assert_doctor_report(&json);

    let human = xtask(&["doctor"]);
    assert_eq!(human.code, json.code);
    let lines: Vec<&str> = human.stdout.lines().collect();
    assert!(lines.len() > TOOLS.len(), "stdout: {}", human.stdout);
    for (line, tool) in lines.iter().zip(&tools) {
        let name = tool["tool"].as_str().expect("`tool` is a string");
        let words: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(words.first(), Some(&name), "line: {line}");
        let state = line
            .trim_start()
            .strip_prefix(name)
            .unwrap_or_else(|| panic!("line does not start with {name}: {line}"))
            .trim_start();
        if tool["present"] == false {
            assert_eq!(state, "missing", "line: {line}");
        } else if let Some(version) = tool["version"].as_str() {
            assert_eq!(state, version, "line: {line}");
        } else {
            assert_eq!(state, "present (no version output)", "line: {line}");
        }
    }
    let result = match findings {
        0 => "doctor: ok".to_string(),
        1 => "doctor: 1 finding".to_string(),
        n => format!("doctor: {n} findings"),
    };
    assert_eq!(
        lines.last(),
        Some(&result.as_str()),
        "stdout: {}",
        human.stdout
    );
}

/// Verifies: XTASK-R080
#[test]
fn doctor_reports_cargo_tools_missing_when_cargo_cannot_start() {
    let run = xtask_with(&["doctor", "--json"], Some(&missing_cargo()));
    assert_eq!(run.code, Some(1));
    let (tools, _) = assert_doctor_report(&run);
    for tool in &tools[1..6] {
        assert_eq!(tool["present"], false, "tool: {tool}");
    }
}

/// Verifies: XTASK-R002
#[test]
fn doctor_rejects_unknown_options() {
    assert_usage_error(&["doctor", "--all"], "unknown option `--all` for `doctor`");
}

/// Verifies: XTASK-R064, XTASK-R009
#[test]
fn coverage_is_skipped_and_fails_when_cargo_cannot_start() {
    let human = xtask_with(&["coverage"], Some(&missing_cargo()));
    assert_eq!(human.code, Some(1));
    assert!(
        human
            .stdout
            .starts_with("coverage: skipped: could not start"),
        "stdout: {}",
        human.stdout
    );
    assert!(
        human.stdout.contains("\n  help: "),
        "stdout: {}",
        human.stdout
    );

    let json = xtask_with(&["coverage", "--json"], Some(&missing_cargo()));
    let report = assert_check_report(&json);
    assert_eq!(report["ok"], false);
    assert!(
        report["skipped"]["reason"].is_string(),
        "stdout: {}",
        json.stdout
    );
    assert!(
        report["skipped"]["help"].is_string(),
        "stdout: {}",
        json.stdout
    );
}

/// Verifies: XTASK-R002
#[test]
fn coverage_rejects_unknown_options() {
    assert_usage_error(
        &["coverage", "--floor"],
        "unknown option `--floor` for `coverage`",
    );
}

/// Verifies: XTASK-R060, XTASK-R009
#[test]
#[ignore = "M0-T011: needs cargo-llvm-cov and runs the test suite; the `coverage` job of the CI workflow runs it"]
fn coverage_measures_every_crate_against_its_floor() {
    let run = xtask(&["coverage", "--json"]);
    let report = assert_check_report(&run);
    assert_eq!(report["ok"], true, "stdout: {}", run.stdout);
    assert!(report.get("skipped").is_none(), "stdout: {}", run.stdout);
    let crates = report["crates"].as_array().expect("`crates` is a list");
    let xtask = crates
        .iter()
        .find(|c| c["crate"] == "xtask")
        .unwrap_or_else(|| panic!("no `xtask` entry: {}", run.stdout));
    assert!(xtask["percent"].is_number(), "stdout: {}", run.stdout);
    assert!(xtask["floor"].is_number(), "stdout: {}", run.stdout);
    assert!(
        xtask["count"].as_u64().is_some_and(|n| n > 0),
        "stdout: {}",
        run.stdout
    );
}
