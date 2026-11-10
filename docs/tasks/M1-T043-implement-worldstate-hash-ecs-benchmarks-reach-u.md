---
id: M1-T043
title: Implement World::state_hash, ECS benchmarks; reach usable
milestone: M1
status: todo
crates: [liminal_ecs]
depends_on: [M1-T042, M1-T017]
requirements: [ECS-R190, ECS-R191, ECS-R192, ECS-R193, ECS-R194, ECS-R220, ECS-R221]
---

# M1-T043: Implement `World::state_hash`, ECS benchmarks; reach `usable`

## Goal

`World::state_hash()` exists, ECS performance is measured, and `liminal_ecs` is ready for the `usable` promotion review.

## Context

- `docs/specs/liminal_ecs.md`; ADR-0009 item 6; ADR-0024.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** State hash over reflected components in canonical order, `tests/determinism.rs`, benchmarks for the competitive workloads (Liminal side), the `usable` evidence.

**Out:** Comparison with `bevy_ecs` and `hecs`: competitive ECS task.

## Acceptance criteria

- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.
- [ ] State hash tests: equal for worlds built in different orders with equal content, different when any reflected field differs.
- [ ] Benchmarks recorded with `cargo xtask bench`; spec budgets filled from them.
- [ ] The spec's §Maturity checklist holds the evidence for every `usable` gate in
      `docs/process/MATURITY.md` (trace with `maturity: usable` applied on a scratch copy, coverage
      against the 85 % target for its crate type, examples, integration tests, Miri passes on the test suite (exceptions listed with reasons), loom tests pass for the lock-free structures).
- [ ] The task is set to `review` with `maturity` still unchanged. The reviewing session performs
      the promotion review (REVIEW.md §3) and, if every gate holds, sets `maturity: usable` in the
      spec and `docs/specs/README.md` and regenerates STATUS.md (MATURITY.md "Promotion
      procedure": a different session changes `maturity`).

## Test requirements

Requirements in frontmatter; property tests; determinism tests.

## Notes

Requirements written by M1-T037 (spec §6 "Assignment"). Points set by the spec: (1) The byte layout of the hash is ECS-R190, with a hand-written known-answer test; what the hash leaves out is ECS-R192. (2) `reference::World::state_hash` (ECS-R194) belongs here. (3) The determinism workloads are `world_churn` and `schedule_ticks` (spec §11), both with recorded literals. (4) Benchmark cases and their IDs are in spec §9, including the pair `liminal_ecs/query/simple_iter` and `liminal_ecs/reference/simple_iter` (ADR-0018).

## Handoff

_Empty._

## Review

_Empty._
