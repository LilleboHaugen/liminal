---
id: M1-T037
title: Write the spec for liminal_ecs
milestone: M1
status: todo
crates: [liminal_ecs]
depends_on: [M1-T030, M1-T034]
requirements: []
---

# M1-T037: Write the spec for `liminal_ecs`

## Goal

`docs/specs/liminal_ecs.md` is a complete, reviewable contract for `liminal_ecs`, so the implementation tasks can be done without guessing.

## Context

- `docs/ARCHITECTURE.md` §2 (module map: archetype ECS: entities, components, resources, events, queries, commands, change detection, schedules with parallel execution).
- `docs/specs/_TEMPLATE.md`; `docs/specs/xtask.md` as the quality bar.
- `architecture.toml` entry for `liminal_ecs` (layer, `unsafe`, `deterministic`, allowed deps).
- ADR-0008 (archetype ECS); ADR-0009; `docs/specs/liminal_reflect.md`, `docs/specs/liminal_jobs.md`.
- PERFORMANCE.md competitive workloads (they shape the budgets section).
- PERFORMANCE.md "Reference implementations reachable from benchmarks" (M1-T054): the spec's API section lists the crate's `reference` items for each optimized path.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Spec for: entities (generational, from `liminal_core`), component registration via `liminal_reflect`, archetype tables and archetype moves, queries and filters, resources, events, deferred commands applied in a defined order, change detection, systems with declared access, schedules, conflict detection, single-threaded and parallel executors (on `liminal_jobs`), `World::state_hash()` (ADR-0009 item 6), and the naive reference world (for example a map from entity to component set) that differential tests compare against.

**Out:** Code. The implementation tasks (M1-T038, M1-T039, M1-T040, M1-T041, M1-T042, M1-T043) write it.

## Acceptance criteria

- [ ] `docs/specs/liminal_ecs.md` exists, follows `docs/specs/_TEMPLATE.md` (all sections), with
      `maturity: planned`, `spec_version: 1` and `adrs` listing every ADR it implements.
- [ ] Every requirement has an ID `ECS-Rnnn`, an RFC 2119 level and a testable statement with
      units, ranges and failure behaviour. No vague words ("fast", "robust", "efficient").
- [ ] §5 Public API covers the scope below and is consistent with `docs/ARCHITECTURE.md` and the
      specs of the crates it depends on.
- [ ] Error codes `LIM-ECS-NNNN` are listed with message and `help` (ADR-0017), or the spec says
      why the crate has no recoverable errors.
- [ ] The test plan names the test kinds that apply (TESTING.md) and, for every optimized path, the
      reference implementation it is checked against (AGENTS.md rule 4).
- [ ] Determinism requirements follow ADR-0024: the spec names the workloads of
      `tests/determinism.rs` and says whether cross-platform bit-identity is claimed for each.
- [ ] Performance budgets name scenarios and machines; numbers that need a measurement say which
      task measures them (PERFORMANCE.md: budgets are never invented).
- [ ] Every MUST requirement is assigned to exactly one implementation task (M1-T038, M1-T039, M1-T040, M1-T041, M1-T042, M1-T043): the spec
      lists the assignment, and this session writes the IDs into those tasks' `requirements`
      frontmatter (`check-docs` R028 validates them).
- [ ] Open questions are listed in the spec instead of being decided silently.
- [ ] `docs/specs/README.md` lists the spec (XTASK-R027); `cargo xtask ci` passes.

## Test requirements

None (documentation task). The review is a spec review (REVIEW.md §2).

## Notes

Requirement IDs leave gaps between groups (template §6). Keep the API small: every public item has to be documented, tested and later snapshotted.

## Handoff

_Empty._

## Review

_Empty._
