---
id: M1-T019
title: Write the spec for liminal_math
milestone: M1
status: todo
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

- [ ] `docs/specs/liminal_math.md` exists, follows `docs/specs/_TEMPLATE.md` (all sections), with
      `maturity: planned`, `spec_version: 1` and `adrs` listing every ADR it implements.
- [ ] Every requirement has an ID `MATH-Rnnn`, an RFC 2119 level and a testable statement with
      units, ranges and failure behaviour. No vague words ("fast", "robust", "efficient").
- [ ] §5 Public API covers the scope below and is consistent with `docs/ARCHITECTURE.md` and the
      specs of the crates it depends on.
- [ ] Error codes `LIM-MATH-NNNN` are listed with message and `help` (ADR-0017), or the spec says
      why the crate has no recoverable errors.
- [ ] The test plan names the test kinds that apply (TESTING.md) and, for every optimized path, the
      reference implementation it is checked against (AGENTS.md rule 4).
- [ ] Determinism requirements follow ADR-0024: the spec names the workloads of
      `tests/determinism.rs` and says whether cross-platform bit-identity is claimed for each.
- [ ] Performance budgets name scenarios and machines; numbers that need a measurement say which
      task measures them (PERFORMANCE.md: budgets are never invented).
- [ ] Every MUST requirement is assigned to exactly one implementation task (M1-T020, M1-T021, M1-T022, M1-T023): the spec
      lists the assignment, and this session writes the IDs into those tasks' `requirements`
      frontmatter (`check-docs` R028 validates them).
- [ ] Open questions are listed in the spec instead of being decided silently.
- [ ] `docs/specs/README.md` lists the spec (XTASK-R027); `cargo xtask ci` passes.

## Test requirements

None (documentation task). The review is a spec review (REVIEW.md §2).

## Notes

Requirement IDs leave gaps between groups (template §6). Keep the API small: every public item has to be documented, tested and later snapshotted.

## Handoff

_Empty._

## Review

_Empty._
