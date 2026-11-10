---
id: M1-T039
title: Implement liminal_ecs — queries and archetype moves
milestone: M1
status: todo
crates: [liminal_ecs]
depends_on: [M1-T038]
requirements: [ECS-R040, ECS-R041, ECS-R042, ECS-R043, ECS-R044, ECS-R060, ECS-R061, ECS-R062, ECS-R063, ECS-R064, ECS-R065, ECS-R066, ECS-R067, ECS-R068, ECS-R073, ECS-R202]
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

Requirements written by M1-T037 (spec §6 "Assignment"). Points set by the spec: (1) Query iteration order is normative (ECS-R043, ECS-R062): archetype number, then row, with append on entry and swap-remove on exit. (2) The aliasing rule is ECS-R065 (LIM-ECS-0020) plus the `&self`/`&mut self` split of ECS-R066. (3) `Added` and `Changed` exist here with baseline 0 (ECS-R067); their behaviour in systems is M1-T041's. (4) Tick comparisons are not part of this task (M1-T040 adds them, ECS-R122). (5) `reference::World::insert` and `remove` (ECS-R202) belong here. ECS-R045 (SHOULD, cache of archetype moves) is implemented here. ECS-R068 (no allocation in iteration) needs its own test binary `tests/no_alloc.rs`.

## Handoff

_Empty._

## Review

_Empty._
