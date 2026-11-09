---
id: M2-T003
title: status write-if-changed requirement tag follow-up from the M1-T007 review
milestone: M2
status: todo
crates: [xtask]
depends_on: [M1-T007]
requirements: [XTASK-R070]
---

# M2-T003: `status` write-if-changed requirement tag follow-up from the M1-T007 review

## Goal

Every `Verifies:` tag on a `status` test names a requirement the test verifies.

## Context

- Follow-up task under ADR-0021, written by the review of M1-T007 (see that task's "Review").
- `docs/specs/xtask.md` v20 "`status` details": "`status` writes `docs/STATUS.md` only if its
  content differs (an up-to-date file is not opened for writing, so its modification time stays)".
- `xtask/src/status/tests.rs`
  `an_up_to_date_file_is_not_rewritten_and_keeps_its_modification_time` carries
  `Verifies: XTASK-R071`. XTASK-R071 is "output is deterministic, no timestamps, generated
  notice"; the test checks write-if-changed, which no requirement row names.
- No M1 exit criterion needs this, so it is in M2 (ADR-0021).

## Scope

**In:**
1. Either retag the test to the requirement whose details hold write-if-changed (XTASK-R070,
   under which the `status` details are listed), or add write-if-changed to a requirement row
   (bump `spec_version`) and tag the test with it.

**Out:** any behaviour change of `status`.

## Acceptance criteria

- [ ] The test's `Verifies:` tag names a requirement whose text or details the test checks.
- [ ] `cargo xtask ci` passes.

## Test requirements

No new behaviour; `cargo xtask trace` still passes.

## Handoff

_Empty._

## Review

_Empty._
