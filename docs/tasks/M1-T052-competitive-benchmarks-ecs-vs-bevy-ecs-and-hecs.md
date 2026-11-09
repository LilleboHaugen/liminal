---
id: M1-T052
title: Competitive benchmarks — ECS vs bevy_ecs and hecs
milestone: M1
status: todo
crates: []
depends_on: [M1-T051, M1-T043]
requirements: []
---

# M1-T052: Competitive benchmarks: ECS vs `bevy_ecs` and `hecs`

## Goal

ECS results are recorded against `bevy_ecs` and `hecs` (M1 exit criterion 3).

## Context

- ADR-0023 item 5; PERFORMANCE.md (workloads modelled on ecs_bench_suite: simple insert/iter, fragmented iter, schedule, add/remove).
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** `bevy_ecs` and `hecs` added to the competitive workspace, the four workloads for all three engines, results file, spec summary.

**Out:** Making Liminal faster (parity is not required in M1).

## Acceptance criteria

- [ ] `cargo deny check` passes on the competitive lock file.
- [ ] The four workloads exist for all three engines and measure the same work.
- [ ] `ci/perf/competitive/liminal_ecs-<date>.json` committed; summary in `docs/specs/liminal_ecs.md`.

## Test requirements

None beyond the benchmarks themselves.

## Notes

_None._

## Handoff

_Empty._

## Review

_Empty._
