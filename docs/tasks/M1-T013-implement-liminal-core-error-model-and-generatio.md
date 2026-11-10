---
id: M1-T013
title: Implement liminal_core — error model and generational handles
milestone: M1
status: in-progress
crates: [liminal_core]
depends_on: [M1-T012, M1-T011, M1-T003, M1-T004, M1-T005]
requirements: [CORE-R001, CORE-R002, CORE-R003, CORE-R004, CORE-R005, CORE-R006, CORE-R007, CORE-R008, CORE-R009, CORE-R010, CORE-R011, CORE-R012, CORE-R013, CORE-R030, CORE-R031, CORE-R032, CORE-R033, CORE-R034, CORE-R035, CORE-R036, CORE-R037, CORE-R038, CORE-R039, CORE-R040]
---

# M1-T013: Implement `liminal_core`: error model and generational handles

## Goal

`liminal_core` exists with the error model and generational IDs/handles specified in its spec.

## Context

- `docs/specs/liminal_core.md` (requirements assigned to this task).
- ADR-0017; CODE_STYLE.md §Errors. Follow-ups M1-T003–T005 harden the gates this first engine crate meets.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** The crate, its error model and its handle types, with unit, integration and property tests.

**Out:** Hashing, RNG, collections, time types: the next `liminal_core` task.

## Acceptance criteria

- [ ] The crate exists at the path in `architecture.toml`, is a workspace member, has
      `#![forbid(unsafe_code)]` if `unsafe = "forbid"`, and has a floor in `ci/coverage.toml`.
- [ ] The crate's clippy determinism configuration is in place and `check-arch` accepts it.
- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.

## Test requirements

Requirements in frontmatter (assigned by the spec task). Property tests for handle generation and reuse rules.

## Notes

_None._

## Handoff

_Empty._

## Review

_Empty._
