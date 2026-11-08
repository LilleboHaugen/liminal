//! The `ci` command: runs the quality steps in order, writes the JSON summary and sets the exit code
//! (`docs/specs/xtask.md` §5, XTASK-R003 … R008, R010, R012).
//!
//! Orchestration ([`run_steps`]) is separated from process spawning through [`StepRunner`], so it is
//! tested with a fake runner; [`crate::process::RealRunner`] is the real one.

use std::collections::BTreeSet;
use std::fmt::{self, Write as _};
use std::io::Write;
use std::path::Path;
use std::time::Instant;

use serde::Serialize;

use crate::check::{self, CheckFn};
use crate::cli::{CommandError, Exit};
use crate::finding::Finding;
use crate::process::RealRunner;
use crate::repo;

/// The summary file, relative to the repository root (XTASK-R005).
pub(crate) const SUMMARY_PATH: &str = "target/xtask/ci-summary.json";

/// Version of the summary schema (spec §5).
const SUMMARY_VERSION: u32 = 1;

/// Synopsis shown with `ci` usage errors.
const SYNOPSIS: &str = "usage: cargo xtask ci [--json] [--fail-fast] [--only <step>[,<step>…]] \
                        [--skip <step>[,<step>…]] [--allow-skip <step>[,<step>…]]";

/// Every step `ci` runs, in run order (XTASK-R003).
///
/// The final order (spec §5) is `fmt, clippy, test, doc, check-arch, check-docs, trace, hygiene,
/// status-check, coverage, deny`. Each task adds its step at its place in that order; a step that
/// does not exist yet is absent, never stubbed as a pass.
pub(crate) const STEPS: &[Step] = &[
    Step {
        name: "fmt",
        action: Action::Cargo(CargoInvocation {
            args: &["fmt", "--all", "--check"],
            env: &[],
            requires: Some(Tool {
                probe: &["fmt", "--version"],
                reason: "rustfmt is not installed for the pinned toolchain",
                help: "rustup component add rustfmt",
            }),
        }),
    },
    Step {
        name: "clippy",
        action: Action::Cargo(CargoInvocation {
            args: &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--locked",
                "--",
                "-D",
                "warnings",
            ],
            env: &[],
            requires: Some(Tool {
                probe: &["clippy", "--version"],
                reason: "clippy is not installed for the pinned toolchain",
                help: "rustup component add clippy",
            }),
        }),
    },
    Step {
        name: "test",
        action: Action::Cargo(CargoInvocation {
            args: &["test", "--workspace", "--all-features", "--locked"],
            env: &[],
            requires: None,
        }),
    },
    Step {
        name: "doc",
        action: Action::Cargo(CargoInvocation {
            args: &[
                "doc",
                "--workspace",
                "--no-deps",
                "--all-features",
                "--locked",
            ],
            env: &[("RUSTDOCFLAGS", "-D warnings")],
            requires: None,
        }),
    },
    Step {
        name: crate::check_arch::NAME,
        action: Action::Check(crate::check_arch::run_check),
    },
    Step {
        name: crate::check_docs::NAME,
        action: Action::Check(crate::check_docs::run_check),
    },
    Step {
        name: crate::trace::NAME,
        action: Action::Check(crate::trace::run_check),
    },
    Step {
        name: crate::hygiene::NAME,
        action: Action::Check(crate::hygiene::run_check),
    },
];

/// One step of `ci`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Step {
    /// Name used in output and by `--only`, `--skip` and `--allow-skip`.
    pub name: &'static str,
    /// What the step does.
    pub action: Action,
}

/// How a step runs.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Action {
    /// A `cargo` child process.
    Cargo(CargoInvocation),
    /// A native check, run inside xtask on the repository root (spec "native check step").
    Check(CheckFn),
}

/// A `cargo` command line (the program itself is chosen by the runner).
#[derive(Debug, Clone, Copy)]
pub(crate) struct CargoInvocation {
    /// Arguments after `cargo`.
    pub args: &'static [&'static str],
    /// Environment variables set for the child process only.
    pub env: &'static [(&'static str, &'static str)],
    /// A tool that must be present; if its probe fails, the step is skipped (XTASK-R006).
    pub requires: Option<Tool>,
}

/// A tool a step needs, and how to tell whether it is installed.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Tool {
    /// Arguments after `cargo` of a command that succeeds only if the tool is installed.
    pub probe: &'static [&'static str],
    /// The `reason` reported when the tool is missing.
    pub reason: &'static str,
    /// The `help` reported when the tool is missing.
    pub help: &'static str,
}

/// How a step's process output is handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutputMode {
    /// The child writes to xtask's stdout and stderr (human mode).
    Inherit,
    /// The child's output is captured; failed steps report its tail (`--json` mode).
    Capture,
}

/// The result of running one step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// The step ran and passed.
    Pass,
    /// The step ran and failed.
    Fail {
        /// Rule violations found by the step (empty for process steps).
        findings: Vec<Finding>,
        /// The last lines of the step's output, when it was captured.
        output_tail: Option<String>,
    },
    /// The step could not run (XTASK-R006). Never counts as a pass.
    Skipped {
        /// Why the step could not run.
        reason: String,
        /// How to make it run.
        help: String,
    },
}

/// Executes steps. The real implementation spawns processes; tests use a fake.
pub(crate) trait StepRunner {
    /// Runs `step` and reports its outcome. Must not panic on failure of the step itself.
    fn run(&mut self, step: &Step, mode: OutputMode) -> Outcome;
}

/// Parsed `ci` options.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Options {
    /// `--json`: print the summary to stdout and capture step output.
    pub json: bool,
    /// `--fail-fast`: stop after the first step that makes `ok` false.
    pub fail_fast: bool,
    /// `--only`: if present, only these steps run.
    pub only: Option<BTreeSet<String>>,
    /// `--skip`: these steps do not run.
    pub skip: BTreeSet<String>,
    /// `--allow-skip`: a `skipped` outcome of these steps does not make `ok` false.
    pub allow_skip: BTreeSet<String>,
}

/// Parses `ci` arguments. Step names are checked against `steps`.
pub(crate) fn parse_options(args: &[String], steps: &[Step]) -> Result<Options, CommandError> {
    let mut options = Options::default();
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        let (flag, inline_value) = match arg.split_once('=') {
            Some((flag, value)) if flag.starts_with("--") => (flag, Some(value)),
            _ => (arg.as_str(), None),
        };
        let is_list_option = matches!(flag, "--only" | "--skip" | "--allow-skip");
        if !is_list_option && inline_value.is_some() {
            return Err(usage(format!("option `{flag}` does not take a value")));
        }
        match flag {
            "--json" => options.json = true,
            "--fail-fast" => options.fail_fast = true,
            "--only" | "--skip" | "--allow-skip" => {
                let value = match inline_value {
                    Some(value) => value,
                    None => args
                        .next()
                        .ok_or_else(|| usage(format!("option `{flag}` needs a step list")))?,
                };
                let names = parse_step_list(flag, value, steps)?;
                let target = match flag {
                    "--only" => options.only.get_or_insert_with(BTreeSet::new),
                    "--skip" => &mut options.skip,
                    _ => &mut options.allow_skip,
                };
                target.extend(names);
            }
            _ if arg.starts_with('-') => {
                return Err(usage(format!("unknown option `{arg}` for `ci`")));
            }
            _ => return Err(usage(format!("unexpected argument `{arg}` for `ci`"))),
        }
    }
    if selected(steps, &options).next().is_none() {
        return Err(usage(
            "no steps selected; `--only` and `--skip` exclude every step".to_string(),
        ));
    }
    Ok(options)
}

/// Splits a comma-separated step list and checks every name (XTASK-R008).
fn parse_step_list(flag: &str, value: &str, steps: &[Step]) -> Result<Vec<String>, CommandError> {
    value
        .split(',')
        .map(|name| {
            if steps.iter().any(|step| step.name == name) {
                Ok(name.to_string())
            } else {
                let known: Vec<&str> = steps.iter().map(|step| step.name).collect();
                Err(usage(format!(
                    "unknown step `{name}` in `{flag}`; steps: {}",
                    known.join(", ")
                )))
            }
        })
        .collect()
}

fn usage(message: impl fmt::Display) -> CommandError {
    CommandError::Usage(format!("{message}\n{SYNOPSIS}"))
}

/// The steps that `options` selects, in table order (XTASK-R003, R008).
fn selected<'a>(steps: &'a [Step], options: &'a Options) -> impl Iterator<Item = &'a Step> {
    steps.iter().filter(|step| {
        options
            .only
            .as_ref()
            .is_none_or(|only| only.contains(step.name))
            && !options.skip.contains(step.name)
    })
}

/// The JSON summary (spec §5).
#[derive(Debug, Serialize)]
pub(crate) struct Summary {
    /// Schema version.
    pub version: u32,
    /// True if no step failed and no step was skipped without `--allow-skip`.
    pub ok: bool,
    /// The steps that ran, in run order.
    pub steps: Vec<StepReport>,
}

/// One step in the summary.
#[derive(Debug, Serialize)]
pub(crate) struct StepReport {
    /// Step name.
    pub name: &'static str,
    /// `pass`, `fail` or `skipped`.
    pub status: Status,
    /// Wall-clock time of the step, in milliseconds.
    pub duration_ms: u64,
    /// Why the step was skipped (only for `skipped`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// How to make the step run (only for `skipped`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    /// Rule violations (only for `fail`, omitted when empty).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<Finding>,
    /// The last lines of captured output (only for `fail` in `--json` mode).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tail: Option<String>,
    /// Whether a skip was allowed by `--allow-skip`. Not serialized: `ok` already reflects it.
    #[serde(skip)]
    pub skip_allowed: bool,
}

/// Step status in the summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Status {
    /// The step passed.
    Pass,
    /// The step failed.
    Fail,
    /// The step could not run.
    Skipped,
}

impl StepReport {
    /// Whether this step keeps `ok` true.
    fn is_ok(&self) -> bool {
        match self.status {
            Status::Pass => true,
            Status::Fail => false,
            Status::Skipped => self.skip_allowed,
        }
    }
}

/// Runs the selected steps in order and builds the summary.
///
/// Every selected step runs even after a failure (XTASK-R004), unless `--fail-fast` is given; then
/// the run stops after the first step that makes `ok` false and later steps are absent from the
/// summary. In human mode, progress lines are written to `out`.
pub(crate) fn run_steps(
    steps: &[Step],
    options: &Options,
    runner: &mut dyn StepRunner,
    out: &mut dyn Write,
) -> Result<Summary, CommandError> {
    let mode = if options.json {
        OutputMode::Capture
    } else {
        OutputMode::Inherit
    };
    let mut reports = Vec::new();
    for step in selected(steps, options) {
        if !options.json {
            let command = match &step.action {
                Action::Cargo(invocation) => invocation.args.join(" "),
                Action::Check(_) => format!("xtask {}", step.name),
            };
            writeln!(out, "==> {}: cargo {command}", step.name)?;
            // The child writes to the same stdout; our line must come first.
            out.flush()?;
        }
        let start = Instant::now();
        let outcome = runner.run(step, mode);
        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        let report = report(step.name, outcome, duration_ms, options);
        if !options.json {
            writeln!(out, "<== {}: {}", step.name, status_text(&report))?;
        }
        let stop = options.fail_fast && !report.is_ok();
        reports.push(report);
        if stop {
            break;
        }
    }
    Ok(Summary {
        version: SUMMARY_VERSION,
        ok: reports.iter().all(StepReport::is_ok),
        steps: reports,
    })
}

fn report(name: &'static str, outcome: Outcome, duration_ms: u64, options: &Options) -> StepReport {
    let mut report = StepReport {
        name,
        status: Status::Pass,
        duration_ms,
        reason: None,
        help: None,
        findings: Vec::new(),
        output_tail: None,
        skip_allowed: false,
    };
    match outcome {
        Outcome::Pass => {}
        Outcome::Fail {
            findings,
            output_tail,
        } => {
            report.status = Status::Fail;
            report.findings = findings;
            report.output_tail = output_tail;
        }
        Outcome::Skipped { reason, help } => {
            report.status = Status::Skipped;
            report.reason = Some(reason);
            report.help = Some(help);
            report.skip_allowed = options.allow_skip.contains(name);
        }
    }
    report
}

fn status_text(report: &StepReport) -> String {
    let status = match report.status {
        Status::Pass => "pass",
        Status::Fail => "FAIL",
        Status::Skipped if report.skip_allowed => "skipped (allowed)",
        Status::Skipped => "SKIPPED",
    };
    format!("{status} ({} ms)", report.duration_ms)
}

/// The summary as pretty-printed JSON with a trailing newline. Field order is fixed by the struct
/// definitions, so the output is deterministic apart from `duration_ms`.
pub(crate) fn summary_json(summary: &Summary) -> String {
    let mut json = serde_json::to_string_pretty(summary)
        .expect("invariant: the summary contains only strings, numbers, booleans and lists");
    json.push('\n');
    json
}

/// The human-readable report printed after all steps in human mode.
fn human_report(summary: &Summary) -> String {
    let width = summary
        .steps
        .iter()
        .map(|step| step.name.len())
        .max()
        .unwrap_or(0);
    let mut text = String::from("\nci summary:\n");
    for step in &summary.steps {
        let _ = writeln!(text, "  {:width$}  {}", step.name, status_text(step));
        if let (Some(reason), Some(help)) = (&step.reason, &step.help) {
            let _ = writeln!(text, "      reason: {reason}\n      help: {help}");
        }
        for line in check::finding_lines(&step.findings).lines() {
            let _ = writeln!(text, "      {line}");
        }
        if let Some(tail) = &step.output_tail {
            for line in tail.lines() {
                let _ = writeln!(text, "      {line}");
            }
        }
    }
    let failed = summary
        .steps
        .iter()
        .filter(|step| step.status == Status::Fail)
        .count();
    let skipped = summary
        .steps
        .iter()
        .filter(|step| step.status == Status::Skipped && !step.skip_allowed)
        .count();
    if summary.ok {
        text.push_str("result: ok\n");
    } else {
        let _ = writeln!(
            text,
            "result: FAILED ({failed} failed, {skipped} skipped without --allow-skip)"
        );
    }
    let _ = writeln!(text, "summary written to {SUMMARY_PATH}");
    text
}

/// Runs `ci` with `args` against the steps `steps`, using `runner`, writing the summary file under
/// `root` (XTASK-R005) and output to `out`.
pub(crate) fn execute(
    args: &[String],
    out: &mut dyn Write,
    root: &Path,
    steps: &[Step],
    runner: &mut dyn StepRunner,
) -> Result<Exit, CommandError> {
    let options = parse_options(args, steps)?;
    let summary = run_steps(steps, &options, runner, out)?;
    let json = summary_json(&summary);
    let path = root.join(SUMMARY_PATH);
    let written = path
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|()| std::fs::write(&path, &json));
    if let Err(error) = written {
        return Err(CommandError::File { path, error });
    }
    if options.json {
        out.write_all(json.as_bytes())?;
    } else {
        out.write_all(human_report(&summary).as_bytes())?;
    }
    Ok(if summary.ok {
        Exit::Success
    } else {
        Exit::Failure
    })
}

/// The `ci` command as registered in the command table.
pub(crate) fn command(args: &[String], out: &mut dyn Write) -> Result<Exit, CommandError> {
    let root = repo::root();
    let mut runner = RealRunner::new(root.clone());
    execute(args, out, &root, STEPS, &mut runner)
}

#[cfg(test)]
mod tests;
