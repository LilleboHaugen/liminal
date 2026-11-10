---
id: ADR-0030
title: Short sessions, two review slots, merge on local CI, spec with code, early first pixels
status: accepted
date: 2026-11-10
supersedes: [ADR-0029]
superseded_by: []
---

# ADR-0030: Short sessions, two review slots, merge on local CI, spec with code, early first pixels

Supersedes ADR-0029. Every decision of ADR-0029 stays in force except Decision 1.5 ("at most 1
of them a review"), Decision 3.1 and 3.3 (spec with code is optional) and the review prompt.
Through ADR-0029, ADR-0021, ADR-0026 and ADR-0027 stay in force where not replaced; of ADR-0026,
Decision 6 (merge needs the branch CI run green) and the review prompt template of Decision 5
are replaced here.

## Context

The parent session of 2026-11-10 (`docs/journal/2026-11-10-session-m1-reviews.md`) ran 26
sub-agents in about three and a half hours and set 16 tasks to `done`. What it observed:

| # | Observation | Source |
|---|---|---|
| 1 | Up to eight tasks waited for the single review slot while implementation slots were empty. | journal, "Sub-agents" |
| 2 | Six of twelve implementation tasks only corrected specs of code that does not exist (M1-T055 … T060). | journal |
| 3 | Reviewers waited 1 to 11 minutes per branch for the GitHub run; twice a macOS runner stayed queued. | review journals of M1-T054, M1-T004 |
| 4 | Each reviewer ran `cargo xtask ci` three times, the last two on identical trees. | review journals |
| 5 | Every sub-agent read all journal entries of the newest date; there were more than 40. | `docs/journal/README.md` |
| 6 | Every merge conflicted in the generated `docs/STATUS.md`. | review journals |
| 7 | On 2026-11-09 two parent sessions ran at once: one switched the branch of the shared checkout, and two agents worked in one worktree. | `2026-11-09-session-m1-subagents.md` |
| 8 | A reviewer declined its review because its worktree's branch carried rules older than `main`. | journal, sub-agent 15 |
| 9 | A spec follow-up (M1-T055) ran beside the task whose requirements it changed (M1-T013); a requirement landed on a `done` task untested (M1-T059). | review journal of M1-T055 |
| 10 | The session lasted over three hours. The owner: "I don't want to sit in a session for an hour." | owner, 2026-11-10 |
| 11 | No pixel on screen after three days; the roadmap puts the first one at M3, after M1 and M2. | `docs/ROADMAP.md` |

The owner was shown a numbered list of 17 changes and answered on 2026-11-10: "Yes to all your
other proposes" (items 1 to 11 below), and for the six owner decisions:

| Question | Owner's answer |
|---|---|
| First pixels earlier | "Yes, as long as it doesn't hurt the project in any way" |
| Platforms | "All three platforms" |
| CI | "Keep it and never wait" |
| An existing GPU library behind Liminal's interface | "No" |
| Parallel graphics track | "If hurting project, no, if not, yes" |
| Nightly toolchain and Miri | "Yes" |

and added: "I need a way to tell sessions that they have a max time, I don't want to sit in a
session for an hour. Keep each session resonable, and close at a clean exit point".

## Decision

1. **Session time budget.**
   1. Every owner-started session has a time budget. The default is **45 minutes**. The owner
      MAY state another budget in the session ("30 minutes", "2 hours", "no limit"); that binds.
   2. The session records its start time (`date`) before any other work and writes it into its
      journal entry.
   3. **Cutoff** = budget minus 20 minutes. After the cutoff the session starts nothing new: a
      parent spawns no sub-agent, and a session without sub-agents does not start its
      implementation task. It checks the clock before every spawn or start.
   4. Running sub-agents are never stopped for the budget. They end on their own; then the
      session hands off (journal, `docs/STATUS_NOTES.md`, session branch merged). The budget is
      therefore a target: a sub-agent that started just before the cutoff can run past it.
   5. The final message to the owner states start time, end time and what is left.
   6. Kickoffs SHOULD size tasks so that one sub-agent finishes one in about 20 minutes. On
      2026-11-10 tasks took 4 to 31 minutes.
2. **Start-of-session check.** Before picking work, an owner-started session runs
   `git worktree list`, `git branch --no-merged origin/main --list 'session/*'` and a process
   listing for other Claude sessions. If there is a worktree besides the primary checkout, an
   unmerged session branch, or another live session, it reports what it found to the owner and
   does no task work until the owner says how to proceed. Sub-agents do not run this check.
3. **Two review slots.** At most 3 sub-agents run at once, of which at most **2** are review
   sub-agents. This replaces ADR-0029 Decision 1.5's "at most 1".
4. **Merge on local CI; one wait per session.** This replaces ADR-0026 Decision 6.
   1. A reviewer that sets `done` merges without waiting for a GitHub run. It runs
      `cargo xtask ci` once, on the final tree: the task branch with `origin/main` merged in and
      its review edits. If the tree of the merge commit on `main` is identical
      (`git rev-parse HEAD^{tree}`), it does not run it again.
   2. To merge it creates the merge commit on a detached `origin/main`
      (`git switch --detach origin/main`, `git merge --no-ff <task branch>`) and pushes it with
      `git push origin HEAD:main`. If the push is rejected because `main` moved, it fetches,
      repeats 4.1 and 4.2 on the new `origin/main`, and pushes again. Never force-push. Two
      reviewers can therefore work at once, and every commit on `main` has passed
      `cargo xtask ci` on its exact tree.
   3. GitHub CI still runs on every push, on all three operating systems (ADR-0005, ADR-0028
      Decision 3, rule 17: unchanged). Nobody waits for it during task work, except a task whose
      own acceptance criterion needs evidence from another platform.
   4. At the end of a session, before the handoff, the session reads the GitHub run for the
      last commit on `main` (`gh run list --branch main`). If it failed, the session writes a
      task for the failure (`status: todo`, current milestone, first in the pick order because
      every other task depends on a green `main`) and names it in `docs/STATUS_NOTES.md`. If it
      has not finished within 10 minutes, the handoff says so and the next session reads it in
      its start check.
5. **Spec with code is the rule.** This replaces ADR-0029 Decisions 3.1 and 3.3.
   1. A spec task ("Write the spec for …") that is `todo` is started only together with the
      first implementation task of the same crate, as one unit, when both are eligible
      (counting the spec task as `done` for the implementation task's `depends_on`). One
      implementation sub-agent, one branch, one review of both. Today this applies to
      M1-T015 with M1-T016, M1-T044 with M1-T045, and M1-T047 with M1-T048.
   2. Kickoffs write no spec-only task. A task covers a spec section, its code and its tests.
   3. Specs already written or in `review` stay as they are.
6. **Reading at session start.**
   1. An owner-started session reads `docs/STATUS.md`, `docs/STATUS_NOTES.md` and the newest
      journal entry whose file name contains `session`, plus entries that one names as needed.
   2. A sub-agent reads its task file, the specs and ADRs the task references, the journal
      entries whose file name contains its task ID, and `AGENTS.md` and the process document for
      its role. It does not read other journal entries unless one of those points to them.
7. **Review prompt.** A review sub-agent's branch can be older than the rules on `main`. The
   review prompt template is now (verbatim, placeholders filled, nothing added):

   > You are a review sub-agent under ADR-0026 and count as a separate session. Repository
   > worktree: `<worktree path>`. The rules in force are the ones on `origin/main`: read
   > `AGENTS.md` and `docs/process/REVIEW.md` from there (`git show origin/main:<file>`), not
   > from the worktree's branch. Review task `<task ID>` following them. Do one review only,
   > then stop.

   For a pair (Decision 5), both task IDs stand in place of `<task ID>`.
8. **Contract-changing follow-ups do not run beside their target.** A parent MUST NOT run a
   contract-changing follow-up task while a task whose requirements or acceptance criteria it
   changes is `in-progress` or in `review`. A follow-up assigns a new requirement to itself or
   to a `todo` task, never to a task that has started.
9. **`xtask` is frozen.** `xtask` and its spec change only through a task that names (a) a
   violation a check failed to catch, or (b) an engine task the tool blocks or slows. The three
   tasks in `review` (M1-T008, T009, T010) are reviewed as they are. The M2 kickoff judges
   M2-T001 … M2-T007 by this rule and deletes those that fail it, with the reason in its journal.
   One task passes (b) now: M2-T008, which removes the `docs/STATUS.md` merge conflict
   (observation 6).
10. **Process changes need a named failure.** An ADR that changes a rule in `AGENTS.md` or a
    process document MUST name, in its Context, the journal entry that records the failure it
    answers. Without one it is not written.
11. **Graphics track in parallel, first pixels first, all three platforms.**
    1. The M3 track (`docs/ROADMAP.md`, "Parallel tracks") is hereby recorded by the owner. It
       starts when `liminal_core` and `liminal_math` are both `usable`. Its first task is the
       M3 kickoff.
    2. M1 keeps priority: a slot goes to an eligible M1 task before an M3 task. M3 tasks use
       slots that M1 cannot fill, which on 2026-11-10 was one or two of three most of the time.
    3. The M3 kickoff orders its tasks so that the first deliverable is a window with a
       triangle, then a rotating cube driven by `liminal_math` and the fixed timestep, on the
       first backend. The first backend is Metal, because the owner's machine is a Mac and the
       owner can look at it.
    4. Nothing in M3 is cut. The GPU interface is designed for Vulkan and Metal from its first
       task (ADR-0006), the Vulkan backend follows inside M3, and the M3 exit criteria are
       unchanged: all three examples on macOS, Windows and Linux, golden images, validation
       layers clean.
    5. Liminal's GPU layer stays its own (ADR-0006). No existing GPU library is used, and no
       throwaway demo outside the engine rules is written.
    6. **"Does not hurt" check.** The owner's yes is conditional. The track is paused, and the
       owner told, if a review finds that an M3 task delayed an eligible M1 task, or that a
       design decision in the GPU interface was taken to suit Metal against ADR-0006. The M1
       exit review states whether either happened.
12. **Installs.** The owner approved the nightly toolchain and Miri for M1-T018 in principle.
    ADR-0022 item 2 still applies: the parent that runs M1-T018 states the exact command, the
    install location and the undo before it runs it, and waits for the owner's yes to that
    command.

## Alternatives considered

- **Do nothing.** Rejected by the owner (observations 1 to 11).
- **Drop GitHub CI and check locally only.** Removes the same waiting as Decision 4 and loses
  the only Windows and Linux test runs. The owner chose "Keep it and never wait".
- **macOS only.** The largest saving in M3 and M7. The owner chose "All three platforms".
- **An existing GPU library behind Liminal's interface for the first months.** The fastest way
  to a first scene. The owner said no; ADR-0006 stands.
- **A throwaway spike with an existing graphics library.** Pixels within one session, but it
  proves nothing about Liminal and uses a library the owner declined for the engine. Not done.
- **A hard time limit that stops running sub-agents.** Rejected: a stopped sub-agent leaves an
  `in-progress` task without a handoff note, which is the opposite of a clean exit.
- **More than 3 sub-agents at once.** Not asked for and not decided. With two review slots the
  limit of 3 leaves one to three implementation slots; revisit if slots are again idle while
  tasks wait.
- **Parent merges `main` into a task branch before the review** (instead of Decision 7).
  Rejected: every such merge conflicts in `docs/STATUS.md`, and resolving it is work on a task
  branch, which a parent does not do.

## Consequences

- **Sessions end.** A default session takes about 45 minutes, of which the last 20 start
  nothing. Throughput per session drops; the owner starts more sessions. Each starts with a
  short context, which also answers the ADR-0027 revisit condition (a summarised parent
  context).
- **A red `main` is possible for some minutes.** A change that passes on macOS and fails on
  Windows or Linux now reaches `main` before anyone sees the failure. Decision 4.4 bounds it to
  one session. Until M3 the code is platform-neutral Rust and every run that executed on
  2026-11-10 was green; from M3 on (GPU, windowing) the risk grows. Revisit then.
- **`cancel-in-progress` on `main`** (M2-T007 item 3) cancels a run when the next merge is
  pushed. With faster merges only the last run of a session completes. Decision 4.4 reads that
  one.
- **Two reviewers at once** double the `cargo xtask ci` load during reviews and can each lose a
  push race and repeat their merge.
- **Spec with code** makes each unit larger than a spec-only task. M1-T044 and M1-T047 now wait
  for their implementation partners; nothing else depends on them earlier.
- **Less reading at start** means a sub-agent can miss a trap recorded only in an unrelated
  journal entry. Traps that matter for later tasks belong in `docs/STATUS_NOTES.md` or in the
  task file.
- **The graphics track** splits attention across two milestones. Agent tooling (M2) arrives
  later relative to graphics than the roadmap's ordering rationale intended, so early M3 work is
  verified with golden images and tests, not through the probe. `capture.screenshot` waits for M2.
- **Revisit** if a session regularly overruns its budget by more than 15 minutes, if `main` is
  red at two session ends in one milestone, or under Decision 11.6.
- **Owner approval and review.** Every decision was approved by the project owner on 2026-11-10
  in the session that wrote this ADR (quotes under Context). As with ADR-0029 there was no
  separate ADR review by another session; the writing session was owner-started (ADR-0026
  Decision 5.4) and checked consistency itself against ADR-0001, ADR-0005, ADR-0006, ADR-0011,
  ADR-0021, ADR-0022, ADR-0026, ADR-0027, ADR-0028 and ADR-0029. That check is not independent.
  One reading is the session's own: "All three platforms" is taken as "no platform is dropped
  and the M3 exit criteria stay", not as "the three backends must be written at the same time".

## Verification

- REVIEW.md item 10 checks, from the journals: start time and budget in the session's journal
  entry; no spawn after the cutoff; at most two review sub-agents at once; the review prompt
  verbatim; the merge made as in Decision 4.2 with `cargo xtask ci` on the final tree.
- The session journal names the GitHub run read at the end (Decision 4.4) and its result.
- No machine check is added (Decision 9).

## Edits on acceptance

Applied in the same change, on 2026-11-10: ADR-0029 frontmatter and `docs/adr/README.md`;
`AGENTS.md` session protocol steps 1 and 6; `docs/process/WORKFLOW.md` §1, §2 and §8;
`docs/process/REVIEW.md` intro and items 2 and 10; `docs/journal/README.md`; `docs/ROADMAP.md`
(kickoff rule, parallel tracks); `docs/STATUS_NOTES.md`; new task M2-T008.
