---
id: M1-T040
title: Implement liminal_ecs — resources, events, commands, change detection
milestone: M1
status: todo
crates: [liminal_ecs]
depends_on: [M1-T039]
requirements: []
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

_None._

## Handoff

_Empty._

## Review

_Empty._
