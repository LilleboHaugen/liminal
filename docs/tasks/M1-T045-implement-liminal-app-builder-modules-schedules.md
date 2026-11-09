---
id: M1-T045
title: Implement liminal_app — builder, modules, schedules
milestone: M1
status: todo
crates: [liminal_app]
depends_on: [M1-T044, M1-T043, M1-T029]
requirements: []
---

# M1-T045: Implement `liminal_app`: builder, modules, schedules

## Goal

Apps are assembled from modules and expose the standard schedules.

## Context

- `docs/specs/liminal_app.md`.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Crate, app builder, module registration and its errors, standard schedules.

**Out:** Main loop and runners: next task.

## Acceptance criteria

- [ ] The crate exists at the path in `architecture.toml`, is a workspace member, has
      `#![forbid(unsafe_code)]` if `unsafe = "forbid"`, and has a floor in `ci/coverage.toml`.
- [ ] The crate's clippy determinism configuration is in place.
- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.

## Test requirements

Requirements in frontmatter.

## Notes

_None._

## Handoff

_Empty._

## Review

_Empty._
