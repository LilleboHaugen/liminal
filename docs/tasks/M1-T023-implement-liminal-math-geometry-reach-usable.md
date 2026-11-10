---
id: M1-T023
title: Implement liminal_math geometry; reach usable
milestone: M1
status: todo
crates: [liminal_math]
depends_on: [M1-T022, M1-T060]
requirements: [MATH-R150, MATH-R151, MATH-R152, MATH-R153, MATH-R155, MATH-R156, MATH-R157, MATH-R160, MATH-R161, MATH-R165, MATH-R166, MATH-R167, MATH-R168, MATH-R169, MATH-R170, MATH-R175, MATH-R176, MATH-R177, MATH-R192]
---

# M1-T023: Implement `liminal_math` geometry; reach `usable`

## Goal

Geometry primitives and intersection tests exist and `liminal_math` is ready for the `usable` promotion review.

## Context

- `docs/specs/liminal_math.md`.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Geometry primitives, intersection tests with edge cases (touching, degenerate, parallel), `tests/determinism.rs`, the `usable` evidence.

**Out:** Physics-specific queries (M6).

## Acceptance criteria

- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.
- [ ] Intersection tests have property tests against a brute-force or f64 reference.
- [ ] The spec's §Maturity checklist holds the evidence for every `usable` gate in
      `docs/process/MATURITY.md` (trace with `maturity: usable` applied on a scratch copy, coverage
      against the 85 % target for its crate type, examples, integration tests, Miri passes on the test suite (exceptions listed with reasons)).
- [ ] The task is set to `review` with `maturity` still unchanged. The reviewing session performs
      the promotion review (REVIEW.md §3) and, if every gate holds, sets `maturity: usable` in the
      spec and `docs/specs/README.md` and regenerates STATUS.md (MATURITY.md "Promotion
      procedure": a different session changes `maturity`).

## Test requirements

Requirements in frontmatter; property and differential tests.

## Notes

Added by the M1-T019 review (2026-11-10): depends on M1-T060, which corrects requirements this task implements (ADR-0029 Decision 2.3).

## Handoff

_Empty._

## Review

_Empty._
