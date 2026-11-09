---
id: M1-T017
title: cargo xtask bench and the first wall-clock baselines
milestone: M1
status: todo
crates: [xtask, liminal_benches]
depends_on: [M1-T016]
requirements: []
---

# M1-T017: `cargo xtask bench` and the first wall-clock baselines

## Goal

Benchmarks are compared with recorded baselines on the reference machine, and the first baselines are committed.

## Context

- ADR-0023 item 3; PERFORMANCE.md §Harness and baselines; `docs/specs/xtask.md`.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** `cargo xtask bench` (run bench targets, compare with `ci/perf/<machine-id>/<crate>.json`, report regressions with the ADR's threshold, `--json`, `--update-baseline`); spec requirements; first baseline file for `liminal_core` on the reference machine.

**Out:** Instruction counts and the CI job: the instruction-count task. Not a step of `cargo xtask ci` (ADR-0023).

## Acceptance criteria

- [ ] New XTASK requirements for `bench`, with `Verifies:` tests on fixture result files (regression, within threshold, missing baseline, unknown machine).
- [ ] Real-binary integration test for `bench` (tool-crate gate).
- [ ] `ci/perf/<machine-id>/liminal_core.json` committed, with the metadata PERFORMANCE.md requires; the journal records the run.
- [ ] `cargo xtask ci` passes.

## Test requirements

New XTASK requirements; comparison logic tested on fixture data only.

## Notes

_None._

## Handoff

_Empty._

## Review

_Empty._
