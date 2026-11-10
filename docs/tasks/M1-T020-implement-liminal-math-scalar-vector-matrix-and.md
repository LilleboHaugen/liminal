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

- [ ] The crate exists at the path in `architecture.toml`, is a workspace member, has
      `#![forbid(unsafe_code)]` if `unsafe = "forbid"`, and has a floor in `ci/coverage.toml`.
- [ ] The crate's clippy determinism configuration is in place.
- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.

## Test requirements

Requirements in frontmatter; property tests (proptest) with explicit tolerances.

## Notes

Added by the M1-T019 review (2026-11-10): depends on M1-T060, which corrects requirements this task implements (ADR-0029 Decision 2.3).

## Handoff

_Empty._

## Review

_Empty._
