---
id: M1-T002
title: check-docs follow-ups from the M0-T003 review
milestone: M1
status: review
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

- [x] A table cell `x\u{a0}` keeps its U+00A0; `  x\t` becomes `x`. Unit tests in `markdown.rs`.
- [x] An R025 test: ADR index title with a trailing U+00A0, frontmatter title without it → one
      R025 finding. (Changed to the status cell; see Handoff "Scope changes".)
- [x] `\u{feff}---\nkey: a\n---\n` is rejected at line 1 with a message naming U+FEFF (test).
- [x] Spec §6 states the table-cell trimming rule; `spec_version` bumped with a History row.
- [x] `cargo xtask ci` passes.

## Test requirements

`Verifies: XTASK-R020` on the BOM test, `Verifies: XTASK-R025` on the index test.

## Notes

- `markdown.rs` line 46 (`line.trim()` in the fence check) and line 230 (`trim_start` in
  `is_table_line`) follow CommonMark rules of their own (up to 3 leading spaces). Check each
  against CommonMark before changing it; only the table-cell trimming is a known defect.

## Handoff

2026-11-09, one session (it reviewed M1-T001 first, journal `2026-11-09-M1-T001-review.md`).
Journal: `2026-11-09-M1-T002.md`. Status: `review`.

**Done:**
1. `xtask/src/markdown.rs`: new `WHITESPACE` (space, tab). `cells` trims the line and each cell
   with it; `is_table_line` skips only spaces and tabs; the closing-fence check
   (`prose_lines`) allows only spaces and tabs after the fence characters.
2. `xtask/src/frontmatter.rs` `parse`: line 1 equal to `\u{feff}---` (after trailing space/tab
   removal) gives "line 1 must be `---`, which opens the frontmatter; it starts with U+FEFF (byte
   order mark), which is not allowed: save the file as UTF-8 without BOM". Any other line 1 keeps
   the old message.
3. Spec `docs/specs/xtask.md` v18: two bullets in §6 "`check-docs` details", History row.

**Tests** (each failed before the change, passes after):
- `markdown::tests::table_cells_trim_only_space_and_tab` (`Verifies: XTASK-R025`),
  `a_line_indented_with_u00a0_is_not_a_table_line`,
  `a_fence_followed_by_u00a0_does_not_close_the_block` (`Verifies: XTASK-R024`).
- `check_docs::tests::r025_a_trailing_u00a0_in_the_status_cell_is_a_mismatch`
  (`Verifies: XTASK-R025`): `accepted` + U+00A0 in the index → one R025 finding at
  `docs/adr/README.md:8`; spaces and tabs around the cell → no finding.
- `frontmatter::tests::a_byte_order_mark_before_the_opening_line_is_named`
  (`Verifies: XTASK-R020`), and the BOM row of `rejects_every_construct_outside_the_subset_with_its_line`
  now expects "starts with U+FEFF".

**Scope changes (recorded per AGENTS.md rule 2):**
- R025 compares the **status** column only (spec §6 "Indexes"; `indexes.rs`); it never compared
  titles, so the Scope's "trailing U+00A0 only in the index title is lost" could not happen. The
  same defect showed in the status cell (`accepted` + U+00A0 was trimmed to `accepted` and not
  reported), so the R025 test uses the status cell. The "false mismatch when both have U+00A0"
  case cannot occur for status: the frontmatter `status` is an enum value.
- Notes asked to check `markdown.rs` line 46 and line 230 against CommonMark. Both were wrong
  for U+00A0 (CommonMark: a closing fence is followed only by spaces or tabs; indentation is
  spaces), so both now use the same space/tab rule, with a test each. Tab indentation of a table
  line (CommonMark: a tab indents to column 4, which makes an indented code block) is unchanged
  and not covered; it was accepted before as well.

**Verified:** macOS 27.2, Rust 1.99.1: `cargo xtask ci` passes (11 steps).

## Review

_Filled in by the reviewing session._
