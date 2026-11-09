---
id: M1-T007
title: status generator follow-ups from the M0-T008 review
milestone: M1
status: review
crates: [xtask]
depends_on: [M0-T008]
requirements: [XTASK-R070]
---

# M1-T007: `status` generator follow-ups from the M0-T008 review

## Goal

The non-blocking findings of the M0-T008 review are resolved: the spec and `xtask/src/status.rs`
agree on the spec link for every input `check-docs` accepts, and the unspecified duplicate-spec
behaviour is either specified and tested or removed.

## Context

- Follow-up task under ADR-0021, written by the review of M0-T008 (see that task's "Review").
- `docs/specs/xtask.md` "`status` details" (`spec_version` 12), `xtask/src/status.rs`.
- No M0 exit criterion needs this, so it is in M1. The M1 kickoff (M1-T001) places it in the M1
  order.

## Scope

**In:**
1. **Spec link text vs. code.** The spec layout says the crate table's spec cell is
   `[<crate>.md](specs/<crate>.md)`. `status.rs` (`render`) writes the spec's actual file name,
   `[<file>](specs/<file>)`. Both give the same output while spec files follow the convention
   `docs/specs/<crate>.md` (`docs/process/DOCUMENTATION.md`). They differ when a spec for a
   crate that is not a workspace member (so R026 does not apply) has a different file name,
   for example `docs/specs/render.md` with `crate: liminal_render`. `check-docs` R022 accepts that
   file. Choose one:
   (a) change the spec wording to the file name (the code's behaviour; the link always resolves), or
   (b) add a `check-docs` rule that a spec file is named `<crate>.md`.
   Add a fixture case that tells the two apart.
2. **Duplicate specs for one crate.** `status.rs` `specs()` keeps the first spec by path when two
   specs name the same crate, and its doc comment says "`check-docs` R022 reports the other".
   That is false: R022 (`xtask/src/check_docs/rules.rs`) checks that `req_prefix` is unique, not
   that `crate` is unique. Input: copy `docs/specs/xtask.md` to `docs/specs/xtask_old.md`, change
   only `req_prefix` and `maturity`. `check-docs` passes. `status` shows whichever file sorts first,
   with nothing reported. Add "`crate` is unique across specs" to R022 (spec + `check-docs` +
   test), and either specify the `status` behaviour with a test or make it an XTASK-R070
   unreadable-input finding like a duplicate task ID. Fix the doc comment.
3. **Write only if changed.** The spec says `status` writes `docs/STATUS.md` only if the content
   differs. The tests check the printed message and the content, but a mutant that always writes
   (and keeps the message logic) passes. If this matters (file timestamps, editor reloads), add a
   test that the file's modification time does not change on an up-to-date run. If not, record
   that the message is the only observable contract.

**Out:** history or graphs (as in M0-T008).

## Acceptance criteria

- [x] Item 1 decided and implemented; a fixture case distinguishes spec file name from crate name.
- [x] Item 2 specified and tested, or turned into an R070 finding with a test.
- [x] Item 3 tested, or the decision recorded in the spec.
- [x] Spec `spec_version` bumped if its text changes; `cargo xtask ci` passes.

## Test requirements

`Verifies: XTASK-R070` tests for each behaviour added or specified.

## Handoff

Session 2026-11-09 (implementation sub-agent). Status `review`.

**Premise correction (scope note).** Items 1 and 2 assume `check-docs` R022 accepts a spec whose
file name differs from its `crate`. It does not: since M0-T003, `xtask/src/check_docs/rules.rs`
reports "`crate` is `<name>`, but the file is <stem>.md" (spec §6 details, "A spec's `crate`
equals its file name without `.md`"). Reproduced on the real repository: copying
`docs/specs/xtask.md` to `xtask_old.md` (only `req_prefix`, `maturity` changed) makes
`check-docs` fail with that R022 finding (and R027, not indexed). So option (b) of item 1 already
exists, and the file-name rule already makes `crate` unique across specs. Adding a separate
"`crate` is unique" check would only give a second finding for the same file; instead the spec
now states the implication and a test pins it. The finding that `status` reported nothing was
correct and is fixed (below).

Done:
1. Item 1: option (a) on top of the existing rule. Spec layout 6 now says the cell is
   `[<file>](specs/<file>)` with the spec's file name (equal to `<crate>.md` when R022 passes;
   the link names an existing file otherwise). Code unchanged in behaviour (`Spec` now keeps the
   path, see item 2). Test `the_spec_cell_links_the_spec_file_by_its_file_name_not_the_crate_name`
   (`xtask/src/status/tests.rs`) renames the fixture's `alpha_lib.md` to `alpha.md`.
2. Item 2: a spec whose `crate` an earlier spec (by path) already names is an XTASK-R070
   unreadable-input finding at its `crate` line, like a duplicate task ID
   (`xtask/src/status.rs` `specs()`, doc comment fixed). Tests:
   `two_specs_for_one_crate_are_unreadable_input` (status) and
   `r022_a_second_spec_for_one_crate_is_reported_by_its_file_name` (`check_docs/tests.rs`).
3. Item 3: tested. `an_up_to_date_file_is_not_rewritten_and_keeps_its_modification_time` sets
   the file's mtime to 2001 with `File::set_modified`, runs `status`, and checks the mtime; it
   also checks that a real write changes it. Verified by hand that the always-write mutant fails
   this test (and only this test).
4. Spec `docs/specs/xtask.md` `spec_version` 18 → 19, history row added.

Verified: `cargo xtask ci` passes on macOS (Darwin 27.2), all 11 steps, none skipped.

Not done: nothing in scope. Traps: none found beyond the premise above.

## Review

_Empty._
