---
id: M1-T005
title: coverage follow-ups from the M0-T006 review
milestone: M1
status: in-progress
crates: [xtask]
depends_on: [M0-T006]
requirements: [XTASK-R060, XTASK-R061]
---

# M1-T005: `coverage` follow-ups from the M0-T006 review

## Goal

The non-blocking findings of the M0-T006 review are fixed: a crate's coverage ratchet cannot be
disabled silently by a crate that measures no lines, or by a member path spelled `./name`.

## Context

- Follow-up task under ADR-0021, written by the review of M0-T006 (see that task's "Review").
- `docs/specs/xtask.md` "`coverage` details" (spec v11): "A crate with no measured lines is shown
  as such, is never below its floor and keeps its floor on update."
- `xtask/src/coverage.rs` (`below_floor`, `evaluate`), `xtask/src/repo/coverage.rs`
  (`parse_export`), `xtask/src/repo/workspace.rs` (`expand`, `owner`).
- No M0 exit criterion needs this, so it is in M1. The M1 kickoff (M1-T001) places it in the M1
  order; it touches only `xtask`.

## Scope

**In:**
1. A crate with a floor above 0.0 and no measured lines (`count == 0`) passes R061 today. If its
   files drop out of the export for any reason (path spelling, a crate turned into a build-only
   crate), its ratchet is gone without any output except "no lines measured". Decide and write
   into the spec (bump `spec_version`): either such a crate is an R061 error (its coverage is
   unknown, not 100 %), or it stays a pass with a stated reason and a `warning` finding. Test it.
2. `repo/workspace.rs::expand` keeps a member entry `./name` as `./name`; `owner` then matches no
   repository-relative path of that crate. Cargo accepts `members = ["./xtask"]`. Normalize a
   leading `./` (and `a/./b`) in member paths, or reject the spelling with LIM-XTASK-0004 and a
   help line. Test it in the workspace loader and in `parse_export`.

**Out:** other behaviour changes to `coverage`; doc-test coverage (needs nightly; M0-T006 "Out").

## Acceptance criteria

- [ ] Item 1 decided, written into the spec, implemented, and tested (a crate with floor > 0 and
      no measured lines gives the decided outcome).
- [ ] Item 2: a workspace with `members = ["./xtask"]` maps `xtask/src/*.rs` to `xtask` in
      `coverage` and in `repo/sources.rs`, or is rejected with a stated error, tested.
- [ ] `cargo xtask ci` passes.

## Test requirements

`Verifies: XTASK-R061` on the item 1 test, `Verifies: XTASK-R060` on the item 2 mapping test.

## Handoff

_No session has worked on this task yet._

## Review

_Empty._
