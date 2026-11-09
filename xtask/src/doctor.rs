//! The `doctor` command: which external tools the checks need are present, their versions, and
//! how to install missing ones (`docs/specs/xtask.md` XTASK-R080 and "`doctor` and `deny`
//! details").
//!
//! [`diagnose`] is a pure function of the probe results; probes go through [`Prober`], so tests
//! use a fake and only [`RealProber`] spawns processes. Probes never install anything
//! (`RUSTUP_AUTO_INSTALL=0`, ADR-0022), and `doctor` writes no file (XTASK-R010).

use std::ffi::OsString;
use std::fmt::Write as _;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde::Serialize;

use crate::check;
use crate::cli::{self, CommandError, Exit};
use crate::finding::{Finding, Severity};
use crate::repo;

/// Command name.
pub(crate) const NAME: &str = "doctor";

/// Summary schema version of `doctor --json`.
const REPORT_VERSION: u32 = 1;

/// Install hint for `rustfmt` (also the `help` of a skipped `fmt` step).
pub(crate) const RUSTFMT_HELP: &str = "rustup component add rustfmt";

/// Install hint for `clippy` (also the `help` of a skipped `clippy` step).
pub(crate) const CLIPPY_HELP: &str = "rustup component add clippy";

/// Install hint for `cargo-deny` (also the `help` of a skipped `deny` step, XTASK-R081).
pub(crate) const CARGO_DENY_HELP: &str = "cargo install cargo-deny --version 0.20.2 --locked (the \
     version CI uses), or a package-manager build of the same version (e.g. brew install cargo-deny)";

/// Install hint for `rustc` and `cargo`.
const RUST_HELP: &str = "Install rustup (https://rustup.rs), then run `rustup toolchain install` \
     in the repository root (it installs the toolchain pinned in rust-toolchain.toml)";

/// Install hint for the nightly toolchain with Miri.
const MIRI_HELP: &str = "rustup toolchain install nightly --component miri";

/// Appended to every install hint in a finding.
const ASK_OWNER: &str = "; ask the project owner before installing (ADR-0022)";

/// The program a probe runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Program {
    /// `rustc` from `PATH`.
    Rustc,
    /// `$CARGO`, else `cargo` from `PATH`.
    Cargo,
    /// `rustup` from `PATH`.
    Rustup,
}

/// One tool `doctor` reports on (spec tool table).
#[derive(Debug, Clone, Copy)]
pub(crate) struct ToolSpec {
    /// Name in output, findings and JSON.
    pub name: &'static str,
    /// The program of the probe.
    pub program: Program,
    /// Arguments of the probe; it exits with status 0 only if the tool is installed.
    pub args: &'static [&'static str],
    /// The `ci` steps that are skipped without this tool (empty: none yet).
    pub needed_by: &'static [&'static str],
    /// Why the tool is needed when no `ci` step needs it yet (empty if `needed_by` is not).
    pub later: &'static str,
    /// How to install it.
    pub help: &'static str,
}

/// Every `ci` step that runs a cargo process.
const CARGO_STEPS: &[&str] = &["fmt", "clippy", "test", "doc", "coverage", "deny"];

/// Every tool, in output order (XTASK-R080).
pub(crate) const TOOLS: &[ToolSpec] = &[
    ToolSpec {
        name: "rustc",
        program: Program::Rustc,
        args: &["--version"],
        needed_by: CARGO_STEPS,
        later: "",
        help: RUST_HELP,
    },
    ToolSpec {
        name: "cargo",
        program: Program::Cargo,
        args: &["--version"],
        needed_by: CARGO_STEPS,
        later: "",
        help: RUST_HELP,
    },
    ToolSpec {
        name: "rustfmt",
        program: Program::Cargo,
        args: &["fmt", "--version"],
        needed_by: &["fmt"],
        later: "",
        help: RUSTFMT_HELP,
    },
    ToolSpec {
        name: "clippy",
        program: Program::Cargo,
        args: &["clippy", "--version"],
        needed_by: &["clippy"],
        later: "",
        help: CLIPPY_HELP,
    },
    ToolSpec {
        name: "cargo-llvm-cov",
        program: Program::Cargo,
        args: crate::repo::coverage::LLVM_COV_PROBE,
        needed_by: &["coverage"],
        later: "",
        help: crate::repo::coverage::MISSING_HELP,
    },
    ToolSpec {
        name: "cargo-deny",
        program: Program::Cargo,
        args: &["deny", "--version"],
        needed_by: &["deny"],
        later: "",
        help: CARGO_DENY_HELP,
    },
    ToolSpec {
        name: "miri",
        program: Program::Rustup,
        args: &["run", "nightly", "cargo", "miri", "--version"],
        needed_by: &[],
        later: "Miri tests of `unsafe` crates from M1 (ADR-0018)",
        help: MIRI_HELP,
    },
];

/// The result of one probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Probe {
    /// The probe started and exited with status 0; its stdout.
    Present(String),
    /// The probe did not start, or exited with a non-zero status.
    Missing,
}

/// Runs probes. The real implementation spawns processes; tests use a fake.
pub(crate) trait Prober {
    /// Runs `program` with `args` and reports whether it succeeded.
    fn probe(&mut self, program: Program, args: &[&str]) -> Probe;
}

/// One tool in the report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ToolReport {
    /// Tool name.
    pub tool: &'static str,
    /// Whether the probe succeeded.
    pub present: bool,
    /// First non-empty line of the probe's stdout; `None` if missing or empty.
    pub version: Option<String>,
    /// The `ci` steps that need the tool.
    pub needed_by: &'static [&'static str],
    /// The install hint if missing, else `None`.
    pub help: Option<&'static str>,
}

/// Probes every tool in `tools` and returns the per-tool report and the findings, sorted.
pub(crate) fn diagnose(
    tools: &[ToolSpec],
    prober: &mut dyn Prober,
) -> (Vec<ToolReport>, Vec<Finding>) {
    let mut reports = Vec::new();
    let mut findings = Vec::new();
    for tool in tools {
        match prober.probe(tool.program, tool.args) {
            Probe::Present(stdout) => reports.push(ToolReport {
                tool: tool.name,
                present: true,
                version: stdout
                    .lines()
                    .map(str::trim)
                    .find(|line| !line.is_empty())
                    .map(str::to_string),
                needed_by: tool.needed_by,
                help: None,
            }),
            Probe::Missing => {
                reports.push(ToolReport {
                    tool: tool.name,
                    present: false,
                    version: None,
                    needed_by: tool.needed_by,
                    help: Some(tool.help),
                });
                findings.push(missing(tool));
            }
        }
    }
    check::sort(&mut findings);
    (reports, findings)
}

/// The XTASK-R080 finding for a missing tool: an error if a `ci` step needs it, else a warning.
fn missing(tool: &ToolSpec) -> Finding {
    let help = format!("{}{ASK_OWNER}", tool.help);
    if tool.needed_by.is_empty() {
        Finding::warning(
            "XTASK-R080",
            tool.name,
            format!(
                "{} is not installed (no `ci` step needs it yet: {})",
                tool.name, tool.later
            ),
            help,
        )
    } else {
        let steps: Vec<String> = tool.needed_by.iter().map(|s| format!("`{s}`")).collect();
        Finding::error(
            "XTASK-R080",
            tool.name,
            format!(
                "{} is not installed; `ci` skips {} without it",
                tool.name,
                steps.join(", ")
            ),
            help,
        )
    }
}

#[derive(Serialize)]
struct Report<'a> {
    version: u32,
    ok: bool,
    findings: &'a [Finding],
    tools: &'a [ToolReport],
}

/// The `doctor` command as registered in the command table.
pub(crate) fn command(args: &[String], out: &mut dyn Write) -> Result<Exit, CommandError> {
    execute(args, out, &mut RealProber::new(repo::root()))
}

/// Runs `doctor [--json]` with `prober`.
pub(crate) fn execute(
    args: &[String],
    out: &mut dyn Write,
    prober: &mut dyn Prober,
) -> Result<Exit, CommandError> {
    let mut json = false;
    for arg in args {
        match arg.as_str() {
            "--json" => json = true,
            _ => return Err(cli::unexpected_argument(NAME, arg)),
        }
    }
    let (tools, findings) = diagnose(TOOLS, prober);
    let ok = !findings.iter().any(|f| f.severity == Severity::Error);
    let text = if json {
        let mut text = serde_json::to_string_pretty(&Report {
            version: REPORT_VERSION,
            ok,
            findings: &findings,
            tools: &tools,
        })
        .expect("invariant: the report contains only strings, booleans and lists");
        text.push('\n');
        text
    } else {
        let mut text = tool_lines(&tools);
        text.push_str(&check::human(NAME, &findings));
        text
    };
    out.write_all(text.as_bytes())?;
    Ok(if ok { Exit::Success } else { Exit::Failure })
}

/// `  <tool>  <version>` or `  <tool>  missing`, names padded to one width.
fn tool_lines(tools: &[ToolReport]) -> String {
    let width = tools.iter().map(|t| t.tool.len()).max().unwrap_or(0);
    let mut text = String::new();
    for tool in tools {
        let state = match (&tool.version, tool.present) {
            (Some(version), true) => version.as_str(),
            (None, true) => "present (no version output)",
            (_, false) => "missing",
        };
        let _ = writeln!(text, "  {:width$}  {state}", tool.tool);
    }
    text
}

/// Spawns probes in the repository root, with stdin closed and `RUSTUP_AUTO_INSTALL=0`.
pub(crate) struct RealProber {
    root: PathBuf,
    cargo: OsString,
}

impl RealProber {
    /// A prober whose probes run in `root`, using the `cargo` that started xtask.
    pub(crate) fn new(root: PathBuf) -> Self {
        let cargo = std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
        Self { root, cargo }
    }

    /// The probe command: `program` with `args` in the repository root, stdin closed, and
    /// `RUSTUP_AUTO_INSTALL=0` so it never installs a toolchain (ADR-0022).
    fn command(&self, program: Program, args: &[&str]) -> Command {
        let program = match program {
            Program::Rustc => OsString::from("rustc"),
            Program::Cargo => self.cargo.clone(),
            Program::Rustup => OsString::from("rustup"),
        };
        let mut command = Command::new(program);
        command
            .args(args)
            .current_dir(&self.root)
            .env("RUSTUP_AUTO_INSTALL", "0")
            .stdin(Stdio::null())
            .stderr(Stdio::null());
        command
    }
}

impl Prober for RealProber {
    fn probe(&mut self, program: Program, args: &[&str]) -> Probe {
        match self.command(program, args).output() {
            Ok(output) if output.status.success() => {
                Probe::Present(String::from_utf8_lossy(&output.stdout).into_owned())
            }
            _ => Probe::Missing,
        }
    }
}

#[cfg(test)]
mod tests;
