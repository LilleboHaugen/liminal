---
id: M1-T005
title: coverage follow-ups from the M0-T006 review
milestone: M1
status: review
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

- [x] Item 1 decided, written into the spec, implemented, and tested (a crate with floor > 0 and
      no measured lines gives the decided outcome).
- [x] Item 2: a workspace with `members = ["./xtask"]` maps `xtask/src/*.rs` to `xtask` in
      `coverage` and in `repo/sources.rs`, or is rejected with a stated error, tested.
- [x] `cargo xtask ci` passes.

## Test requirements

`Verifies: XTASK-R061` on the item 1 test, `Verifies: XTASK-R060` on the item 2 mapping test.

## Handoff

2026-11-09, implementation sub-agent (parent session `session/2026-11-09-m1-subagents`).

Done:
- Item 1, decided: a crate with a floor above 0.0 and no measured lines is an XTASK-R061
  **error** (its coverage is unknown, not 100 %). A `warning` was rejected: the `ci` step fails
  on any finding anyway, and a ratchet that can vanish needs a hard stop. Floor 0.0 with no lines
  still passes; `--update-floors` still keeps the floor. Spec `docs/specs/xtask.md` v18
  ("`coverage` details"). Code: `xtask/src/coverage.rs::evaluate` (new match arm before
  `below_floor`). Test: `coverage::tests::a_crate_with_a_floor_above_0_and_no_measured_lines_fails`.
  The existing test `floors_below_the_tolerance_and_crates_without_lines_never_fail` asserted the
  old behaviour (floor 100.0, no lines → pass); it is renamed
  `..._and_unmeasured_crates_with_floor_0_never_fail` and uses floor 0.0 (intended behaviour change).
- Item 2: `xtask/src/repo/workspace.rs::normalize` removes `.` segments, empty segments and a
  trailing `/` from each member entry before expansion (`./xtask` → `xtask`, `./crates/*` →
  `crates/*`). Spec v18 "`check-docs` details" (member entries). Tests (`Verifies: XTASK-R060`):
  `repo::workspace::tests::member_paths_with_dot_segments_are_normalized`,
  `repo::coverage::tests::a_member_spelled_dot_slash_owns_its_files` (loader + `parse_export`),
  `repo::sources::tests::a_member_spelled_dot_slash_owns_its_files`.

Not done / traps:
- `..` segments in member paths are not normalized (out of scope; cargo resolves them, xtask
  would then map no files). A member entry that is only `.` stays `.` (the root package).
- `\` in member entries (Windows spelling) is not normalized; not verified on Windows.

## Review

_Empty._
