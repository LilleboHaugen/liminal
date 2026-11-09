---
id: M1-T020
title: Implement liminal_math — scalar vector, matrix and quaternion types
milestone: M1
status: todo
crates: [liminal_math]
depends_on: [M1-T019, M1-T014]
requirements: []
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

_None._

## Handoff

_Empty._

## Review

_Empty._
