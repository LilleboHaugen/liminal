---
id: M1-T021
title: Implement liminal_math::det
milestone: M1
status: todo
crates: [liminal_math]
depends_on: [M1-T020]
requirements: []
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

_None._

## Handoff

_Empty._

## Review

_Empty._
