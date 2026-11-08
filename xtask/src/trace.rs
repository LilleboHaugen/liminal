//! The `trace` command and `ci` step (`docs/specs/xtask.md` XTASK-R040 … R045 and the "`trace`
//! and `hygiene` details").
//!
//! [`trace`] is a pure function of the [`SourceModel`]: it builds the requirement → test matrix and
//! the findings. Only [`run_check`] and [`command`] read files, through the shared loader in
//! [`crate::repo`].

mod annotations;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Write;
use std::path::Path;

use serde::Serialize;

use crate::check;
use crate::cli::{self, CommandError, Exit};
use crate::finding::{Finding, Severity};
use crate::repo::sources::{self, SourceModel};
use crate::repo::{self, LoadError};
use crate::requirements::{self, Extracted, Level, Spec};
use annotations::{Content, annotations};

/// Command and step name.
pub(crate) const NAME: &str = "trace";

/// Report schema version of `trace --json`.
const REPORT_VERSION: u32 = 1;

/// A test that verifies a requirement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct TestRef {
    /// `path:line` of the function's name.
    pub location: String,
    /// The function name.
    pub function: String,
}

/// One requirement in the matrix (XTASK-R042).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Row {
    /// `PREFIX-Rnnn`.
    pub id: String,
    /// `MUST`, `SHOULD` or `MAY`.
    pub level: &'static str,
    /// The crate of the spec that defines it.
    #[serde(rename = "crate")]
    pub crate_name: String,
    /// `path:line` of the requirement row.
    pub spec: String,
    /// Whether it is withdrawn (not counted, not gated).
    pub withdrawn: bool,
    /// The verifying tests, sorted by location.
    pub tests: Vec<TestRef>,
}

/// Tested and untested counts for one level.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub(crate) struct Count {
    /// Non-withdrawn requirements with at least one verifying test.
    pub tested: usize,
    /// Non-withdrawn requirements without a verifying test.
    pub untested: usize,
}

/// Per-crate counts (XTASK-R042).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CrateSummary {
    /// The crate (spec file name).
    #[serde(rename = "crate")]
    pub crate_name: String,
    /// The spec file.
    pub spec: String,
    /// The spec's `maturity`, if readable.
    pub maturity: Option<String>,
    /// Whether XTASK-R044 makes untested MUST requirements errors.
    pub gated: bool,
    /// `MUST`, `SHOULD`, `MAY` → counts.
    pub levels: BTreeMap<&'static str, Count>,
}

/// The result of `trace`: the matrix and the findings.
#[derive(Debug, Clone, Default)]
pub(crate) struct Trace {
    /// Every requirement, in spec and row order.
    pub rows: Vec<Row>,
    /// Every spec, sorted by path.
    pub crates: Vec<CrateSummary>,
    /// `(crate, finding)`: the crate the finding belongs to, for `--crate`. Sorted by finding.
    pub findings: Vec<(String, Finding)>,
}

/// The `trace` command: `[--json] [--crate <name>]`.
pub(crate) fn command(args: &[String], out: &mut dyn Write) -> Result<Exit, CommandError> {
    execute(args, out, &repo::root())
}

/// The `trace` command on the repository at `root`.
fn execute(args: &[String], out: &mut dyn Write, root: &Path) -> Result<Exit, CommandError> {
    let mut json = false;
    let mut only: Option<String> = None;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--crate" => match rest.next() {
                Some(name) if only.is_none() => only = Some(name.clone()),
                Some(_) => return Err(CommandError::Usage("`--crate` is given twice".into())),
                None => return Err(CommandError::Usage("`--crate` needs a crate name".into())),
            },
            _ => {
                if let Some(name) = arg.strip_prefix("--crate=") {
                    if only.is_some() {
                        return Err(CommandError::Usage("`--crate` is given twice".into()));
                    }
                    only = Some(name.to_string());
                } else {
                    return Err(cli::unexpected_argument(NAME, arg));
                }
            }
        }
    }
    let mut result = trace(&sources::load(root)?);
    if let Some(name) = &only {
        if !result.crates.iter().any(|c| &c.crate_name == name) {
            return Err(CommandError::Usage(format!(
                "`--crate {name}`: no spec docs/specs/{name}.md"
            )));
        }
        result.rows.retain(|row| &row.crate_name == name);
        result.crates.retain(|c| &c.crate_name == name);
        result.findings.retain(|(crate_name, _)| crate_name == name);
    }
    let findings: Vec<Finding> = result.findings.iter().map(|(_, f)| f.clone()).collect();
    let ok = !findings.iter().any(|f| f.severity == Severity::Error);
    if json {
        #[derive(Serialize)]
        struct Report<'a> {
            version: u32,
            ok: bool,
            findings: &'a [Finding],
            requirements: &'a [Row],
            crates: &'a [CrateSummary],
        }
        let report = Report {
            version: REPORT_VERSION,
            ok,
            findings: &findings,
            requirements: &result.rows,
            crates: &result.crates,
        };
        let mut text = serde_json::to_string_pretty(&report)
            .expect("invariant: the report contains only strings, numbers, booleans and lists");
        text.push('\n');
        out.write_all(text.as_bytes())?;
    } else {
        let mut text = matrix(&result);
        text.push_str(&check::human(NAME, &findings));
        out.write_all(text.as_bytes())?;
    }
    Ok(if ok { Exit::Success } else { Exit::Failure })
}

/// The human-readable matrix: per crate, one line per requirement with its tests (or `untested`,
/// `withdrawn`), then the counts.
fn matrix(result: &Trace) -> String {
    let mut text = String::new();
    for summary in &result.crates {
        let maturity = summary.maturity.as_deref().unwrap_or("unknown");
        let gate = if summary.gated {
            "untested MUST requirements are errors (XTASK-R044)"
        } else {
            "untested MUST requirements are information only"
        };
        let _ = writeln!(
            text,
            "{} ({}, maturity {maturity}): {gate}",
            summary.crate_name, summary.spec
        );
        for row in result
            .rows
            .iter()
            .filter(|row| row.crate_name == summary.crate_name)
        {
            let head = format!("  {:<12} {:<6}", row.id, row.level);
            if row.withdrawn {
                let _ = writeln!(text, "{head} withdrawn");
            } else if row.tests.is_empty() {
                let _ = writeln!(text, "{head} untested");
            } else {
                for (index, test) in row.tests.iter().enumerate() {
                    let lead = if index == 0 { head.as_str() } else { "" };
                    let _ = writeln!(text, "{lead:<21} {} {}", test.location, test.function);
                }
            }
        }
        let counts: Vec<String> = Level::ALL
            .iter()
            .map(|level| {
                let count = summary.levels[level.as_str()];
                format!(
                    "{} {}/{}",
                    level.as_str(),
                    count.tested,
                    count.tested + count.untested
                )
            })
            .collect();
        let _ = writeln!(text, "  tested: {}\n", counts.join(", "));
    }
    text
}

/// Loads the repository at `root` and returns the `trace` findings (the [`check::CheckFn`] of the
/// `ci` step).
///
/// # Errors
///
/// [`LoadError`] if the repository, `ci/hygiene.toml` or a source file cannot be read.
pub(crate) fn run_check(root: &Path) -> Result<Vec<Finding>, LoadError> {
    Ok(trace(&sources::load(root)?)
        .findings
        .into_iter()
        .map(|(_, finding)| finding)
        .collect())
}

/// `(crate, finding)` pairs: each finding with the crate it belongs to.
type Findings = Vec<(String, Finding)>;

/// The matrix and findings for `model` (XTASK-R040 … R045).
pub(crate) fn trace(model: &SourceModel) -> Trace {
    let extracted = requirements::extract(&model.repo);
    let mut findings: Findings = Vec::new();
    for finding in &extracted.findings {
        let path = finding.location.split(':').next().unwrap_or("");
        let crate_name = extracted
            .specs
            .iter()
            .find(|spec| spec.path == path)
            .map(|spec| spec.crate_name.clone())
            .unwrap_or_default();
        findings.push((crate_name, finding.clone()));
    }
    let mut tests = verifying_tests(model, &extracted, &mut findings);
    let rows = rows(&extracted, &mut tests, &mut findings);
    let crates = extracted
        .specs
        .iter()
        .map(|spec| summary(spec, &rows))
        .collect();
    findings.sort_by(|(_, a), (_, b)| check::order(a, b));
    Trace {
        rows,
        crates,
        findings,
    }
}

/// XTASK-R041 and R043: reads the annotations of every source file not excluded by
/// `[[trace.exclude]]`. Returns requirement ID → verifying tests.
fn verifying_tests<'a>(
    model: &SourceModel,
    extracted: &'a Extracted,
    findings: &mut Findings,
) -> BTreeMap<&'a str, Vec<TestRef>> {
    let mut tests: BTreeMap<&str, Vec<TestRef>> = BTreeMap::new();
    for file in &model.sources {
        if model
            .config
            .trace
            .iter()
            .any(|exclusion| exclusion.covers(&file.path))
        {
            continue;
        }
        for annotation in annotations(&file.text) {
            let location = format!("{}:{}", file.path, annotation.line);
            let mut report = |rule: &str, message: &str, help: &str| {
                findings.push((
                    file.crate_name.clone(),
                    Finding::error(rule, &location, message, help),
                ));
            };
            let (ids, function) = match annotation.content {
                Content::Malformed => {
                    report(
                        "XTASK-R041",
                        "malformed annotation: write `/// Verifies: ID, ID` with requirement IDs \
                         (`PREFIX-Rnnn`) only",
                        "Fix the line so that trace counts the test, e.g. \
                         `/// Verifies: XTASK-R001, XTASK-R002`",
                    );
                    continue;
                }
                Content::Verifies { ids, function } => (ids, function),
            };
            let Some((function, function_line)) = function else {
                report(
                    "XTASK-R041",
                    "annotation is not attached to a function: the next item after it is not \
                     a `fn`",
                    "Put the annotation in the doc comment directly above the test function",
                );
                continue;
            };
            for id in ids {
                match extracted.get(&id) {
                    Some(requirement) if requirement.withdrawn => report(
                        "XTASK-R043",
                        &format!("annotation names `{id}`, which is withdrawn"),
                        "Remove the ID from the annotation; a withdrawn requirement is not tested",
                    ),
                    Some(requirement) => {
                        tests
                            .entry(requirement.id.as_str())
                            .or_default()
                            .push(TestRef {
                                location: format!("{}:{function_line}", file.path),
                                function: function.clone(),
                            });
                    }
                    // A malformed row is reported by XTASK-R040 only.
                    None if extracted.is_known(&id) => {}
                    None => report(
                        "XTASK-R043",
                        &format!("annotation names `{id}`, which is not a requirement in any spec"),
                        "Use an ID from a spec's section 6 table (check for typos), or add the \
                         requirement to the spec first",
                    ),
                }
            }
        }
    }
    tests
}

/// The matrix rows, in spec and row order, and XTASK-R044 for gated specs.
fn rows(
    extracted: &Extracted,
    tests: &mut BTreeMap<&str, Vec<TestRef>>,
    findings: &mut Findings,
) -> Vec<Row> {
    let mut rows = Vec::new();
    for requirement in &extracted.requirements {
        let spec = &extracted.specs[requirement.spec];
        let mut verifying = tests.remove(requirement.id.as_str()).unwrap_or_default();
        verifying.sort_by(|a, b| {
            (check::location_key(&a.location), &a.function)
                .cmp(&(check::location_key(&b.location), &b.function))
        });
        verifying.dedup();
        let location = format!("{}:{}", spec.path, requirement.line);
        if spec.gated()
            && requirement.level == Level::Must
            && !requirement.withdrawn
            && verifying.is_empty()
        {
            findings.push((
                spec.crate_name.clone(),
                Finding::error(
                    "XTASK-R044",
                    &location,
                    format!(
                        "MUST requirement `{}` has no verifying test, and the spec's maturity is \
                         `{}`",
                        requirement.id,
                        spec.maturity.as_deref().unwrap_or("")
                    ),
                    "Add a test with `/// Verifies: <ID>` in its doc comment \
                     (docs/process/TESTING.md §Requirement traceability)",
                ),
            ));
        }
        rows.push(Row {
            id: requirement.id.clone(),
            level: requirement.level.as_str(),
            crate_name: spec.crate_name.clone(),
            spec: location,
            withdrawn: requirement.withdrawn,
            tests: verifying,
        });
    }
    rows
}

/// Tested and untested counts per level for `spec` (withdrawn requirements not counted).
fn summary(spec: &Spec, rows: &[Row]) -> CrateSummary {
    let mut levels: BTreeMap<&'static str, Count> = Level::ALL
        .iter()
        .map(|level| (level.as_str(), Count::default()))
        .collect();
    for row in rows
        .iter()
        .filter(|row| row.crate_name == spec.crate_name && !row.withdrawn)
    {
        let count = levels.entry(row.level).or_default();
        if row.tests.is_empty() {
            count.untested += 1;
        } else {
            count.tested += 1;
        }
    }
    CrateSummary {
        crate_name: spec.crate_name.clone(),
        spec: spec.path.clone(),
        maturity: spec.maturity.clone(),
        gated: spec.gated(),
        levels,
    }
}

#[cfg(test)]
mod fixture_tests;
#[cfg(test)]
mod tests;
