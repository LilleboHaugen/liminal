---
id: M1-T040
title: Implement liminal_ecs — resources, events, commands, change detection
milestone: M1
status: todo
crates: [liminal_ecs]
depends_on: [M1-T039]
requirements: [ECS-R090, ECS-R091, ECS-R100, ECS-R101, ECS-R102, ECS-R110, ECS-R111, ECS-R112, ECS-R113, ECS-R114, ECS-R120, ECS-R121, ECS-R122]
---

# M1-T040: Implement `liminal_ecs`: resources, events, commands, change detection

## Goal

Resources, events, deferred commands and change detection work as specified.

## Context

- `docs/specs/liminal_ecs.md`; ADR-0009 item 4 (command order).
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Resources, events, command buffers applied in the specified order, change ticks.

**Out:** Schedules and executors: next tasks.

## Acceptance criteria

- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.
- [ ] A test shows command application order is the specified order for commands recorded from several sources.

## Test requirements

Requirements in frontmatter.

## Notes

Requirements written by M1-T037 (spec §6 "Assignment"). Points set by the spec: (1) `CommandBuffer` is a public type usable without a schedule, applied with `World::apply_commands`; "several sources" in this task's acceptance criteria is several buffers (ECS-R113). (2) Skipped (entity not live) and rejected (type errors, LIM-ECS-0040) commands are different things (ECS-R111, ECS-R112). (3) This task adds the tick columns, stamping in `Mut`, `World::change_tick`, `advance_change_tick`, `component_ticks`, the same three on `reference::World`, and the tick comparisons in the existing differential tests (ECS-R121, ECS-R122). (4) The `ecs.commands_skipped` counter (ECS-R211) is added by M1-T041 together with the `liminal_telemetry` dependency. ECS-R115 (SHOULD, allocation-free command recording) is decided here: implemented, or reported with the reason in the handoff.

## Handoff

_Empty._

## Review

_Empty._
