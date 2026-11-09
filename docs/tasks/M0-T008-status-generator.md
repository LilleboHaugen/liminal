---
id: M0-T008
title: status generator
milestone: M0
status: done
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

**CI:** run 42612728836 on `main` (1a0a0ef): all 4 jobs pass (macOS, Windows, Linux, coverage).

**Next:** review (fresh session). Then M0-T009 (`cargo deny` policy and `doctor`).

## Review

2026-11-09, new owner-started session (fresh context; did not write M0-T008). Outcome:
**done**, no blocking findings. Follow-ups in M1-T007.

Checked, macOS 27.2, Rust 1.99.1, cargo-llvm-cov 0.9.1:
1. `cargo xtask ci` locally: pass, 10 steps (`status-check` 6 ms, coverage pass).
2. Acceptance criteria, by running and reading:
   - `status` twice → identical file, second run "up to date" (test
     `running_status_twice_…`, and `status-check` passes on the real repository).
   - `--check` fails after a task status edit, at the right line, and writes nothing (test
     `check_fails_after_a_task_status_is_edited_without_regenerating`).
   - Snapshot `xtask/tests/snapshots/STATUS.md` against the fixture: read against the spec
     layout line by line (notice, intro, current milestone = lowest with an open task, `\|`
     escape, `M2` before `M10`, task-number order, milestone without dash heading, crates by
     layer then name, `ghost` spec ignored). Matches.
   - `status-check` is in `ci` after `hygiene`; `docs/STATUS.md` is generated and current.
3. Tests vs. mutants. Five own mutants, each reverted, all killed: current milestone = first
   task's milestone; crates sorted by name only; duplicate task IDs accepted; `--check` silent
   when the file is missing; spec `maturity` not validated.
4. Spec agreement: code read against "`status` details". `depends_on` checked for list form
   only; R023 ID existence is a cross-file rule, so that matches "not of the R023 form". One
   wording difference (spec link `<crate>.md` vs. actual file name) is identical for every
   file following `docs/process/DOCUMENTATION.md`; follow-up, not B5.
5. Docs: AGENTS.md (orient step, table rows that already named `STATUS_NOTES.md`), WORKFLOW.md
   §1 and REVIEW.md changes are reference updates in task scope, not rule changes. ADR-0021
   ("`docs/STATUS.md` lists it under Blockers") still holds through `## Blocked tasks`. No
   content of the old hand-written STATUS.md was lost (prerequisites, tools, follow-up table,
   module notes are in `docs/STATUS_NOTES.md`).
6. CI: `gh run view 42612728836`: head 1a0a0ef, all 4 jobs `success`.
7. No placeholders, no weakened tests, no new dependency, no `unsafe`. Coverage 97.56 % vs. floor
   97.6 within the 0.5-point tolerance; floor unchanged. Session boundaries: one review, then
   this implementation, in an owner-started session; no sub-agents.

Follow-ups (non-blocking, ADR-0021) in `docs/tasks/M1-T007-status-follow-ups.md`:
1. Spec says spec link `[<crate>.md](specs/<crate>.md)`, code writes the actual file name; they
   differ for a misnamed spec of a non-member crate, which `check-docs` accepts.
2. Two specs with the same `crate` pass `check-docs` (R022 checks `req_prefix` uniqueness only);
   `status` silently uses the first by path, and its doc comment wrongly says R022 reports it.
3. "Write only if changed" is not observable in tests (an always-write mutant passes).
