---
id: M1-T019
title: Write the spec for liminal_math
milestone: M1
status: review
crates: [liminal_math]
depends_on: [M1-T001, M1-T054]
requirements: []
---

# M1-T019: Write the spec for `liminal_math`

## Goal

`docs/specs/liminal_math.md` is a complete, reviewable contract for `liminal_math`, so the implementation tasks can be done without guessing.

## Context

- `docs/ARCHITECTURE.md` §2 (module map: scalar and SIMD linear algebra, geometry primitives, intersection tests, deterministic transcendental functions (`det`)).
- `docs/specs/_TEMPLATE.md`; `docs/specs/xtask.md` as the quality bar.
- `architecture.toml` entry for `liminal_math` (layer, `unsafe`, `deterministic`, allowed deps).
- ADR-0009 item 5 (`det`); ADR-0018 (`unsafe` for intrinsics).
- PERFORMANCE.md "Reference implementations reachable from benchmarks" (M1-T054): the spec's API section lists the crate's `reference` items for each optimized path.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Spec for: vector, matrix, quaternion and affine types (f32), their operations and conventions (handedness, column/row major, angle units); `det` functions with accuracy in ULP against an f64 reference and the cross-platform identity claim; SIMD paths per target (x86_64, aarch64) with the scalar code as the reference; geometry primitives (AABB, sphere, plane, ray, frustum) and intersection tests with their edge cases.

**Out:** Code. The implementation tasks (M1-T020, M1-T021, M1-T022, M1-T023) write it.

## Acceptance criteria

- [x] `docs/specs/liminal_math.md` exists, follows `docs/specs/_TEMPLATE.md` (all sections), with
      `maturity: planned`, `spec_version: 1` and `adrs` listing every ADR it implements.
- [x] Every requirement has an ID `MATH-Rnnn`, an RFC 2119 level and a testable statement with
      units, ranges and failure behaviour. No vague words ("fast", "robust", "efficient").
- [x] §5 Public API covers the scope below and is consistent with `docs/ARCHITECTURE.md` and the
      specs of the crates it depends on.
- [x] Error codes `LIM-MATH-NNNN` are listed with message and `help` (ADR-0017), or the spec says
      why the crate has no recoverable errors.
- [x] The test plan names the test kinds that apply (TESTING.md) and, for every optimized path, the
      reference implementation it is checked against (AGENTS.md rule 4).
- [x] Determinism requirements follow ADR-0024: the spec names the workloads of
      `tests/determinism.rs` and says whether cross-platform bit-identity is claimed for each.
- [x] Performance budgets name scenarios and machines; numbers that need a measurement say which
      task measures them (PERFORMANCE.md: budgets are never invented).
- [x] Every MUST requirement is assigned to exactly one implementation task (M1-T020, M1-T021, M1-T022, M1-T023): the spec
      lists the assignment, and this session writes the IDs into those tasks' `requirements`
      frontmatter (`check-docs` R028 validates them).
- [x] Open questions are listed in the spec instead of being decided silently.
- [x] `docs/specs/README.md` lists the spec (XTASK-R027); `cargo xtask ci` passes.

## Test requirements

None (documentation task). The review is a spec review (REVIEW.md §2).

## Notes

Requirement IDs leave gaps between groups (template §6). Keep the API small: every public item has to be documented, tested and later snapshotted.

## Handoff

2026-11-10, implementation sub-agent (ADR-0026/0027) on branch
`task/M1-T019-write-the-spec-for-liminal-math`, worktree `liminal-worktrees/M1-T019`. Journal:
`2026-11-10-M1-T019.md`. Status: `review`.

**Done:**
1. `docs/specs/liminal_math.md` (`spec_version` 1, `maturity: planned`, all 14 template sections,
   `adrs: [ADR-0009, ADR-0017, ADR-0018, ADR-0023, ADR-0024]`). 71 requirements, all MUST:
   crate-wide R001–R007, vectors R020–R026, matrices R040–R051, quaternions R070–R074, affine
   R090–R094, `det` R100–R110, reference and SIMD R130–R132, geometry R150–R177, determinism
   workloads R110, R190–R192. 10 error codes LIM-MATH-0001 … 0014.
2. Requirement IDs written into the `requirements` frontmatter of M1-T020 (35), M1-T021 (14),
   M1-T022 (3), M1-T023 (19); the assignment table is in spec §6. Each ID is in exactly one task.
3. `docs/specs/README.md` index row; `docs/STATUS.md` regenerated.
4. "Notes" of M1-T021 and M1-T022 (see "Scope changes to other tasks").
5. `cargo xtask ci`: result in the journal entry.

**Decisions taken in the spec (reviewer, check these):**
- Conventions (§4): right-handed, +Y up, −Z forward, column vectors, column-major storage,
  radians, clip depth 0 to 1. No ADR covers them; the task scope assigns them to the spec
  (spec §12 item 9 asks the review whether an ADR is wanted).
- Bit-exact contract: the core operations are "exact expressions" with a fixed evaluation order
  and no FMA (§4), so SIMD and scalar paths return the same bits and cross-platform identity is
  claimed for all four determinism workloads. NaN sign and payload are excluded (§4 "NaN bits");
  `det` returns the canonical NaN.
- `det`: 11 functions, bound 1.0 ULP against the `f64` std function; `sin`, `cos`, `tan` bounded
  only for `abs(x) ≤ 2^20` (§12 item 2). The algorithm is not prescribed, only the allowed
  operation set (MATH-R004); evaluating in `f64` and rounding once meets the bound (not verified
  by code).
- SIMD: four operations (`Mat4 * Mat4`, `Mat4 * Vec4`, `Quat * Quat`,
  `Affine3::transform_points`), SSE2 and NEON baseline only, compile-time selection. Reference
  items are free functions named after type and operation because three counterparts are
  operators (§12 item 8: a reading of PERFORMANCE.md rule 1 the reviewer should accept or reject).
- Types at the crate root through re-exports of private modules (one public path each); public
  modules only `det` and `reference`. `liminal_core` chose module paths with no re-exports; the
  difference is deliberate (PERFORMANCE.md's example path is `liminal_math::reference::mat4_mul`
  next to `crate::x`).
- Errors: `Result<_, MathError>` for `normalize`, `inverse` and every validating constructor;
  `normalize_or` for callers that do not want a `Result`. Geometry types have private fields and
  invariants (§7); linear algebra types have public fields and none.
- Geometry rounding rule: answers are required only in "stable cases" (same `f64` answer for the
  shape grown and shrunk by 0.01), plus hand-computed exact cases.
- Small API: no Euler angles, `Quat` from matrix, `nlerp`, integer or `f64` vectors (§12 item 6).

**Tolerances (not verified by engine code):** chosen from a throwaway scalar Rust prototype run in
this session (aarch64, 2,000,000 seeded cases per property; numbers in the journal). MATH-R044,
R049–R051, R073, R092, R161 and R177 were not prototyped. Spec §12 item 1 makes M1-T020 and
M1-T023 confirm or change them.

**Scope changes to other tasks (written into their "Notes"):**
- M1-T021 also implements `Mat4::perspective`, `Quat::from_axis_angle` and `Quat::slerp`
  (MATH-R050, R073, R074), because they call `det`. M1-T020 leaves them out.
- M1-T022 also records the `det` benchmark cases (spec §9).

**Not done / for others:**
- No ADR for the conventions (spec §12 item 9).
- CODE_STYLE.md's banned-function list misses `asin`, `atan2`, `powi`, `hypot`, `mul_add` and
  others (spec §12 item 10); M1-T011 is the place. Not edited here (outside this task).
- Benchmark evidence for the x86_64 SIMD paths has no machine (spec §12 item 3).
- Reflection of math types (spec §12 item 7) is a question for M1-T030.
- No task was created.

**Traps:** no `|` in requirement table cells (the spec writes `abs(x)`); the word "withdrawn" in
parentheses marks a requirement withdrawn; `docs/specs/README.md` and `docs/STATUS.md` will
conflict with other spec branches (keep both rows; regenerate STATUS.md).

## Review

_Empty._
