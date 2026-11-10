//! The real step runner: spawns `cargo` for cargo steps (`docs/specs/xtask.md` §10: "real runner
//! spawns processes") and calls the check function for native check steps. Orchestration logic
//! lives in `ci` and is tested with a fake runner.

use std::ffi::OsString;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};

use crate::check::CheckFn;
use crate::ci::{Action, CargoInvocation, Outcome, OutputMode, Step, StepRunner};

/// Number of output lines kept in `output_tail` for a failed step.
pub(crate) const TAIL_LINES: usize = 50;

/// Help text when `cargo` itself cannot be started.
pub(crate) const CARGO_HELP: &str = "Install Rust through rustup (https://rustup.rs) and run from a shell \
                          where `cargo` works; `cargo xtask doctor` lists required tools.";

/// Runs cargo steps as child processes in the repository root, and native check steps in-process.
pub(crate) struct RealRunner {
    /// Working directory of every child process.
    root: PathBuf,
    /// The `cargo` program: `$CARGO` (set by cargo for `cargo run`), else `cargo` from `PATH`.
    cargo: OsString,
}

impl RealRunner {
    /// A runner whose children run in `root`, using the `cargo` that started xtask.
    pub(crate) fn new(root: PathBuf) -> Self {
        let cargo = std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
        Self::with_cargo(root, cargo)
    }

    /// A runner using an explicit `cargo` program (tests use a missing one).
    pub(crate) fn with_cargo(root: PathBuf, cargo: OsString) -> Self {
        Self { root, cargo }
    }

    fn command(&self, args: &[&str], env: &[(&str, &str)]) -> Command {
        let mut command = Command::new(&self.cargo);
        command
            .args(args)
            .current_dir(&self.root)
            .stdin(Stdio::null());
        for (key, value) in env {
            // Child process only; xtask's own environment is never changed (M0-T002 notes).
            command.env(key, value);
        }
        command
    }

    fn cannot_start(&self, error: &io::Error) -> Outcome {
        Outcome::Skipped {
            reason: format!(
                "could not start `{}`: {error}",
                self.cargo.to_string_lossy()
            ),
            help: CARGO_HELP.to_string(),
        }
    }
}

impl StepRunner for RealRunner {
    fn run(&mut self, step: &Step, mode: OutputMode) -> Outcome {
        match &step.action {
            Action::Cargo(invocation) => self.run_cargo(invocation, mode),
            Action::Check(check) => self.run_check(*check),
            Action::Coverage => crate::coverage::run_real_step(&self.root, &self.cargo, mode),
            Action::FeaturesOff => self.run_features_off(mode),
        }
    }
}

impl RealRunner {
    /// A native check: findings fail the step; an internal error fails it with the message as
    /// `output_tail` (spec "native check step").
    fn run_check(&self, check: CheckFn) -> Outcome {
        match check(&self.root) {
            Ok(findings) if findings.is_empty() => Outcome::Pass,
            Ok(findings) => Outcome::Fail {
                findings,
                output_tail: None,
            },
            Err(error) => Outcome::Fail {
                findings: Vec::new(),
                output_tail: Some(error.to_string()),
            },
        }
    }

    fn run_cargo(&self, invocation: &CargoInvocation, mode: OutputMode) -> Outcome {
        let CargoInvocation {
            args,
            env,
            requires,
        } = invocation;
        if let Some(tool) = requires {
            let probe = self
                .command(tool.probe, &[])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            match probe {
                Ok(status) if status.success() => {}
                Ok(_) => {
                    return Outcome::Skipped {
                        reason: tool.reason.to_string(),
                        help: tool.help.to_string(),
                    };
                }
                Err(error) => return self.cannot_start(&error),
            }
        }
        self.run_command(args, env, mode)
    }

    /// Runs `cargo <args>` and maps its exit status to pass or fail.
    fn run_command(&self, args: &[&str], env: &[(&str, &str)], mode: OutputMode) -> Outcome {
        let mut command = self.command(args, env);
        match mode {
            OutputMode::Inherit => match command.status() {
                Ok(status) if status.success() => Outcome::Pass,
                Ok(_) => Outcome::Fail {
                    findings: Vec::new(),
                    output_tail: None,
                },
                Err(error) => self.cannot_start(&error),
            },
            OutputMode::Capture => match run_captured(command) {
                Ok((status, _)) if status.success() => Outcome::Pass,
                Ok((_, output)) => Outcome::Fail {
                    findings: Vec::new(),
                    output_tail: Some(tail(&output, TAIL_LINES)),
                },
                Err(error) => self.cannot_start(&error),
            },
        }
    }

    /// The `test-features-off` step (XTASK-R014): lists the workspace packages with
    /// `cargo metadata`, then tests each one that has features with the features off.
    ///
    /// A `cargo` that cannot start skips the step (XTASK-R006). A `cargo metadata` failure or
    /// unreadable output fails it, with the error message as `output_tail` in both output modes
    /// (as for a native check step).
    fn run_features_off(&self, mode: OutputMode) -> Outcome {
        let probe = self
            .command(&["--version"], &[])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        if let Err(error) = probe {
            return self.cannot_start(&error);
        }
        let packages = crate::repo::metadata::run_cargo(&self.root, &self.cargo)
            .and_then(|json| crate::repo::metadata::parse(&json));
        match packages {
            Ok(packages) => {
                crate::features_off::run(&packages, |args| self.run_command(args, &[], mode))
            }
            Err(error) => Outcome::Fail {
                findings: Vec::new(),
                output_tail: Some(error.to_string()),
            },
        }
    }
}

/// Runs `command` with stdout and stderr sent to one pipe, so the captured output keeps the order
/// in which the child wrote it. Returns the exit status and all output.
fn run_captured(mut command: Command) -> io::Result<(ExitStatus, Vec<u8>)> {
    let (mut reader, writer) = io::pipe()?;
    command.stdout(writer.try_clone()?).stderr(writer);
    let mut child = command.spawn()?;
    // `command` holds our copies of the pipe's write end; drop them so `read_to_end` sees EOF
    // when the child (and any grandchildren holding the pipe) exit.
    drop(command);
    let mut output = Vec::new();
    let read = reader.read_to_end(&mut output);
    let status = child.wait()?;
    read?;
    Ok((status, output))
}

/// The last `lines` lines of `output`, decoded lossily as UTF-8, without a trailing newline.
fn tail(output: &[u8], lines: usize) -> String {
    let text = String::from_utf8_lossy(output);
    let all: Vec<&str> = text.lines().collect();
    all[all.len().saturating_sub(lines)..].join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ci::Tool;
    use crate::finding::Finding;
    use crate::repo::LoadError;

    fn step(args: &'static [&'static str], requires: Option<Tool>) -> Step {
        cargo_step(CargoInvocation {
            args,
            env: &[],
            requires,
        })
    }

    fn cargo_step(invocation: CargoInvocation) -> Step {
        Step {
            name: "probe-test",
            action: Action::Cargo(invocation),
        }
    }

    fn runner() -> RealRunner {
        RealRunner::new(crate::repo::root())
    }

    const MISSING_TOOL: Tool = Tool {
        probe: &["xtask-test-no-such-subcommand"],
        reason: "the test tool is not installed",
        help: "install the test tool",
    };

    #[test]
    fn tail_keeps_the_last_lines() {
        assert_eq!(tail(b"a\nb\nc\n", 2), "b\nc");
        assert_eq!(tail(b"a\nb", 5), "a\nb");
        assert_eq!(tail(b"", 5), "");
        assert_eq!(tail(b"x\r\ny\r\n", 1), "y");
    }

    #[test]
    fn successful_process_is_a_pass_in_both_modes() {
        for mode in [OutputMode::Capture, OutputMode::Inherit] {
            assert_eq!(
                runner().run(&step(&["--version"], None), mode),
                Outcome::Pass
            );
        }
    }

    #[test]
    fn failing_process_is_a_fail_with_its_output_tail_when_captured() {
        let outcome = runner().run(
            &step(&["xtask-test-no-such-subcommand"], None),
            OutputMode::Capture,
        );
        let Outcome::Fail {
            findings,
            output_tail: Some(output_tail),
        } = outcome
        else {
            panic!("expected a failure with output, got {outcome:?}");
        };
        assert_eq!(findings, Vec::new());
        assert!(
            output_tail.contains("xtask-test-no-such-subcommand"),
            "output_tail: {output_tail}"
        );
    }

    #[test]
    fn child_environment_is_set_for_the_child_only() {
        // `cargo --version` ignores the variable; this checks it never leaks into xtask itself.
        let step = cargo_step(CargoInvocation {
            args: &["--version"],
            env: &[("XTASK_TEST_CHILD_ONLY", "1")],
            requires: None,
        });
        assert_eq!(runner().run(&step, OutputMode::Capture), Outcome::Pass);
        assert_eq!(std::env::var_os("XTASK_TEST_CHILD_ONLY"), None);
    }

    /// Verifies: XTASK-R006
    #[test]
    fn failing_tool_probe_skips_the_step_with_reason_and_help() {
        let outcome = runner().run(
            &step(&["--version"], Some(MISSING_TOOL)),
            OutputMode::Capture,
        );
        assert_eq!(
            outcome,
            Outcome::Skipped {
                reason: "the test tool is not installed".to_string(),
                help: "install the test tool".to_string(),
            }
        );
    }

    /// Verifies: XTASK-R006
    #[test]
    fn missing_cargo_skips_the_step_with_reason_and_help() {
        let mut runner = RealRunner::with_cargo(
            crate::repo::root(),
            OsString::from("xtask-test-no-such-program"),
        );
        for requires in [None, Some(MISSING_TOOL)] {
            let outcome = runner.run(&step(&["--version"], requires), OutputMode::Capture);
            let Outcome::Skipped { reason, help } = outcome else {
                panic!("expected a skip, got {outcome:?}");
            };
            assert!(
                reason.starts_with("could not start `xtask-test-no-such-program`"),
                "reason: {reason}"
            );
            assert_eq!(help, CARGO_HELP);
        }
    }

    fn check_step(check: CheckFn) -> Step {
        Step {
            name: "check-test",
            action: Action::Check(check),
        }
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "must have the signature of check::CheckFn"
    )]
    fn finds_one(root: &std::path::Path) -> Result<Vec<Finding>, LoadError> {
        Ok(vec![Finding::error(
            "XTASK-R024",
            root.join("a.md").to_string_lossy(),
            "m",
            "h",
        )])
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "must have the signature of check::CheckFn"
    )]
    fn finds_none(_: &std::path::Path) -> Result<Vec<Finding>, LoadError> {
        Ok(Vec::new())
    }

    fn cannot_load(_: &std::path::Path) -> Result<Vec<Finding>, LoadError> {
        Err(LoadError::read("docs", &io::Error::other("denied")))
    }

    #[test]
    fn native_checks_run_on_the_root_and_map_findings_and_errors() {
        let mut runner = RealRunner::new(PathBuf::from("/repo"));
        for mode in [OutputMode::Capture, OutputMode::Inherit] {
            assert_eq!(runner.run(&check_step(finds_none), mode), Outcome::Pass);
            assert_eq!(
                runner.run(&check_step(finds_one), mode),
                Outcome::Fail {
                    findings: vec![Finding::error(
                        "XTASK-R024",
                        // Built like `finds_one` builds it: `\` separator on Windows.
                        PathBuf::from("/repo").join("a.md").to_string_lossy(),
                        "m",
                        "h"
                    )],
                    output_tail: None,
                }
            );
            let Outcome::Fail {
                findings,
                output_tail: Some(tail),
            } = runner.run(&check_step(cannot_load), mode)
            else {
                panic!("expected a failure with output_tail");
            };
            assert_eq!(findings, []);
            assert!(
                tail.starts_with("LIM-XTASK-0001: docs: could not be read"),
                "{tail}"
            );
        }
    }
}
