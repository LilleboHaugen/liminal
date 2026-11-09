---
id: ADR-0026
title: Sub-agents for task work under the same rules as owner-started sessions
status: superseded
date: 2026-11-09
supersedes: [ADR-0020]
superseded_by: [ADR-0027]
---

# ADR-0026: Sub-agents for task work under the same rules as owner-started sessions

Supersedes ADR-0020. While this ADR was `proposed`, its `supersedes` field stayed empty, because
`cargo xtask check-docs` (XTASK-R021) requires symmetric supersession links and would have marked
ADR-0020 superseded before this ADR was in force. The accepting session set both fields ("Edits on
acceptance", step 1).

## Context

ADR-0020 (accepted 2026-11-08) forbids a session to start other agents for task work. Every
review and every implementation needs a session the project owner starts. M1 has 54 tasks. One
session does at most one review and one implementation, so M1 needs at least 54 owner-started
sessions, run one after another unless the owner runs parallel sessions by hand.

On 2026-11-09 the project owner asked for a rule change: a session may spawn sub-agents for task
work, and every sub-agent must meet exactly the requirements a separate owner-started session
meets. "Nothing gets weaker: no shortcuts, no skipped gates, no self-review."

### Risks named by ADR-0020, and where this ADR answers them

ADR-0020's Context and Alternatives name these risks. Each one MUST stay answered by this
decision:

| # | Risk in ADR-0020 | Answered by |
|---|---|---|
| R1 | The owner did not start the sub-agent, and the session did not ask first. | Decision 1 (the owner gives standing permission in this ADR; each spawn is announced to the owner) |
| R2 | The owner could not see the sub-agent's work, only the summary it sends back. | Decisions 2, 8 (each sub-agent leaves its commits, task handoff and journal entry in the repository; the parent reports outcomes from the repository, not from summaries). Remaining cost: Consequences. |
| R3 | The author session chose the reviewer and wrote its brief, so the review was not independent of the author. | Decision 5 (exclusion set covers author, parent and siblings; fixed review prompt with nothing from the author; prompt recorded verbatim) |
| R4 | The pattern could repeat as a chain (review → merge → next task → another sub-agent) and move work forward without the owner choosing to start each session. | Decision 9 (per-session totals, not only a concurrency limit; no nested sub-agents; the parent stops after its wave, so every wave still needs an owner-started session) |
| R5 | ADR-0020 rejected "ask the owner each time" because it adds a confirmation step to every task. | Decision 1 (no confirmation step: standing permission, announcement only) |

ADR-0021 (review round limit) and ADR-0022 (owner approval for installs) also constrain every
session; Decisions 5 and 7 state how they apply to sub-agents.

## Decision

Terms: the **parent** is a session the project owner started that spawns sub-agents. A
**sub-agent** is an agent the parent spawns (Claude Code `Agent` tool or any equivalent in another
harness). A **wave** is all sub-agents one parent spawns.

1. **Standing permission.** A session the project owner started MAY spawn sub-agents for task work
   without being asked to, under every rule of this ADR. The owner gives this permission once, in
   this ADR (Context, Owner approval). The owner MAY forbid sub-agents or lower the limits of
   Decision 9 for a session by saying so in that session; that instruction binds the session. It
   never raises the limits. A session that spawns no sub-agents keeps ADR-0020's shape: one
   review, then one implementation. Before each spawn, the parent tells the owner the sub-agent's
   kind (review or implementation), task ID and branch. This is an announcement, not a request for
   confirmation.
2. **Equivalence.** A sub-agent counts as a separate session. Everything that binds a session the
   owner starts binds it: `AGENTS.md` in full, the session protocol (orient, pick, load context,
   test-first, hand off), the definition of done, `cargo xtask ci` passing with no failed or skipped
   step before `review`, the task handoff note, its own journal entry, and rule 16 (Decision 7).
   A sub-agent MUST NOT rely on anything the parent knows that is not in the repository or in its
   prompt (Decision 5, 6).
3. **Roles.**
   1. A parent orchestrates only. It MUST NOT review, implement, or change the status of a task,
      MUST NOT merge a task branch, and MUST NOT set anything to `done`. It picks tasks (Decision
      10), spawns and announces sub-agents, relays install requests (Decision 7), writes its own
      journal entry (Decision 8), and hands off to the owner.
   2. An **implementation sub-agent** does exactly one implementation task: continue an
      `in-progress` task with a handoff note, or start a `todo` task. It stops when it sets the task
      to `review`, or leaves it `in-progress` or `blocked` with a handoff note. It does no review.
   3. A **review sub-agent** does exactly one review (`docs/process/REVIEW.md`) of a task in
      `review`. If it sets the task to `done`, it merges the branch (Decision 6). It does no
      implementation, not even the fix for its own findings.
   4. Sub-agents MUST NOT spawn agents of any kind, including read-only helpers. The read-only
      helper exception of ADR-0020 item 1 applies to parents only.
4. **Isolation.**
   1. Each implementation sub-agent works on one task, on its own branch `task/<ID>-<slug>`
      (WORKFLOW.md §8), in its own git worktree. Worktrees live outside the repository directory
      (`../liminal-worktrees/<ID>` relative to the repository root), so that no Cargo workspace,
      `check-docs` or `hygiene` scan of one worktree sees another. A review sub-agent also works in
      its own worktree, `../liminal-worktrees/<ID>-review`. Git checks out a branch in one worktree
      at a time. Therefore, before its first spawn, the parent checks out its session branch
      (Decision 8.3) in the primary checkout, so that `main` is free for the merging reviewer.
      Before it spawns a review, the parent removes any worktree that still has the task branch
      checked out (Decision 4.4). If that worktree is not clean, the parent does not assign the
      review, and records why (Decision 10).
   2. Concurrent implementation sub-agents MUST have pairwise disjoint, non-empty `crates` lists
      in their task frontmatter. A task with `crates: []` (kickoff, exit review, docs-only) runs
      only when no other implementation sub-agent of the parent runs. The same crate rule applies
      against tasks that owner-started parallel sessions hold `in-progress` at spawn time.
   3. Shared files (`docs/STATUS_NOTES.md`, `docs/tasks/README.md`, `docs/adr/README.md`,
      `architecture.toml`, the workspace `Cargo.toml`, `Cargo.lock`, `ci/*`) are combined by rebasing
      or merging and keeping both sides' content. Generated files are regenerated, never edited by
      hand (`cargo xtask status` for `docs/STATUS.md`; `cargo` for `Cargo.lock`). Resolving a
      conflict by taking one side whole (`--ours`, `--theirs`, overwriting the file) is a B2 finding.
   4. A worktree is removed only after its branch is committed and pushed and `git status` in it is
      clean.
5. **Review independence.**
   1. A task is never reviewed by any agent of a session that did implementation work on it
      (original implementation or any fix round): not the author, not the author's parent, not any
      sub-agent of that parent. A parent and all its sub-agents count as **one session** for this
      rule and for every independence rule in `AGENTS.md`, `REVIEW.md`, ADR-0011 (maturity
      promotion "by a different session") and ADR-0021.
   2. A review sub-agent starts from a fresh context (never a fork of the parent). Its prompt is the
      review template below, verbatim, with only the placeholders filled. The parent adds nothing:
      no summary, conclusion, hint, list of things to check, or opinion about the task, its author
      or a previous review. The review sub-agent reads everything else from the repository, as an
      owner-started reviewer does.
   3. Only a reviewer sets `done`. ADR-0021 applies unchanged: B1–B5 block, everything else goes
      into one follow-up task, at most two "changes required" outcomes per task, then `blocked` for
      the owner.
   4. The ADR review (REVIEW.md §4) of an ADR that changes a rule in `AGENTS.md` or a process doc
      is done in an owner-started session, never by a sub-agent.

   Review prompt template:

   > You are a review sub-agent under ADR-0026 and count as a separate session. Repository
   > worktree: `<worktree path>`. Review task `<task ID>` following `AGENTS.md` and
   > `docs/process/REVIEW.md`. Read them first. Do one review only, then stop.

   Implementation prompt template (the parent MAY add, as quoted text, only facts the owner stated
   in the parent session that bind the task, e.g. an install approval under Decision 7):

   > You are an implementation sub-agent under ADR-0026 and count as a separate session.
   > Repository worktree: `<worktree path>`, branch `<branch>`. Work on task `<task ID>` following
   > `AGENTS.md` and `docs/process/WORKFLOW.md`. Read them first. Do this one task only, then stop.
6. **Merging.** Merges to `main` happen one at a time, each with `cargo xtask ci` green on the
   merge result and the branch CI run green (WORKFLOW.md §8). The review sub-agent that set the task
   to `done` does the merge. It merges from a worktree that has `main` checked out; it fixes shared
   files by Decision 4.3 and regenerates `docs/STATUS.md` before running `cargo xtask ci`. A parent
   runs at most one review sub-agent at a time, so its merges are serial by construction. A parent
   never merges a task branch, and never one whose review it spawned.
7. **Installs (ADR-0022).** A sub-agent cannot get the owner's yes first-hand, so it MUST NOT run any
   command listed in ADR-0022 item 1. If it needs a tool, the step stays not run and the task stays
   `in-progress` with the missing tool named in the handoff (ADR-0022 item 4). The parent MAY pass
   the request to the owner, with all of ADR-0022 item 2 (what, why, where and how big, alternatives,
   undo). Only on the owner's explicit yes in the parent session does the parent run the install
   itself. The parent's journal records the request and the owner's answer.
8. **Traceability.**
   1. The parent's journal entry (`docs/journal/YYYY-MM-DD-session-<topic>.md`, session type
      `parent`) quotes any instruction of the owner in that session that limits or forbids
      sub-agents (Decision 1), and lists every sub-agent it spawned: kind, task ID, branch, worktree, task status
      before and after, the sub-agent's journal entry file, the last commit on the branch, and the
      prompt (either "review template verbatim" / "implementation template verbatim", or the full
      text including any additions). Outcomes are read from the repository (task frontmatter, `git
      log`), not taken from the sub-agent's summary. The entry states facts only, no assessment of
      a sub-agent's work.
   2. Each sub-agent's own journal entry names its session type as `implementation (sub-agent,
      parent: <parent journal file>)` or `review (sub-agent, parent: <parent journal file>)`.
   3. The parent commits its journal entry, plus `docs/STATUS_NOTES.md` if it changed, on its own
      branch `session/YYYY-MM-DD-<topic>`. That branch touches no other files, so it contains no
      task work. The parent merges it to `main` with `cargo xtask ci` green, after all its
      sub-agents have ended, by the rules in Decision 6. This is the only merge a parent does.
   4. REVIEW.md item 10 checks Decisions 1, 3, 5 and 8 instead of "owner-started sessions only".
9. **Limits.**
   1. At most **3** sub-agents run concurrently per parent, of which at most **1** is a review
      sub-agent.
   2. In total, a parent spawns at most **3** implementation sub-agents and **3** review sub-agents.
      A sub-agent that ends counts against the total. It is not restarted or replaced in the same
      session. One owner action therefore moves at most 3 implementations and 3 reviews.
   3. Sub-agents MUST NOT spawn sub-agents (Decision 3.4).
   4. When its wave has ended, the parent writes its journal entry, merges its session branch, and
      hands off. It does not start a second wave.
10. **Task choice.** Sub-agents follow the pick rule and dependency rules of `AGENTS.md`: reviews
    first, then `in-progress` tasks with a handoff note, then `todo` tasks of the current milestone
    whose `depends_on` are all `done` at spawn time. The parent assigns tasks in that order and gives
    each task to exactly one sub-agent. It skips a task only if Decision 4.2 or 5.1 forbids it for
    that sub-agent, or Decision 4.1 (a review whose task branch is held by a worktree that is not clean), and it records the skip and the reason in its journal. A task that becomes
    `done` during the wave makes its dependents eligible for sub-agents spawned after that.
11. **Unchanged from ADR-0020.** Sessions are started by the project owner. Parallel tracks
    (`docs/ROADMAP.md`) are still parallel sessions the owner starts. A session that spawns no
    sub-agents keeps ADR-0020's shape. When a task written in the current session (by the
    parent's sub-agents included) is in `review` and the owner asks what is next, the answer is
    still: "Start a new session. It will review `<task>`, then continue with `<next task>`."

## Alternatives considered

- **Keep ADR-0020 (owner-started parallel sessions only).** This gives the strongest
  independence and full owner visibility, at one owner action per session. Rejected because the
  owner asked for higher throughput with the same gates. Parallel owner-started sessions remain
  allowed and remain the fallback if this ADR is revisited.
- **Sub-agents for implementation only, reviews in owner-started sessions.** This removes the review
  bias risk (R3) completely. Throughput then depends on reviews: a wave of 3 implementations needs
  3 owner-started review sessions, or 3 reviews spread over sessions that each do one review.
  Rejected as the default, because Decision 5 already keeps an author's session out of its reviews:
  a review sub-agent only ever reviews work of *another* session, with a fixed prompt. It is the
  fallback named under "Revisit".
- **Allow sub-agents, but ask the owner before each spawn.** ADR-0020 rejected this for its
  per-task confirmation cost. Decision 1 announces each spawn instead.
- **Allow sub-agents only when the owner's message in the session opts in.** This was proposed in
  an earlier draft of this ADR, to answer R1 per session. The owner rejected it on 2026-11-09: "I
  do not need to tell a session to spawn subagents. It can do so as long as every other rule is
  followed." The owner can still forbid or limit sub-agents in a session (Decision 1). The chain
  risk R4 stays bounded by Decision 9.
- **Let the parent merge after reviews.** This is simpler, because one agent serialises all merges.
  Rejected: the owner's rule forbids the parent to merge a branch whose review it influenced, and
  spawning the reviewer is influence. The reviewer already merges under WORKFLOW.md today.
- **Same-session review by a fresh-context sub-agent, if the parent did not write the task.**
  Rejected: a sibling sub-agent wrote it, and a parent and its sub-agents count as one session
  (Decision 5.1). This is the ADR-0020 incident under another name.
- **A higher concurrency limit (5–8).** Rejected for now. The first 10 M1 tasks
  (M1-T002 … M1-T011) all have `crates: [xtask]`, so only one of them can run at a time. The next
  wave of spec tasks (M1-T012, T024, T027, T030, plus T019 after M1-T054) touches 4–5 different
  crates, and after that the engine crates form a dependency chain. More than 3 concurrent agents
  would rarely find 3 eligible, crate-disjoint tasks, and every extra one adds a worktree build
  and conflicts on shared files.
- **No limit on the total, only on concurrency.** Rejected: this is risk R4 (chains).
- **Do nothing.** Same as the first alternative.

## Consequences

- **Throughput.** One owner action can move up to 3 implementations and 3 reviews instead of 1 + 1.
  Real gains are smaller while eligible tasks share a crate (the first ten M1 tasks are all
  `xtask`).
- **Token cost.** Each sub-agent orients from scratch: AGENTS.md, STATUS, journals, task, specs
  and code. A wave costs roughly the sum of that many separate sessions plus the parent's
  orchestration, and none of it is shared. That is the price of equivalence. A wave whose
  implementations fail review adds review and fix cost in later sessions.
- **Machine load.** Each worktree has its own `target/` (1.1 GB for `xtask` alone today; it
  grows with every engine crate). Three concurrent `cargo xtask ci` runs, including the coverage
  build, compete for 18 cores and 24 GB RAM on the owner's Mac. Timing-sensitive tests can become
  flaky under load. Assumption, unverified: no `xtask` test depends on a fixed path or port shared
  between worktrees. Nobody has run `cargo xtask ci` in two worktrees at once yet.
- **Merge conflicts.** Every task edits `docs/STATUS.md` (generated), usually
  `docs/STATUS_NOTES.md` and `docs/tasks/README.md`, and new crates edit `architecture.toml`,
  `Cargo.toml` and `Cargo.lock`. With up to 3 branches per wave, the later merges need rebasing
  and regeneration. Decision 4.3 makes this explicit. It still takes time, and a careless
  resolution can lose content. The reviewer that merges is accountable for it, and the next
  reviewer checks it.
- **Summary loss.** The parent sees only what a sub-agent returns. The owner sees the parent, not
  the sub-agents, live. The repository record makes up for this only partly: commits, task
  handoff, a journal per sub-agent, and the parent's journal written from repository state.
  Reasoning a sub-agent did not write down is lost. With an owner-started session, the owner could
  have read or interrupted it.
- **Review bias.** Review sub-agents run on the same model and harness as the authors, so they
  may share blind spots. That was already true for owner-started sessions. The parent still
  chooses *which* task a reviewer gets and *when*. The fixed prompt keeps the parent's opinion out,
  and Decision 5.1 keeps the author's session out completely. Bias that comes from the same
  model is not removed. Only the owner's review of samples would show it.
- **Owner visibility and control.** Sessions spawn sub-agents by default. The owner sees each
  announcement and can forbid or limit sub-agents in a session, but does not choose each spawn.
  The owner also cannot approve an install a sub-agent needs while it runs (Decision 7). Such tasks stall at
  `in-progress` exactly as they would in an unattended owner-started session.
- **Rule complexity.** Reviewers check more under item 10. A violation is visible in journals, but
  no machine check enforces it yet.
- **Revisit** if any review finds a Decision 5 violation, if more than one merge in a wave needs a
  conflict fix that a later review flags, if waves regularly end with fewer than 2 eligible tasks,
  or if defects found later trace back to reviews by review sub-agents. The first fallback is
  "sub-agents for implementation only".
- **Owner approval.** Requested by the project owner on 2026-11-09 (quoted under Context). This
  ADR changes rules in `AGENTS.md` (session protocol step 6, definition of done, "Changing the
  rules"). On the same day the owner removed a per-session opt-in from the draft: "I do not need
  to tell a session to spawn subagents. It can do so as long as every other rule is followed, as
  if only done by one agent. The goal here is to speed up things as I notice things are moving
  incredibly slow with just one agent running at a time." Approved by the project owner on
  2026-11-09, conditional on the ADR review (REVIEW.md §4)
  finding it consistent with the other accepted ADRs.
- **ADR review.** Done 2026-11-09 in an owner-started session
  (`docs/journal/2026-11-09-adr-0026-review.md`). It found the ADR consistent with ADR-0001, ADR-0011,
  ADR-0020 (superseded explicitly), ADR-0021 and ADR-0022. Before acceptance, the review added these
  clarifications: ADR-0011 is named in 5.1; the review sub-agent's worktree, and keeping `main` free
  for the merge, in 4.1 and 10; and the placement of edit 6. Accepted on the same day.

## Verification

- REVIEW.md item 10 (new wording below) checks, from the journals and task file: the
  implementing session (owner-started, or a sub-agent listed in a parent journal, within any limit
  the owner set for that session); that no agent of the implementing session reviewed the task; that the review
  sub-agent's prompt was the template verbatim; and that merges were serial with CI green. A
  violation is a B2 finding (ADR-0021).
- A missing parent journal, a sub-agent not listed in it, or a nested spawn is a B2 finding on every
  task that sub-agent touched.
- A machine check is possible but not part of this ADR: `check-docs` could check that each journal
  entry with session type `… (sub-agent, parent: X)` names an existing parent journal X that lists
  it. Candidate follow-up task if item-10 reviews prove error-prone.

## Edits on acceptance

Applied in the acceptance commit on 2026-11-09. These edits took effect only on acceptance. They
were not made while the ADR was `proposed`.
`AGENTS.md` can only change by an accepted ADR ("Changing the rules"). A conditional sentence there
would itself be a rule change without one, and it would contradict step 6 as it stands. The
accepting session applies them in its acceptance commit, verbatim apart from fixes the ADR review
asks for:

1. **Frontmatter.** This ADR: `status: accepted`, `supersedes: [ADR-0020]`. ADR-0020:
   `status: superseded`, `superseded_by: [ADR-0026]` (the only edits allowed to an accepted ADR,
   ADR-0001).
2. **`docs/adr/README.md`.** Row 0020: status `superseded by ADR-0026`. Row 0026: status
   `accepted`.
3. **`AGENTS.md`, session protocol step 6**, replaced by:

   > 6. **Stop at the session boundary (ADR-0026).** A session does at most one review, then one
   >    implementation task. When you set your task to `review`, stop task work. Never review
   >    your own work. Only the project owner starts sessions. A session MAY spawn sub-agents for
   >    task work unless the owner forbids it in that session. It then follows ADR-0026:
   >    every sub-agent counts as a separate session and meets every rule here. The parent only
   >    orchestrates (no review, implementation, status change, task merge or `done`). Each
   >    implementation sub-agent gets one task on its own branch and worktree, and concurrent ones
   >    share no crate. A task is never reviewed by any agent of the session that wrote it. Review
   >    sub-agents start fresh with the fixed prompt only. At most 3 sub-agents run at once (1
   >    review), with at most 3 + 3 per session, and sub-agents never spawn agents. When the owner
   >    asks what is next or for the status, and a task written in this session is in `review`,
   >    answer: "Start a new session. It will review `<task>`, then continue with `<next task>`."
   >    Do not start that work yourself.

4. **`AGENTS.md`, "Definition of done"**, last bullet replaced by:

   > - The task is set to `review`, not `done`. A reviewer from another session reviews it with
   >   `docs/process/REVIEW.md` and sets it to `done`, possibly with a follow-up task for
   >   non-blocking findings (ADR-0021). The reviewer is a session the project owner started, or
   >   a review sub-agent of such a session under ADR-0026.

5. **`AGENTS.md`, "Changing the rules"**, replaced by:

   > These rules are changed only by an accepted ADR that names the rule and the reason. Edit this
   > file in the same change as the ADR. An ADR that changes these rules is reviewed and accepted in
   > a session the project owner started, never by a sub-agent (ADR-0026).

6. **`docs/process/WORKFLOW.md` §2**: the paragraph starting "**Session boundary (ADR-0020).**"
   is replaced by the text below. The ADR-0021 paragraph after it ("A review blocks a task only …")
   stays where it is. The paragraph starting "Never start sub-agents" is deleted.

   > **Session boundary (ADR-0026).** Only the project owner starts sessions; parallel agents are
   > parallel sessions the owner started, or sub-agents of one (below). One session does at
   > most:
   > 1. one review (priority 1 above). If it sets the task to `done`, merge the branch to `main` and
   >    push (§8). Then:
   > 2. one implementation task (priority 2 or 3).
   >
   > **Sub-agents (ADR-0026).** Instead, a session may act as a *parent* and spawn sub-agents for
   > task work, unless the owner forbids it in that session. A limit the owner states binds. Each sub-agent counts as a
   > separate session and follows this whole document. The parent:
   > - does no review, implementation or status change itself, merges no task branch, and sets
   >   nothing to `done`;
   > - announces each spawn to the owner (kind, task, branch);
   > - assigns tasks by the priority order above, each task to one sub-agent. Concurrent
   >   implementation sub-agents have disjoint, non-empty `crates`. A task with `crates: []` runs
   >   alone;
   > - never gives a review to a sub-agent if any agent of this session worked on that task;
   > - starts review sub-agents with a fresh context and the review prompt template of ADR-0026,
   >   verbatim, adding nothing;
   > - runs at most 3 sub-agents at once, at most 1 of them a review, and at most 3 implementation
   >   and 3 review sub-agents in total;
   > - relays install requests to the owner and runs an approved install itself (ADR-0022). Sub-agents
   >   never install;
   > - ends with its own journal entry listing every sub-agent (ADR-0026 Decision 8), committed on
   >   `session/YYYY-MM-DD-<topic>` and merged with CI green.
   >
   > Sub-agents never spawn agents. Read-only lookups that change no files and decide nothing may
   > use helpers in a parent session or a session without sub-agents. Never review a task that any
   > agent of your session worked on.

7. **`docs/process/WORKFLOW.md` §7 step 5**, last sentence replaced by:

   > Do not start that work yourself, in this session or through a sub-agent: a sub-agent of this
   > session may not review a task this session wrote (ADR-0026).

8. **`docs/process/WORKFLOW.md` §8**, append:

   > - Sub-agents (ADR-0026): each implementation sub-agent works in its own worktree under
   >   `../liminal-worktrees/<ID>`, on its task branch. Shared files are combined by rebase or
   >   merge, keeping both sides. Generated files (`docs/STATUS.md`, `Cargo.lock`) are regenerated,
   >   never resolved by taking one side. Merges to `main` are one at a time, by the reviewer that
   >   set `done`, with `cargo xtask ci` green on the merge result and the branch CI run green. A
   >   parent's own branch `session/YYYY-MM-DD-<topic>` holds only its journal entry and
   >   `docs/STATUS_NOTES.md`. A worktree is removed only when its branch is pushed and clean.
   >   A review sub-agent works in its own worktree `../liminal-worktrees/<ID>-review`. The parent
   >   keeps its own session branch checked out in the primary checkout, so `main` is free for the
   >   merge.

9. **`docs/process/REVIEW.md`**: the intro's second sentence replaced by:

   > That reviewer is a session the project owner started, or a review sub-agent spawned by such
   > a session with a fresh context and only the ADR-0026 review prompt. It is never the author, the
   > author's parent session, or any sub-agent of that session (ADR-0026).

   Item 10 replaced by:

   > 10. **Session boundaries (ADR-0026):** from the journals and the task file: each session that
   >     worked on the task was owner-started, or a sub-agent listed in its parent's journal (within
   >     any limit the owner set for that session); no agent of a session that implemented any part of the
   >     task reviewed it; a review sub-agent's prompt was the ADR-0026 template verbatim; no
   >     sub-agent spawned agents or ran an install; merges to `main` were serial with CI green.

10. **`docs/journal/README.md`**: the session type line becomes
    `implementation | review | kickoff | planning | parent`, and below the format add:

    > A sub-agent writes `implementation (sub-agent, parent: <parent journal file>)` or
    > `review (sub-agent, parent: <parent journal file>)`. A parent entry lists every sub-agent it
    > spawned (ADR-0026 Decision 8).

11. **Unchanged:** `docs/ROADMAP.md` (parallel tracks remain owner-started sessions),
    `docs/tasks/README.md` ("set to `done` only by a reviewing session" holds), ADR-0021 and ADR-0022
    (accepted, immutable; Decisions 5.3 and 7 apply them to sub-agents).
