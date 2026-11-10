---
id: M1-T035
title: Implement liminal_jobs — pool, work-stealing deques, scoped tasks
milestone: M1
status: todo
crates: [liminal_jobs]
depends_on: [M1-T034, M1-T028, M1-T018]
requirements: [JOBS-R001, JOBS-R002, JOBS-R003, JOBS-R004, JOBS-R005, JOBS-R006, JOBS-R007, JOBS-R008, JOBS-R009, JOBS-R010, JOBS-R020, JOBS-R021, JOBS-R022, JOBS-R023, JOBS-R024, JOBS-R025, JOBS-R026, JOBS-R027, JOBS-R028, JOBS-R040, JOBS-R041, JOBS-R042, JOBS-R043, JOBS-R044, JOBS-R101, JOBS-R102, JOBS-R140]
---

# M1-T035: Implement `liminal_jobs`: pool, work-stealing deques, scoped tasks

## Goal

A work-stealing thread pool runs scoped tasks and `join`, checked by loom and Miri.

## Context

- `docs/specs/liminal_jobs.md`; ADR-0018.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Crate, pool, deques with a locked reference implementation, scoped tasks and `join`, loom and Miri.

**Out:** Parallel-for and job graphs: next task.

## Acceptance criteria

- [ ] The crate exists at the path in `architecture.toml`, is a workspace member, has
      `#![forbid(unsafe_code)]` if `unsafe = "forbid"`, and has a floor in `ci/coverage.toml`.
- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.
- [ ] Differential tests: the lock-free deque and the locked reference agree on generated operation sequences.
- [ ] loom tests cover push, pop and steal interleavings; Miri passes; `// SAFETY:` on every `unsafe` block.

## Test requirements

Requirements in frontmatter; differential, loom, Miri.

## Notes

Requirements written by M1-T034 (spec §6 "Assignment"). Scope additions made by M1-T034, with reasons: (1) the panic rules of `scope` and `join` (JOBS-R101, JOBS-R102) are in this task, not M1-T036: both are unsound unless they wait for their jobs when a closure panics, so they cannot ship without the panic path. (2) The deque is public (`liminal_jobs::deque`) and its locked reference is the public module `liminal_jobs::reference` (PERFORMANCE.md "Reference implementations reachable from benchmarks"); the differential test is `tests/deque_differential.rs` (JOBS-R028). (3) This task adds the `liminal_telemetry` dependency but records no span (JOBS-R120 is M1-T036's). JOBS-R011 (SHOULD, parking) is implemented here. `unsafe` only in `deque`, `job`, `pool` (spec §3). JOBS-R002 and JOBS-R004 are tested through the private worker-start function.

## Handoff

_Empty._

## Review

_Empty._
