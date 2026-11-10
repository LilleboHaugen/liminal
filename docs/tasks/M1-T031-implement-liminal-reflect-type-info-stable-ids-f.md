---
id: M1-T031
title: Implement liminal_reflect — type info, stable IDs, field access
milestone: M1
status: in-progress
crates: [liminal_reflect]
depends_on: [M1-T030, M1-T014]
requirements: [REFL-R001, REFL-R002, REFL-R003, REFL-R004, REFL-R005, REFL-R020, REFL-R021, REFL-R040, REFL-R041, REFL-R042, REFL-R043, REFL-R044, REFL-R045, REFL-R060, REFL-R061, REFL-R062, REFL-R063, REFL-R064, REFL-R065, REFL-R080, REFL-R100, REFL-R101, REFL-R102, REFL-R103, REFL-R104, REFL-R140]
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

- 2026-11-10 (M1-T057): the spec is at `spec_version` 2. New requirement REFL-R065
  (`TypeRegistry` `Debug` without `TypeId`) is in this task's `requirements`. REFL-R043,
  REFL-R080 and REFL-R100 now ask for tests with deliberately wrong hand-written impls
  (`field_at(1)` / `get(1)` returning `None` below the count); REFL-R061 and REFL-R062 fix the
  check order; the §5.2 example changed (the `spec_version` 1 form did not compile).

## Handoff

_Empty._

## Review

_Empty._
