---
id: M1-T051
title: Competitive benchmarks — workspace, math vs glam, jobs vs rayon
milestone: M1
status: todo
crates: []
depends_on: [M1-T017, M1-T023, M1-T036, M1-T008, M1-T009]
requirements: []
---

# M1-T051: Competitive benchmarks: workspace, math vs `glam`, jobs vs `rayon`

## Goal

Math and jobs results are recorded against `glam` and `rayon` from the separate competitive workspace.

## Context

- ADR-0023 item 5; PERFORMANCE.md §Competitive benchmarks; DEPENDENCIES.md (procedure, licences); `architecture.toml` entries `liminal_benches_competitive`, `glam`, `rayon`.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** `benches/competitive/` workspace excluded from the root workspace, `glam` and `rayon` at pinned versions, `cargo deny` on its lock file, `check-arch` coverage of the competitive crate (or the recorded reason it is not possible), workloads described in prose and implemented for both sides, `cargo xtask bench --competitive`, results files.

**Out:** ECS comparison: next task.

## Acceptance criteria

- [ ] `cargo xtask ci` does not build the competitive workspace (shown by its step list and the root `Cargo.lock` not containing `glam` or `rayon`).
- [ ] `cargo deny check` passes on the competitive lock file; any licence or advisory issue is reported to the owner.
- [ ] For `glam` and `rayon` each, the competitive crate's documentation records licence, maintenance status
      (date of the last release), transitive dependency count at the pinned version and whether it
      uses `unsafe` (DEPENDENCIES.md "Adding a dependency" step 2; ADR-0023 is accepted and
      immutable, so the data lives there; added by M1-T054).
- [ ] Workloads documented in the competitive crate; each measures the same work on both sides (reviewer can check by reading).
- [ ] `ci/perf/competitive/liminal_math-<date>.json` and `liminal_jobs-<date>.json` committed; summary in each spec.
- [ ] XTASK spec and tests for `--competitive`; `cargo xtask ci` passes.

## Test requirements

XTASK requirements for `--competitive`.

## Notes

_None._

## Handoff

_Empty._

## Review

_Empty._
