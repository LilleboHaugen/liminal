---
id: M1-T031
title: Implement liminal_reflect — type info, stable IDs, field access
milestone: M1
status: todo
crates: [liminal_reflect]
depends_on: [M1-T030, M1-T014]
requirements: []
---

# M1-T031: Implement `liminal_reflect`: type info, stable IDs, field access

## Goal

Types can describe themselves at runtime and be read and written by field name, with hand-written impls.

## Context

- `docs/specs/liminal_reflect.md`.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Crate, type descriptors, stable type IDs, field access, hand-written impls for primitives and `liminal_math` types if the spec asks for them.

**Out:** The derive macro: next task.

## Acceptance criteria

- [ ] The crate exists at the path in `architecture.toml`, is a workspace member, has
      `#![forbid(unsafe_code)]` if `unsafe = "forbid"`, and has a floor in `ci/coverage.toml`.
- [ ] The crate's clippy determinism configuration is in place.
- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.

## Test requirements

Requirements in frontmatter; property tests for get/set round trips.

## Notes

_None._

## Handoff

_Empty._

## Review

_Empty._
