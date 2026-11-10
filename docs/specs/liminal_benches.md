---
crate: liminal_benches
req_prefix: BENCH
layer: 5
maturity: planned
spec_version: 2
last_reviewed: 2026-11-10
adrs: [ADR-0003, ADR-0017, ADR-0018, ADR-0023]
---

# Spec: `liminal_benches`

## 1. Purpose

`liminal_benches` is Liminal's benchmark crate (ADR-0023). Its library is the benchmark harness:
`std` only, it warms a routine up, calibrates the iterations per sample, takes samples, computes
median, median absolute deviation (MAD), minimum, maximum and throughput, and writes one JSON
result file per bench target. Its bench targets measure the engine crates through their public
API. It is one separate crate in the tools layer because layer rules (XTASK-R032, R036) forbid
engine crates from depending on a layer 5 crate, and because engine crates have no `benches/`
targets of their own (ADR-0023 Decision 1). `cargo xtask bench` (M1-T017) and the CI
instruction-count job read the result files.

## 2. Scope

**In scope:**
- The harness: `Bench`, `Case`, `Config`, `Summary`, `Report`, `Metadata` (§5).
- Case IDs `<crate>/<group>/<case>` and the single-threaded marker the instruction-count gate
  uses (ADR-0023 item 4).
- The JSON result file, format version 1, with machine metadata (PERFORMANCE.md "Machines").
- The command line of bench binaries (`cargo bench -p liminal_benches --bench <crate> -- …`).
- Bench target layout, and the bench target `liminal_core` (state hasher and RNG).
- The crate's error enum `BenchError` (codes `LIM-BENCH-NNNN`, §8).

**Out of scope / non-goals:**
- Comparing results with baselines, writing baseline files, the regression threshold:
  `cargo xtask bench` in `xtask` (M1-T017, ADR-0023 item 3).
- Instruction counting under Valgrind and the CI job: the instruction-count task (ADR-0023 item 4).
  The harness only provides fixed iteration counts (BENCH-R013) and the threading marker.
- Comparison benchmarks against `bevy_ecs`, `hecs`, `glam`, `rayon`: the separate workspace
  `benches/competitive/` (`liminal_benches_competitive`), which uses this harness.
- Checking that engine crates have no `benches/` targets: `xtask check-arch` (XTASK-R090).
- Statistical tests (confidence intervals, outlier classification, bootstrap), HTML reports,
  plots. Asynchronous or setup-per-iteration routines.
- Benchmarking private items (ADR-0023: benchmarks see only public API; optimized paths expose
  their oracle through a `reference` module, PERFORMANCE.md).

## 3. Dependencies

- Engine crates: `liminal_core` (layer 0) as a normal dependency: `BenchError` implements
  `liminal_core::error::EngineError` (ADR-0017), and the `liminal_core` bench target measures it.
  Later bench targets add the crate they measure (any lower layer, XTASK-R032).
- External crates: none (`architecture.toml`: `external = []`, `dev_external = []`; ADR-0003,
  ADR-0023 rejects `criterion` and `divan`). `check-arch` (XTASK-R033, R034) enforces this.
- `unsafe = "forbid"` (ADR-0018): `src/lib.rs` and every bench target start with
  `#![forbid(unsafe_code)]`.
- `deterministic = false`: the harness reads the wall clock and the environment by design.

## 4. Concepts

- **Routine.** A closure `FnMut() -> R`; one call is one **iteration**. Its return value is passed
  through `std::hint::black_box` so the compiler cannot remove the work.
- **Case.** One measured routine with a stable **case ID** `<crate>/<group>/<case>`: three
  segments of 1 to 64 characters of `[a-z0-9_]`, separated by `/`. The first segment is the crate
  the case measures. A case may declare **elements** (items one iteration processes, ≥ 1) for
  throughput, and is **single-threaded** unless marked multi-threaded.
- **Bench target.** A `[[bench]]` target of this crate, `harness = false`, at
  `benches/benches/<crate>.rs`, named after the crate it measures. It has one result file,
  `<out_dir>/<crate>.json`. Every case of the target has the target name as its first segment.
- **Sample.** One timed run of `n` consecutive iterations, where `n` (iterations per sample) is
  the same for every sample of a case. A sample's value is its time divided by `n`, in
  nanoseconds per iteration (an `f64`). The time includes the loop and the `black_box` call;
  nothing is subtracted.
- **Calibrated mode** (default): warm-up, then calibration of `n`, then the samples.
  **Fixed mode** (`--iterations k`): `n = k`, no warm-up, no calibration. The CI instruction-count
  gate runs each case once with `k` and once with `2k` iterations (ADR-0023 item 4).
- **Summary.** Median, MAD (median of absolute deviations from the median, **unscaled**: no
  1.4826 factor), minimum and maximum of a case's samples, and the sample count.

Flow of one bench binary run:

```
main ─ Bench::from_env(target) ─┬─ run(case 1, routine) ─ warm-up ─ calibrate ─ samples ─ Summary
                                ├─ run(case 2, routine) ─ …
                                └─ finish() ─ <out_dir>/<target>.json
```

## 5. Public API

All items are re-exported at the crate root; modules are private.

```rust
pub struct Case { /* id, elements, threading */ }
impl Case {
    pub fn new(id: &str) -> Result<Case, BenchError>;            // R001
    pub fn with_elements(self, elements: NonZeroU64) -> Case;    // R023
    pub fn multi_threaded(self) -> Case;                         // R005
    pub fn id(&self) -> &str;
    pub fn crate_name(&self) -> &str;                            // first segment
    pub fn elements(&self) -> Option<u64>;
    pub fn threading(&self) -> Threading;
}
pub enum Threading { Single, Multi }

pub const MAX_ITERATIONS: u64 = 1 << 40;
pub const MAX_SAMPLES: u32 = 10_000;
pub const OUT_DIR_ENV: &str = "LIMINAL_BENCH_OUT_DIR";
pub struct Config {
    pub warm_up: Duration,            // default 300 ms
    pub min_sample_time: Duration,    // default 10 ms (ADR-0023)
    pub samples: u32,                 // default 30 (ADR-0023), 1..=MAX_SAMPLES
    pub iterations: Option<NonZeroU64>, // default None; Some = fixed mode, ≤ MAX_ITERATIONS
    pub filter: Option<String>,       // default None
    pub out_dir: PathBuf,             // default R042
}
impl Default for Config;
impl Config {
    pub fn from_args(args: impl IntoIterator<Item = String>) -> Result<Config, BenchError>; // R040, R041
}

pub struct Bench { /* target, config, results */ }
impl Bench {
    pub fn new(target: &str, config: Config) -> Result<Bench, BenchError>;     // R003
    pub fn from_env(target: &str) -> Result<Bench, BenchError>;                // args of this process
    pub fn run<R>(&mut self, case: &Case, routine: impl FnMut() -> R)
        -> Result<Option<&CaseResult>, BenchError>;                           // R002, R004, R010–R015
    pub fn report(&self) -> Report;                                            // with Metadata::detect()
    pub fn finish(self) -> Result<PathBuf, BenchError>;                        // R030, R033
}

pub struct Summary { pub samples: u32, pub median_ns: f64, pub mad_ns: f64,
                     pub min_ns: f64, pub max_ns: f64 }
impl Summary {
    pub fn from_samples(samples: &[f64]) -> Option<Summary>;                   // R020–R022, R024
    pub fn throughput_per_s(&self, elements: Option<u64>) -> Option<f64>;      // R023
}

pub const FORMAT_VERSION: u32 = 1;
pub struct CaseResult { pub id: String, pub threading: Threading, pub elements: Option<u64>,
                        pub iterations_per_sample: u64, pub summary: Summary }
impl Display for CaseResult;   // one terminal line; wording not a stable interface
pub struct Report { pub target: String, pub metadata: Metadata, pub cases: Vec<CaseResult> }
impl Report { pub fn to_json(&self) -> String; }                               // R031, R034, R035

pub const UNKNOWN: &str = "unknown";
pub struct Metadata { pub machine_id: String, pub os: String, pub arch: String,
                      pub os_version: String, pub toolchain: String, pub commit: String,
                      pub power: String }
impl Metadata { pub fn detect() -> Metadata; }                                 // R032

#[non_exhaustive]
pub enum BenchError { InvalidCaseId { id }, InvalidTarget { target },
    CaseOutsideTarget { id, target }, DuplicateCase { id },
    InvalidArgument { argument, reason }, WriteFailed { path, reason } }       // §8
impl EngineError for BenchError; impl Display; impl std::error::Error;
```

### Command line of a bench binary

`cargo bench -p liminal_benches --bench <crate> -- [arguments]`. `Config::from_args` reads the
arguments after the program name:

| Argument | Effect |
|---|---|
| `--bench` | Ignored: cargo passes it to `harness = false` targets. |
| `--filter <text>` | Run only cases whose ID contains `<text>` (substring, case-sensitive). |
| `--iterations <n>` | Fixed mode with `n` iterations per sample, 1 ≤ n ≤ 2⁴⁰. |
| `--samples <n>` | Samples per case, 1 ≤ n ≤ 10 000. |
| `--out-dir <path>` | Directory of the result file. |

A later repetition of an option replaces the earlier value. The binary prints one line per case
(`CaseResult`'s `Display`) and `wrote <path>` to standard output, exits 0 on success, and prints
the error's `Display` to standard error and exits 1 on any `BenchError`.

### Result file (format version 1)

`<out_dir>/<target>.json`, UTF-8, `\n` line ends, two-space indentation, a final `\n`, keys in
exactly this order:

```json
{
  "version": 1,
  "target": "liminal_core",
  "machine": {
    "id": "unknown",
    "os": "macos",
    "arch": "aarch64",
    "os_version": "27.2",
    "toolchain": "rustc 1.99.1 (58cf8f9c3 2026-10-13)",
    "commit": "e652b0c23dbb127718bc1cde8a6b1d5cb6fd3bd8",
    "power": "ac"
  },
  "cases": [
    {
      "id": "liminal_core/hash/state_hasher_bytes_1k",
      "single_threaded": true,
      "elements": 1024,
      "iterations_per_sample": 524288,
      "samples": 30,
      "median_ns": 35.782,
      "mad_ns": 0.208,
      "min_ns": 35.284,
      "max_ns": 36.603,
      "throughput_per_s": 28617503886.3
    }
  ]
}
```

| Key | Type | Meaning |
|---|---|---|
| `version` | integer | `1`. A change of a key's name, type or meaning bumps it. |
| `target` | string | Bench target name = file stem. |
| `machine.*` | string | `Metadata` fields (`id` = `machine_id`), never empty; `"unknown"` if not detectable. |
| `cases` | array | One object per case that ran, in run order; `[]` if none ran. |
| `id` | string | Case ID. |
| `single_threaded` | bool | `false` iff the case is marked multi-threaded. |
| `elements` | integer or `null` | Elements per iteration. |
| `iterations_per_sample` | integer | `n`. |
| `samples` | integer | Sample count. |
| `median_ns`, `mad_ns`, `min_ns`, `max_ns` | number | Nanoseconds per iteration. |
| `throughput_per_s` | number or `null` | Elements per second at the median; `null` without elements or with median 0. |

Numbers are written in Rust's shortest round-trip decimal form for `f64` (no exponent); NaN and
infinities are written as `null`.

### Machine metadata

Each field is taken from its environment variable if set and not empty after trimming, else
detected, else `"unknown"`. Detection spawns processes in the workspace root; a missing program
or a non-zero exit gives `"unknown"`.

| Field | Variable | Detection |
|---|---|---|
| `machine_id` | `LIMINAL_BENCH_MACHINE_ID` | none |
| `os` | — | `std::env::consts::OS` |
| `arch` | — | `std::env::consts::ARCH` |
| `os_version` | `LIMINAL_BENCH_OS_VERSION` | macOS: `sw_vers -productVersion`; other Unix: `uname -r`; Windows: none |
| `toolchain` | `LIMINAL_BENCH_TOOLCHAIN` | first line of `$RUSTC -V` (`rustc` if `RUSTC` is unset) |
| `commit` | `LIMINAL_BENCH_COMMIT` | `git rev-parse HEAD`, plus `-dirty` if `git status --porcelain` prints anything |
| `power` | `LIMINAL_BENCH_POWER` | macOS: first line of `pmset -g ps` contains `'AC Power'` → `ac`, `'Battery Power'` → `battery`; else none |

Assumption: the `rustc` the binary finds at run time is the one that compiled it. `cargo bench`
runs the binary in `benches/`, inside the repository, so `rust-toolchain.toml` selects the same
toolchain through rustup. Unverified when `RUSTC` points elsewhere.

## 6. Requirements

### Cases and targets (R001–R009)

| ID | Level | Requirement |
|---|---|---|
| BENCH-R001 | MUST | `Case::new(id)` succeeds exactly when `id` is three segments separated by `/`, each 1 to 64 bytes of `[a-z0-9_]`; otherwise it returns `BenchError::InvalidCaseId` (LIM-BENCH-0001) carrying `id`. |
| BENCH-R002 | MUST | `Bench::run` returns `BenchError::CaseOutsideTarget` (LIM-BENCH-0003) without calling the routine if the case ID's first segment is not the bench target name. |
| BENCH-R003 | MUST | `Bench::new(target, _)` returns `BenchError::InvalidTarget` (LIM-BENCH-0002) unless `target` is 1 to 64 bytes of `[a-z0-9_]`. |
| BENCH-R004 | MUST | A second `Bench::run` with a case ID already given to the same `Bench` (whether it ran or was filtered out) returns `BenchError::DuplicateCase` (LIM-BENCH-0004) without calling the routine, and the report keeps one result for that ID. |
| BENCH-R005 | MUST | A case is `Threading::Single` unless `multi_threaded()` was called; its result and JSON (`single_threaded`) carry the marker. |

### Measurement (R010–R019)

| ID | Level | Requirement |
|---|---|---|
| BENCH-R010 | MUST | In calibrated mode, `run` first calls the routine repeatedly until at least `Config::warm_up` has elapsed since warm-up started, at least once; these calls are in no sample. |
| BENCH-R011 | MUST | In calibrated mode, after warm-up, the iterations per sample `n` is the smallest power of two from 1 to 2⁴⁰ for which one timed run of `n` iterations takes at least `Config::min_sample_time`, or 2⁴⁰ if none does. Calibration runs are in no sample. |
| BENCH-R012 | MUST | `run` takes `Config::samples` samples of `n` iterations each (`max(samples, 1)` when built without `from_args`); each sample's value is its elapsed nanoseconds divided by `n`. |
| BENCH-R013 | MUST | With `Config::iterations = Some(k)`, `run` does no warm-up and no calibration: the routine is called exactly `k × samples` times and `iterations_per_sample` is `k`. |
| BENCH-R014 | MUST | Every return value of the routine is passed to `std::hint::black_box` and dropped before the next call. |
| BENCH-R015 | MUST | With `Config::filter = Some(text)`, a case whose ID does not contain `text` is not run (`run` returns `Ok(None)`, the routine is not called) and is not in the report. |

### Statistics (R020–R029)

| ID | Level | Requirement |
|---|---|---|
| BENCH-R020 | MUST | `Summary::median_ns` is the middle value of the sorted samples for an odd count, and the mean of the two middle values for an even count. |
| BENCH-R021 | MUST | `Summary::mad_ns` is the median (as R020) of the absolute differences between each sample and the median, without a scale factor. |
| BENCH-R022 | MUST | `Summary::min_ns` and `max_ns` are the smallest and largest sample, and `samples` the sample count. |
| BENCH-R023 | MUST | `Summary::throughput_per_s(Some(e))` is `e × 10⁹ / median_ns` elements per second when `median_ns > 0`; it is `None` for `None` or a median of 0. |
| BENCH-R024 | MUST | `Summary::from_samples` returns `None` for an empty slice, a slice containing NaN or an infinity, or a slice longer than `u32::MAX` elements (its count does not fit `Summary::samples`), and `Some` otherwise. `Bench::run` passes at most `u32::MAX` samples (`Config::samples` is a `u32`), so it always gets `Some`. |

### Result file (R030–R039)

| ID | Level | Requirement |
|---|---|---|
| BENCH-R030 | MUST | `Bench::finish` creates `Config::out_dir` (with parents) if missing, writes `Report::to_json()` to `<out_dir>/<target>.json`, replacing an existing file, and returns that path. |
| BENCH-R031 | MUST | `Report::to_json` produces exactly the layout of §5 "Result file": the keys in the order listed, `version` 1, two-space indentation, `\n` line ends and a final `\n`. |
| BENCH-R032 | MUST | `Metadata::detect` fills every field as §5 "Machine metadata" states: environment variable first (trimmed, if not empty), then detection, then `"unknown"`; no field is empty. |
| BENCH-R033 | MUST | If the directory or the file cannot be written, `finish` returns `BenchError::WriteFailed` (LIM-BENCH-0006) with the file path and the operating system's error text. |
| BENCH-R034 | MUST | JSON strings escape `"` and `\` with a backslash and U+0000 to U+001F as `\u00XX` (RFC 8259 §7); NaN and infinite numbers are written as `null`. |
| BENCH-R035 | MUST | `cases` lists the cases in the order `run` measured them. |

### Command line (R040–R049)

| ID | Level | Requirement |
|---|---|---|
| BENCH-R040 | MUST | `Config::from_args` starts from `Config::default()` and applies the arguments of §5 "Command line" in order; `--bench` is ignored and a repeated option replaces the earlier value. |
| BENCH-R041 | MUST | `Config::from_args` returns `BenchError::InvalidArgument` (LIM-BENCH-0005) for an unknown argument (including a positional one), an option without a value, or an `--iterations` or `--samples` value that is not a decimal integer in its range (1 to 2⁴⁰, 1 to 10 000). |
| BENCH-R042 | MUST | `Config::default().out_dir` is `$LIMINAL_BENCH_OUT_DIR` if set and not empty, else `<workspace root>/target/liminal-bench`, where the workspace root is the parent of this crate's manifest directory at compile time. |

### Errors (R050–R059)

| ID | Level | Requirement |
|---|---|---|
| BENCH-R050 | MUST | Every `BenchError` variant has the code of §8 and passes `liminal_core::error::validate_error` with area `BENCH`. |

### Bench targets (R060–R069)

| ID | Level | Requirement |
|---|---|---|
| BENCH-R060 | MUST | Every `[[bench]]` target in `benches/Cargo.toml` has `harness = false`, a name equal to a crate directory under `crates/`, and its source at `benches/benches/<name>.rs`; every file in `benches/benches/` is such a target. |
| BENCH-R061 | SHOULD | The `liminal_core` target measures, through the public API: `liminal_core/hash/state_hasher_u64` (one `write_u64` and `finish` on a new `StateHasher`), `liminal_core/hash/state_hasher_bytes_1k` (`write_bytes` of 1024 bytes and `finish`, 1024 elements), `liminal_core/rng/next_u64`, `liminal_core/rng/below_u32_1000` (bound 1000) and `liminal_core/rng/next_f64` (one call each on an `Rng` with world seed 42 and stream `"bench"`), all single-threaded. |

### Assignment to implementation tasks

Every MUST requirement is assigned to exactly one task; the IDs are in the tasks' `requirements`
frontmatter. BENCH-R061 (SHOULD) is implemented by M1-T016 but not in its frontmatter: it has no
`Verifies:` test, because `cargo test` does not run bench targets; its evidence is the run of the
target recorded in the M1-T016 journal entry.

| Task | Requirements |
|---|---|
| M1-T016 (harness, `liminal_core` target) | BENCH-R001 to BENCH-R005, BENCH-R010 to BENCH-R015, BENCH-R020 to BENCH-R024, BENCH-R030 to BENCH-R035, BENCH-R040 to BENCH-R042, BENCH-R050, BENCH-R060 |
| M1-T017 (`cargo xtask bench`) | none in this spec; it adds XTASK requirements and reads the result file of BENCH-R031 |

M1-T016 also implements XTASK-R090 (`check-arch`: engine crates have no bench targets) in
`docs/specs/xtask.md`.

## 7. Invariants

- INV-1: Every `CaseResult` has `iterations_per_sample` in 1 to 2⁴⁰ and `summary.samples ≥ 1`,
  with `min_ns ≤ median_ns ≤ max_ns` and `mad_ns ≥ 0`, all finite (measurement produces only
  finite, non-negative samples; `Bench::run` states this as an `expect("invariant: …")`).
- INV-2: Case IDs in one report are distinct and all start with the report's target name
  (R002, R004).
- INV-3: The result file is a pure function of the `Report` value (R031); the same report gives
  the same bytes.

## 8. Error codes

| Code | Meaning | Help |
|---|---|---|
| LIM-BENCH-0001 | Case ID is not `<crate>/<group>/<case>` (R001) | Use three segments of 1 to 64 characters of `[a-z0-9_]`, separated by `/` |
| LIM-BENCH-0002 | Bench target name is not 1 to 64 characters of `[a-z0-9_]` (R003) | Name the bench target after the crate it measures |
| LIM-BENCH-0003 | Case does not belong to the bench target (R002) | Start the case ID with the target name, or move the case to its crate's target |
| LIM-BENCH-0004 | Case ID run twice in one bench target (R004) | Give every case its own ID |
| LIM-BENCH-0005 | Invalid bench binary argument (R041) | Use `--filter`, `--iterations`, `--samples`, `--out-dir`; numbers are positive integers |
| LIM-BENCH-0006 | Result file could not be written (R033) | Check the output directory, or pass another with `--out-dir` |

Context keys: `id`; `target`; `id`, `target`; `id`; `argument`, `reason`; `path`, `reason`.

## 9. Performance budgets

None. `liminal_benches` is a tool; it has no runtime budget. The harness's own overhead per
iteration (loop and `black_box`) is part of every sample and is not subtracted (§4); the first
baselines that show its size are recorded by M1-T017 (`ci/perf/<machine-id>/liminal_core.json`).

## 10. Test plan

No path in this crate is an optimized variant of another; the statistics are the simple
sort-based definitions, so no differential oracle applies (AGENTS.md rule 4).

| Area | Test kinds | Oracle |
|---|---|---|
| Case IDs, target names (R001, R003, R005) | unit | Hand-picked valid and invalid IDs from the grammar of §4 |
| Measurement (R010–R015) | unit, with a fake clock the routine advances (no wall-clock assertions, TESTING.md) | Hand-computed call counts, iteration counts and sample values |
| Statistics (R020–R024) | unit, doc test | Hand-computed medians, MADs, minima, maxima, rates |
| Result file (R030, R031, R033–R035) | unit (exact expected text), integration (`benches/tests/output.rs`: a real `Bench` with fixed iterations writes a file that a test-local JSON reader parses and checks against the §5 schema) | §5 "Result file" |
| Metadata (R032) | unit with injected environment and process runner; one real `detect()` | §5 "Machine metadata" table |
| Command line (R040–R042) | unit | §5 "Command line" table |
| Errors (R050) | unit | `validate_error`, §8 table |
| Bench target layout (R060) | integration (reads `benches/Cargo.toml` and the directory) | R060 text |
| `liminal_core` target (R061) | run by hand with `cargo bench -p liminal_benches --bench liminal_core`; output recorded in the M1-T016 journal | R061 case list |

The bench binaries themselves are not run by `cargo xtask ci` (`cargo test` does not build bench
targets; `clippy --all-targets` compiles them). M1-T017's `cargo xtask bench` runs them.

## 11. Determinism

Not deterministic (`deterministic = false`): measured times vary between runs by design. The
result file is deterministic for a given `Report` value (INV-3), and case order is the run order.

## 12. Open questions

1. **Machine IDs.** The harness records `$LIMINAL_BENCH_MACHINE_ID` or `"unknown"`; it does not
   derive an ID from the hardware. Which IDs exist (for example one for the reference machine)
   and whether `cargo xtask bench` sets the variable: M1-T017.
2. **Exact case selection for the instruction-count job.** `--filter` is a substring match, so
   `liminal_core/rng/next_u64` also selects a later case `liminal_core/rng/next_u64_x`. The
   instruction-count task decides whether it needs an exact-ID option (a `spec_version` bump
   here) or runs the whole target with `--iterations k` and `2k`.
3. **Power state off macOS.** `power` is `"unknown"` on Linux and Windows unless
   `LIMINAL_BENCH_POWER` is set. Resolve when baselines are recorded on a non-macOS machine (the
   low-end target, PERFORMANCE.md).
4. **Raw samples.** The file stores the summary only. If `cargo xtask bench` needs the samples
   (for example for another regression test than ADR-0023's threshold), add them with a
   `version` bump.

## 13. Maturity

Current: `planned` (spec written with the first implementation, M1-T015 with M1-T016,
2026-11-10). Next target: `experimental`, after the spec review of M1-T015 (REVIEW.md §2).

- [ ] Spec follows `docs/specs/_TEMPLATE.md` and passed a spec review (pending: review of
      M1-T015).
- [x] Compiles, all lints pass, registered in `ci/coverage.toml` (M1-T016, `cargo xtask ci`).
- [x] Every implemented behaviour has tests (M1-T016: every MUST has a `Verifies:` test).

## 14. History

| spec_version | Date | Change | By task |
|---|---|---|---|
| 1 | 2026-11-10 | Initial spec | M1-T015 |
| 2 | 2026-11-10 | BENCH-R024 states the `None` result for a slice longer than `u32::MAX` (review of M1-T016, finding 5) | M1-T016 |
