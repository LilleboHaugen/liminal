---
id: M1-T042
title: Implement liminal_ecs — parallel executor
milestone: M1
status: todo
crates: [liminal_ecs]
depends_on: [M1-T041, M1-T036]
requirements: [ECS-R180, ECS-R181, ECS-R182, ECS-R183]
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

Requirements written by M1-T037 (spec §6 "Assignment"). Points set by the spec: (1) The oracle is `Schedule::run` (ECS-R180). (2) The intended design is one `liminal_jobs::graph::JobGraph` per stage, one node per system named with the system's name, one dependency per order; with it the spec has no shared state of its own and no loom test (spec §10, §12 item 16). If this task adds atomics or other shared state, it adds loom models and lists them in spec §10. (3) ECS-R210 covers `run_parallel`: extend the telemetry test of M1-T041 (job-graph node spans named with the system name count as the system spans; do not record a second one). (4) ECS-R184 (SHOULD, no allocation) is decided here; report the allocation count per run (spec §12 item 8). (5) The equivalence and ordering tests run under Miri with 2 threads (spec §7 INV-7).

## Handoff

_Empty._

## Review

_Empty._
