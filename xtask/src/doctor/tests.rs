//! Tests of `doctor` (`docs/specs/xtask.md` §10: fake probe runner; the real probes run on the
//! developer machine).

use std::collections::BTreeMap;

use super::*;

/// A prober that answers from a table and records every call. Unlisted probes are `Missing`.
#[derive(Default)]
struct FakeProber {
    answers: BTreeMap<String, Probe>,
    calls: Vec<(Program, Vec<String>)>,
}

impl FakeProber {
    /// Every tool present, with the stdout `<tool> 1.0.0\n`.
    fn all_present() -> Self {
        let mut prober = Self::default();
        for tool in TOOLS {
            prober.set(tool.name, Probe::Present(format!("{} 1.0.0\n", tool.name)));
        }
        prober
    }

    fn set(&mut self, tool: &str, probe: Probe) {
        self.answers.insert(tool.to_string(), probe);
    }
}

impl Prober for FakeProber {
    fn probe(&mut self, program: Program, args: &[&str]) -> Probe {
        self.calls
            .push((program, args.iter().map(ToString::to_string).collect()));
        // Identify the tool by its probe, as the real prober only sees the command line.
        let tool = TOOLS
            .iter()
            .find(|tool| tool.program == program && tool.args == args)
            .map_or("", |tool| tool.name);
        self.answers.get(tool).cloned().unwrap_or(Probe::Missing)
    }
}

fn run(args: &[&str], prober: &mut FakeProber) -> (Result<Exit, CommandError>, String) {
    let args: Vec<String> = args.iter().map(ToString::to_string).collect();
    let mut out = Vec::new();
    let result = execute(&args, &mut out, prober);
    (result, String::from_utf8(out).unwrap())
}

/// Verifies: XTASK-R080
#[test]
fn every_tool_of_the_spec_is_probed_with_its_exact_command() {
    let mut prober = FakeProber::all_present();
    let _ = run(&[], &mut prober);
    let calls: Vec<(Program, Vec<&str>)> = prober
        .calls
        .iter()
        .map(|(program, args)| (*program, args.iter().map(String::as_str).collect()))
        .collect();
    assert_eq!(
        calls,
        [
            (Program::Rustc, vec!["--version"]),
            (Program::Cargo, vec!["--version"]),
            (Program::Cargo, vec!["fmt", "--version"]),
            (Program::Cargo, vec!["clippy", "--version"]),
            (Program::Cargo, vec!["llvm-cov", "--version"]),
            (Program::Cargo, vec!["deny", "--version"]),
            (
                Program::Rustup,
                vec!["run", "nightly", "cargo", "miri", "--version"]
            ),
        ]
    );
    let names: Vec<&str> = TOOLS.iter().map(|tool| tool.name).collect();
    assert_eq!(
        names,
        [
            "rustc",
            "cargo",
            "rustfmt",
            "clippy",
            "cargo-llvm-cov",
            "cargo-deny",
            "miri"
        ]
    );
}

/// Verifies: XTASK-R080
#[test]
fn all_present_lists_each_version_and_passes() {
    let mut prober = FakeProber::all_present();
    let (result, out) = run(&[], &mut prober);
    assert!(matches!(result, Ok(Exit::Success)));
    assert_eq!(
        out,
        "  rustc           rustc 1.0.0\n\
         \x20 cargo           cargo 1.0.0\n\
         \x20 rustfmt         rustfmt 1.0.0\n\
         \x20 clippy          clippy 1.0.0\n\
         \x20 cargo-llvm-cov  cargo-llvm-cov 1.0.0\n\
         \x20 cargo-deny      cargo-deny 1.0.0\n\
         \x20 miri            miri 1.0.0\n\
         doctor: ok\n"
    );
}

/// Verifies: XTASK-R080
#[test]
fn a_tool_a_ci_step_needs_is_an_error_with_the_install_hint() {
    let mut prober = FakeProber::all_present();
    prober.set("cargo-deny", Probe::Missing);
    let (result, out) = run(&[], &mut prober);
    assert!(matches!(result, Ok(Exit::Failure)), "{out}");
    assert!(out.contains("  cargo-deny      missing\n"), "{out}");
    assert!(
        out.contains(
            "error XTASK-R080 cargo-deny: cargo-deny is not installed; `ci` skips `deny` without it\n"
        ),
        "{out}"
    );
    assert!(
        out.contains(&format!(
            "  help: {CARGO_DENY_HELP}; ask the project owner before installing (ADR-0022)\n"
        )),
        "{out}"
    );
    assert!(out.ends_with("doctor: 1 finding\n"), "{out}");
}

/// Verifies: XTASK-R080
#[test]
fn missing_rustc_names_every_cargo_step() {
    let mut prober = FakeProber::all_present();
    prober.set("rustc", Probe::Missing);
    let (result, out) = run(&[], &mut prober);
    assert!(matches!(result, Ok(Exit::Failure)));
    assert!(
        out.contains(
            "rustc is not installed; `ci` skips `fmt`, `clippy`, `test`, `doc`, `coverage`, \
             `deny` without it"
        ),
        "{out}"
    );
}

/// Verifies: XTASK-R080
#[test]
fn a_tool_no_ci_step_needs_yet_is_a_warning_and_passes() {
    let mut prober = FakeProber::all_present();
    prober.set("miri", Probe::Missing);
    let (result, out) = run(&[], &mut prober);
    assert!(matches!(result, Ok(Exit::Success)), "{out}");
    assert!(
        out.contains(
            "warning XTASK-R080 miri: miri is not installed (no `ci` step needs it yet: Miri \
             tests of `unsafe` crates from M1 (ADR-0018))\n  help: rustup toolchain install \
             nightly --component miri; ask the project owner before installing (ADR-0022)\n"
        ),
        "{out}"
    );
    assert!(out.ends_with("doctor: 1 finding\n"), "{out}");
}

/// Verifies: XTASK-R080
#[test]
fn version_is_the_first_non_empty_stdout_line_trimmed() {
    let mut prober = FakeProber::all_present();
    prober.set(
        "cargo",
        Probe::Present("\n  cargo 9.9 (abc)  \nmore\n".to_string()),
    );
    prober.set("clippy", Probe::Present(String::new()));
    let (tools, findings) = diagnose(TOOLS, &mut prober);
    assert_eq!(findings, Vec::new());
    assert_eq!(tools[1].version.as_deref(), Some("cargo 9.9 (abc)"));
    assert!(tools[3].present);
    assert_eq!(tools[3].version, None);
    let (_, out) = run(&[], &mut prober);
    assert!(
        out.contains("  clippy          present (no version output)\n"),
        "{out}"
    );
}

/// Verifies: XTASK-R080, XTASK-R009
#[test]
fn json_lists_every_tool_and_the_findings() {
    let mut prober = FakeProber::all_present();
    prober.set("cargo-llvm-cov", Probe::Missing);
    prober.set("miri", Probe::Missing);
    let (result, out) = run(&["--json"], &mut prober);
    assert!(matches!(result, Ok(Exit::Failure)));
    let report: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(report["version"], 1);
    assert_eq!(report["ok"], false);
    let tools = report["tools"].as_array().unwrap();
    assert_eq!(tools.len(), TOOLS.len());
    assert_eq!(
        tools[0],
        serde_json::json!({
            "tool": "rustc", "present": true, "version": "rustc 1.0.0",
            "needed_by": ["fmt", "clippy", "test", "doc", "coverage", "deny"], "help": null
        })
    );
    assert_eq!(
        tools[4],
        serde_json::json!({
            "tool": "cargo-llvm-cov", "present": false, "version": null,
            "needed_by": ["coverage"], "help": crate::repo::coverage::MISSING_HELP
        })
    );
    assert_eq!(tools[6]["needed_by"], serde_json::json!([]));
    let findings = report["findings"].as_array().unwrap();
    let summary: Vec<(&str, &str, &str)> = findings
        .iter()
        .map(|f| {
            (
                f["severity"].as_str().unwrap(),
                f["rule"].as_str().unwrap(),
                f["location"].as_str().unwrap(),
            )
        })
        .collect();
    // Sorted by location.
    assert_eq!(
        summary,
        [
            ("error", "XTASK-R080", "cargo-llvm-cov"),
            ("warning", "XTASK-R080", "miri")
        ]
    );
}

/// Verifies: XTASK-R080
#[test]
fn json_ok_is_true_with_only_warnings() {
    let mut prober = FakeProber::all_present();
    prober.set("miri", Probe::Missing);
    let (result, out) = run(&["--json"], &mut prober);
    assert!(matches!(result, Ok(Exit::Success)));
    let report: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(report["ok"], true);
}

#[test]
fn unknown_arguments_are_usage_errors() {
    for args in [&["--check"][..], &["extra"][..]] {
        let mut prober = FakeProber::all_present();
        assert!(matches!(
            run(args, &mut prober).0,
            Err(CommandError::Usage(_))
        ));
        assert!(prober.calls.is_empty(), "probed before parsing arguments");
    }
}

/// Verifies: XTASK-R080, XTASK-R010
#[test]
fn real_probes_run_in_the_root_and_never_auto_install() {
    let root = repo::root();
    let prober = RealProber::new(root.clone());
    for program in [Program::Rustc, Program::Cargo, Program::Rustup] {
        let command = prober.command(program, &["--version"]);
        assert_eq!(command.get_current_dir(), Some(root.as_path()));
        let envs: Vec<_> = command.get_envs().collect();
        assert_eq!(
            envs,
            [(
                std::ffi::OsStr::new("RUSTUP_AUTO_INSTALL"),
                Some(std::ffi::OsStr::new("0"))
            )]
        );
    }
    assert_eq!(
        prober.command(Program::Rustc, &[]).get_program(),
        std::ffi::OsStr::new("rustc")
    );
    assert_eq!(
        prober.command(Program::Rustup, &[]).get_program(),
        std::ffi::OsStr::new("rustup")
    );
}

#[test]
fn real_prober_finds_cargo_and_reports_a_failing_probe_as_missing() {
    let mut prober = RealProber::new(repo::root());
    match prober.probe(Program::Cargo, &["--version"]) {
        Probe::Present(stdout) => assert!(stdout.starts_with("cargo "), "{stdout}"),
        Probe::Missing => panic!("cargo runs these tests, so it is present"),
    }
    assert_eq!(
        prober.probe(Program::Cargo, &["xtask-test-no-such-subcommand"]),
        Probe::Missing
    );
    let mut absent = RealProber {
        root: repo::root(),
        cargo: OsString::from("xtask-test-no-such-program"),
    };
    assert_eq!(absent.probe(Program::Cargo, &["--version"]), Probe::Missing);
}

/// Verifies: XTASK-R080, XTASK-R009
#[test]
fn the_real_command_reports_every_tool_as_json() {
    let mut out = Vec::new();
    // The exit depends on the machine (a missing tool is an error); the report shape does not.
    let exit = command(&["--json".to_string()], &mut out).unwrap();
    let report: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(report["ok"], exit == Exit::Success);
    let tools = report["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t["tool"].as_str().unwrap()).collect();
    let expected: Vec<&str> = TOOLS.iter().map(|tool| tool.name).collect();
    assert_eq!(names, expected);
    // cargo runs this test, so it is present.
    assert_eq!(tools[1]["present"], true);
    assert!(
        tools[1]["version"].as_str().unwrap().starts_with("cargo "),
        "{}",
        tools[1]
    );
}

/// The extra test-name filter that [`stdin_reader_child`] requires before it reads stdin, so a
/// normal `--include-ignored` run (whose stdin may be a terminal) never blocks on it.
const CHILD_FILTER: &str = "xtask-doctor-probe-child";

/// The line [`stdin_reader_child`] writes to stderr; a probe's stdout must not contain it.
const STDERR_MARKER: &str = "xtask-doctor-stderr-marker";

/// Child process of [`a_probe_that_reads_stdin_sees_it_closed`]: reads stdin to its end, writes
/// [`STDERR_MARKER`] to stderr and the number of bytes read to stdout. It does nothing unless
/// its arguments contain [`CHILD_FILTER`].
#[test]
#[ignore = "M1-T008: runs only as the probe of a_probe_that_reads_stdin_sees_it_closed"]
fn stdin_reader_child() {
    if !std::env::args().any(|arg| arg == CHILD_FILTER) {
        return;
    }
    let mut input = Vec::new();
    std::io::Read::read_to_end(&mut std::io::stdin(), &mut input).unwrap();
    eprintln!("{STDERR_MARKER}");
    println!("xtask-stdin-bytes={}", input.len());
}

/// Probes run with stdin closed (spec "`doctor` and `deny` details"): a probe of a program that
/// reads stdin to its end returns at once, having read nothing. Its stderr is not part of the
/// probe's stdout (the version comes from stdout only).
///
/// The probe program is this test binary, run as `cargo` (the [`Program`] whose path is
/// configurable), with [`stdin_reader_child`] as the only test.
///
/// Verifies: XTASK-R080
#[test]
fn a_probe_that_reads_stdin_sees_it_closed() {
    let mut prober = RealProber {
        root: repo::root(),
        cargo: std::env::current_exe().unwrap().into_os_string(),
    };
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let probe = prober.probe(
            Program::Cargo,
            &[
                "doctor::tests::stdin_reader_child",
                CHILD_FILTER,
                "--exact",
                "--ignored",
                "--nocapture",
                "--test-threads=1",
            ],
        );
        let _ = sender.send(probe);
    });
    let probe = receiver
        .recv_timeout(std::time::Duration::from_secs(120))
        .expect("the probe returned within 120 s (it blocks if stdin is not closed)");
    let Probe::Present(stdout) = probe else {
        panic!("the child test failed: {probe:?}");
    };
    assert!(stdout.contains("xtask-stdin-bytes=0"), "{stdout}");
    assert!(stdout.contains("1 passed"), "{stdout}");
    assert!(!stdout.contains(STDERR_MARKER), "{stdout}");
}
