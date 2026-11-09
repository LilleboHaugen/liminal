---
id: M1-T041
title: Implement liminal_ecs — systems, schedules, single-threaded executor
milestone: M1
status: todo
crates: [liminal_ecs]
depends_on: [M1-T040]
requirements: []
---

# M1-T041: Implement `liminal_ecs`: systems, schedules, single-threaded executor

## Goal

Systems with declared access run in schedules, and conflicting access is detected.

## Context

- `docs/specs/liminal_ecs.md`; ADR-0008.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** System trait/functions, access sets, schedules with ordering constraints, conflict detection, single-threaded executor.

**Out:** Parallel executor: next task.

## Acceptance criteria

- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.
- [ ] Tests for each conflict the spec defines (rejected with its error code) and for ordering constraints.

## Test requirements

Requirements in frontmatter.

## Notes

_None._

## Handoff

_Empty._

## Review

_Empty._
