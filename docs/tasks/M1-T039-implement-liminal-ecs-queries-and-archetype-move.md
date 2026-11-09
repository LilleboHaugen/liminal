---
id: M1-T039
title: Implement liminal_ecs — queries and archetype moves
milestone: M1
status: todo
crates: [liminal_ecs]
depends_on: [M1-T038]
requirements: []
---

# M1-T039: Implement `liminal_ecs`: queries and archetype moves

## Goal

Queries with filters iterate matching entities, and components can be added and removed.

## Context

- `docs/specs/liminal_ecs.md`.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Queries (read, write, optional, filters), add/remove component (archetype moves), differential tests extended to these operations.

**Out:** Resources, events, commands: next task.

## Acceptance criteria

- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.
- [ ] Differential tests cover queries and moves against the reference world.
- [ ] Aliasing rules (no two mutable borrows of one component) are enforced as the spec says, with a test for the rejected case.

## Test requirements

Requirements in frontmatter; differential tests; Miri for any `unsafe`.

## Notes

_None._

## Handoff

_Empty._

## Review

_Empty._
