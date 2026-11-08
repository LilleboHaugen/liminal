---
id: M0-T008
title: status generator
milestone: M0
status: todo
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

- [ ] `cargo xtask status` regenerates `docs/STATUS.md`. Running it twice gives identical files.
- [ ] `cargo xtask status --check` fails after a task's status is edited without regenerating.
- [ ] Snapshot test of the output for a fixture repository.
- [ ] `status-check` is a step in `ci`. Generated file committed.

## Test requirements

`Verifies:` tests for R070–R072.

## Handoff

_Empty._

## Review

_Empty._
