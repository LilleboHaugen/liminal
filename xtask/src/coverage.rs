//! The `coverage` command and `ci` step: per-crate line coverage against the ratchet floors in
//! `ci/coverage.toml` (`docs/specs/xtask.md` XTASK-R060 … R064, R009, R010, "`coverage` details";
//! ADR-0010; `docs/process/TESTING.md` §Coverage).
//!
//! Measuring spawns `cargo llvm-cov` and is a loader (`repo::coverage`, INV-3). The rules
//! ([`evaluate`], [`raised_floors`]) are pure functions of the measurement and the floors, in
//! exact integer arithmetic: a measurement is (covered lines, lines), a floor is in tenths of a
//! percentage point.

use std::ffi::{OsStr, OsString};
use std::fmt::Write as _;
use std::io::Write;
use std::path::Path;

use serde::Serialize;

use crate::check;
use crate::ci::{Outcome, OutputMode};
use crate::cli::{self, CommandError, Exit};
use crate::finding::{Finding, Severity};
use crate::repo::coverage::{
    self as loader, CONFIG_PATH, Floors, LLVM_COV_PROBE, Lines, MeasureError, Measurement, Stderr,
};
use crate::repo::{self, LoadError};

/// Command and step name.
pub(crate) const NAME: &str = "coverage";

/// Tolerance below a floor, in tenths of a percentage point (XTASK-R061: 0.5 points).
const TOLERANCE_TENTHS: u128 = 5;

/// `n` tenths of a percentage point as text: `784` → `78.4`, `1000` → `100.0`.
pub(crate) fn tenths_text(tenths: u32) -> String {
    format!("{}.{}", tenths / 10, tenths % 10)
}

/// The measured percentage with two decimals, rounded down: (4220, 4290) → `98.36`.
fn percent_text(lines: Lines) -> String {
    let hundredths = u128::from(lines.covered) * 10_000 / u128::from(lines.count.max(1));
    format!("{}.{:02}", hundredths / 100, hundredths % 100)
}

/// The measured percentage rounded down to 0.1, in tenths (XTASK-R063). `None` without lines.
pub(crate) fn measured_tenths(lines: Lines) -> Option<u32> {
    if lines.count == 0 {
        return None;
    }
    let tenths = u128::from(lines.covered) * 1000 / u128::from(lines.count);
    Some(u32::try_from(tenths).expect("invariant: covered <= count, so tenths <= 1000"))
}

/// Whether `lines` is more than 0.5 points below `floor_tenths` (XTASK-R061). Without measured
/// lines this is false; [`evaluate`] reports such a crate separately if its floor is above 0.
pub(crate) fn below_floor(lines: Lines, floor_tenths: u32) -> bool {
    // covered / count * 100 < floor / 10 - 0.5  ⇔  covered * 1000 < (floor - 5) * count
    let Some(threshold) = u128::from(floor_tenths).checked_sub(TOLERANCE_TENTHS) else {
        return false;
    };
    // With `count == 0` the right side is 0 and the comparison is false.
    u128::from(lines.covered) * 1000 < threshold * u128::from(lines.count)
}

/// The findings of the coverage rules (XTASK-R061, R062), sorted. `measurement` has an entry for
/// every workspace crate. A crate with a floor above 0 and no measured lines is an R061 error.
pub(crate) fn evaluate(measurement: &Measurement, floors: &Floors) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (name, lines) in measurement {
        match floors.get(name) {
            None => {
                let suggested = tenths_text(measured_tenths(*lines).unwrap_or(0));
                findings.push(Finding::error(
                    "XTASK-R062",
                    CONFIG_PATH,
                    format!("workspace crate `{name}` has no coverage floor"),
                    format!("add `{name} = {suggested}` under `[floors]` (its measured coverage, rounded down to 0.1)"),
                ));
            }
            // Coverage unknown, not 100 %: its files dropped out of the export, so its ratchet
            // would be gone without a finding (spec v18).
            Some(floor) if lines.count == 0 && floor.tenths > 0 => findings.push(Finding::error(
                "XTASK-R061",
                format!("{CONFIG_PATH}:{}", floor.line),
                format!(
                    "`{name}` has no measured lines but a coverage floor of {}%: its coverage is \
                     unknown",
                    tenths_text(floor.tenths)
                ),
                "check that the crate's files are in the `cargo llvm-cov` export (its member path \
                 in Cargo.toml, its source files); if it has no code to measure, set its floor to \
                 0.0 with a journal entry that says why (docs/process/TESTING.md §Coverage)",
            )),
            Some(floor) if below_floor(*lines, floor.tenths) => findings.push(Finding::error(
                "XTASK-R061",
                format!("{CONFIG_PATH}:{}", floor.line),
                format!(
                    "`{name}` line coverage {}% ({} of {} lines) is more than 0.5 points below \
                     its floor {}%",
                    percent_text(*lines),
                    lines.covered,
                    lines.count,
                    tenths_text(floor.tenths)
                ),
                "add tests for the uncovered lines; lowering a floor needs a journal entry that \
                 says why (docs/process/TESTING.md §Coverage)",
            )),
            Some(_) => {}
        }
    }
    for (name, floor) in floors {
        if !measurement.contains_key(name) {
            findings.push(Finding::error(
                "XTASK-R062",
                format!("{CONFIG_PATH}:{}", floor.line),
                format!("coverage floor for `{name}`, which is not a workspace crate"),
                "remove the entry, or rename it to the crate's current name",
            ));
        }
    }
    check::sort(&mut findings);
    findings
}

/// One raised floor: crate, old and new value in tenths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Raise {
    /// The crate.
    pub name: String,
    /// The floor before.
    pub from: u32,
    /// The floor after: the measured value rounded down to 0.1, above `from`.
    pub to: u32,
}

/// The floors `--update-floors` raises (XTASK-R063): each floor of a measured crate becomes
/// `max(floor, measured rounded down to 0.1)`. Floors are never lowered, never added (a missing
/// floor stays an R062 finding), and a crate without measured lines keeps its floor.
pub(crate) fn raised_floors(measurement: &Measurement, floors: &Floors) -> Vec<Raise> {
    floors
        .iter()
        .filter_map(|(name, floor)| {
            let measured = measured_tenths(*measurement.get(name)?)?;
            (measured > floor.tenths).then(|| Raise {
                name: name.clone(),
                from: floor.tenths,
                to: measured,
            })
        })
        .collect()
}

/// `text` (of `ci/coverage.toml`) with the values of the raised floors replaced. Everything else,
/// comments included, is kept byte for byte.
pub(crate) fn apply_raises(text: &str, floors: &Floors, raises: &[Raise]) -> String {
    let mut spans: Vec<((usize, usize), String)> = raises
        .iter()
        .filter_map(|raise| Some((floors.get(&raise.name)?.span, tenths_text(raise.to))))
        .collect();
    // Replace from the end, so earlier spans keep their offsets.
    spans.sort_by_key(|((start, _), _)| std::cmp::Reverse(*start));
    let mut text = text.to_string();
    for ((start, end), value) in spans {
        text.replace_range(start..end, &value);
    }
    text
}

#[derive(Serialize)]
struct CrateReport<'a> {
    #[serde(rename = "crate")]
    name: &'a str,
    covered: u64,
    count: u64,
    /// Measured line coverage in percent; `null` without measured lines.
    percent: Option<f64>,
    /// The floor in percent; `null` without an entry.
    floor: Option<f64>,
}

#[derive(Serialize)]
struct Skip<'a> {
    reason: &'a str,
    help: &'a str,
}

#[derive(Serialize)]
struct Report<'a> {
    version: u32,
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    skipped: Option<Skip<'a>>,
    findings: &'a [Finding],
    crates: Vec<CrateReport<'a>>,
}

fn json_text(report: &Report<'_>) -> String {
    let mut text = serde_json::to_string_pretty(report)
        .expect("invariant: the report contains only strings, numbers, booleans and lists");
    text.push('\n');
    text
}

/// Lossy `u64` → `f64` for the JSON report only; never used in a comparison.
#[expect(
    clippy::cast_precision_loss,
    reason = "report value only; line counts are far below 2^52"
)]
fn as_f64(value: u64) -> f64 {
    value as f64
}

fn crate_reports<'a>(measurement: &'a Measurement, floors: &Floors) -> Vec<CrateReport<'a>> {
    measurement
        .iter()
        .map(|(name, lines)| CrateReport {
            name,
            covered: lines.covered,
            count: lines.count,
            percent: (lines.count > 0).then(|| as_f64(lines.covered) * 100.0 / as_f64(lines.count)),
            floor: floors.get(name).map(|floor| f64::from(floor.tenths) / 10.0),
        })
        .collect()
}

/// The human-readable table: one line per crate.
fn crate_lines(measurement: &Measurement, floors: &Floors) -> String {
    let width = measurement.keys().map(String::len).max().unwrap_or(0);
    let mut text = String::new();
    for (name, lines) in measurement {
        let measured = if lines.count == 0 {
            "no lines measured".to_string()
        } else {
            format!(
                "{}% ({}/{} lines)",
                percent_text(*lines),
                lines.covered,
                lines.count
            )
        };
        let floor = floors.get(name).map_or_else(
            || "no floor".to_string(),
            |floor| format!("floor {}%", tenths_text(floor.tenths)),
        );
        let _ = writeln!(text, "{name:width$}  {measured}, {floor}");
    }
    text
}

/// Command options.
#[derive(Debug, Default, PartialEq, Eq)]
struct Options {
    json: bool,
    update_floors: bool,
}

fn parse_options(args: &[String]) -> Result<Options, CommandError> {
    let mut options = Options::default();
    for arg in args {
        match arg.as_str() {
            "--json" => options.json = true,
            "--update-floors" => options.update_floors = true,
            _ => return Err(cli::unexpected_argument(NAME, arg)),
        }
    }
    Ok(options)
}

/// The `cargo` program: `$CARGO` (set by cargo for `cargo run`), else `cargo` from `PATH`.
fn cargo() -> OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"))
}

/// The `coverage` command as registered in the command table.
pub(crate) fn command(args: &[String], out: &mut dyn Write) -> Result<Exit, CommandError> {
    let options = parse_options(args)?;
    let root = repo::root();
    execute(&options, out, &root, |members| {
        loader::measure(&root, &cargo(), LLVM_COV_PROBE, members, Stderr::Inherit)
    })
}

/// Runs `coverage` on the repository at `root`, measuring with `measure` (tests pass a fixed
/// measurement).
fn execute(
    options: &Options,
    out: &mut dyn Write,
    root: &Path,
    measure: impl FnOnce(&[repo::Member]) -> Result<Measurement, MeasureError>,
) -> Result<Exit, CommandError> {
    let members = repo::workspace::load(root)?;
    let config = repo::read(root, CONFIG_PATH)?;
    let mut floors = loader::parse_floors(&config)?;
    let measurement = match measure(&members) {
        Ok(measurement) => measurement,
        Err(MeasureError::Failed(error)) => return Err(error.into()),
        Err(MeasureError::Skipped { reason, help }) => {
            let text = if options.json {
                json_text(&Report {
                    version: 1,
                    ok: false,
                    skipped: Some(Skip {
                        reason: &reason,
                        help: &help,
                    }),
                    findings: &[],
                    crates: Vec::new(),
                })
            } else {
                format!("{NAME}: skipped: {reason}\n  help: {help}\n")
            };
            out.write_all(text.as_bytes())?;
            return Ok(Exit::Failure);
        }
    };
    let mut updates = String::new();
    if options.update_floors {
        let raises = raised_floors(&measurement, &floors);
        if !raises.is_empty() {
            let new_config = apply_raises(&config, &floors, &raises);
            let path = root.join(CONFIG_PATH);
            std::fs::write(&path, &new_config)
                .map_err(|error| CommandError::File { path, error })?;
            floors = loader::parse_floors(&new_config)?;
        }
        for raise in &raises {
            let _ = writeln!(
                updates,
                "raised floor of `{}`: {} -> {}",
                raise.name,
                tenths_text(raise.from),
                tenths_text(raise.to)
            );
        }
    }
    let findings = evaluate(&measurement, &floors);
    let ok = !findings
        .iter()
        .any(|finding| finding.severity == Severity::Error);
    let text = if options.json {
        json_text(&Report {
            version: 1,
            ok,
            skipped: None,
            findings: &findings,
            crates: crate_reports(&measurement, &floors),
        })
    } else {
        format!(
            "{}{updates}{}",
            crate_lines(&measurement, &floors),
            check::human(NAME, &findings)
        )
    };
    out.write_all(text.as_bytes())?;
    Ok(if ok { Exit::Success } else { Exit::Failure })
}

/// The `coverage` step of `ci`: measures with `measure`, never updates floors, and maps the
/// result to a step outcome. A missing tool is `skipped` (XTASK-R064); an internal error fails the
/// step with its message as `output_tail` (spec "native check step").
pub(crate) fn run_step(
    root: &Path,
    mode: OutputMode,
    measure: impl FnOnce(&[repo::Member], Stderr) -> Result<Measurement, MeasureError>,
) -> Outcome {
    let stderr = match mode {
        OutputMode::Inherit => Stderr::Inherit,
        OutputMode::Capture => Stderr::Capture,
    };
    let internal = |error: LoadError| Outcome::Fail {
        findings: Vec::new(),
        output_tail: Some(error.to_string()),
    };
    let loaded = repo::workspace::load(root).and_then(|members| {
        let floors = loader::parse_floors(&repo::read(root, CONFIG_PATH)?)?;
        Ok((members, floors))
    });
    let (members, floors) = match loaded {
        Ok(loaded) => loaded,
        Err(error) => return internal(error),
    };
    match measure(&members, stderr) {
        Ok(measurement) => {
            let findings = evaluate(&measurement, &floors);
            if findings.is_empty() {
                Outcome::Pass
            } else {
                Outcome::Fail {
                    findings,
                    output_tail: None,
                }
            }
        }
        Err(MeasureError::Skipped { reason, help }) => Outcome::Skipped { reason, help },
        Err(MeasureError::Failed(error)) => internal(error),
    }
}

/// [`run_step`] with the real measurement: `cargo llvm-cov` run with the `cargo` program `cargo`.
pub(crate) fn run_real_step(root: &Path, cargo: &OsStr, mode: OutputMode) -> Outcome {
    run_step(root, mode, |members, stderr| {
        loader::measure(root, cargo, LLVM_COV_PROBE, members, stderr)
    })
}

#[cfg(test)]
mod tests;
