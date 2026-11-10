---
id: M2-T007
title: Dependabot handling and `main` run cancellation follow-ups from the M1-T006 review
milestone: M2
status: todo
crates: []
depends_on: [M1-T006]
requirements: []
---

# M2-T007: Dependabot handling and `main` run cancellation follow-ups from the M1-T006 review

## Goal

The text that tells an agent what to do with a Dependabot pull request agrees with ADR-0028
Decision 5 and `AGENTS.md` rule 17, what Dependabot does in this repository is recorded from
observation, and a merge to `main` no longer loses the CI result of the merge before it.

## Context

- Follow-up task under ADR-0021, written by the review of M1-T006 (see that task's "Review").
- `.github/workflows/ci.yml` (header comment, `concurrency`), `.github/dependabot.yml`,
  `docs/STATUS_NOTES.md` "Module notes", ADR-0028 (Decisions 3, 5, 6), `docs/process/WORKFLOW.md` §8.
- M1-T006 was written before ADR-0028 was accepted (task commits 2026-11-09 20:45–21:07 CET,
  acceptance 21:49 CET), so its wording predates rule 17.
- No M1 exit criterion needs this, so it is in M2 (ADR-0021). Item 3 affects every merge to
  `main`; an earlier session MAY pick the task up if the owner moves it.

## Scope

**In:**
1. **Workflow header wording.** The header of `.github/workflows/ci.yml` says about a Dependabot
   pull request: "Handle such a pull request in a task like any other change (do not merge it on
   GitHub without a reviewed task)". Read literally, it allows merging the bot's branch through a
   reviewed task, and it lets an agent start on such a pull request by itself. ADR-0028 Decision 5
   and rule 17 say: an agent never merges, checks out or cherry-picks a bot's branch; it acts on
   the pull request only when the owner points to it; in a task it makes the change itself and
   resolves each new SHA from the upstream tag with `git ls-remote`. Reword the header to say
   that. Check the comment in `.github/dependabot.yml` ("one task and one CI run cover them")
   against the same rule. `docs/STATUS_NOTES.md` was already reworded by the M1-T006 review.
2. **Dependabot, observed.** M1-T006 could not observe Dependabot (`dependabot.yml` was not on
   `main`). Record from read-only observation (`gh pr list --author app/dependabot --state all`,
   `gh run list`), without acting on any pull request unless the owner points to it:
   - whether a grouped pull request was opened, and for which actions;
   - whether its branch got a `push` run of `ci.yml` and with what result (the header claims
     "its branch gets a normal `push` run"; unverified);
   - what happens to an open Dependabot pull request once the pins on `main` were updated by
     hand (assumption: Dependabot closes it itself; unverified). Closing it by hand needs the
     owner's yes (ADR-0028 Decision 6).
   Then decide whether `commit-message` in `dependabot.yml` stays: the bot's commits are never
   merged, so the prefix only affects the pull-request title.
3. **`cancel-in-progress` on `main`.** The concurrency group is per ref with
   `cancel-in-progress: true`, so a push to `main` cancels the running `main` run of the
   previous push. Observed: `main` runs 42693473514 (`2c8f334`), 42857415487 (`bb94e3a`) and
   42859718551 (`bbd28ee`) are `cancelled`. Each cancelled run belonged to a merge whose
   reviewer needs "CI green on `main`" (WORKFLOW.md §8, task acceptance criteria); that result
   then exists only indirectly, through a later commit's run. Decide: cancel only on refs other
   than `main` (for example `cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}`), or
   keep it and state in WORKFLOW.md §8 that a later green `main` run that contains the merge
   counts. Record the decision and the reason in the workflow header and the journal. Not
   cancelling is not a cost concern (ADR-0028 Decision 3: standard runners are free).

**Out:** a `pull_request` trigger for fork pull requests (ADR-0028 Consequences, OQ-011); Cargo
dependency updates through Dependabot (need an ADR, `docs/process/DEPENDENCIES.md`); Dependabot
security alerts and other repository settings (owner, ADR-0028 Decision 6); a machine check for
workflow rules (ADR-0028 "Alternatives considered").

## Acceptance criteria

- [ ] The header of `.github/workflows/ci.yml` and the comment in `.github/dependabot.yml` state
      the handling of a Dependabot pull request as ADR-0028 Decision 5 does; no sentence in the
      repository outside journals and task files says such a pull request is merged.
- [ ] The three observations of item 2 are in the journal with pull-request numbers and run IDs,
      or "none opened as of <date>"; the `commit-message` decision is recorded.
- [ ] Item 3 is decided and implemented; if `cancel-in-progress` changes, two pushes to one
      non-`main` branch in quick succession still cancel the older run (run IDs in the journal),
      and the `main` behaviour is observed on the task's own merge or stated as not verified.
- [ ] ADR-0028 Decision 4 still holds for every workflow (REVIEW.md item 11).
- [ ] `cargo xtask ci` passes; branch CI run green.

## Test requirements

None in code. Verification is the CI runs and `gh` observations linked in the journal.

## Notes

Written by the review of M1-T006 (2026-11-10). Also seen in that review, no work: the M1-T006
journal says "all six jobs" once; the workflow has five (three `checks`, `coverage`, `deny`).
Journal entries are append-only.

## Handoff

_Empty._

## Review

_Empty._
