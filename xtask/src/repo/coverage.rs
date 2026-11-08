//! Loaders for `coverage` (`docs/specs/xtask.md` XTASK-R060 … R064 and the "`coverage` details"):
//! the floors in `ci/coverage.toml`, and the per-crate line counts from `cargo llvm-cov`.
//!
//! Percentages are never stored as floating point. A floor is an integer number of tenths of a
//! percentage point, and a measurement is the pair (covered lines, line count), so every
//! comparison in the check is exact integer arithmetic.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Stdio};

use serde::Deserialize;
use toml::Spanned;

use super::LoadError;
use super::workspace::{self, Member};

/// Repository-relative path of the floors file.
pub(crate) const CONFIG_PATH: &str = "ci/coverage.toml";

/// Arguments after `cargo` that measure coverage (XTASK-R060). `--locked`: like every cargo step,
/// it must not update `Cargo.lock` (XTASK-R010).
pub(crate) const LLVM_COV_ARGS: &[&str] = &[
    "llvm-cov",
    "--workspace",
    "--all-features",
    "--locked",
    "--json",
    "--summary-only",
];

/// Arguments after `cargo` of a command that succeeds only if `cargo-llvm-cov` is installed
/// (XTASK-R064).
pub(crate) const LLVM_COV_PROBE: &[&str] = &["llvm-cov", "--version"];

/// `reason` of the skip when `cargo-llvm-cov` is missing (XTASK-R064).
pub(crate) const MISSING_REASON: &str = "cargo-llvm-cov is not installed";

/// `help` of the skip when `cargo-llvm-cov` is missing (XTASK-R064).
pub(crate) const MISSING_HELP: &str = "cargo install cargo-llvm-cov --locked (it uses the \
                                       llvm-tools component pinned in rust-toolchain.toml)";

/// One crate's floor in `ci/coverage.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Floor {
    /// The floor in tenths of a percentage point: `784` is 78.4 %. At most 1000.
    pub tenths: u32,
    /// 1-based line of the entry.
    pub line: usize,
    /// Byte range of the value in the file text, which `--update-floors` replaces.
    pub span: (usize, usize),
}

/// `ci/coverage.toml`, parsed: crate name → floor.
pub(crate) type Floors = BTreeMap<String, Floor>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    floors: BTreeMap<String, Spanned<toml::Value>>,
}

fn invalid(detail: impl Into<String>) -> LoadError {
    LoadError {
        code: "LIM-XTASK-0004",
        path: CONFIG_PATH.to_string(),
        detail: detail.into(),
        help: "Fix the file at the reported location",
    }
}

/// Parses the text of `ci/coverage.toml`.
///
/// # Errors
///
/// LIM-XTASK-0004 if the text is not valid TOML, has no `[floors]` table or another top-level
/// key, or a floor is not a number from 0 to 100 with at most one decimal.
pub(crate) fn parse_floors(text: &str) -> Result<Floors, LoadError> {
    let raw: RawConfig = toml::from_str(text)
        .map_err(|error| invalid(format!("is not valid: {}", error.to_string().trim_end())))?;
    let line = |offset: usize| text[..offset].matches('\n').count() + 1;
    let mut floors = Floors::new();
    for (name, value) in raw.floors {
        let span = value.span();
        let line = line(span.start);
        let tenths = match value.get_ref() {
            toml::Value::Integer(percent) => u32::try_from(*percent)
                .ok()
                .filter(|percent| *percent <= 100)
                .map(|percent| percent * 10),
            toml::Value::Float(percent) => tenths_of(*percent),
            _ => None,
        };
        let Some(tenths) = tenths else {
            return Err(invalid(format!(
                "line {line}: the floor of `{name}` must be a number from 0 to 100 with at most \
                 one decimal, e.g. `{name} = 78.4`"
            )));
        };
        floors.insert(
            name,
            Floor {
                tenths,
                line,
                span: (span.start, span.end),
            },
        );
    }
    Ok(floors)
}

/// `percent` in tenths, if it is in 0 … 100 and has at most one decimal (as written in TOML;
/// `78.4` is not exactly representable, so "one decimal" allows a 1e-9 rounding error).
fn tenths_of(percent: f64) -> Option<u32> {
    let scaled = percent * 10.0;
    let rounded = scaled.round();
    let exact = (scaled - rounded).abs() < 1e-9;
    if !(exact && (0.0..=1000.0).contains(&rounded)) {
        return None;
    }
    // `rounded` is an integer in 0 … 1000, so the cast is exact.
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "checked above: an integer in 0 ..= 1000"
    )]
    Some(rounded as u32)
}

/// Measured line counts of one crate.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Lines {
    /// Lines executed at least once.
    pub covered: u64,
    /// Instrumented lines.
    pub count: u64,
}

/// Measured coverage: every workspace crate → its lines (zero if no file of it was measured).
pub(crate) type Measurement = BTreeMap<String, Lines>;

#[derive(Deserialize)]
struct Export {
    #[serde(rename = "type")]
    kind: String,
    data: Vec<ExportData>,
}

#[derive(Deserialize)]
struct ExportData {
    files: Vec<ExportFile>,
}

#[derive(Deserialize)]
struct ExportFile {
    filename: String,
    summary: ExportSummary,
}

#[derive(Deserialize)]
struct ExportSummary {
    lines: ExportLines,
}

#[derive(Deserialize)]
struct ExportLines {
    count: u64,
    covered: u64,
}

fn unparseable(detail: impl Into<String>) -> LoadError {
    LoadError {
        code: "LIM-XTASK-0002",
        path: "cargo llvm-cov output".to_string(),
        detail: detail.into(),
        help: "Run `cargo xtask doctor`; tool version may be unsupported",
    }
}

/// Sums the line counts of an llvm-cov JSON export (`--json --summary-only`) per workspace crate.
///
/// `root` is the repository root as the tool sees it (absolute). A file belongs to the member
/// whose manifest directory contains it (innermost if nested). Files outside `root` or outside
/// every member's directory are ignored. `\` in paths is read as `/`.
///
/// # Errors
///
/// LIM-XTASK-0002 if the text is not an llvm-cov JSON export, or a file has more covered lines
/// than lines.
pub(crate) fn parse_export(
    json: &str,
    root: &str,
    members: &[Member],
) -> Result<Measurement, LoadError> {
    let export: Export = serde_json::from_str(json)
        .map_err(|error| unparseable(format!("is not an llvm-cov JSON export: {error}")))?;
    if export.kind != "llvm.coverage.json.export" {
        return Err(unparseable(format!(
            "has type `{}`, expected `llvm.coverage.json.export`",
            export.kind
        )));
    }
    let root = format!("{}/", root.replace('\\', "/").trim_end_matches('/'));
    let mut measurement: Measurement = members
        .iter()
        .map(|member| (member.name.clone(), Lines::default()))
        .collect();
    let mut files = 0_usize;
    let mut inside_root = 0_usize;
    for file in export.data.iter().flat_map(|data| &data.files) {
        let ExportLines { count, covered } = file.summary.lines;
        if covered > count {
            return Err(unparseable(format!(
                "`{}` has {covered} covered lines but only {count} lines",
                file.filename
            )));
        }
        files += 1;
        let filename = file.filename.replace('\\', "/");
        let Some(relative) = filename.strip_prefix(&root) else {
            continue;
        };
        inside_root += 1;
        if let Some(member) = workspace::owner(members, relative) {
            let lines = measurement.entry(member.name.clone()).or_default();
            lines.covered += covered;
            lines.count += count;
        }
    }
    // Paths spelled differently from `root` (a symlink, another drive letter case) would make
    // every crate look unmeasured. That is an error, never a silent pass.
    if files > 0 && inside_root == 0 {
        return Err(unparseable(format!(
            "none of its {files} files is inside the repository root `{root}`"
        )));
    }
    Ok(measurement)
}

/// Why coverage could not be measured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MeasureError {
    /// A tool is missing: the step is `skipped` (XTASK-R006, R064).
    Skipped {
        /// Why the step could not run.
        reason: String,
        /// How to make it run.
        help: String,
    },
    /// The tool ran and failed, or its output could not be parsed.
    Failed(LoadError),
}

/// What `cargo llvm-cov` writes to stderr (build and test output).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stderr {
    /// Passed through to xtask's stderr.
    Inherit,
    /// Captured; its last lines go into the error if the tool fails.
    Capture,
}

/// Runs `cargo llvm-cov` (XTASK-R060) in `root` with the `cargo` program `cargo`, after checking
/// with `probe` that the tool is installed, and parses its output.
///
/// # Errors
///
/// [`MeasureError::Skipped`] if `cargo` cannot be started or `probe` fails (XTASK-R064);
/// [`MeasureError::Failed`] with LIM-XTASK-0006 if `cargo llvm-cov` fails (for example a test
/// fails), or with LIM-XTASK-0002 if its output cannot be parsed.
pub(crate) fn measure(
    root: &Path,
    cargo: &OsStr,
    probe: &[&str],
    members: &[Member],
    stderr: Stderr,
) -> Result<Measurement, MeasureError> {
    let cannot_start = |error: std::io::Error| MeasureError::Skipped {
        reason: format!("could not start `{}`: {error}", cargo.to_string_lossy()),
        help: crate::process::CARGO_HELP.to_string(),
    };
    let probed = Command::new(cargo)
        .args(probe)
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(cannot_start)?;
    if !probed.success() {
        return Err(MeasureError::Skipped {
            reason: MISSING_REASON.to_string(),
            help: MISSING_HELP.to_string(),
        });
    }
    let mut command = Command::new(cargo);
    command
        .args(LLVM_COV_ARGS)
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped());
    match stderr {
        Stderr::Inherit => command.stderr(Stdio::inherit()),
        Stderr::Capture => command.stderr(Stdio::piped()),
    };
    let output = command.output().map_err(cannot_start)?;
    if !output.status.success() {
        let text = String::from_utf8_lossy(&output.stderr);
        let lines: Vec<&str> = text.lines().collect();
        let tail = lines[lines.len().saturating_sub(crate::process::TAIL_LINES)..].join("\n");
        let mut detail = format!("failed ({})", output.status);
        if !tail.is_empty() {
            detail.push('\n');
            detail.push_str(&tail);
        }
        return Err(MeasureError::Failed(LoadError {
            code: "LIM-XTASK-0006",
            path: format!("cargo {}", LLVM_COV_ARGS.join(" ")),
            detail,
            help: "Run `cargo llvm-cov --workspace` in the repository root to see the error; \
                   a failing test fails coverage too",
        }));
    }
    let json = String::from_utf8(output.stdout)
        .map_err(|_| MeasureError::Failed(unparseable("is not valid UTF-8")))?;
    parse_export(&json, &root.to_string_lossy(), members).map_err(MeasureError::Failed)
}

#[cfg(test)]
mod tests;
