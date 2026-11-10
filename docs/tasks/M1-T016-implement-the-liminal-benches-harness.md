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

## Review

_Empty._
