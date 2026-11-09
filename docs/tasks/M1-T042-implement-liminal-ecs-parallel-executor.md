---
id: M1-T042
title: Implement liminal_ecs — parallel executor
milestone: M1
status: todo
crates: [liminal_ecs]
depends_on: [M1-T041, M1-T036]
requirements: []
---

# M1-T042: Implement `liminal_ecs`: parallel executor

## Goal

Schedules run on `liminal_jobs` in parallel with results identical to the single-threaded executor.

## Context

- `docs/specs/liminal_ecs.md`; ADR-0009 item 4; `docs/specs/liminal_jobs.md`.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Parallel executor, equivalence tests, loom tests for shared executor state.

**Out:** State hash and `usable`: next task.

## Acceptance criteria

- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.
- [ ] Equivalence property test: for generated worlds and schedules, single- and multithreaded execution give the same world state.
- [ ] loom tests for the executor's shared structures; Miri passes on code that loom does not cover.

## Test requirements

Requirements in frontmatter; equivalence and loom tests.

## Notes

_None._

## Handoff

_Empty._

## Review

_Empty._
