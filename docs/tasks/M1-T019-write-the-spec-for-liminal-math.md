---
id: M1-T019
title: Write the spec for liminal_math
milestone: M1
status: done
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

2026-11-10, review sub-agent (ADR-0026, ADR-0029) of parent session
`session/2026-11-10-m1-reviews` (journal `2026-11-10-session-m1-reviews.md`), fresh context,
prompt: the ADR-0026 review template with worktree and task ID filled in. The author was an
implementation sub-agent of the same parent; ADR-0029 Decision 1.2 (on `main` since `c75a54e`,
merged into this branch before the review commit) allows that. This agent did no implementation
work on M1-T019. macOS 27.2 (Darwin, arm64), worktree `liminal-worktrees/M1-T019-review`,
reviewed commit `bf8c884`. Task review (REVIEW.md §1) plus spec review (REVIEW.md §2).

Outcome: **done**, no blocking finding (B1–B5). Contract-changing follow-ups: M1-T060 (5 items;
added to `depends_on` of M1-T020 and M1-T023). Backlog: 9 entries in `docs/BACKLOG.md`.

Checked:
- AC 1: 14 template sections in order; `maturity: planned`, `spec_version: 1`,
  `adrs: [ADR-0009, ADR-0017, ADR-0018, ADR-0023, ADR-0024]`.
- AC 2: 71 requirement rows, IDs unique, all MUST (script); a word search for "fast", "robust",
  "efficient" and similar finds none in the spec.
- AC 3: §5 covers every item of the task scope. Consistent with `docs/ARCHITECTURE.md` §2 (no
  other document states a handedness, axis or matrix convention), `architecture.toml` (layer 0,
  `same_layer_deps = ["liminal_core"]`, no external, dev `proptest`, `unsafe = "allow"`,
  `deterministic = true`) and `docs/specs/liminal_core.md` (`EngineError`, `write_display`,
  `validate_error`, `ContextValue::Float`/`Text`, CORE-R010 `null` for non-finite floats,
  `StateHasher::write_f32` without canonicalization, context keys match `^[a-z][a-z0-9_]*$`).
- AC 4: 10 codes with message, help and context.
- AC 5: test kinds per area; the four SIMD paths name their `reference` item and differential
  test (PERFORMANCE.md rules 1 to 4). Reference names accepted (spec §12 item 8).
- AC 6: four workloads of `tests/determinism.rs`, each with the bit-identity claim; §11 says it
  is not verified.
- AC 7: budgets "none yet", scenarios on the reference machine, measuring tasks M1-T022 and
  M1-T051 named (both exist).
- AC 8: the 71 spec IDs equal the union of the `requirements` frontmatter of M1-T020 (35),
  M1-T021 (14), M1-T022 (3), M1-T023 (19), no ID twice (script); §6 table agrees.
- AC 9: 12 open questions, each with a resolving task or milestone.
- AC 10: `docs/specs/README.md` row; `cargo xtask ci` on `bf8c884`: pass, 11 of 11 steps, none
  skipped; branch CI run 42863576835 on `bf8c884` green.
- Mathematics, by derivation: quaternion product (R071) against the Hamilton product; rotation
  formula (R072) against `v + 2w(u × v) + 2u × (u × v)`; `look_to` (R049), `perspective`
  (R050: depth 0 at `z_near`, 1 at `z_far`) and `orthographic` (R051) against right-handed
  0-to-1-depth matrices; `from_axis_angle` and the Rodrigues identity (R073); slerp (R074);
  affine product and inverse (R092, R093); frustum plane extraction for column vectors and
  depth 0 to 1, normals inward (R175); positive-vertex test (R176); sizes and alignments
  (R001); determinant exactness bound `24 × 8^4 < 2^24` (R044); the `det` special-value table
  row by row against the C99 Annex F / IEEE 754-2019 §9.2 cases; ULP definition (§4).
- Numbers, with a scratch `rustc -O` program (aarch64, not committed): `2.3561945` is the
  `f32` nearest to 3π/4; `ln(f32::MAX) = 88.7228`, `ln(2^−150) = −103.97` (R107 thresholds);
  `tan` of half the largest `f32` below `PI` is finite and positive (R050); the two
  counterexamples of M1-T060 items 1 and 2.
- No fake progress, oracle changes: none. Docs: STATUS.md regenerated; scope changes to
  M1-T021 and M1-T022 are in their "Notes".
- Session boundaries: the journal entry names its parent; that parent's journal is "in
  progress" on its session branch and its sub-agent table is not written yet, so the listing
  is not verifiable here. The journal records no spawn and no install. The owner's instruction
  behind ADR-0029 is known to this review only from the ADR text on `main`.
- ADR-0028: no workflow change in the task's commit; no credentials or personal data.

Follow-up task M1-T060 (contract-changing, each changes a requirement or invariant that
M1-T020 or M1-T023 implements): MATH-R023 antisymmetry false for finite inputs whose products
overflow; INV-3 to INV-5 broken by inputs that MATH-R160, R165, R175 accept; MATH-R004 omits
`f32`/`f64` conversion; MATH-R072 identity clause without domain; "stable case" undefined for
two shapes.

Not verified by this review: every claim about code that does not exist (the 1.0 ULP bound,
cross-platform identity, Miri support, SIMD speed); the journal's prototype numbers (the
prototype is not in the repository).
