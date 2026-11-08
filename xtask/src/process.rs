//! The real step runner: spawns `cargo` for each step (`docs/specs/xtask.md` §10: "real runner
//! spawns processes"). Orchestration logic lives in `ci` and is tested with a fake runner.

use std::ffi::OsString;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};

use crate::ci::{CargoInvocation, Outcome, OutputMode, Step, StepRunner};

/// Number of output lines kept in `output_tail` for a failed step.
pub(crate) const TAIL_LINES: usize = 50;

/// Help text when `cargo` itself cannot be started.
const CARGO_HELP: &str = "Install Rust through rustup (https://rustup.rs) and run from a shell \
                          where `cargo` works; `cargo xtask doctor` lists required tools.";

/// Runs steps as `cargo` child processes in the repository root.
pub(crate) struct ProcessRunner {
    /// Working directory of every child process.
    root: PathBuf,
    /// The `cargo` program: `$CARGO` (set by cargo for `cargo run`), else `cargo` from `PATH`.
    cargo: OsString,
}

impl ProcessRunner {
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

impl StepRunner for ProcessRunner {
    fn run(&mut self, step: &Step, mode: OutputMode) -> Outcome {
        let CargoInvocation {
            args,
            env,
            requires,
        } = &step.invocation;
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

    fn step(args: &'static [&'static str], requires: Option<Tool>) -> Step {
        Step {
            name: "probe-test",
            invocation: CargoInvocation {
                args,
                env: &[],
                requires,
            },
        }
    }

    fn runner() -> ProcessRunner {
        ProcessRunner::new(crate::repo::root())
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
        let mut step = step(&["--version"], None);
        step.invocation.env = &[("XTASK_TEST_CHILD_ONLY", "1")];
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
        let mut runner = ProcessRunner::with_cargo(
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
}
