---
id: M1-T014
title: Implement liminal_core — hashing, RNG, collections, time; reach usable
milestone: M1
status: review
crates: [liminal_core]
depends_on: [M1-T013, M1-T059]
requirements: [CORE-R050, CORE-R051, CORE-R052, CORE-R053, CORE-R054, CORE-R055, CORE-R056, CORE-R057, CORE-R060, CORE-R061, CORE-R070, CORE-R071, CORE-R072, CORE-R073, CORE-R074, CORE-R075, CORE-R076, CORE-R090, CORE-R091, CORE-R092, CORE-R093, CORE-R094, CORE-R095, CORE-R096, CORE-R097, CORE-R098, CORE-R110, CORE-R111, CORE-R112, CORE-R113, CORE-R114, CORE-R130, CORE-R131, CORE-R132, CORE-R133]
---

# M1-T014: Implement `liminal_core`: hashing, RNG, collections, time; reach `usable`

## Goal

`liminal_core` is complete per its spec and ready for the `usable` promotion review.

## Context

- `docs/specs/liminal_core.md`; ADR-0009; ADR-0024.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** State hasher, fixed-seed hasher and collections, RNG streams, time types; `tests/determinism.rs`; the `usable` evidence.

**Out:** Benchmarks of the hasher and RNG: the `liminal_benches` tasks.

## Acceptance criteria

- [x] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [x] New public items are documented; primary entry points have runnable `# Examples`.
- [x] `cargo xtask ci` passes with no failed or skipped step.
- [x] RNG and hasher outputs are checked against published reference vectors of the chosen algorithms, or against values computed by an independent implementation named in the test.
- [x] `tests/determinism.rs` per ADR-0024, with recorded hash literals for the workloads the spec claims to be cross-platform.
- [x] The spec's §Maturity checklist holds the evidence for every `usable` gate in
      `docs/process/MATURITY.md` (trace with `maturity: usable` applied on a scratch copy, coverage
      against the 85 % target for its crate type, examples, integration tests).
- [x] The task is set to `review` with `maturity` still unchanged. The reviewing session performs
      the promotion review (REVIEW.md §3) and, if every gate holds, sets `maturity: usable` in the
      spec and `docs/specs/README.md` and regenerates STATUS.md (MATURITY.md "Promotion
      procedure": a different session changes `maturity`).

## Test requirements

Requirements in frontmatter. Known-answer tests for hash and RNG; determinism tests.

## Notes

- 2026-11-10 (M1-T014): the spec's test plan named the reference C implementations
  (`xoshiro256starstar.c`, `splitmix64.c`) and `xxhsum` as oracles. The session was not
  permitted to compile and run downloaded code. Oracles used instead (spec §10, `spec_version`
  4): published vectors copied as data (xxHash sanity vectors; the `rand_xoshiro` `reference`
  tests, which record outputs of the reference C files) and an independent Python
  implementation, `crates/liminal_core/tests/reference/kat.py`. This meets the acceptance
  criterion ("published reference vectors … or … an independent implementation named in the
  test"); it is not a scope reduction.

## Handoff

2026-11-10, implementation sub-agent (parent journal `2026-11-10-session-m1-reviews.md`), branch
`task/M1-T014-implement-liminal-core-hashing-rng-collections-t`, journal
`docs/journal/2026-11-10-M1-T014.md`.

**Done**
- `crates/liminal_core/src/hash.rs` and `hash/`: streaming XXH64 (seed 0), `StateHasher`,
  `StateHash` with the impls of CORE-R054, `state_hash_of`, `FixedState`, `DetHasher`.
- `src/collections.rs` and `collections/`: `DetHashMap`, `DetHashSet` (entry list in insertion
  order plus an open-addressing index table; `remove` rebuilds the table, O(len)).
- `src/rng.rs`: `StreamKey`, `Rng` (xoshiro256\*\*, SplitMix64 seeding, Lemire bounded integers).
- `src/time.rs`: `TickRate`, `Tick`, `TickDuration`.
- Tests: `tests/hashing.rs`, `collections.rs`, `rng.rs`, `time.rs`, `determinism.rs`, helpers
  `tests/common/xxh64_vectors.rs`, `vec_model.rs`; unit tests in `hash/xxh64.rs`,
  `collections/index_table.rs`, `rng.rs`. Every requirement of this task has a `Verifies:` test.
- `examples/seeded_world.rs` (the "example or dependent crate" gate).
- Spec `spec_version` 4 (own commit) and the §13 evidence; `ci/coverage.toml` floor 98.9 → 99.3.
- Backlog: the `Tick` / `TickDuration` half of the M1-T055 entry on `Default` values is fixed
  (CORE-R114) and removed from `docs/BACKLOG.md`; the `ErrorContext` half stays.

**Verified**
- `cargo xtask ci`: pass, 11 steps, none skipped (macOS 27.2 arm64, Rust 1.99.1).
- Branch CI run 42882724790 on `9760608`: success on macOS, Windows and Linux (`checks`),
  `coverage`, `deny`. That run has the code and tests; the later commits add one test
  assertion, docs and the coverage floor. Their CI run was not waited for.
- Trace with `maturity: usable` on a scratch copy: MUST 58/58, `trace: ok`.
- Coverage 99.37 % (1753/1764 lines); the new modules have no uncovered line.
- Mutation check: 33 hand-written mutants of the new modules, each failed at least one test
  (list in the journal).
- `python3 crates/liminal_core/tests/reference/kat.py` self-check passes and reproduces the
  recorded literals it is named for, including the `rng_streams` determinism hash.

**Not done / for the reviewer**
- `maturity` is unchanged (`planned`). The promotion review (REVIEW.md §3) sets it.
- Spec changes to judge (§14 row 4): hand-written `Debug` for the two collections instead of
  the derive; the oracle wording of §10; CORE-R114 `Default`.
- The tests were written after the code, not before. The mutation check stands in for
  "seen failing".

**Traps**
- `allow-unwrap-in-tests` covers only `#[test]` functions: helper functions in `tests/*.rs`
  need `expect("test setup: …")`.
- `cargo xtask coverage --update-floors` raises every crate's floor. This task kept the `xtask`
  floor at 97.6 (not its crate).
- A failing proptest writes `tests/<name>.proptest-regressions`. The ones produced by test bugs
  and by the mutation check were deleted; none is committed for the new tests.
- Recorded literals are oracles. Regenerate one only for an intended change, with `kat.py`
  where the test names it.

## Review

_Empty._
