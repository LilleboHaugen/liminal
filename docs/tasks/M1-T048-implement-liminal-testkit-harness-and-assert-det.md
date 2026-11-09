---
id: M1-T048
title: Implement liminal_testkit — harness and assert_deterministic
milestone: M1
status: todo
crates: [liminal_testkit]
depends_on: [M1-T047, M1-T046]
requirements: []
---

# M1-T048: Implement `liminal_testkit`: harness and `assert_deterministic`

## Goal

Tests can run an app headless and assert determinism across runs and executors.

## Context

- `docs/specs/liminal_testkit.md`; ADR-0024.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Crate, harness, `step(n)`, `assert_deterministic` with divergence report.

**Out:** Replay: next task.

## Acceptance criteria

- [ ] The crate exists at the path in `architecture.toml`, is a workspace member, has
      `#![forbid(unsafe_code)]` if `unsafe = "forbid"`, and has a floor in `ci/coverage.toml`.
- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.
- [ ] A test with a deliberately nondeterministic system shows `assert_deterministic` reports the first diverging step and the differing component.

## Test requirements

Requirements in frontmatter.

## Notes

_None._

## Handoff

_Empty._

## Review

_Empty._
