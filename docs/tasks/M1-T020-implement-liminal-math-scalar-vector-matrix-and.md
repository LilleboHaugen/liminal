---
id: M1-T020
title: Implement liminal_math — scalar vector, matrix and quaternion types
milestone: M1
status: in-progress
crates: [liminal_math]
depends_on: [M1-T019, M1-T014, M1-T060]
requirements: [MATH-R001, MATH-R002, MATH-R003, MATH-R004, MATH-R005, MATH-R006, MATH-R007, MATH-R020, MATH-R021, MATH-R022, MATH-R023, MATH-R024, MATH-R025, MATH-R026, MATH-R040, MATH-R041, MATH-R042, MATH-R043, MATH-R044, MATH-R045, MATH-R046, MATH-R047, MATH-R048, MATH-R049, MATH-R051, MATH-R070, MATH-R071, MATH-R072, MATH-R090, MATH-R091, MATH-R092, MATH-R093, MATH-R094, MATH-R130, MATH-R190]
---

# M1-T020: Implement `liminal_math`: scalar vector, matrix and quaternion types

## Goal

The scalar math types and operations in the spec exist and are the reference for later SIMD paths.

## Context

- `docs/specs/liminal_math.md`.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Crate, scalar types and operations, property tests for algebraic identities with stated tolerances.

**Out:** `det` functions, SIMD, geometry: later `liminal_math` tasks.

## Acceptance criteria

- [x] The crate exists at the path in `architecture.toml`, is a workspace member, has
      `#![forbid(unsafe_code)]` if `unsafe = "forbid"`, and has a floor in `ci/coverage.toml`.
- [x] The crate's clippy determinism configuration is in place.
- [x] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [x] New public items are documented; primary entry points have runnable `# Examples`.
- [x] `cargo xtask ci` passes with no failed or skipped step.

## Test requirements

Requirements in frontmatter; property tests (proptest) with explicit tolerances.

## Notes

Added by the M1-T019 review (2026-11-10): depends on M1-T060, which corrects requirements this task implements (ADR-0029 Decision 2.3).

## Handoff

2026-11-10, implementation sub-agent (journal `docs/journal/2026-11-10-M1-T020.md`). Status
`review`.

**Done.**
- Crate `crates/liminal_math` (workspace member, depends on `liminal_core` only, dev-dependency
  `proptest`). `src/`: `vector.rs` (`Vec2`, `Vec3`, `Vec4` through one private macro),
  `mat3.rs`, `mat4.rs`, `quat.rs`, `affine3.rs`, `error.rs` (`MathError`, all 11 variants of
  spec §5.8), `reference.rs` (public module, the four oracles of §5.6). The public operators
  `Mat4 * Mat4`, `Mat4 * Vec4`, `Quat * Quat` and `Affine3::transform_points` call the
  `reference` items (spec §6 assignment notes); M1-T022 adds the SIMD paths behind them.
- `#![deny(unsafe_code)]` at the crate root: `unsafe = "allow"` in `architecture.toml`, so
  `forbid` would block M1-T022; no module uses `unsafe` yet.
- `crates/liminal_math/clippy.toml`: the root settings, the XTASK-R039 list and every path of
  MATH-R004. The `core::arch` intrinsic entries carry `allow-invalid = true`, because each path
  exists on one architecture only (clippy would warn about the other).
- Tests (`crates/liminal_math/tests/`): one file per area, every requirement of the frontmatter
  traced (`cargo xtask trace --crate liminal_math`). Snapshot
  `tests/snapshots/math_errors.txt` (one value of every variant). `tests/determinism.rs`
  workload `linear_algebra` with literal `0xDDA3_807E_8CE9_3C5F` (macOS arm64).
  `tests/tolerance_margins.rs` measures the tolerance properties over 100,000 seeded cases
  each; the maxima are in the journal (open question 1). Largest: 0.375 of the
  `4 × f32::EPSILON` bound of MATH-R025; all others below 0.1 of their bound.
- `ci/coverage.toml`: `liminal_math = 97.3` (measured 97.33 %).
- Spec §13: the two `experimental` gates M1-T020 owns are ticked with evidence; the promotion
  itself is the reviewer's decision (MATURITY.md).

**Not done / not verified.**
- Cross-platform bit-identity of the `linear_algebra` literal is verified only on macOS arm64
  here; GitHub CI on Windows and Linux x86_64 checks it after the merge (spec §11).
- Not in scope (spec §6 assignment): `Mat4::perspective`, `Quat::from_axis_angle`,
  `Quat::slerp` (M1-T021), SIMD (M1-T022), geometry (M1-T023).
- `MathError::with_parameter` (crate-private) has arms for variants whose producers arrive in
  M1-T021 and M1-T023; those arms are uncovered for now (part of the 2.7 % uncovered lines).

**Traps.**
- Clippy pedantic `float_cmp` and `many_single_char_names` fire on exact-comparison tests and on
  the one-letter names of the spec formulas; test files allow them at file level with a
  reason. In library code the one `float_cmp` site (`Mat4::orthographic`) has a local
  `#[expect]`.
- The determinism workload must not divide by a value that can be 0: `0 / 0` NaN bits differ
  between x86_64 and aarch64 (spec §4 "NaN bits").

**Next step.** Review of M1-T020 (REVIEW.md). Then M1-T021 (`det`) is the next `liminal_math`
task.

## Review

2026-11-10, review sub-agent of the parent session on branch
`session/2026-11-10-m1-t020-t031-review`, fresh context (journal `2026-11-10-M1-T020-review.md`).
First review of this task. Rules read from `origin/main` at `115b556` (`AGENTS.md`,
`docs/process/REVIEW.md` §1). **Outcome: changes required (first "changes required"); status
`in-progress`.** The `experimental` promotion of spec §13 is not decided in this review.

**Required changes.**
1. **B4.** `Mat4::inverse` is tested only on affine matrices (fourth row `(0, 0, 0, 1)`): the
   D-trs property of MATH-R045, the hand-computed case in `tests/matrices.rs`, the no-panic test
   and the `linear_algebra` workload. For those matrices the 2×2 minors `c[0]`, `c[1]` and `c[3]`
   of `Minors` in `crates/liminal_math/src/mat4.rs` are 0, so every adjugate term multiplied by
   them is unchecked. Mutation run by the reviewer: in `Minors::adjugate`, row 2, the entry
   `-a(0, 0) * c[4] + a(0, 1) * c[2] - a(0, 3) * c[0]` changed to `... + a(0, 3) * c[0]`:
   `cargo test -p liminal_math` passes. MATH-R045 requires "`Ok` of the adjugate divided by
   `d`" for every matrix with a finite non-zero determinant, and the inverse of a
   view-projection matrix (non-affine) is a realistic use. Required: a test that fails under such
   a mutation, for example a hand-computed exact inverse of a full integer matrix with a
   power-of-two determinant and non-zero entries in every row (including the fourth), and/or a
   property over general well-conditioned matrices. (The reviewer's probe with that mutation:
   max `abs(m * m⁻¹ − I)` 58.7 over 10,000 random matrices with elements in −2 to 2 and
   `abs(det) ≥ 0.5`; 7.6e-6 without it, so the implementation itself is correct.)
2. **B1.** `orthographic_maps_the_box_corners` in `crates/liminal_math/tests/matrices.rs` fails
   intermittently with "Test aborted: Too many global rejects" (proptest default
   `max_global_rejects = 1024`): its `prop_assume!(right <= 10.0 && top <= 10.0 && …)` keeps
   about 23 % of the cases (P(`left + width ≤ 10`) ≈ 0.475 per axis), so 256 successes need
   about 880 rejections on average. Reproduced: 4 failures in 200 runs of the test binary
   (output: 255 successes, 1024 global rejects at line 483). `cargo xtask ci` therefore fails on
   about 2 % of runs. Required: generate the inputs inside the domain (for example `left` in −10
   to 9, `right` in `left + 1` to 10) instead of rejecting them.

**Checked, no finding.**
- Acceptance criteria: crate at `crates/liminal_math` (workspace member, `liminal_core` only,
  dev `proptest` as in `architecture.toml`); `#![deny(unsafe_code)]` (not `forbid`, justified:
  `unsafe = "allow"`); floor `liminal_math = 97.3` in `ci/coverage.toml`;
  `crates/liminal_math/clippy.toml` repeats the root settings, the XTASK-R039 list and every
  MATH-R004 path (compared with the requirement text by hand and by `tests/operation_set.rs`).
  `cargo xtask trace --crate liminal_math`: every frontmatter requirement has `Verifies:` tests.
  Doc examples on `Vec3`, `Mat4`, `Quat`, `Affine3`, `MathError`, `reference` and the crate root.
- `cargo xtask ci` on the task branch with `origin/main` (`115b556`) merged (`3d75a73`): pass,
  12 steps, none skipped, macOS 27.2 arm64. 15 further full runs of `cargo test -p liminal_math`:
  no failure other than finding 2 (which did not occur in those runs).
- Spec agreement, read line by line against §5.1 to §5.6, §5.8, §6 and §8: the exact expressions
  of MATH-R021 to R023, R026, R041, R042, R047, R048, R049, R051, R070 to R072, R090 to R094 and
  R130 (left-to-right sums, no FMA, division not by reciprocal); error order of `look_to` and
  `orthographic`; `normalize` error order; every message and help text of §8 and the snapshot.
  `perspective`, `from_axis_angle` and `slerp` are absent, as the §6 assignment notes require.
- Tests: other mutations tried and caught — `Mat3::inverse` without the transpose
  (`inverse_undoes_the_transform`), `Quat * Vec3` reassociated (`linear_algebra` literal and
  `rotation_is_the_exact_expression`). Exact-expression tests include association-order and FMA
  probes. No assertion-free test except the no-panic test of MATH-R005, which also checks the
  error variants.
- No placeholders, no `todo!()`, no ignored test; new oracles (snapshot, hash literal, floor)
  are justified in the author's journal. No `unsafe`, no banned function or collection in
  `src/`. No new external dependency. No `.github/workflows/` change.
- Session boundaries: the author is row 5 of the parent journal
  `2026-11-10-session-m1-t014-review.md` (spawned 22:26, before the 22:40 cutoff); this reviewer
  did not work on the task.

**Backlog entries:** 0. **Follow-up task:** none.
