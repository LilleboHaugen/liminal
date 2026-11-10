---
id: M1-T006
title: CI workflow follow-ups from the M0-T007 review
milestone: M1
status: done
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
- [x] `cargo xtask ci` passes; CI green on `main` after the change. (Review 2026-11-10:
      `cargo xtask ci` passes on the merge result with `main` `3f1b06f`, and the branch CI runs
      are green. The `main` run of the merge can only exist after this commit is merged and
      pushed; the reviewer waits for it and reports its ID to the parent session, whose journal
      `docs/journal/2026-11-10-session-m1-reviews.md` records it. If it is not green, the reviewer
      says so there and the task is reopened.)

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

Reviewer: review sub-agent (ADR-0026), 2026-11-10, parent
`docs/journal/2026-11-10-session-m1-reviews.md`; did not write M1-T006 (its author was sub-agent
no. 7 of `2026-11-09-session-m1-subagents.md`). Prompt: the ADR-0026 review template verbatim.
First review with an outcome (sub-agent no. 3 of `2026-11-09-session-m1-reviews.md` declined
without one). Outcome: **done**, no blocking findings. Follow-up task:
`docs/tasks/M2-T007-dependabot-and-main-run-follow-ups.md`.

Checked, macOS 27.2, Rust 1.99.1, on `efe662f` and on its merge with `main` `3f1b06f`
(`10d627d`):
1. Acceptance criteria (by reading the diff `d6bc2b6..efe662f` and with `gh`, read-only):
   - `pull_request` run: run 42686383503 is event `pull_request`, commit `2719f6c`, attempt 2,
     `success`, 5 jobs `success`. Met.
   - Trigger set: `on: push:` only, `event_name` gone from the concurrency group; reason in the
     workflow header and the journal. The claims behind the reason hold: `push` run 42686372259
     and `pull_request` run 42686383503 on the same commit `2719f6c` (duplicate); exactly one
     run (42689078830, `push`, `success`) for `1d99a37`, pushed while PR #1 was open (PR #1
     closed 20:07:25Z, run created 20:02:22Z). Met.
   - Action-update path: `.github/dependabot.yml` committed (`github-actions`, monthly, one
     group, `ci` prefix with scope). Keys read against the Dependabot options; parsed as YAML.
     Met. Its behaviour on GitHub is not verified (needs the file on `main`); follow-up item 2.
   - `cargo xtask ci`: pass, 11 steps, none skipped, on `10d627d`. Branch CI: run 42689928818
     on `efe662f`, `success` (attempt 2; attempt 1 lost the macOS job to runner capacity, as
     recorded in `2026-11-09-session-m1-reviews.md`). "Green on `main`": see the criterion.
2. Tests: the task has none in code ("Test requirements"). The evidence is the runs above.
3. No fake progress: no test, oracle, floor or baseline touched. Scope items 1 and 2 are both
   decided and implemented; the "Note (no work)" item is carried in the journal.
4. Spec: `docs/specs/xtask.md` does not describe workflow triggers; unchanged, no disagreement.
5. Unsafe, determinism: not applicable (no Rust code changed).
6. Docs: `docs/STATUS_NOTES.md` updated by the task. Stale against ADR-0028: follow-up item 1.
7. Architecture: no crate dependency. Dependabot is a GitHub service configured by a file, not
   a dependency in the sense of `docs/process/DEPENDENCIES.md`.
8. Session boundaries (item 10): the author is listed as sub-agent 7 in its parent's journal;
   this reviewer's parent is a different owner-started session; no install; no agent spawned.
9. Public repository (item 11, ADR-0028): the workflow keeps `permissions: contents: read`,
   every action pinned by full SHA, `persist-credentials: false` on every checkout, no secrets,
   no `pull_request_target` or `workflow_run`, hosted runners only. Dropping `pull_request` is
   the removal of a duplicate run, not a cut (Decision 3). No platform or job removed. Opening
   and closing draft PR #1 (author: the owner's account, the task's own branch) was asked for
   by this task's scope item 1 and happened before ADR-0028 was accepted (PR closed 21:07 CET,
   acceptance commit `bda846d` 21:49 CET); nothing outside was merged or acted on. No violation.

Merge with `main`: `.github/workflows/ci.yml` header conflict, both sides kept verbatim (the
task's two paragraphs, then the ADR-0028 paragraph); `docs/STATUS.md` regenerated. In
`docs/STATUS_NOTES.md` the reviewer replaced the task's clause "it is merged only through a
task" by the ADR-0028 Decision 5 handling, because `main`'s side of that file already states
rule 17 and the two sides contradicted each other. The workflow header was not reworded by the
reviewer (follow-up item 1).

Findings, all non-blocking (ADR-0021), in M2-T007:
1. The workflow header ("handle such a pull request in a task like any other change (do not
   merge it on GitHub without a reviewed task)") reads as allowing a merge of the bot branch
   through a task. ADR-0028 Decision 5 and rule 17 forbid that. Not B2: no rule was broken, the
   text predates the rule, and `AGENTS.md` wins over it. Reword.
2. Dependabot is unobserved: whether a pull request appears, whether its branch gets a `push`
   run, how the pull request ends after a manual re-pin; whether `commit-message` has a use.
3. `cancel-in-progress: true` on `main` cancelled `main` runs 42693473514, 42857415487 and
   42859718551 (annotation "Canceling since a higher priority waiting request for
   ci-ci-push-refs/heads/main exists"). The task's handoff names the trap; it is unchanged code
   and outside the task's scope.
Seen, no work: the journal says "all six jobs" once; there are five. `crates: [xtask]` in the
frontmatter although no `xtask` file changed.
