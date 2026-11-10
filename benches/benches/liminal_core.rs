//! Benchmarks of `liminal_core` through its public API (spec `docs/specs/liminal_benches.md`
//! §4 "Bench targets"): the state hasher and the RNG.
//!
//! `cargo bench -p liminal_benches --bench liminal_core -- [options]` writes
//! `liminal_core.json` to the output directory (`--out-dir`, else `$LIMINAL_BENCH_OUT_DIR`, else
//! `target/liminal-bench/`).

#![forbid(unsafe_code)]
#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "a bench binary reports its results on the terminal"
)]

use std::hint::black_box;
use std::num::{NonZeroU32, NonZeroU64};
use std::process::ExitCode;

use liminal_benches::{Bench, BenchError, Case};
use liminal_core::hash::StateHasher;
use liminal_core::rng::{Rng, StreamKey};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), BenchError> {
    let mut bench = Bench::from_env("liminal_core")?;
    let report = |result: Option<&liminal_benches::CaseResult>| {
        if let Some(result) = result {
            println!("{result}");
        }
    };

    report(
        bench.run(&Case::new("liminal_core/hash/state_hasher_u64")?, || {
            let mut hasher = StateHasher::new();
            hasher.write_u64(black_box(0x0123_4567_89ab_cdef));
            hasher.finish()
        })?,
    );

    let bytes = vec![0x5a_u8; 1024];
    let kib = NonZeroU64::new(1024).expect("invariant: 1024 is not zero");
    report(bench.run(
        &Case::new("liminal_core/hash/state_hasher_bytes_1k")?.with_elements(kib),
        || {
            let mut hasher = StateHasher::new();
            hasher.write_bytes(black_box(&bytes));
            hasher.finish()
        },
    )?);

    let mut rng = Rng::new(42, StreamKey::from_name("bench"));
    report(bench.run(&Case::new("liminal_core/rng/next_u64")?, || rng.next_u64())?);

    let bound = NonZeroU32::new(1000).expect("invariant: 1000 is not zero");
    report(
        bench.run(&Case::new("liminal_core/rng/below_u32_1000")?, || {
            rng.below_u32(black_box(bound))
        })?,
    );

    report(bench.run(&Case::new("liminal_core/rng/next_f64")?, || rng.next_f64())?);

    let path = bench.finish()?;
    println!("wrote {}", path.display());
    Ok(())
}
