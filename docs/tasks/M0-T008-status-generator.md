---
id: M0-T008
title: status generator
milestone: M0
status: review
crates: [xtask]
depends_on: [M0-T003]
requirements: [XTASK-R010, XTASK-R070, XTASK-R071, XTASK-R072]
---

# M0-T008: `status` generator

## Goal

`docs/STATUS.md` is generated from the single sources of truth (task and spec frontmatter,
`architecture.toml`) and cannot drift.

## Context

`docs/specs/xtask.md` R070–R072. The current hand-written `docs/STATUS.md` shows the intended content.
It also lists prerequisites and blockers in prose, and the generator cannot know those. Keep that prose
as a separate hand-written file, `docs/STATUS_NOTES.md`, linked from the generated file.

## Scope

**In:** `status` and `status --check`; deterministic Markdown output; `status-check` step in `ci`;
moving hand-written notes to `docs/STATUS_NOTES.md`; updating references in AGENTS.md and
WORKFLOW.md if wording changes.

**Out:** history or graphs.

## Acceptance criteria

- [x] `cargo xtask status` regenerates `docs/STATUS.md`. Running it twice gives identical files.
- [x] `cargo xtask status --check` fails after a task's status is edited without regenerating.
- [x] Snapshot test of the output for a fixture repository.
- [x] `status-check` is a step in `ci`. Generated file committed.

## Test requirements

`Verifies:` tests for R070–R072.

## Handoff

2026-11-09, implementation session (after the review of M0-T007 in the same session). Status →
`review`.

**Done (all acceptance criteria met):**
- Spec `docs/specs/xtask.md` v12: "`status` details" (sources, unreadable input is an R070
  finding and nothing is written, sort order, exact layout, write only if changed, `--check`
  finding at the first differing line).
- `xtask/src/status.rs`: `status` and `status --check` commands; `status-check` native step in
  `ci` after `hygiene` (`xtask/src/ci.rs`). Human `ci` output shows the step as
  `cargo xtask status --check`. Reuses `check_docs::fields::{is_milestone, is_task_id}` and
  `check_docs::rules::{MATURITIES, TASK_STATUSES}` (made `pub(crate)`).
- Tests `xtask/src/status/tests.rs` (10): snapshot of fixture `xtask/tests/fixtures/status/valid/`
  against `xtask/tests/snapshots/STATUS.md` (written by hand from the spec before the code);
  running twice is identical and ignores the old file; `--check` fails at line 17 after a task
  status edit and writes nothing; missing file / missing final newline; 8 unreadable-input
  cases; no open tasks; milestone heading form; real repository readable; usage errors.
  `ci` tests: step list and the `status --check` display.
- Own mutants (3, each reverted): no `|` escape, lexical task sort, `first_difference` ignoring
  length. All killed.
- `docs/STATUS.md` is generated and committed. Hand-written prose moved to
  `docs/STATUS_NOTES.md`. References updated: AGENTS.md (orient step reads both files; table
  rows), `docs/process/WORKFLOW.md` §1, `docs/process/REVIEW.md` (blocked-task decisions go to
  STATUS_NOTES; the generated file lists `blocked` tasks by itself, which keeps ADR-0021's
  "STATUS.md lists it" true).

**Decisions beyond the task text:** a "Milestones" count table and milestone names from
`docs/ROADMAP.md` headings (`## M<n> — <name>`) are extra output, specified in spec v12. The
spec column links each crate's spec.

**Coverage:** `xtask` 97.56 % (4879 of 5001 lines) against floor 97.6: passes within the
0.5-point tolerance. Floor unchanged (it only rises).

**Not verified:** Windows and Linux locally; CI runs them after the push.

**Next:** review (fresh session). Then M0-T009 (`cargo deny` policy and `doctor`).

## Review

_Empty._
