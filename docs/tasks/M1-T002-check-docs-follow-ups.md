---
id: M1-T002
title: check-docs follow-ups from the M0-T003 review
milestone: M1
status: todo
crates: [xtask]
depends_on: [M0-T003]
requirements: [XTASK-R020, XTASK-R025]
---

# M1-T002: `check-docs` follow-ups from the M0-T003 review

## Goal

The two non-blocking findings recorded during the M0-T003 reviews are fixed: Markdown table cells
are trimmed the way GitHub Flavored Markdown trims them, and a file that starts with a byte order
mark gets an error that names it.

## Context

- Follow-up task under ADR-0021, written by the fifth review of M0-T003 (see that task's "Review 5").
- `docs/specs/xtask.md` §6 "`check-docs` details" (frontmatter subset, indexes R025/R027).
- No M0 exit criterion needs this, so it is in M1. The M1 kickoff (M1-T001) places it in the M1
  order; it touches only `xtask`.

## Scope

**In:**
1. `xtask/src/markdown.rs` (`table_cells`, around lines 241–259): cells and the whole line are
   trimmed with Unicode `str::trim`. GFM trims only space and tab (and the line ending). Frontmatter
   values keep U+00A0 (spec v6). So a trailing U+00A0 only in the index title is lost and the
   mismatch is not reported; a trailing U+00A0 in both titles gives a false R025 mismatch. Trim
   only space and tab, the same
   `WHITESPACE` rule `frontmatter.rs` uses (spec v6). Specify it in spec §6 (bump `spec_version`).
   Origin: observation in M0-T003 "Review 3 changes".
2. `xtask/src/frontmatter.rs`: a file whose first character is U+FEFF (some Windows editors save
   UTF-8 with a BOM) is rejected with "line 1 must be `---`, which opens the frontmatter". The
   rejection is correct (spec v7); the message does not say why, because the BOM is invisible.
   Name U+FEFF in the message when line 1 is `\u{feff}---`.

**Out:** accepting a BOM. The spec rejects it so the subset means the same in YAML 1.1 and 1.2.

## Acceptance criteria

- [ ] A table cell `x\u{a0}` keeps its U+00A0; `  x\t` becomes `x`. Unit tests in `markdown.rs`.
- [ ] An R025 test: ADR index title with a trailing U+00A0, frontmatter title without it → one
      R025 finding.
- [ ] `\u{feff}---\nkey: a\n---\n` is rejected at line 1 with a message naming U+FEFF (test).
- [ ] Spec §6 states the table-cell trimming rule; `spec_version` bumped with a History row.
- [ ] `cargo xtask ci` passes.

## Test requirements

`Verifies: XTASK-R020` on the BOM test, `Verifies: XTASK-R025` on the index test.

## Notes

- `markdown.rs` line 46 (`line.trim()` in the fence check) and line 230 (`trim_start` in
  `is_table_line`) follow CommonMark rules of their own (up to 3 leading spaces). Check each
  against CommonMark before changing it; only the table-cell trimming is a known defect.

## Handoff

_Filled in by the implementing session(s)._

## Review

_Filled in by the reviewing session._
