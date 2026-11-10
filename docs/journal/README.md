# Journal

One entry per session, append-only. Entries are how the next session (with no memory) learns what
happened, what was decided, and what to watch out for. An owner-started session reads the newest
entry whose file name contains `session`; a sub-agent reads the entries named after its task
(ADR-0030 Decision 6).

File name: `YYYY-MM-DD-<task-id-or-topic>.md`. If two sessions share a date and topic, add `-2`.
A parent or owner-started session names its entry `YYYY-MM-DD-session-<topic>.md` and gives its
start time, time budget and end time. "Newest" means latest date, then latest commit.

## Entry format

```markdown
# YYYY-MM-DD — <task ID>: <short title>

**Session type:** implementation | review | kickoff | planning | parent
**Tasks touched:** M1-T014 (in-progress → review)

## Done
- Concrete changes, with file paths.

## Verified
- What was run and the result (`cargo xtask ci`: pass on macOS 27.2). What could NOT be verified, and why.

## Decisions
- Decisions made, with links to ADRs or spec changes. "None" is a valid answer.

## Oracle changes
- Golden images, snapshots, coverage floors or baselines changed, and why. "None" if none.

## Problems and traps
- Things that went wrong or surprised you, so the next session avoids them.

## Next
- The exact next step.
```

A sub-agent writes `implementation (sub-agent, parent: <parent journal file>)` or
`review (sub-agent, parent: <parent journal file>)`. A parent entry lists every sub-agent it
spawned (ADR-0026 Decision 8).

Keep entries factual and short. A journal entry is not a place for plans: those go in task files.
