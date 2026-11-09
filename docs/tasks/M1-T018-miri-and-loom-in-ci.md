---
id: M1-T018
title: Miri and loom in CI
milestone: M1
status: todo
crates: [xtask]
depends_on: [M1-T014, M1-T006]
requirements: []
---

# M1-T018: Miri and loom in CI

## Goal

Unsafe and lock-free code in M1 crates is checked by Miri and loom on every push, before the first `unsafe = "allow"` crate gets code.

## Context

- ADR-0018; TESTING.md (Miri, Concurrency rows); CODE_STYLE.md (nightly only for Miri, fuzzing, API snapshots); ADR-0022 (installs need the owner's yes); M1 exit criterion 4.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** A CI job running Miri (pinned nightly) on every workspace crate with `unsafe = "allow"` and loom (`--cfg loom`) on crates that have loom tests; how a test excluded from Miri is listed (attribute plus reason naming a task); whether these run in `cargo xtask ci` locally.

**Out:** The tests themselves: each crate's tasks.

## Acceptance criteria

- [ ] CI job exists and runs on every push; it is shown to fail on a seeded Miri error (scratch branch, journal records it).
- [ ] The nightly date is pinned in one place, referenced by CODE_STYLE.md.
- [ ] Before any local install the owner was asked (ADR-0022) with tool, command, reason, location and undo; the answer is in the journal. If local Miri is declined, the task says how agents run Miri (CI only) and DoD wording is checked against it.
- [ ] Spec changes for any new xtask step or command, with tests; `cargo xtask ci` passes.

## Test requirements

Tests for any new xtask behaviour; the seeded-failure run.

## Notes

Ask the owner early in the session; the rest of the work does not depend on the answer.

## Handoff

_Empty._

## Review

_Empty._
