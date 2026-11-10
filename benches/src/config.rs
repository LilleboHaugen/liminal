//! Harness configuration and the bench binary command line (spec §5, BENCH-R040 … R042).

use std::ffi::OsString;
use std::num::NonZeroU64;
use std::path::PathBuf;
use std::time::Duration;

use crate::BenchError;

/// Largest iteration count per sample, in calibration and with `--iterations` (2⁴⁰).
pub const MAX_ITERATIONS: u64 = 1 << 40;
/// Largest number of samples per case.
pub const MAX_SAMPLES: u32 = 10_000;
/// Environment variable naming the default output directory (BENCH-R042).
pub const OUT_DIR_ENV: &str = "LIMINAL_BENCH_OUT_DIR";

/// How cases are measured and where the result file goes.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
/// use liminal_benches::Config;
///
/// let config = Config::default();
/// assert_eq!(config.warm_up, Duration::from_millis(300));
/// assert_eq!(config.min_sample_time, Duration::from_millis(10));
/// assert_eq!(config.samples, 30);
/// assert_eq!(config.iterations, None);
///
/// // `cargo bench --bench liminal_core -- --iterations 1000 --samples 1`
/// let args = ["--bench", "--iterations", "1000", "--samples", "1"].map(String::from);
/// let fixed = Config::from_args(args)?;
/// assert_eq!(fixed.iterations.map(|n| n.get()), Some(1000));
/// assert_eq!(fixed.samples, 1);
/// # Ok::<(), liminal_benches::BenchError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    /// Minimum time the routine runs before calibration (BENCH-R010). Default 300 ms.
    pub warm_up: Duration,
    /// Minimum time of one sample, used by calibration (BENCH-R011). Default 10 ms (ADR-0023).
    pub min_sample_time: Duration,
    /// Samples per case, 1 to [`MAX_SAMPLES`] (BENCH-R012). Default 30 (ADR-0023).
    pub samples: u32,
    /// Fixed iterations per sample, 1 to [`MAX_ITERATIONS`]: no warm-up and no calibration
    /// (BENCH-R013). Default `None`.
    pub iterations: Option<NonZeroU64>,
    /// Run only cases whose ID contains this text (BENCH-R015). Default `None`.
    pub filter: Option<String>,
    /// Directory the result file is written to (BENCH-R030, R042).
    pub out_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            warm_up: Duration::from_millis(300),
            min_sample_time: Duration::from_millis(10),
            samples: 30,
            iterations: None,
            filter: None,
            out_dir: default_out_dir(std::env::var_os(OUT_DIR_ENV)),
        }
    }
}

impl Config {
    /// The default configuration changed by the bench binary arguments `args` (without the
    /// program name), BENCH-R040:
    ///
    /// | Argument | Effect |
    /// |---|---|
    /// | `--bench` | ignored (cargo passes it to `harness = false` targets) |
    /// | `--filter <text>` | [`Config::filter`] |
    /// | `--iterations <n>` | [`Config::iterations`], 1 to [`MAX_ITERATIONS`] |
    /// | `--samples <n>` | [`Config::samples`], 1 to [`MAX_SAMPLES`] |
    /// | `--out-dir <path>` | [`Config::out_dir`] |
    ///
    /// A later repetition of an option replaces the earlier value.
    ///
    /// # Errors
    ///
    /// [`BenchError::InvalidArgument`] for an unknown argument, an option without a value, or a
    /// number that is not an integer in its range (BENCH-R041).
    pub fn from_args(args: impl IntoIterator<Item = String>) -> Result<Config, BenchError> {
        let mut config = Config::default();
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            if arg == "--bench" {
                continue;
            }
            if !matches!(
                arg.as_str(),
                "--filter" | "--iterations" | "--samples" | "--out-dir"
            ) {
                return Err(invalid(&arg, "unknown argument"));
            }
            let Some(value) = args.next() else {
                return Err(invalid(&arg, "the option needs a value"));
            };
            match arg.as_str() {
                "--filter" => config.filter = Some(value),
                "--iterations" => {
                    let n = number(&arg, &value, MAX_ITERATIONS)?;
                    config.iterations = NonZeroU64::new(n);
                }
                "--samples" => {
                    let n = number(&arg, &value, u64::from(MAX_SAMPLES))?;
                    config.samples = u32::try_from(n).map_err(|_| invalid(&arg, "too large"))?;
                }
                _ => config.out_dir = PathBuf::from(value),
            }
        }
        Ok(config)
    }
}

/// The default output directory: `$LIMINAL_BENCH_OUT_DIR` if set and not empty, otherwise
/// `target/liminal-bench` in the workspace root (BENCH-R042).
pub(crate) fn default_out_dir(env: Option<OsString>) -> PathBuf {
    match env {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("target")
            .join("liminal-bench"),
    }
}

fn invalid(argument: &str, reason: &str) -> BenchError {
    BenchError::InvalidArgument {
        argument: argument.to_string(),
        reason: reason.to_string(),
    }
}

/// `value` as an integer in `1..=max`.
fn number(option: &str, value: &str, max: u64) -> Result<u64, BenchError> {
    let argument = format!("{option} {value}");
    match value.parse::<u64>() {
        Ok(n) if (1..=max).contains(&n) => Ok(n),
        _ => Err(invalid(
            &argument,
            &format!("expected an integer from 1 to {max}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Config, BenchError> {
        Config::from_args(args.iter().map(|a| (*a).to_string()))
    }

    /// Verifies: BENCH-R040
    #[test]
    fn arguments_set_the_configuration() {
        assert_eq!(parse(&[]), Ok(Config::default()));
        assert_eq!(parse(&["--bench"]), Ok(Config::default()));
        let config = parse(&[
            "--bench",
            "--filter",
            "hash",
            "--iterations",
            "7",
            "--samples",
            "1",
            "--out-dir",
            "/tmp/x",
            "--samples",
            "3",
        ])
        .expect("valid");
        assert_eq!(config.filter.as_deref(), Some("hash"));
        assert_eq!(config.iterations, NonZeroU64::new(7));
        assert_eq!(config.samples, 3);
        assert_eq!(config.out_dir, PathBuf::from("/tmp/x"));
        let max = parse(&["--iterations", "1099511627776", "--samples", "10000"]).expect("max");
        assert_eq!(max.iterations, NonZeroU64::new(MAX_ITERATIONS));
        assert_eq!(max.samples, MAX_SAMPLES);
    }

    /// Verifies: BENCH-R041
    #[test]
    fn invalid_arguments_are_errors() {
        let reason = |args: &[&str]| match parse(args) {
            Err(BenchError::InvalidArgument { argument, reason }) => (argument, reason),
            other => panic!("expected InvalidArgument, got {other:?}"),
        };
        assert_eq!(reason(&["--fast"]).1, "unknown argument");
        assert_eq!(reason(&["positional"]).0, "positional");
        assert_eq!(reason(&["--filter"]).1, "the option needs a value");
        for bad in ["0", "-1", "x", "1.5", "1099511627777"] {
            assert_eq!(
                reason(&["--iterations", bad]),
                (
                    format!("--iterations {bad}"),
                    "expected an integer from 1 to 1099511627776".to_string()
                )
            );
        }
        assert_eq!(reason(&["--samples", "10001"]).0, "--samples 10001");
        assert_eq!(reason(&["--samples", "0"]).0, "--samples 0");
    }

    /// Verifies: BENCH-R042
    #[test]
    fn the_output_directory_defaults_to_the_environment_or_the_workspace_target() {
        assert_eq!(
            default_out_dir(Some(OsString::from("/out"))),
            PathBuf::from("/out")
        );
        let fallback = default_out_dir(None);
        assert!(
            fallback.ends_with("../target/liminal-bench"),
            "{fallback:?}"
        );
        assert!(fallback.starts_with(env!("CARGO_MANIFEST_DIR")));
        assert_eq!(default_out_dir(Some(OsString::new())), fallback);
    }
}
