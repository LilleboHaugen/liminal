---
id: ADR-0027
title: Sub-agents by default, no per-session total
status: proposed
date: 2026-11-09
supersedes: []
superseded_by: []
---

# ADR-0027: Sub-agents by default, no per-session total

Supersedes ADR-0026 on acceptance. Every decision of ADR-0026 stays in force, except Decision 1
(the last two sentences are replaced by Decision 1 here) and Decisions 9.2 and 9.4 (replaced by
Decision 2 here). The ADR review must treat ADR-0026's text as part of this decision.
`supersedes` stays empty while this ADR is `proposed`, for the same reason given in ADR-0026
(XTASK-R021 symmetric links). The accepting session sets both fields.

## Context

ADR-0026 (accepted 2026-11-09) lets a session spawn sub-agents, but only *permits* it ("MAY"). It
also caps a session at 3 implementation and 3 review sub-agents in total. After that the parent
stops and the owner must start a new session. ADR-0026 adopted the total cap to answer risk R4
(chains that move work forward without the owner starting each session).

On 2026-11-09, the same day, the project owner asked for a change: "I want sessions to prefer to
spawn sub-agent where they can, and spawn enough to work as efficiently as possible, while
following the rules of course. But right now things move too slow." Asked whether to change the
limits, the owner chose to remove the per-session total and keep the concurrency limit.

Fact that bounds the gain: ADR-0026 Decision 5.1 (unchanged) counts a parent and all its
sub-agents as one session. A task implemented in a session is never reviewed in that session, so
it becomes `done` only in a later session. Within one session, dependents become eligible only
through reviews of tasks that *other* sessions wrote. The gain is that a session keeps all
3 slots busy until no eligible task is left, instead of stopping after 3 + 3.

## Decision

1. **Prefer sub-agents.** A session the project owner started SHOULD act as a parent (ADR-0026)
   whenever two or more tasks are eligible for sub-agents at the same time (ADR-0026 Decisions 4.2,
   5.1, 10), counting reviews and implementations. With at most one eligible task, it MAY do that
   task itself in the shape of ADR-0020 (one review, then one implementation). The owner MAY forbid
   sub-agents or lower the concurrency limit in a session by saying so in that session. That
   instruction binds the session and never raises the limit. Each spawn is still announced to the
   owner (kind, task ID, branch). The announcement is not a request for confirmation.
2. **Keep the slots full; no total.**
   1. The concurrency limit of ADR-0026 Decision 9.1 is unchanged: at most **3** sub-agents run at
      once per parent, at most **1** of them a review sub-agent.
   2. There is no per-session total. Whenever a slot is free and an eligible task exists, the parent
      SHOULD spawn a sub-agent for it at once, in ADR-0026's pick order (reviews first).
   3. Each task is given to at most one sub-agent per session, and a sub-agent that ends is never
      restarted or replaced for the same task in that session. A task the session's sub-agent left
      `in-progress` or `blocked` waits for a later session.
   4. The parent ends when no sub-agent is running and no task is eligible, or when the owner tells
      it to stop. On a stop it spawns nothing new and waits for running sub-agents to end. Then it
      writes its journal entry, merges its session branch (ADR-0026 Decision 8.3) and hands off.
3. **Unchanged.** Every other rule of ADR-0026 applies: equivalence, parent-only orchestration,
   isolation, review independence, fixed prompts, serial merges, installs, traceability, no
   nested sub-agents. In ADR-0026 Decision 8.1, "wave" now means all sub-agents of one parent
   session.

## Alternatives considered

- **Keep ADR-0026 unchanged.** Rejected: the owner reports that work moves too slowly. The total
  cap ends a session while eligible tasks remain.
- **Only add the preference, keep the 3 + 3 total.** The owner rejected it when asked. It changes
  little: a session would still stop after six sub-agents.
- **Raise the concurrency limit (5 concurrent, 2 reviews).** The owner rejected it when asked.
  ADR-0026's analysis still holds: M1-T002 … M1-T011 all have `crates: [xtask]`, so only one of
  them runs at a time, and more slots add machine load and conflicts on shared files.
- **Let a session review tasks its own sub-agents wrote.** Rejected: it breaks ADR-0026 Decision
  5.1 and the ADR-0020 incident. The owner asked to keep every rule.

## Consequences

- **Throughput.** A session works until nothing is eligible. One owner action can now move every
  pending review and every eligible implementation. Dependency chains (e.g. M1-T013 → M1-T014)
  still advance by one link per session, because of Decision 5.1.
- **Risk R4 is accepted.** ADR-0026 answered R4 (chains) with the total cap. With no total, one
  owner action can trigger many sub-agents. What bounds it now: the finite set of eligible tasks, the
  one-sub-agent-per-task rule (2.3), the rule that a session never reviews its own tasks, the
  announcement of every spawn, and the owner's right to stop the session. The owner chose this
  trade.
- **Cost.** Long sessions with many sub-agents use more tokens per owner action. The parent's
  context grows with each announcement and result. Longer runs at 3 concurrent `cargo xtask ci`
  builds keep the owner's Mac under load for longer.
- **Review load per session.** One review sub-agent at a time, with serial merges, now limits how
  fast work becomes `done`.
- **Revisit** if a session runs long enough that the parent's context is summarised, and its
  journal then misses a sub-agent, or if the ADR-0026 revisit conditions occur.
- **Owner approval.** Requested by the project owner on 2026-11-09 (quoted under Context), with
  the choice "Remove total cap" when asked about the limits. This ADR changes rules in `AGENTS.md`
  (session protocol step 6). Its ADR review (REVIEW.md §4) happens in an owner-started session
  (ADR-0026 Decision 5.4).

## Verification

- REVIEW.md item 10 is unchanged. A parent journal that lists a second sub-agent for the same task in
  one session, or a review of a task that the same session wrote, is a B2 finding (ADR-0021).
- A session with two or more eligible tasks that did not spawn sub-agents is not a violation
  (SHOULD). Its journal says why.

## Edits on acceptance

These edits are applied in the acceptance commit, verbatim apart from fixes the ADR review asks
for:

1. **Frontmatter.** This ADR: `status: accepted`, `supersedes: [ADR-0026]`. ADR-0026:
   `status: superseded`, `superseded_by: [ADR-0027]`.
2. **`docs/adr/README.md`.** Row 0026: `superseded by ADR-0027`. Row 0027: `accepted`.
3. **`AGENTS.md`, session protocol step 6**:
   - "(ADR-0026)" in the heading becomes "(ADR-0026, ADR-0027)".
   - "A session MAY spawn sub-agents for task work unless the owner forbids it in that session.
     It then follows ADR-0026:" becomes "A session SHOULD spawn sub-agents for task work whenever
     two or more tasks are eligible, unless the owner forbids it in that session. It then
     follows ADR-0026 and ADR-0027:".
   - "At most 3 sub-agents run at once (1 review), with at most 3 + 3 per session, and sub-agents
     never spawn agents." becomes "At most 3 sub-agents run at once (1 review). Keep the slots full
     until no task is eligible; each task gets at most one sub-agent per session. Sub-agents never
     spawn agents."
4. **`docs/process/WORKFLOW.md` §2**:
   - "**Sub-agents (ADR-0026).** Instead, a session may act as a *parent*" becomes
     "**Sub-agents (ADR-0026, ADR-0027).** Instead, a session should act as a *parent* whenever
     two or more tasks are eligible, and".
   - The bullet "runs at most 3 sub-agents at once, at most 1 of them a review, and at most 3
     implementation and 3 review sub-agents in total;" becomes "runs at most 3 sub-agents at
     once, at most 1 of them a review, and refills a free slot at once while an eligible task
     exists; gives each task to at most one sub-agent per session, and ends when nothing runs and
     nothing is eligible, or when the owner says stop;".
5. **`docs/process/REVIEW.md`, `docs/journal/README.md`:** unchanged (they cite ADR-0026, which
   stays in force through this ADR).
