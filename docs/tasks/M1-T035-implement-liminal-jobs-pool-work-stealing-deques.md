---
id: M1-T035
title: Implement liminal_jobs — pool, work-stealing deques, scoped tasks
milestone: M1
status: todo
crates: [liminal_jobs]
depends_on: [M1-T034, M1-T028, M1-T018]
requirements: []
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

_None._

## Handoff

_Empty._

## Review

_Empty._
