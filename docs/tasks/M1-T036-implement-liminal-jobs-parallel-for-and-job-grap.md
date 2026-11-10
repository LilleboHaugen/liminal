---
id: M1-T036
title: Implement liminal_jobs — parallel-for and job graphs; reach usable
milestone: M1
status: todo
crates: [liminal_jobs]
depends_on: [M1-T035, M1-T017]
requirements: [JOBS-R045, JOBS-R060, JOBS-R061, JOBS-R062, JOBS-R063, JOBS-R064, JOBS-R065, JOBS-R080, JOBS-R081, JOBS-R082, JOBS-R083, JOBS-R084, JOBS-R085, JOBS-R086, JOBS-R100, JOBS-R103, JOBS-R104, JOBS-R110, JOBS-R111, JOBS-R120, JOBS-R121, JOBS-R122, JOBS-R141]
---

# M1-T036: Implement `liminal_jobs`: parallel-for and job graphs; reach `usable`

## Goal

`liminal_jobs` is complete per its spec and ready for the `usable` promotion review.

## Context

- `docs/specs/liminal_jobs.md`.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Parallel-for, job graphs, panic propagation, telemetry spans, scaling benchmarks, the `usable` evidence.

**Out:** Comparison with `rayon`: competitive benchmark task.

## Acceptance criteria

- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.
- [ ] Order-independence tests: results of parallel-for reductions declared deterministic in the spec are equal to the sequential result for generated inputs.
- [ ] Benchmarks (1, 2, 4, all cores) recorded with `cargo xtask bench`.
- [ ] The spec's §Maturity checklist holds the evidence for every `usable` gate in
      `docs/process/MATURITY.md` (trace with `maturity: usable` applied on a scratch copy, coverage
      against the 85 % target for its crate type, examples, integration tests, Miri passes on the test suite (exceptions listed with reasons), loom tests pass for the lock-free structures).
- [ ] The task is set to `review` with `maturity` still unchanged. The reviewing session performs
      the promotion review (REVIEW.md §3) and, if every gate holds, sets `maturity: usable` in the
      spec and `docs/specs/README.md` and regenerates STATUS.md (MATURITY.md "Promotion
      procedure": a different session changes `maturity`).

## Test requirements

Requirements in frontmatter; property and loom tests.

## Notes

Requirements written by M1-T034 (spec §6 "Assignment"). Scope changes made by M1-T034, with reasons: (1) panic propagation of `scope` and `join` (JOBS-R101, JOBS-R102) moved to M1-T035 (soundness); this task has the panic rules of the loop functions and graphs and the test over all six pool calls (JOBS-R100). (2) This task adds the `jobs.scope` span to M1-T035's `scope` (JOBS-R120). (3) Benchmarks are the cases of spec §9, including the `deque` / `reference` pair that ADR-0018 requires, and the queue-capacity comparison of spec open question 3. (4) JOBS-R087 (SHOULD, no allocation in `run_graph` from the second run): implement it, or report in the handoff why not. `parallel`, `graph`, `reference` and `error` declare `#![forbid(unsafe_code)]` (spec §3); the allocation test of JOBS-R064 is a test binary of its own.

## Handoff

_Empty._

## Review

_Empty._
