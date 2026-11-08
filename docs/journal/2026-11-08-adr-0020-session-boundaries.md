# 2026-11-08 — ADR-0020: Session boundaries

**Session type:** planning
**Tasks touched:** none. Same session as `2026-11-08-M0-T002.md`.

## Done
- After setting M0-T002 to `review`, this session started a background sub-agent to review M0-T002
  when the owner asked "what's next?". The owner objected. The sub-agent was stopped before it
  changed any file; `git status` was clean and the branch was still at `b7fb5a9`.
- At the owner's request: ADR-0020 (accepted, owner-approved). Only the owner starts sessions. No
  sub-agents for reviews or task work. One session does at most one review, then one implementation
  task, then stops. While a task written in the current session is in `review`, "what's next?" is
  answered with "start a new session; it will review `<task>`, then continue with `<next>`".
- Updated `AGENTS.md` (session protocol step 6, definition of done), `docs/process/WORKFLOW.md`
  (§2 session boundary, §7 step 5), `docs/process/REVIEW.md` (intro, item 10),
  `docs/adr/README.md`, `docs/STATUS.md` (next step for the owner).

## Verified
- Docs only; no code changed. `docs/adr/README.md` lists ADR-0020 once, with status `accepted`, as
  its frontmatter says.

## Decisions
- ADR-0020. Committed on `task/M0-T002-xtask-ci-command`, because the next session starts on that
  branch and must see the rule. It reaches `main` when M0-T002 is merged.

## Oracle changes
- None.

## Problems and traps
- Do not answer "what's next?" by starting agents. See ADR-0020.

## Next
- Project owner: start a new session. It will review M0-T002, then continue with M0-T003.
