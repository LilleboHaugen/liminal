---
id: M2-T001
title: check-docs Markdown follow-ups from the M1-T002 review
milestone: M2
status: todo
crates: [xtask]
depends_on: [M1-T002]
requirements: [XTASK-R025]
---

# M2-T001: `check-docs` Markdown follow-ups from the M1-T002 review

## Goal

The two non-blocking findings of the M1-T002 review are fixed: `fence_marker` in
`xtask/src/markdown.rs` has its own doc comment again, and the line-level space/tab trimming in
`cells` is covered by a test.

## Context

- Follow-up task under ADR-0021, written by the review of M1-T002 (see that task's "Review").
- `docs/specs/xtask.md` §6 "`check-docs` details", bullet "Markdown tables and fences"
  (`spec_version` 18).
- No M1 exit criterion needs this, so it is in M2 (REVIEW.md §1, "Findings and outcome").
  Assumption: the M2 kickoff task, created by M1-T053, takes the next free M2 ID.

## Scope

**In:**
1. `xtask/src/markdown.rs`: M1-T002 inserted `const WHITESPACE` between the doc comment of
   `fence_marker` ("The fence character and run length if `line` starts a fence …") and the
   function. Rustdoc now joins both paragraphs onto `WHITESPACE`, and `fence_marker` has no doc
   comment. Move `WHITESPACE` (with its own doc comment) above the `fence_marker` doc comment.
2. `xtask/src/markdown.rs` `cells`: the first line, `line.trim_matches(WHITESPACE)`, is untested.
   Replacing it with `line.trim()` passes every test (mutation run in the M1-T002 review). The
   inputs that tell them apart have Unicode whitespace other than space/tab outside the outer
   pipes, e.g. `| a | b |\u{a0}`: with the space/tab rule the trailing pipe is not the last
   character, so the row has a third cell `\u{a0}` (GFM agrees: the text after the last pipe is a
   cell); with `str::trim` the row has two cells. Add a unit test that pins the space/tab rule
   for the whole line.

**Out:** tab indentation of table lines (CommonMark: a tab indents to column 4, an indented code
block). M1-T002 "Handoff" records it as unchanged; open a task only if a real file needs it.

## Acceptance criteria

- [ ] `fence_marker` has its doc comment; `WHITESPACE` has only its own.
- [ ] A unit test in `markdown.rs` fails when the line-level trim in `cells` uses `str::trim`.
- [ ] `cargo xtask ci` passes.

## Test requirements

`Verifies: XTASK-R025` on the new `cells` test (index tables are read with `cells`).

## Notes

- Check `cargo doc --document-private-items -p xtask` output, or read the source, for item 1.

## Handoff

_Filled in by the implementing session(s): date, what is done, what is not, the exact next step,
traps found._

## Review

_Filled in by the reviewing session: date, what was checked, outcome._
