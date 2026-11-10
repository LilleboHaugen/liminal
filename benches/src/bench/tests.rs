//! Unit tests of the measurement logic on a clock the routine advances: no wall-clock
//! assertions (TESTING.md).

use std::cell::Cell;
use std::num::NonZeroU64;
use std::rc::Rc;
use std::time::Duration;

use super::*;
use crate::Threading;

/// A clock that only moves when a routine advances it.
#[derive(Clone, Default)]
struct FakeClock(Rc<Cell<u64>>);

impl FakeClock {
    fn advance(&self, ns: u64) {
        self.0.set(self.0.get() + ns);
    }
}

impl Clock for FakeClock {
    fn now_ns(&self) -> u64 {
        self.0.get()
    }
}

fn config(warm_up_ns: u64, min_sample_ns: u64, samples: u32) -> Config {
    Config {
        warm_up: Duration::from_nanos(warm_up_ns),
        min_sample_time: Duration::from_nanos(min_sample_ns),
        samples,
        iterations: None,
        filter: None,
        out_dir: std::env::temp_dir(),
    }
}

/// Runs `measure` with a routine that costs `cost_ns` per call; returns (iterations, samples,
/// calls).
fn measured(config: &Config, cost_ns: u64) -> (u64, Vec<f64>, u64) {
    let clock = FakeClock::default();
    let calls = Cell::new(0_u64);
    let (n, samples) = measure(&clock, config, || {
        clock.advance(cost_ns);
        calls.set(calls.get() + 1);
    });
    (n, samples, calls.get())
}

/// Verifies: BENCH-R010
#[test]
fn warm_up_runs_the_routine_until_the_warm_up_time_has_passed() {
    let clock = FakeClock::default();
    let calls = Cell::new(0);
    warm_up(&clock, 10_000, &mut || {
        clock.advance(1_000);
        calls.set(calls.get() + 1);
    });
    assert_eq!(calls.get(), 10);
    // A zero warm-up still calls the routine once.
    warm_up(&clock, 0, &mut || calls.set(calls.get() + 1));
    assert_eq!(calls.get(), 11);
    // In `measure`: warm-up 10 calls, calibration 1 + 2 + 4 + 8 calls, 2 samples of 8 calls.
    assert_eq!(measured(&config(10_000, 5_000, 2), 1_000).2, 10 + 15 + 16);
}

/// Verifies: BENCH-R011
#[test]
fn calibration_picks_the_smallest_power_of_two_reaching_the_sample_time() {
    let run = |min_ns: u64, cost_ns: u64, max: u64| {
        let clock = FakeClock::default();
        calibrate(&clock, min_ns, max, &mut || clock.advance(cost_ns))
    };
    assert_eq!(run(5_000, 1_000, MAX_ITERATIONS), 8);
    assert_eq!(run(8_000, 1_000, MAX_ITERATIONS), 8);
    assert_eq!(run(8_001, 1_000, MAX_ITERATIONS), 16);
    assert_eq!(run(1, 1_000, MAX_ITERATIONS), 1);
    assert_eq!(run(10_000_000, 3, MAX_ITERATIONS), 1 << 22);
    // A routine too fast to reach the sample time stops at the cap.
    assert_eq!(run(1_000, 0, 64), 64);
    assert_eq!(MAX_ITERATIONS, 1 << 40);
}

/// Verifies: BENCH-R012
#[test]
fn samples_are_time_per_iteration_of_calibrated_batches() {
    let (n, samples, _) = measured(&config(0, 10_000, 30), 250);
    assert_eq!(n, 64); // 64 × 250 ns = 16 µs ≥ 10 µs; 32 × 250 ns = 8 µs < 10 µs.
    assert_eq!(samples, vec![250.0; 30]);
}

/// Verifies: BENCH-R013
#[test]
fn fixed_iterations_skip_warm_up_and_calibration() {
    let mut fixed = config(1_000_000, 1_000_000, 4);
    fixed.iterations = NonZeroU64::new(3);
    let (n, samples, calls) = measured(&fixed, 100);
    assert_eq!((n, calls), (3, 12));
    assert_eq!(samples, vec![100.0; 4]);
}

/// Verifies: BENCH-R014
#[test]
fn every_return_value_is_consumed_before_the_next_call() {
    struct Counted<'a>(&'a Cell<i64>);
    impl Drop for Counted<'_> {
        fn drop(&mut self) {
            self.0.set(self.0.get() - 1);
        }
    }
    let live = Cell::new(0_i64);
    let most = Cell::new(0_i64);
    let calls = Cell::new(0_u64);
    let mut fixed = config(0, 0, 2);
    fixed.iterations = NonZeroU64::new(5);
    measure(&FakeClock::default(), &fixed, || {
        live.set(live.get() + 1);
        most.set(most.get().max(live.get()));
        calls.set(calls.get() + 1);
        Counted(&live)
    });
    assert_eq!((calls.get(), live.get(), most.get()), (10, 0, 1));
}

fn bench(target: &str, config: Config) -> Bench {
    Bench::with_clock(target, config, Box::new(FakeClock::default())).expect("valid target")
}

fn fixed(samples: u32, iterations: u64) -> Config {
    let mut config = config(0, 0, samples);
    config.iterations = NonZeroU64::new(iterations);
    config
}

/// Verifies: BENCH-R002, BENCH-R004
#[test]
fn foreign_and_repeated_cases_are_rejected_without_running() {
    let mut bench = bench("liminal_core", fixed(1, 1));
    let calls = Cell::new(0);
    let foreign = Case::new("liminal_math/vec/add").expect("valid");
    assert_eq!(
        bench.run(&foreign, || calls.set(calls.get() + 1)).err(),
        Some(BenchError::CaseOutsideTarget {
            id: "liminal_math/vec/add".into(),
            target: "liminal_core".into()
        })
    );
    let case = Case::new("liminal_core/rng/next_u64").expect("valid");
    assert!(bench.run(&case, || calls.set(calls.get() + 1)).is_ok());
    assert_eq!(calls.get(), 1);
    assert_eq!(
        bench.run(&case, || calls.set(calls.get() + 1)).err(),
        Some(BenchError::DuplicateCase {
            id: "liminal_core/rng/next_u64".into()
        })
    );
    assert_eq!(calls.get(), 1);
    assert_eq!(bench.report().cases.len(), 1);
}

/// Verifies: BENCH-R003
#[test]
fn target_names_are_one_lowercase_segment() {
    for target in ["", "Core", "a/b", "a-b", &"a".repeat(65)] {
        assert_eq!(
            Bench::new(target, Config::default()).err(),
            Some(BenchError::InvalidTarget {
                target: target.to_string()
            })
        );
    }
    assert!(Bench::new("liminal_core", Config::default()).is_ok());
}

/// Verifies: BENCH-R015
#[test]
fn the_filter_skips_cases_whose_id_does_not_contain_it() {
    let mut config = fixed(2, 4);
    config.filter = Some("hash".into());
    let mut bench = bench("liminal_core", config);
    let calls = Cell::new(0);
    let rng = Case::new("liminal_core/rng/next_u64").expect("valid");
    let hash = Case::new("liminal_core/hash/u64")
        .expect("valid")
        .with_elements(NonZeroU64::new(2).expect("non-zero"))
        .multi_threaded();
    assert_eq!(bench.run(&rng, || calls.set(calls.get() + 1)), Ok(None));
    assert_eq!(calls.get(), 0);
    let result = bench
        .run(&hash, || calls.set(calls.get() + 1))
        .expect("runs")
        .expect("not filtered")
        .clone();
    assert_eq!(calls.get(), 8);
    assert_eq!(result.id, "liminal_core/hash/u64");
    assert_eq!(result.threading, Threading::Multi);
    assert_eq!(result.elements, Some(2));
    assert_eq!(result.iterations_per_sample, 4);
    assert_eq!(result.summary.samples, 2);
    assert_eq!(bench.report().cases, vec![result]);
}

/// Verifies: BENCH-R030
#[test]
fn finish_writes_the_report_to_the_target_file() {
    let dir = std::env::temp_dir().join(format!("liminal-bench-unit-{}", std::process::id()));
    let mut config = fixed(1, 1);
    config.out_dir = dir.join("nested");
    // The directory does not exist yet: finish creates it.
    let path = bench("demo", config).finish().expect("written");
    assert_eq!(path, dir.join("nested").join("demo.json"));
    let json = std::fs::read_to_string(&path).expect("readable");
    assert!(json.starts_with("{\n  \"version\": 1,\n  \"target\": \"demo\",\n"));
    // A second finish replaces the file.
    let mut again = fixed(1, 1);
    again.out_dir = dir.join("nested");
    let mut second = bench("demo", again);
    second
        .run(&Case::new("demo/a/b").expect("valid"), || ())
        .expect("runs");
    second.finish().expect("written");
    let replaced = std::fs::read_to_string(&path).expect("readable");
    assert!(replaced.contains("\"id\": \"demo/a/b\""));
    std::fs::remove_dir_all(&dir).expect("cleanup");
}

/// Verifies: BENCH-R033
#[test]
fn an_unwritable_output_directory_is_an_error() {
    let file = std::env::temp_dir().join(format!("liminal-bench-file-{}", std::process::id()));
    std::fs::write(&file, "not a directory").expect("temp file");
    let mut config = fixed(1, 1);
    config.out_dir = file.join("sub");
    match bench("demo", config).finish() {
        Err(BenchError::WriteFailed { path, reason }) => {
            assert_eq!(
                path,
                file.join("sub").join("demo.json").display().to_string()
            );
            assert_ne!(reason, "");
        }
        other => panic!("expected WriteFailed, got {other:?}"),
    }
    std::fs::remove_file(&file).expect("cleanup");
}
