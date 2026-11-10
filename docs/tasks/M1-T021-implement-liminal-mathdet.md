---
id: M1-T021
title: Implement liminal_math::det
milestone: M1
status: todo
crates: [liminal_math]
depends_on: [M1-T020]
requirements: [MATH-R050, MATH-R073, MATH-R074, MATH-R100, MATH-R101, MATH-R102, MATH-R103, MATH-R104, MATH-R105, MATH-R106, MATH-R107, MATH-R108, MATH-R109, MATH-R110]
---

# M1-T021: Implement `liminal_math::det`

## Goal

Deterministic transcendental functions exist with measured accuracy.

## Context

- `docs/specs/liminal_math.md` (`det` requirements); ADR-0009 item 5.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** The `det` functions, accuracy tests against an f64 reference over the spec's input ranges, recorded-output tests for the cross-platform claim.

**Out:** SIMD versions of `det`: only if the spec asks for them, in the SIMD task.

## Acceptance criteria

- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.
- [ ] Accuracy: maximum error in ULP measured over the stated range and recorded in the spec; tests fail if it exceeds the spec bound.
- [ ] Results compared against recorded literals so CI on three OSes shows cross-platform identity (ADR-0024 item 3).

## Test requirements

Requirements in frontmatter; differential tests against `f64` std functions as oracle.

## Notes

Added by M1-T019 (2026-11-10): `docs/specs/liminal_math.md` §6 assigns `Mat4::perspective` (MATH-R050), `Quat::from_axis_angle` (MATH-R073) and `Quat::slerp` (MATH-R074) to this task, because they call `det` functions. This widens the scope above by these three operations. The exhaustive accuracy sweep is an `#[ignore]` test run once here (spec §10); measured maxima go into the spec's §10 table.

## Handoff

_Empty._

## Review

_Empty._
