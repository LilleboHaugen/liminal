//! Orchestration tests for `ci` with a fake step runner (`docs/specs/xtask.md` §10). The real
//! runner is tested in `process.rs`; the real steps run in CI on the real repository.

use std::collections::BTreeMap;
use std::path::PathBuf;

use super::*;
use crate::finding::Severity;
use crate::test_support::TempRoot;

/// Steps with the names used in the fixture runs. Their invocations are never executed.
const FIXTURE_STEPS: &[Step] = &[
    fixture_step("fmt"),
    fixture_step("clippy"),
    fixture_step("trace"),
    fixture_step("coverage"),
];

const fn fixture_step(name: &'static str) -> Step {
    Step {
        name,
        action: Action::Cargo(CargoInvocation {
            args: &["fixture"],
            env: &[],
            requires: None,
        }),
    }
}

/// Returns scripted outcomes (default: pass) and records which steps ran in which mode.
#[derive(Default)]
struct FakeRunner {
    outcomes: BTreeMap<&'static str, Outcome>,
    calls: Vec<(&'static str, OutputMode)>,
}

impl FakeRunner {
    fn with(outcomes: &[(&'static str, Outcome)]) -> Self {
        Self {
            outcomes: outcomes.iter().cloned().collect(),
            calls: Vec::new(),
        }
    }

    fn ran(&self) -> Vec<&'static str> {
        self.calls.iter().map(|(name, _)| *name).collect()
    }
}

impl StepRunner for FakeRunner {
    fn run(&mut self, step: &Step, mode: OutputMode) -> Outcome {
        self.calls.push((step.name, mode));
        self.outcomes
            .get(step.name)
            .cloned()
            .unwrap_or(Outcome::Pass)
    }
}

fn fail() -> Outcome {
    Outcome::Fail {
        findings: Vec::new(),
        output_tail: None,
    }
}

fn skipped() -> Outcome {
    Outcome::Skipped {
        reason: "cargo-llvm-cov not installed".to_string(),
        help: "cargo install cargo-llvm-cov".to_string(),
    }
}

struct Run {
    result: Result<Exit, CommandError>,
    out: String,
    summary_file: Option<String>,
    runner: FakeRunner,
}

/// Runs `ci` over `FIXTURE_STEPS` with `outcomes` in a temporary root.
fn run_ci(name: &str, args: &[&str], outcomes: &[(&'static str, Outcome)]) -> Run {
    let root = TempRoot::new(&format!("ci-{name}"));
    let args: Vec<String> = args.iter().map(ToString::to_string).collect();
    let mut runner = FakeRunner::with(outcomes);
    let mut out = Vec::new();
    let result = execute(&args, &mut out, root.path(), FIXTURE_STEPS, &mut runner);
    Run {
        result,
        out: String::from_utf8(out).unwrap(),
        summary_file: std::fs::read_to_string(root.path().join(SUMMARY_PATH)).ok(),
        runner,
    }
}

fn exit(run: &Run) -> Exit {
    match &run.result {
        Ok(exit) => *exit,
        Err(error) => panic!("unexpected error: {error}"),
    }
}

fn summary(run: &Run) -> serde_json::Value {
    serde_json::from_str(run.summary_file.as_deref().unwrap()).unwrap()
}

fn step_names(summary: &serde_json::Value) -> Vec<&str> {
    summary["steps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|step| step["name"].as_str().unwrap())
        .collect()
}

fn usage_message(args: &[&str]) -> String {
    let args: Vec<String> = args.iter().map(ToString::to_string).collect();
    match parse_options(&args, FIXTURE_STEPS) {
        Err(CommandError::Usage(message)) => message,
        other => panic!("expected a usage error for {args:?}, got {other:?}"),
    }
}

// ---------------------------------------------------------------- R003 order

/// Verifies: XTASK-R003
#[test]
fn steps_run_in_table_order_whatever_the_option_order() {
    let run = run_ci(
        "order",
        &["--only", "coverage,fmt", "--only", "clippy"],
        &[],
    );
    assert_eq!(run.runner.ran(), ["fmt", "clippy", "coverage"]);
    assert_eq!(step_names(&summary(&run)), ["fmt", "clippy", "coverage"]);
}

/// Verifies: XTASK-R003
#[test]
fn step_table_follows_the_final_order_in_the_spec() {
    let spec = std::fs::read_to_string(repo::root().join("docs/specs/xtask.md")).unwrap();
    let line = spec
        .lines()
        .find(|line| line.starts_with("`ci` steps, in order:"))
        .unwrap();
    let mut final_order: Vec<&str> = line.split('`').skip(3).step_by(2).collect();
    // The sentence continues on the next line in the spec; take the rest from there.
    let next = spec
        .lines()
        .skip_while(|candidate| *candidate != line)
        .nth(1)
        .unwrap();
    final_order.extend(next.split('`').skip(1).step_by(2));
    assert_eq!(
        final_order,
        [
            "fmt",
            "clippy",
            "test",
            "doc",
            "check-arch",
            "check-docs",
            "trace",
            "hygiene",
            "status-check",
            "coverage",
            "deny"
        ],
        "spec line: {line}"
    );
    // STEPS must be a subsequence of the final order (later tasks insert at their place).
    let mut remaining = final_order.iter();
    for step in STEPS {
        assert!(
            remaining.any(|name| *name == step.name),
            "step `{}` is out of order",
            step.name
        );
    }
}

// ---------------------------------------------------------------- R004 failure handling

/// Verifies: XTASK-R004
#[test]
fn every_step_runs_after_a_failure_without_fail_fast() {
    let run = run_ci("continue", &[], &[("fmt", fail()), ("trace", fail())]);
    assert_eq!(run.runner.ran(), ["fmt", "clippy", "trace", "coverage"]);
    assert_eq!(exit(&run), Exit::Failure);
    let summary = summary(&run);
    assert_eq!(summary["ok"], false);
    let statuses: Vec<&str> = summary["steps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|step| step["status"].as_str().unwrap())
        .collect();
    assert_eq!(statuses, ["fail", "pass", "fail", "pass"]);
}

/// Verifies: XTASK-R004
#[test]
fn fail_fast_stops_after_the_first_failure() {
    let run = run_ci("fail-fast", &["--fail-fast"], &[("clippy", fail())]);
    assert_eq!(run.runner.ran(), ["fmt", "clippy"]);
    assert_eq!(exit(&run), Exit::Failure);
    assert_eq!(step_names(&summary(&run)), ["fmt", "clippy"]);
}

/// Verifies: XTASK-R004
#[test]
fn fail_fast_stops_at_an_unallowed_skip_but_not_at_an_allowed_one() {
    let run = run_ci("ff-skip", &["--fail-fast"], &[("clippy", skipped())]);
    assert_eq!(run.runner.ran(), ["fmt", "clippy"]);

    let run = run_ci(
        "ff-allowed",
        &["--fail-fast", "--allow-skip", "clippy"],
        &[("clippy", skipped())],
    );
    assert_eq!(run.runner.ran(), ["fmt", "clippy", "trace", "coverage"]);
    assert_eq!(exit(&run), Exit::Success);
}

// ---------------------------------------------------------------- R005 summary

/// Verifies: XTASK-R005
#[test]
fn json_mode_prints_the_summary_and_writes_the_same_file() {
    let run = run_ci("json", &["--json"], &[("trace", fail())]);
    assert_eq!(run.summary_file.as_deref(), Some(run.out.as_str()));
    let summary = summary(&run);
    assert_eq!(summary["version"], 1);
    assert_eq!(summary["ok"], false);
}

/// Verifies: XTASK-R005
#[test]
fn human_mode_writes_the_summary_file_and_prints_a_report() {
    let run = run_ci("human", &[], &[]);
    assert_eq!(exit(&run), Exit::Success);
    assert_eq!(summary(&run)["ok"], true);
    assert!(serde_json::from_str::<serde_json::Value>(&run.out).is_err());
    let headers: Vec<&str> = run
        .out
        .lines()
        .filter(|line| line.starts_with("==> "))
        .collect();
    assert_eq!(
        headers,
        [
            "==> fmt: cargo fixture",
            "==> clippy: cargo fixture",
            "==> trace: cargo fixture",
            "==> coverage: cargo fixture"
        ]
    );
    assert!(run.out.contains("\nci summary:\n"), "out: {}", run.out);
    assert!(run.out.contains("result: ok\n"), "out: {}", run.out);
    assert!(
        run.out
            .ends_with(&format!("summary written to {SUMMARY_PATH}\n"))
    );
}

/// Verifies: XTASK-R005
#[test]
fn json_summary_matches_the_approved_snapshot() {
    let finding = Finding {
        rule: "XTASK-R044".to_string(),
        severity: Severity::Error,
        location: "docs/specs/liminal_ecs.md".to_string(),
        message: "MUST requirement ECS-R001 has no verifying test".to_string(),
        help: "add a test with `/// Verifies: ECS-R001`".to_string(),
    };
    let run = run_ci(
        "snapshot",
        &["--json", "--allow-skip", "coverage"],
        &[
            (
                "clippy",
                Outcome::Fail {
                    findings: Vec::new(),
                    output_tail: Some(
                        "error: unused variable: `x`\nerror: could not compile".to_string(),
                    ),
                },
            ),
            (
                "trace",
                Outcome::Fail {
                    findings: vec![finding],
                    output_tail: None,
                },
            ),
            ("coverage", skipped()),
        ],
    );
    // Compare the exact text (field order included); only the durations vary between runs.
    let actual: String = run
        .out
        .split_inclusive('\n')
        .map(
            |line| match line.trim_start().strip_prefix("\"duration_ms\": ") {
                Some(value) => {
                    let digits = value.trim_end().trim_end_matches(',');
                    assert!(digits.parse::<u64>().is_ok(), "duration: {line}");
                    line.replace(digits, "\"<duration>\"")
                }
                None => line.to_string(),
            },
        )
        .collect();
    let snapshot_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/snapshots/ci-summary.json"
    );
    let approved = std::fs::read_to_string(snapshot_path).unwrap();
    assert_eq!(
        actual, approved,
        "summary differs from {snapshot_path}; update the snapshot only for an intended schema \
         change, and say why in the journal (AGENTS.md rule 5)"
    );
}

#[test]
fn unwritable_summary_file_is_a_file_error() {
    let root = TempRoot::new("ci-unwritable");
    // `target` is a file, so `target/xtask/` cannot be created.
    std::fs::write(root.path().join("target"), "").unwrap();
    let mut out = Vec::new();
    let result = execute(
        &[],
        &mut out,
        root.path(),
        FIXTURE_STEPS,
        &mut FakeRunner::default(),
    );
    let Err(CommandError::File { path, .. }) = result else {
        panic!("expected a file error, got {result:?}");
    };
    assert_eq!(path, root.path().join(SUMMARY_PATH));
}

// ---------------------------------------------------------------- R006 skips

/// Verifies: XTASK-R006
#[test]
fn skipped_step_is_reported_with_reason_and_help_and_fails_the_run() {
    let run = run_ci("skip", &["--json"], &[("coverage", skipped())]);
    assert_eq!(exit(&run), Exit::Failure);
    let summary = summary(&run);
    assert_eq!(summary["ok"], false);
    let coverage = &summary["steps"][3];
    assert_eq!(coverage["name"], "coverage");
    assert_eq!(coverage["status"], "skipped");
    assert_eq!(coverage["reason"], "cargo-llvm-cov not installed");
    assert_eq!(coverage["help"], "cargo install cargo-llvm-cov");
}

/// Verifies: XTASK-R006
#[test]
fn allowed_skip_keeps_ok_but_is_still_reported_as_skipped() {
    let run = run_ci(
        "allow-skip",
        &["--allow-skip", "coverage"],
        &[("coverage", skipped())],
    );
    assert_eq!(exit(&run), Exit::Success);
    let summary = summary(&run);
    assert_eq!(summary["ok"], true);
    assert_eq!(summary["steps"][3]["status"], "skipped");
    assert!(run.out.contains("skipped (allowed)"), "out: {}", run.out);
}

/// Verifies: XTASK-R006
#[test]
fn allow_skip_does_not_excuse_a_failure() {
    let run = run_ci(
        "allow-fail",
        &["--allow-skip", "trace"],
        &[("trace", fail())],
    );
    assert_eq!(exit(&run), Exit::Failure);
}

// ---------------------------------------------------------------- R007 exit codes

/// Verifies: XTASK-R007
#[test]
fn exit_code_is_0_when_ok_1_when_not_and_2_for_usage_errors() {
    assert_eq!(exit(&run_ci("exit-ok", &[], &[])).code(), 0);
    assert_eq!(
        exit(&run_ci("exit-fail", &[], &[("fmt", fail())])).code(),
        1
    );
    assert_eq!(
        exit(&run_ci("exit-skip", &[], &[("fmt", skipped())])).code(),
        1
    );
    // `crate::run` turns a usage error into exit 2 (tested in cli.rs and tests/ci.rs).
    let run = run_ci("exit-usage", &["--bogus"], &[]);
    assert!(matches!(run.result, Err(CommandError::Usage(_))));
    assert_eq!(run.runner.ran(), Vec::<&str>::new());
    assert_eq!(run.summary_file, None);
}

// ---------------------------------------------------------------- R008 selection

/// Verifies: XTASK-R008
#[test]
fn deselected_steps_are_omitted_and_do_not_affect_ok() {
    let run = run_ci(
        "deselect",
        &["--skip", "trace", "--json"],
        &[("trace", fail())],
    );
    assert_eq!(run.runner.ran(), ["fmt", "clippy", "coverage"]);
    assert_eq!(exit(&run), Exit::Success);
    let summary = summary(&run);
    assert_eq!(summary["ok"], true);
    assert_eq!(step_names(&summary), ["fmt", "clippy", "coverage"]);

    let run = run_ci("only", &["--only=clippy"], &[("fmt", fail())]);
    assert_eq!(run.runner.ran(), ["clippy"]);
    assert_eq!(exit(&run), Exit::Success);
}

/// Verifies: XTASK-R008
#[test]
fn unknown_step_names_are_usage_errors() {
    for flag in ["--only", "--skip", "--allow-skip"] {
        let message = usage_message(&[flag, "fmt,nonsense"]);
        assert!(
            message.starts_with(&format!("unknown step `nonsense` in `{flag}`")),
            "message: {message}"
        );
    }
    assert!(usage_message(&["--only", "fmt,"]).starts_with("unknown step `` in `--only`"));
}

/// Verifies: XTASK-R008
#[test]
fn selecting_no_steps_is_a_usage_error() {
    let message = usage_message(&["--only", "fmt", "--skip", "fmt"]);
    assert!(
        message.starts_with("no steps selected"),
        "message: {message}"
    );
}

#[test]
fn malformed_options_are_usage_errors() {
    let cases: [(&[&str], &str); 5] = [
        (&["--only"], "option `--only` needs a step list"),
        (&["--json=yes"], "option `--json` does not take a value"),
        (&["--verbose"], "unknown option `--verbose` for `ci`"),
        (&["fmt"], "unexpected argument `fmt` for `ci`"),
        (&["-j"], "unknown option `-j` for `ci`"),
    ];
    for (args, expected) in cases {
        let message = usage_message(args);
        assert!(message.starts_with(expected), "args {args:?}: {message}");
        assert!(message.ends_with(SYNOPSIS), "args {args:?}: {message}");
    }
}

#[test]
fn options_parse_flags_and_both_list_forms() {
    let args: Vec<String> = [
        "--json",
        "--fail-fast",
        "--skip=fmt,clippy",
        "--allow-skip",
        "trace",
    ]
    .iter()
    .map(ToString::to_string)
    .collect();
    let options = parse_options(&args, FIXTURE_STEPS).unwrap();
    assert_eq!(
        options,
        Options {
            json: true,
            fail_fast: true,
            only: None,
            skip: ["fmt", "clippy"].map(String::from).into(),
            allow_skip: ["trace"].map(String::from).into(),
        }
    );
}

// ---------------------------------------------------------------- output modes

#[test]
fn json_mode_captures_step_output_and_human_mode_inherits_it() {
    let run = run_ci("mode-json", &["--json", "--only", "fmt"], &[]);
    assert_eq!(run.runner.calls, [("fmt", OutputMode::Capture)]);
    let run = run_ci("mode-human", &["--only", "fmt"], &[]);
    assert_eq!(run.runner.calls, [("fmt", OutputMode::Inherit)]);
}

#[test]
fn human_report_lists_failures_skips_and_findings() {
    let finding = Finding {
        rule: "XTASK-R050".to_string(),
        severity: Severity::Error,
        location: "xtask/src/ci.rs:3".to_string(),
        message: "untracked TODO".to_string(),
        help: "name a task".to_string(),
    };
    let run = run_ci(
        "human-report",
        &[],
        &[
            (
                "trace",
                Outcome::Fail {
                    findings: vec![finding],
                    output_tail: None,
                },
            ),
            ("coverage", skipped()),
        ],
    );
    for expected in [
        "<== trace: FAIL (",
        "<== coverage: SKIPPED (",
        "      error XTASK-R050 xtask/src/ci.rs:3: untracked TODO\n        help: name a task\n",
        "      reason: cargo-llvm-cov not installed\n      help: cargo install cargo-llvm-cov\n",
        "result: FAILED (1 failed, 1 skipped without --allow-skip)\n",
    ] {
        assert!(
            run.out.contains(expected),
            "missing {expected:?} in:\n{}",
            run.out
        );
    }
}

// ---------------------------------------------------------------- R010, R012 real steps

/// Verifies: XTASK-R010
#[test]
fn ci_writes_only_under_target() {
    let root = TempRoot::new("ci-writes");
    let mut out = Vec::new();
    let result = execute(
        &["--json".to_string()],
        &mut out,
        root.path(),
        FIXTURE_STEPS,
        &mut FakeRunner::default(),
    );
    assert!(matches!(result, Ok(Exit::Success)));
    let mut stack = vec![root.path().to_path_buf()];
    let mut files = Vec::new();
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else {
                files.push(path.strip_prefix(root.path()).unwrap().to_path_buf());
            }
        }
    }
    assert_eq!(files, [PathBuf::from(SUMMARY_PATH)]);
}

/// Verifies: XTASK-R010
#[test]
fn real_steps_do_not_modify_the_repository() {
    // The cargo steps write only to `target/`: fmt checks instead of rewriting, and `--locked`
    // forbids Cargo.lock updates. Native checks only read: they are `CheckFn`s, which load the
    // repository model and return findings (INV-1, INV-3).
    for step in STEPS {
        match (step.name, &step.action) {
            ("fmt", Action::Cargo(invocation)) => assert!(invocation.args.contains(&"--check")),
            (_, Action::Cargo(invocation)) => {
                assert!(invocation.args.contains(&"--locked"), "step {}", step.name);
            }
            (_, Action::Check(_)) => {}
        }
    }
}

/// Verifies: XTASK-R012
#[test]
fn real_steps_run_the_exact_commands_of_the_spec() {
    type Command<'a> = (&'a str, Vec<&'a str>, Vec<(&'a str, &'a str)>);
    let commands: Vec<Command> = STEPS
        .iter()
        .filter_map(|step| match &step.action {
            Action::Cargo(invocation) => {
                Some((step.name, invocation.args.to_vec(), invocation.env.to_vec()))
            }
            Action::Check(_) => None,
        })
        .collect();
    assert_eq!(
        commands,
        [
            ("fmt", vec!["fmt", "--all", "--check"], vec![]),
            (
                "clippy",
                vec![
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--all-features",
                    "--locked",
                    "--",
                    "-D",
                    "warnings"
                ],
                vec![]
            ),
            (
                "test",
                vec!["test", "--workspace", "--all-features", "--locked"],
                vec![]
            ),
            (
                "doc",
                vec![
                    "doc",
                    "--workspace",
                    "--no-deps",
                    "--all-features",
                    "--locked"
                ],
                vec![("RUSTDOCFLAGS", "-D warnings")]
            ),
        ]
    );
}

/// Verifies: XTASK-R003
#[test]
fn native_check_steps_are_registered_by_name() {
    let native: Vec<&str> = STEPS
        .iter()
        .filter(|step| matches!(step.action, Action::Check(_)))
        .map(|step| step.name)
        .collect();
    assert_eq!(native, ["check-arch", "check-docs"]);
}

#[test]
fn human_mode_names_native_steps_and_prints_an_internal_error() {
    #[expect(
        clippy::unnecessary_wraps,
        reason = "must have the signature of check::CheckFn"
    )]
    fn no_findings(_: &std::path::Path) -> Result<Vec<Finding>, crate::repo::LoadError> {
        Ok(Vec::new())
    }
    const STEPS: &[Step] = &[Step {
        name: "check-x",
        action: Action::Check(no_findings),
    }];
    let root = TempRoot::new("ci-native-human");
    let mut runner = FakeRunner::with(&[(
        "check-x",
        Outcome::Fail {
            findings: Vec::new(),
            output_tail: Some("LIM-XTASK-0001: docs: could not be read".to_string()),
        },
    )]);
    let mut out = Vec::new();
    let result = execute(&[], &mut out, root.path(), STEPS, &mut runner);
    assert!(matches!(result, Ok(Exit::Failure)));
    let out = String::from_utf8(out).unwrap();
    assert!(
        out.starts_with("==> check-x: cargo xtask check-x\n"),
        "{out}"
    );
    assert!(
        out.contains("  check-x  FAIL (")
            && out.contains("\n      LIM-XTASK-0001: docs: could not be read\n"),
        "{out}"
    );
}
