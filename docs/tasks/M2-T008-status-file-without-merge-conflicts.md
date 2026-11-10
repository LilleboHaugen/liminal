---
id: M2-T008
title: Remove the docs/STATUS.md merge conflict from task branches
milestone: M2
status: todo
crates: [xtask]
depends_on: [M1-T007]
requirements: []
---

# M2-T008: Remove the `docs/STATUS.md` merge conflict from task branches

## Goal

A task branch that changes a task status merges into `main` without a conflict in the generated
`docs/STATUS.md`, and `main` still never carries a stale `docs/STATUS.md`.

## Context

- ADR-0030 Context observation 6 and Decision 9: every merge on 2026-11-10 conflicted in
  `docs/STATUS.md`, because each task branch commits its own regenerated copy. Each reviewer
  regenerated it by hand (`cargo xtask status`). `xtask` is frozen except for tooling that slows
  engine work; this task is the one named exception.
- `docs/specs/xtask.md`: XTASK-R070 … R073 (`status`), the `status-check` step of
  `cargo xtask ci`.
- `docs/process/WORKFLOW.md` §7 step 3 and §8.

## Scope

**In:** one design that meets the goal, its spec change, its tests, and the matching wording in
`docs/process/WORKFLOW.md`. Candidates (the task chooses and says why):
1. `status-check` compares against a stale file only on `main` (or when a flag is given), and
   task branches do not commit `docs/STATUS.md`.
2. A git merge driver for `docs/STATUS.md` that regenerates the file, registered in
   `.gitattributes`, with the setup step documented.
3. `docs/STATUS.md` is no longer tracked; `cargo xtask status` prints it or writes it untracked.

**Out:** any other `xtask` change. The `spec_version` collisions in `docs/specs/xtask.md` need
no change: with `xtask` frozen, only M1-T008, T009 and T010 still bump it.

## Acceptance criteria

- [ ] A test builds two branches that each change a task status, merges them, and shows no
      conflict in `docs/STATUS.md` (or shows the file is not part of the merge).
- [ ] `cargo xtask ci` on `main` still fails when the status overview does not match the task
      and spec frontmatter, shown by a seeded test.
- [ ] `docs/specs/xtask.md` states the new behaviour; `spec_version` bumped with a history row.
- [ ] `docs/process/WORKFLOW.md` and `AGENTS.md` ("Where things live", if affected) agree with it.
- [ ] `cargo xtask ci` passes.

## Test requirements

`Verifies:` tests for every XTASK requirement the task adds or changes.

## Notes

- If candidate 3 is chosen, `docs/STATUS.md` links in other documents and the session protocol
  (AGENTS.md step 1) need a replacement; that is a rule change and needs the owner.

## Handoff

_Filled in by the implementing session(s): date, what is done, what is not, the exact next step,
traps found._

## Review

_Filled in by the reviewing session: date, what was checked, outcome._
