//! [`Bench`]: runs the cases of one bench target and writes its result file (spec §5,
//! BENCH-R002 … R004, R010 … R015, R030, R033).

use std::collections::BTreeSet;
use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

use crate::case::is_segment;
use crate::config::MAX_ITERATIONS;
use crate::{BenchError, Case, CaseResult, Config, Metadata, Report, Summary};

/// A monotonic clock in nanoseconds. The harness uses [`WallClock`]; tests use a clock the
/// routine advances, so measurement logic is tested without reading wall-clock time.
pub(crate) trait Clock {
    /// Nanoseconds since an arbitrary fixed start; never decreases.
    fn now_ns(&self) -> u64;
}

/// [`Clock`] on `std::time::Instant`.
struct WallClock(Instant);

impl Clock for WallClock {
    fn now_ns(&self) -> u64 {
        u64::try_from(self.0.elapsed().as_nanos()).unwrap_or(u64::MAX)
    }
}

/// The cases of one bench target, run one after another, and the result file they produce.
///
/// # Examples
///
/// ```
/// use std::hint::black_box;
/// use liminal_benches::{Bench, Case, Config};
///
/// let out_dir = std::env::temp_dir().join(format!("liminal-bench-doc-{}", std::process::id()));
/// let config = Config {
///     iterations: std::num::NonZeroU64::new(100),
///     samples: 3,
///     out_dir: out_dir.clone(),
///     ..Config::default()
/// };
/// let mut bench = Bench::new("demo", config)?;
/// let result = bench.run(&Case::new("demo/math/add")?, || black_box(2_u64) + 2)?;
/// assert_eq!(result.map(|r| r.iterations_per_sample), Some(100));
///
/// let path = bench.finish()?;
/// assert_eq!(path, out_dir.join("demo.json"));
/// let json = std::fs::read_to_string(&path).expect("written");
/// assert!(json.contains("\"id\": \"demo/math/add\""));
/// # std::fs::remove_dir_all(out_dir).ok();
/// # Ok::<(), liminal_benches::BenchError>(())
/// ```
pub struct Bench {
    target: String,
    config: Config,
    seen: BTreeSet<String>,
    results: Vec<CaseResult>,
    clock: Box<dyn Clock>,
}

impl Bench {
    /// A bench target named `target` with `config`.
    ///
    /// # Errors
    ///
    /// [`BenchError::InvalidTarget`] unless `target` is 1 to 64 characters of `[a-z0-9_]`
    /// (BENCH-R003).
    pub fn new(target: &str, config: Config) -> Result<Bench, BenchError> {
        Bench::with_clock(target, config, Box::new(WallClock(Instant::now())))
    }

    /// [`Bench::new`] with [`Config::from_args`] of this process's arguments (BENCH-R040): the
    /// usual first line of a bench target's `main`.
    ///
    /// # Errors
    ///
    /// As [`Bench::new`] and [`Config::from_args`].
    pub fn from_env(target: &str) -> Result<Bench, BenchError> {
        Bench::new(target, Config::from_args(std::env::args().skip(1))?)
    }

    pub(crate) fn with_clock(
        target: &str,
        config: Config,
        clock: Box<dyn Clock>,
    ) -> Result<Bench, BenchError> {
        if !is_segment(target) {
            return Err(BenchError::InvalidTarget {
                target: target.to_string(),
            });
        }
        Ok(Bench {
            target: target.to_string(),
            config,
            seen: BTreeSet::new(),
            results: Vec::new(),
            clock,
        })
    }

    /// Measures `case` by calling `routine` repeatedly, passing each return value through
    /// `std::hint::black_box` (BENCH-R014). Returns `None` without calling `routine` if the
    /// configured filter excludes the case (BENCH-R015).
    ///
    /// # Errors
    ///
    /// Without calling `routine`: [`BenchError::CaseOutsideTarget`] if the case's crate segment
    /// is not the target name (BENCH-R002), [`BenchError::DuplicateCase`] if a case with this ID
    /// was already given to this `Bench` (BENCH-R004).
    ///
    /// # Panics
    ///
    /// Only if `routine` panics. (The internal `expect` states an invariant: measurement always
    /// yields at least one finite sample.)
    pub fn run<R>(
        &mut self,
        case: &Case,
        routine: impl FnMut() -> R,
    ) -> Result<Option<&CaseResult>, BenchError> {
        if case.crate_name() != self.target {
            return Err(BenchError::CaseOutsideTarget {
                id: case.id().to_string(),
                target: self.target.clone(),
            });
        }
        if !self.seen.insert(case.id().to_string()) {
            return Err(BenchError::DuplicateCase {
                id: case.id().to_string(),
            });
        }
        if let Some(filter) = &self.config.filter
            && !case.id().contains(filter.as_str())
        {
            return Ok(None);
        }
        let (iterations, samples) = measure(&*self.clock, &self.config, routine);
        let summary = Summary::from_samples(&samples).expect("invariant: 1 or more finite samples");
        self.results.push(CaseResult {
            id: case.id().to_string(),
            threading: case.threading(),
            elements: case.elements(),
            iterations_per_sample: iterations,
            summary,
        });
        Ok(self.results.last())
    }

    /// The report of the cases run so far, with [`Metadata::detect`].
    #[must_use]
    pub fn report(&self) -> Report {
        Report {
            target: self.target.clone(),
            metadata: Metadata::detect(),
            cases: self.results.clone(),
        }
    }

    /// Writes [`Report::to_json`] to `<out_dir>/<target>.json`, creating the directory and
    /// replacing an existing file (BENCH-R030). Returns the file path.
    ///
    /// # Errors
    ///
    /// [`BenchError::WriteFailed`] if the directory or file cannot be written (BENCH-R033).
    pub fn finish(self) -> Result<PathBuf, BenchError> {
        let path = self.config.out_dir.join(format!("{}.json", self.target));
        let failed = |error: std::io::Error| BenchError::WriteFailed {
            path: path.display().to_string(),
            reason: error.to_string(),
        };
        std::fs::create_dir_all(&self.config.out_dir).map_err(failed)?;
        std::fs::write(&path, self.report().to_json()).map_err(failed)?;
        Ok(path)
    }
}

/// Runs warm-up and calibration (unless iterations are fixed) and the samples. Returns the
/// iterations per sample and each sample's ns per iteration (BENCH-R010 … R013).
pub(crate) fn measure<R>(
    clock: &dyn Clock,
    config: &Config,
    mut routine: impl FnMut() -> R,
) -> (u64, Vec<f64>) {
    let iterations = if let Some(fixed) = config.iterations {
        fixed.get()
    } else {
        warm_up(clock, nanos(config.warm_up), &mut routine);
        calibrate(
            clock,
            nanos(config.min_sample_time),
            MAX_ITERATIONS,
            &mut routine,
        )
    };
    #[expect(
        clippy::cast_precision_loss,
        reason = "iteration counts are at most 2^40, exact in f64; sample times above 2^53 ns \
                  (104 days) are not expected"
    )]
    let samples = (0..config.samples.max(1))
        .map(|_| batch(clock, iterations, &mut routine) as f64 / iterations as f64)
        .collect();
    (iterations, samples)
}

fn nanos(duration: std::time::Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

/// Calls `routine` until at least `ns` nanoseconds have passed; at least once.
fn warm_up<R>(clock: &dyn Clock, ns: u64, routine: &mut impl FnMut() -> R) {
    let start = clock.now_ns();
    loop {
        black_box(routine());
        if clock.now_ns().saturating_sub(start) >= ns {
            break;
        }
    }
}

/// The smallest power of two `n ≤ max` whose timed batch takes at least `ns`, or `max`
/// (BENCH-R011; the harness passes `max` = [`MAX_ITERATIONS`], a power of two).
fn calibrate<R>(clock: &dyn Clock, ns: u64, max: u64, routine: &mut impl FnMut() -> R) -> u64 {
    let mut n = 1;
    while n < max && batch(clock, n, routine) < ns {
        n *= 2;
    }
    n
}

/// Nanoseconds taken by `n` calls of `routine`.
fn batch<R>(clock: &dyn Clock, n: u64, routine: &mut impl FnMut() -> R) -> u64 {
    let start = clock.now_ns();
    for _ in 0..n {
        black_box(routine());
    }
    clock.now_ns().saturating_sub(start)
}

#[cfg(test)]
mod tests;
