---
id: M2-T002
title: workspace member path follow-ups from the M1-T005 review
milestone: M2
status: todo
crates: [xtask]
depends_on: [M1-T005]
requirements: [XTASK-R060]
---

# M2-T002: workspace member path follow-ups from the M1-T005 review

## Goal

Every member entry spelling that cargo accepts either maps the member's files to the member in
every xtask command, or is rejected with LIM-XTASK-0004 and a help line. No spelling makes a
crate's files silently unowned.

## Context

- Follow-up task under ADR-0021, written by the review of M1-T005 (see that task's "Review").
- `docs/specs/xtask.md` v19 "`check-docs` details" (member entries are normalized: `.` segments,
  empty segments, trailing `/`).
- `xtask/src/repo/workspace.rs` (`normalize`, `expand`, `owner`); `xtask/src/repo/sources.rs`;
  `xtask/src/repo/coverage.rs` (`parse_export`).
- M1-T005 handoff "Not done / traps" lists both items below as out of its scope.
- No M1 exit criterion needs this, so it is in M2 (ADR-0021: the reviewer picks a later
  milestone).

## Scope

**In:**
1. `..` segments in a member entry (`members = ["tools/../xtask"]`) are kept as written; `owner`
   then matches no repository-relative path. Since spec v19 `coverage` reports this as an R061
   error (floor above 0, no measured lines), but `repo/sources.rs` users (`hygiene`,
   `check-arch`, `trace`) silently see no files for that crate. Either resolve `..` lexically
   (rejecting an entry that leaves the repository root) or reject `..` with LIM-XTASK-0004.
2. `\` in a member entry (Windows spelling, `members = ["crates\\a"]`) is not read as `/`, while
   `\` in export paths is. Decide: read as `/`, or reject with LIM-XTASK-0004.

**Out:** glob patterns other than `<dir>/*` (already LIM-XTASK-0004).

## Acceptance criteria

- [ ] Item 1 decided, written into the spec (bump `spec_version`), implemented and tested in the
      workspace loader.
- [ ] Item 2 decided, written into the spec, implemented and tested.
- [ ] `cargo xtask ci` passes.

## Test requirements

`Verifies: XTASK-R060` on the mapping tests (or on the rejection tests if rejection is chosen).

## Notes

Unverified assumption: cargo resolves `..` in member paths relative to the root manifest.

## Handoff

_Empty._

## Review

_Empty._
