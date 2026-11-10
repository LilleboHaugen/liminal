---
id: M1-T041
title: Implement liminal_ecs — systems, schedules, single-threaded executor
milestone: M1
status: todo
crates: [liminal_ecs]
depends_on: [M1-T040]
requirements: [ECS-R124, ECS-R125, ECS-R130, ECS-R131, ECS-R132, ECS-R133, ECS-R134, ECS-R135, ECS-R150, ECS-R151, ECS-R152, ECS-R153, ECS-R154, ECS-R155, ECS-R170, ECS-R171, ECS-R172, ECS-R173, ECS-R174, ECS-R175, ECS-R210, ECS-R211]
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

Requirements written by M1-T037 (spec §6 "Assignment"). Points set by the spec: (1) The conflict rule is spec §4 "Conflict", with one test per case of ECS-R154. (2) `ScheduleBuilder::build` binds the schedule to one world (ECS-R155). (3) Run ticks follow the execution order (ECS-R124); the §4 example is the test. (4) Missing resources are a pre-run check (ECS-R171); rejected commands are reported after the run (ECS-R173). (5) This task adds the `liminal_telemetry` dependency, the spans of ECS-R210 for `run`, and the counter of ECS-R211 in `World::apply_commands` and in sync points. (6) ECS-R175 (no allocation after warm-up) is tested in `tests/no_alloc.rs`. The signatures of spec §5.6 and §5.7 were type-checked with stub bodies only; if an implementation needs a different trait shape, change the spec in its own commit (WORKFLOW.md §6).

## Handoff

_Empty._

## Review

_Empty._
