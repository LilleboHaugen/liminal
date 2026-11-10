---
id: M1-T016
title: Implement the liminal_benches harness
milestone: M1
status: review
crates: [liminal_benches]
depends_on: [M1-T015]
requirements: [BENCH-R001, BENCH-R002, BENCH-R003, BENCH-R004, BENCH-R005, BENCH-R010, BENCH-R011, BENCH-R012, BENCH-R013, BENCH-R014, BENCH-R015, BENCH-R020, BENCH-R021, BENCH-R022, BENCH-R023, BENCH-R024, BENCH-R030, BENCH-R031, BENCH-R032, BENCH-R033, BENCH-R034, BENCH-R035, BENCH-R040, BENCH-R041, BENCH-R042, BENCH-R050, BENCH-R060, XTASK-R090]
---

# M1-T016: Implement the `liminal_benches` harness

## Goal

Benchmarks can be written and produce the JSON result files ADR-0023 defines.

## Context

- `docs/specs/liminal_benches.md`; ADR-0023 items 1–2.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** The crate at `benches/`, the harness, one bench target measuring the `liminal_core` hasher and RNG; `check-arch` rule that engine crates have no `benches/` targets (ADR-0023 "Verification"; XTASK spec change).

**Out:** Comparison with baselines (`cargo xtask bench`): next task.

## Acceptance criteria

- [x] The crate exists at the path in `architecture.toml`, is a workspace member, has
      `#![forbid(unsafe_code)]` if `unsafe = "forbid"`, and has a floor in `ci/coverage.toml`.
- [x] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [x] New public items are documented; primary entry points have runnable `# Examples`.
- [x] `cargo xtask ci` passes with no failed or skipped step.
- [x] Statistics functions have unit tests with hand-computed expected values (no wall-clock assertions, TESTING.md).
- [x] Running the bench target writes a JSON file that matches the spec's format (integration test on a trivial case with a fixed iteration count).

## Test requirements

Requirements in frontmatter; the new XTASK requirement for engine `benches/` targets.

## Notes

_None._

## Handoff

Done with M1-T015 as one unit (ADR-0029 Decision 3), 2026-11-10. Status `review`.

- Crate `benches/` (`liminal_benches`, workspace member, `#![forbid(unsafe_code)]`, floor 96.8 in
  `ci/coverage.toml`): `src/{lib,bench,case,config,stats,report,machine,error}.rs`,
  `src/bench/tests.rs` (measurement tests on a fake clock the routine advances), bench target
  `benches/liminal_core.rs` (five cases, spec BENCH-R061), integration tests
  `tests/output.rs` (a real `Bench` with fixed iterations writes a file that a test-local JSON
  reader checks against spec §5; bench target layout, R060). Normal dependency `liminal_core`
  (spec §3).
- `check-arch` XTASK-R090 (`xtask/src/check_arch/benches.rs`, test
  `r090_engine_crates_have_no_benchmark_targets`); `docs/specs/xtask.md` `spec_version` 27.
  Seeded once: a `crates/liminal_core/benches/seed.rs` gave one R090 finding, then removed.
- Scope addition, with reason: cargo-llvm-cov's default file filter skips every `benches/`
  directory, so the whole crate measured "no lines". `xtask` now passes its own filter
  (`IGNORE_FILENAME_REGEX` in `xtask/src/repo/coverage.rs`, XTASK-R060 reworded); `liminal_core`
  and `xtask` line counts are unchanged by it (1753/1764, 5940/6047).
- Two xtask tests adapted: `r031_members_are_registered_with_their_path_and_kind` now asserts no
  R031 finding (its layer 1 test crate has a `bench` target, which is R090 now);
  `loads_the_rust_files_of_the_real_workspace` knows `liminal_benches` lives at `benches/`.
- Not done here: `cargo xtask bench` and baselines (M1-T017); instruction counts (later task).
- Trap: `cargo test` does not build bench targets, so the bench binary is only compiled by
  `clippy --all-targets` in CI; run it by hand with
  `cargo bench -p liminal_benches --bench liminal_core`.

Second round, 2026-11-10 (implementation sub-agent, after the first "changes required"):
all five required changes addressed. Status `review`.

1. BENCH-R004: `a_filtered_out_case_still_counts_as_given` (`benches/src/bench/tests.rs`).
2. BENCH-R032: `empty_and_blank_overrides_fall_back_to_detection` (`""`, `" "`, `"\t\n"` fall
   back to detection and to `unknown`) in `benches/src/machine.rs`.
3. BENCH-R032: `the_toolchain_comes_from_the_rustc_named_by_rustc` (injected `RUSTC` is the
   program run; `rustc` without it).
4. BENCH-R012: `zero_samples_without_from_args_take_one_sample` (`measure` and `Bench::run`
   with `samples: 0`).
5. BENCH-R024: spec changed, code kept. `docs/specs/liminal_benches.md` `spec_version` 2: R024
   names the `None` result for a slice longer than `u32::MAX` (its count does not fit
   `Summary::samples: u32`); `Bench::run` never reaches it. Doc comment of
   `Summary::from_samples` updated. That clause has no test: it needs a slice of more than
   2³² `f64` (32 GiB).

Each reviewer mutant (1–4) was re-applied by hand and makes exactly one test fail; reverted.
`cargo xtask ci`: 12 steps pass, none skipped (macOS 27.2 arm64). Coverage floor unchanged.

## Review

2026-11-10, review sub-agent (ADR-0026, ADR-0029 Decision 3, ADR-0030; journal
`docs/journal/2026-11-10-M1-T015-M1-T016-review.md`). Outcome: **changes required** (first
"changes required" outcome of this task).

Required changes:

1. **B4, BENCH-R004.** "whether it ran or was filtered out" is untested. Mutant (reverted):
   in `benches/src/bench.rs` `Bench::run`, move the filter check before `self.seen.insert`;
   every test passes, yet a second `run` of a filtered-out ID then returns `Ok(None)` instead
   of `DuplicateCase`. Add a test that runs a filtered-out case twice.
2. **B4, BENCH-R032.** The "not empty after trimming" rule and "no field is empty" are untested.
   Mutant (reverted): remove `.filter(|v| !v.is_empty())` in `benches/src/machine.rs`
   `detect_with`; every test passes, yet `LIMINAL_BENCH_MACHINE_ID=" "` gives `id: ""`. Test an
   empty and a whitespace-only variable falling back to detection (and to `unknown`).
3. **B4, BENCH-R032 / spec §5 "Machine metadata".** Detection through `$RUSTC -V` is untested.
   Mutant (reverted): `let rustc = "rustc".to_string();`; every test passes. Test that the
   injected `RUSTC` value is the program run.
4. **B4, BENCH-R012.** `max(samples, 1)` when built without `from_args` is untested. Mutant
   (reverted): `(0..config.samples)` in `measure`; every test passes, yet
   `Config { samples: 0, .. }` then panics in `Bench::run` on the `expect("invariant: …")`
   instead of taking one sample. Test `measure`/`run` with `samples: 0`.
5. **B5, BENCH-R024.** The spec says `Summary::from_samples` returns `Some` for every non-empty
   finite slice; `benches/src/stats.rs` returns `None` when the length exceeds `u32::MAX`
   (`u32::try_from(...).ok()?`). Either state that case in R024 (spec bump) or change the code.

Checked without a finding: every other acceptance criterion (crate at `benches/`, workspace
member, `#![forbid(unsafe_code)]` in `src/lib.rs` and the bench target, floor 96.8 in
`ci/coverage.toml`; public items documented with examples; statistics tests with hand-computed
values and a fake clock; `tests/output.rs` parses a real fixed-iteration result file);
`Verifies:` tags of R001–R003, R005, R010, R011, R013–R015, R020–R023, R030, R031, R033–R035,
R040–R042, R050, R060 and XTASK-R090 against their tests; spec §5 API against the code; no
`unsafe`, crate not deterministic; no new external dependency, `architecture.toml` entry
matches. Bench target run by hand on the merged tree (`--iterations 1000 --samples 3`): five
cases of BENCH-R061, file in the §5 layout; `--samples 0` prints LIM-BENCH-0005 and exits 1.
`xtask` changes (XTASK-R090, coverage file filter) judged inside ADR-0030 Decision 9 (b): the
task's scope names R090, and without the filter the new crate measured no lines, so its floor
blocked this task. The two adapted `xtask` tests still check what they checked for their rule.
Coverage floor raise `xtask` 97.6 → 98.2: by `--update-floors`, allowed (XTASK-R063), and
`cargo xtask ci` coverage passes with it on the merged tree.
Merge of `origin/main` (`483f277`): conflicts `Cargo.toml` (members: both), `ci/coverage.toml`
(both floors), `docs/specs/xtask.md` (this task's history row renumbered **27 → 28**,
`spec_version: 28`; M1-T010 took 27), `docs/STATUS.md` regenerated. The handoff above keeps
"`spec_version` 27" as written. `cargo xtask ci` on the merged tree: 12 steps pass, none
skipped, macOS 27.2 arm64, Rust 1.99.1.
Session boundaries: implementation sub-agent #10 of `2026-11-10-session-m1-t014-review.md`
(started 22:35, before the 22:40 cutoff), pair template. No workflow change (ADR-0028).

Backlog: 4 entries in `docs/BACKLOG.md` ("M1-T015, M1-T016 — review 2026-11-10"). Follow-up
task: none.
