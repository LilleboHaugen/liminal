---
id: M1-T020
title: Implement liminal_math — scalar vector, matrix and quaternion types
milestone: M1
status: review
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

_Empty._
