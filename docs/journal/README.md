# Journal

One entry per session, append-only. Entries are how the next session (with no memory) learns what
happened, what was decided, and what to watch out for. Read the three newest entries at session start.

File name: `YYYY-MM-DD-<task-id-or-topic>.md`. If two sessions share a date and topic, add `-2`.
"Newest" means latest date. For entries on the same date, read all of them.

## Entry format

```markdown
# YYYY-MM-DD — <task ID>: <short title>

**Session type:** implementation | review | kickoff | planning
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

Keep entries factual and short. A journal entry is not a place for plans: those go in task files.
