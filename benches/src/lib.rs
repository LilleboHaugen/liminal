//! Liminal's benchmark harness and the engine benchmarks (ADR-0023).
//!
//! The library is the harness: `std` only, no external crate. A bench target
//! (`benches/benches/<crate>.rs`, `harness = false`) builds a [`Bench`], runs each [`Case`]
//! through the benchmarked crate's public API, and calls [`Bench::finish`], which writes one JSON
//! result file per target (format version 1, [`Report::to_json`]). For each case the harness
//! warms up, calibrates the iterations per sample so a sample takes at least 10 ms, takes 30
//! samples and reports median, median absolute deviation, minimum, maximum and throughput
//! ([`Summary`]).
//!
//! Specification: `docs/specs/liminal_benches.md`.
//!
//! Run a target with `cargo bench -p liminal_benches --bench <crate> -- [--filter <text>]
//! [--iterations <n>] [--samples <n>] [--out-dir <path>]` ([`Config::from_args`]).
//!
//! # Examples
//!
//! The `main` of a bench target:
//!
//! ```no_run
//! use std::hint::black_box;
//! use liminal_benches::{Bench, BenchError, Case};
//!
//! fn main() -> Result<(), BenchError> {
//!     let mut bench = Bench::from_env("liminal_core")?;
//!     let mut x = 0_u64;
//!     bench.run(&Case::new("liminal_core/demo/add")?, || {
//!         x = x.wrapping_add(black_box(3));
//!         x
//!     })?;
//!     bench.finish()?;
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]

mod bench;
mod case;
mod config;
mod error;
mod machine;
mod report;
mod stats;

pub use bench::Bench;
pub use case::{Case, Threading};
pub use config::{Config, MAX_ITERATIONS, MAX_SAMPLES, OUT_DIR_ENV};
pub use error::BenchError;
pub use machine::{Metadata, UNKNOWN};
pub use report::{CaseResult, FORMAT_VERSION, Report};
pub use stats::Summary;

impl core::fmt::Display for CaseResult {
    /// One line for a terminal: `<id>: median <m> ns, MAD <d> ns, min <a> ns, max <b> ns
    /// (<n> iterations × <s> samples)`, then `, <t> elements/s` if the case has elements.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = &self.summary;
        write!(
            f,
            "{}: median {:.3} ns, MAD {:.3} ns, min {:.3} ns, max {:.3} ns ({} iterations × {} \
             samples)",
            self.id,
            s.median_ns,
            s.mad_ns,
            s.min_ns,
            s.max_ns,
            self.iterations_per_sample,
            s.samples
        )?;
        if let Some(rate) = s.throughput_per_s(self.elements) {
            write!(f, ", {rate:.0} elements/s")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_results_print_one_line() {
        let summary = Summary::from_samples(&[1.0, 2.0, 3.0]).expect("valid");
        let mut result = CaseResult {
            id: "a/b/c".into(),
            threading: Threading::Single,
            elements: None,
            iterations_per_sample: 8,
            summary,
        };
        assert_eq!(
            result.to_string(),
            "a/b/c: median 2.000 ns, MAD 1.000 ns, min 1.000 ns, max 3.000 ns (8 iterations × 3 \
             samples)"
        );
        result.elements = Some(4);
        assert!(result.to_string().ends_with(", 2000000000 elements/s"));
    }
}
