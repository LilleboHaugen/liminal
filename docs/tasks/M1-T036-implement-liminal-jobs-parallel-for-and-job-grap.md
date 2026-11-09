---
id: M1-T036
title: Implement liminal_jobs — parallel-for and job graphs; reach usable
milestone: M1
status: todo
crates: [liminal_jobs]
depends_on: [M1-T035, M1-T017]
requirements: []
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

_None._

## Handoff

_Empty._

## Review

_Empty._
