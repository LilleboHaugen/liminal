---
id: ADR-0029
title: Faster loop — same-session review by sub-agents, backlog for minor findings, spec with code
status: accepted
date: 2026-11-10
supersedes: [ADR-0021, ADR-0027]
superseded_by: []
---

# ADR-0029: Faster loop — same-session review by sub-agents, backlog for minor findings, spec with code

Supersedes ADR-0021 and ADR-0027. Every decision of ADR-0021 stays in force except its Decision 2
(replaced by Decision 2 here). Every decision of ADR-0027 stays in force except its Decision 2.3
(replaced by Decision 1.4 here). ADR-0027 keeps ADR-0026 in force; of ADR-0026, Decision 5.1 is
replaced by Decision 1 here and Decision 3.2 is extended by Decision 3 here. The texts of
ADR-0021, ADR-0026 and ADR-0027 are part of this decision where not replaced.

## Context

After two days (2026-11-08 to 2026-11-10) the project owner asked whether the working method is
worth its cost. Facts on 2026-11-10, from `docs/STATUS.md`, the task files and the journals:

| Fact | Value |
|---|---|
| Engine code on `main` | none; `crates/liminal_core` (M1-T013) is in `review` on its branch |
| M1 tasks `done` | 11 of 58 |
| Reviews in the parent session of 2026-11-10 | 7, each set its task to `done` |
| Follow-up tasks those 7 reviews created | 7 (M1-T055 … M1-T058, M2-T004 … M2-T006) |
| M1 task count during that session | 54 → 58 |
| Tasks that session wrote and may not review | 7 (M1-T013, T019, T034, T055 … T058) |

Three causes:

1. **Review independence is per session (ADR-0026 Decision 5.1).** A parent and all its
   sub-agents count as one session, so a task written in a session becomes `done` only in a
   later session. A dependency chain (M1-T013 → M1-T014 → …, about 15 links to the M1 exit)
   moves one link per owner-started session.
2. **Every review creates a task (ADR-0021 Decision 2).** All follow-ups of a review go into one
   new task. Each such task then needs an implementation, a review, a journal entry and a merge,
   and its review creates the next follow-up task. Most items are small (wording, an untested
   branch with no effect on outcomes).
3. **Specs are written and reviewed before code exists.** The reviews of M1-T030 and M1-T024
   found a normative example that does not compile and a diagram that contradicts its own rule.
   Code written with the spec would have shown both at once.

The owner's decision (2026-11-10, in the parent session `2026-11-10-session-m1-reviews.md`), after
that session listed these three changes: "You are hereby allowed by me to change the ADR with
those points directly, and merge to main".

## Decision

1. **A fresh review sub-agent may review a sibling's task.** This replaces ADR-0026 Decision 5.1.
   1. A task is never reviewed by an agent that did implementation work on it (original
      implementation or any fix round), and never by a parent. A session without sub-agents
      never reviews a task it worked on.
   2. A review sub-agent MAY review a task that an implementation sub-agent of the same parent
      wrote. It starts from a fresh context (never a fork), with the ADR-0026 review prompt
      template verbatim and nothing added (ADR-0026 Decision 5.2, unchanged). The parent has
      done no implementation work, so it has no opinion of its own to pass on, and it passes on
      none of the author's.
   3. For every independence rule in `AGENTS.md`, `REVIEW.md`, ADR-0011 (maturity promotion "by a
      different session") and ADR-0021, each sub-agent counts as its own session. A parent
      counts as the same session as each of its sub-agents.
   4. Per parent session, each task gets at most one implementation sub-agent and at most one
      review sub-agent, in that order if both. A sub-agent that ends is not restarted or replaced
      for the same role on the same task. A task a review sets back to `in-progress`, or a
      sub-agent leaves `in-progress` or `blocked`, waits for a later session. This replaces
      ADR-0027 Decision 2.3.
   5. Unchanged: at most 3 sub-agents at once, at most 1 of them a review; the reviewer that sets
      `done` merges; the parent only orchestrates; ADR-0026 Decision 5.4 (an ADR that changes a
      rule is reviewed in an owner-started session, never by a sub-agent).
2. **Minor findings go to the backlog, not into a task.** This replaces ADR-0021 Decision 2.
   1. Follow-ups never block `done`. With no blocking finding, the reviewer sets the task to
      `done` in the same review.
   2. The reviewer appends each follow-up to `docs/BACKLOG.md`, under a heading that names the
      reviewed task and the date. An entry states the finding and the file it concerns in one to
      three lines. The review note says how many entries were added.
   3. A follow-up becomes a task only if it is **contract-changing**: it changes a spec
      requirement, a public API, an error code or an acceptance criterion that a task in `todo`
      or `in-progress` implements, or a milestone exit criterion needs it. All contract-changing
      follow-ups of one review go into one new task (`status: todo`, next free ID, current
      milestone), named in the review note. Such a task SHOULD be listed in `depends_on` of the
      tasks whose contract it changes, so that they do not start against the old contract.
   4. The backlog is worked at milestone kickoff and exit reviews: each entry is turned into a
      task, or deleted with a one-line reason in that session's journal entry. Between those, an
      implementation task MAY fix backlog entries in files it changes anyway; it deletes the
      entry and says so in its handoff.
   5. Follow-up tasks that exist on 2026-11-10 (M1-T055 … M1-T058, M2-T001 … M2-T006) stay as
      they are.
3. **Spec and first implementation may be one unit of work.**
   1. When a spec task ("Write the spec for …") and the first implementation task of the same
      crate are both eligible, counting the spec task as `done` for the implementation task's
      `depends_on`, the parent MAY give both to one implementation sub-agent, on one branch
      named after the spec task. The sub-agent writes the spec with the code, sets both tasks to
      `review`, and writes one journal entry. One review sub-agent reviews both (task review and
      spec review, REVIEW.md §1 and §2) and sets each to `done` or not.
   2. Prompt template for the pair, an addition to ADR-0026 Decision 3.2 (same rule for added
      text as the implementation template):

      > You are an implementation sub-agent under ADR-0026 and count as a separate session.
      > Repository worktree: `<worktree path>`, branch `<branch>`. Work on tasks `<spec task ID>`
      > and `<implementation task ID>` as one unit (ADR-0029 Decision 3) following `AGENTS.md`
      > and `docs/process/WORKFLOW.md`. Read them first. Do these two tasks only, then stop.

      The review prompt for the pair is the review template with both task IDs in place of
      `<task ID>`.
   3. A spec task alone stays valid, for example when another spec depends on it before the
      implementation task is eligible.
   4. From the M2 kickoff on, a kickoff SHOULD plan one task per crate slice that covers the
      spec section, the code and the tests, instead of separate spec tasks.
4. **Not decided here.** The owner was also offered "move the remaining `xtask` follow-ups
   (M1-T008 … M1-T010) to M2". They are already implemented and in `review`; only their reviews
   remain, so they stay in M1.

## Alternatives considered

- **Do nothing.** Rejected by the owner: about 15 to 20 more owner-started sessions for M1 at one
  chain link per session (estimate from the dependency chain, not measured).
- **Drop the method: one agent writes the engine directly, no specs, tasks or reviews.** Faster
  for the first weeks. Rejected: nothing would then check an agent's claims except the agent
  itself, which is the failure `AGENTS.md` ("The one idea behind every rule") exists to prevent.
  CI gates, traceability, oracles and review by another agent stay.
- **Let an agent review its own task.** Rejected: ADR-0020's incident. Decision 1.1 keeps it out.
- **Same-session review only for docs and spec tasks.** Keeps full independence for code.
  Rejected: the chain that limits throughput is code (M1-T013 → M1-T014 → …).
- **Keep one follow-up task per review, but in a later milestone.** This is what ADR-0021
  allowed; the task count still grows by one per review, and M2 already has six such tasks.
- **Drop minor findings.** Rejected: some are real gaps (an untested rule, a spec sentence that
  misleads the next implementer). The backlog keeps them at the cost of one file entry.

## Consequences

- **Throughput.** One owner-started session can now move a chain several links: implement,
  review, merge, implement the dependent. The review slot (1 at a time, serial merges) becomes
  the limit.
- **Risk R3 of ADR-0020 is accepted in part.** The parent chooses which task a reviewer gets
  and when, and now also spawned the author. What remains against bias: the reviewer's fresh
  context, the fixed prompt, the parent's ban on implementation work, and the repository record.
  Reviewer and author still share model and harness, as before. The owner chose this trade.
- **Risk R4 (chains without the owner) grows.** A session can now carry work from `todo` to
  `done` to the next task with no owner action. Bounds: one implementation and one review
  sub-agent per task per session, the announcement of every spawn, and the owner's right to stop
  the session.
- **Backlog.** `docs/BACKLOG.md` is a shared file that every review appends to; merges combine
  it keeping both sides (ADR-0026 Decision 4.3). Entries have no acceptance criteria and no
  owner until a kickoff or exit review. An unread backlog is lost knowledge: the exit review of
  each milestone MUST empty it or carry entries forward on purpose.
- **Judgement.** "Contract-changing" is a judgement by the reviewer, like B3. A wrong "minor"
  leaves a spec defect for the implementer to find; a wrong "contract-changing" costs one task.
- **Spec with code.** A paired review is larger. A spec defect found in it costs code rework in
  the same branch instead of a spec-only fix.
- **Revisit** if a defect found later traces to a same-session review, if the backlog holds more
  than 50 entries at a milestone exit, or if paired reviews regularly end in "changes required".
  The first fallback is ADR-0026 Decision 5.1 for code tasks only.
- **Owner approval and review.** Requested and approved by the project owner on 2026-11-10
  (quote under Context). This ADR changes rules in `AGENTS.md` (session protocol steps 2 and 6,
  definition of done). The owner told the session that wrote it to apply it directly and merge,
  so there was no separate ADR review (REVIEW.md §4) by another session. The writing session
  was owner-started (ADR-0026 Decision 5.4). It checked consistency itself against ADR-0001,
  ADR-0011, ADR-0020, ADR-0021, ADR-0022, ADR-0026, ADR-0027 and ADR-0028; that check is not
  independent.

## Verification

- REVIEW.md item 10 (new wording, applied with this ADR) checks from the journals and the task
  file: no agent that implemented any part of the task reviewed it; the reviewer was an
  owner-started session that did not work on the task, or a review sub-agent with the template
  verbatim; a parent did no implementation; at most one implementation and one review sub-agent
  per task per parent session. A violation is a B2 finding.
- A review note names the number of backlog entries and, if any, the follow-up task and why its
  items are contract-changing.
- `cargo xtask check-docs` checks links and task references in `docs/BACKLOG.md` like in any
  other document. No machine check of the backlog's content is part of this ADR.

## Edits on acceptance

Applied in the same change as this ADR, on 2026-11-10:

1. Frontmatter: ADR-0021 and ADR-0027 `status: superseded`, `superseded_by: [ADR-0029]`;
   `docs/adr/README.md` rows 0021, 0027, 0029.
2. `AGENTS.md`: session protocol step 2.1 (backlog), step 6 (review by a sibling's fresh review
   sub-agent, one implementation and one review sub-agent per task, the "what is next" answer),
   definition of done (reviewer wording), "Where things live" (`docs/BACKLOG.md`).
3. `docs/process/WORKFLOW.md` §2, §7 step 5 and §8; `docs/process/REVIEW.md` intro, item 10 and
   "Findings and outcome".
4. New file `docs/BACKLOG.md`.
