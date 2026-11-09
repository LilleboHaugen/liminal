---
id: ADR-0020
title: Session boundaries — owner-started sessions only, no sub-agents for task work
status: superseded
date: 2026-11-08
supersedes: []
superseded_by: [ADR-0026]
---

# ADR-0020: Session boundaries — owner-started sessions only, no sub-agents for task work

## Context

AGENTS.md requires that reviews are done by "another session (fresh context)" and that only a
reviewer sets a task to `done`. It did not say who starts that session.

On 2026-11-08 a session implemented M0-T002 and set it to `review`. When the project owner asked
"what's next?", the session started a background sub-agent to review M0-T002. That sub-agent had a
fresh context, but:

- The owner did not start it, and the session did not ask first. The owner could not see the
  sub-agent's work, only the summary it would send back.
- The author session chose the reviewer and wrote its brief, so the review was not independent of
  the author.
- The pattern could repeat as a chain (review → merge → next task → another sub-agent). That moves
  work forward without the owner choosing to start each session.

The project owner stopped this and asked for the rule recorded here (approval below).

## Decision

1. **Sessions are started by the project owner.** A session MUST NOT start other agents
   (sub-agents, background agents, forks, or new sessions) to review a task, implement a task,
   or change a task's status. Read-only lookups that change no files and make no decisions MAY use
   helpers. Their results are the session's responsibility, as if the session had done them.
2. **Shape of a session.** In one session, an agent does at most:
   1. one review of a task in `review` that a previous session wrote (AGENTS.md pick rule 1). If the
      review sets the task to `done`, merge its branch to `main` and push (WORKFLOW.md §8). Then:
   2. one implementation task (continue an `in-progress` task, or start the next `todo`).
   When the session sets its implementation task to `review`, it MUST stop task work and hand off.
   It MUST NOT review its own work, directly or through another agent.
3. **What to tell the owner.** When the owner asks what is next or what the status is, and a task
   written in the current session is in `review`, the agent MUST answer that the owner should start
   a new session. That session will review `<task>` and then continue with `<next task>`. The agent
   does not start that work itself. The same instruction ends the session's final handoff message.
   A session that has not yet done task work answers with the work it would pick (review first),
   and does it when asked.
4. **Parallel tracks** (`docs/ROADMAP.md`) are sessions the owner starts in parallel, never agents
   a session starts.

## Alternatives considered

- **Sub-agent reviews (status quo of that session).** Fast, and the reviewer has a fresh context.
  Rejected: the owner does not see or control the session, and the author writes the reviewer's
  brief. That brief anchors the review.
- **Allow sub-agents, but ask the owner each time.** Rejected: it adds a confirmation step to every
  task. A new session gives a cleaner fresh context with the same owner action (one prompt).
- **Do nothing.** Rejected: the owner explicitly does not want sessions starting agents for task work.

## Consequences

- Progress needs one owner action per task: starting a session. This is intended.
- Each review starts from the repository alone, with no brief from the author. That is the
  independence REVIEW.md already asks for.
- Owner approval: requested by the project owner on 2026-11-08 ("I don't want one session to start
  another agent to work on the next task"). This ADR changes rules in AGENTS.md ("Session protocol").

## Verification

- Review item: REVIEW.md §1 checks that the task's journal entries show no sub-agent did task work
  or a review.
- Journal entries name the session type, and a session's journal lists every task it touched. A
  journal showing a review and a second implementation in one session, or a review of the session's
  own work, is a violation.
