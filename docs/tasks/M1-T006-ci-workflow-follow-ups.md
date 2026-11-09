---
id: M1-T006
title: CI workflow follow-ups from the M0-T007 review
milestone: M1
status: review
crates: [xtask]
depends_on: [M0-T007]
requirements: []
---

# M1-T006: CI workflow follow-ups from the M0-T007 review

## Goal

The non-blocking findings of the M0-T007 review are resolved: pull-request runs are verified and
not duplicated, and pinned action SHAs have a defined update path.

## Context

- Follow-up task under ADR-0021, written by the review of M0-T007 (see that task's "Review").
- `.github/workflows/ci.yml` (M0-T007), ADR-0005.
- No M0 exit criterion needs this, so it is in M1. The M1 kickoff (M1-T001) places it in the M1
  order.

## Scope

**In:**
1. The workflow triggers on every `push` and every `pull_request`, and the concurrency group
   includes `github.event_name`. A branch with an open pull request therefore runs all jobs
   twice per push (once per event), and neither run cancels the other. No `pull_request` run
   has been observed yet (M0-T007 handoff, "Not verified"). Open a pull request from a task
   branch, confirm the `pull_request` run passes, and decide the trigger set (for example
   `push: branches: [main]` plus `pull_request`, or keep both and accept the duplicate cost).
   Record the decision and the reason in the workflow header comment and the journal.
2. Third-party actions are pinned by commit SHA with a manual re-pin procedure (workflow header).
   Nothing tells an agent when a pinned version is outdated or has a security advisory. Decide
   whether to enable Dependabot version updates for the `github-actions` ecosystem
   (`.github/dependabot.yml`, GitHub-native, no new crate dependency) or to keep manual updates
   with a stated review cadence. Record the decision.

Note (no work): the deliberate-failure experiment in M0-T007 (`fn   badly_formatted ( ) { }` in
`xtask/src/lib.rs`) also failed `clippy` (`dead_code` under `-D warnings`). A future failure
experiment should use an input that fails only the targeted step (e.g. mis-indenting an existing
line).

**Out:** Miri job (M1, first unsafe crate), GPU jobs (M3), release workflows (M7), the `deny` job
(M0-T009).

## Acceptance criteria

- [x] A `pull_request`-triggered run on a task branch passes; run ID in the journal.
- [x] Trigger set decided, implemented, reason in the workflow header and the journal.
- [x] Action-update path decided (Dependabot config committed, or manual cadence written into the
      workflow header).
- [ ] `cargo xtask ci` passes; CI green on `main` after the change. (`cargo xtask ci` and branch
      CI pass; "green on `main`" is checked by the reviewer after the merge, WORKFLOW.md §8.)

## Test requirements

None in code. Verification is the CI runs linked in the journal.

## Handoff

2026-11-09, implementation sub-agent (journal `docs/journal/2026-11-09-M1-T006.md`).

**Done:**
- `pull_request` run verified on draft PR #1 (commit 2719f6c, old workflow): run 42686383503,
  attempt 2 green (attempt 1: macOS job never got a hosted runner; re-ran that job only). The
  same commit's `push` run 42686372259 ran in parallel: duplicate cost confirmed.
- Trigger set decided: `push` on every branch, no `pull_request`; `event_name` dropped from the
  concurrency group. Reason in the header of `.github/workflows/ci.yml` and the journal.
- Action updates: `.github/dependabot.yml`, `github-actions` ecosystem, monthly, one grouped pull
  request, prefix `ci(deps)`. Handling rule (merge only through a task) in the workflow header and
  `docs/STATUS_NOTES.md`.
- PR #1 closed without merging after the new-trigger check (see journal "Verified").

**Not done / not verified:**
- "CI green on `main` after the change": the reviewer checks it after merging.
- Dependabot behaviour: observable only after `dependabot.yml` is on `main`.
- Dependabot *security* alerts are a repository setting (owner), not configured here.

**Traps:** `cancel-in-progress: true` also cancels an older `main` run when two merges are pushed
quickly (journal "Problems"); not changed, outside scope.

## Review

_Empty._
