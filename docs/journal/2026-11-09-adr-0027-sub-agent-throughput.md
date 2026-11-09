# 2026-11-09 — ADR-0027: Sub-agents by default, no per-session total (proposed)

**Session type:** planning (same owner-started session as `2026-11-09-adr-0026-review.md`)
**Tasks touched:** none.

## Done
- The owner asked for sessions to prefer sub-agents and spawn enough to work efficiently, within
  the rules. Asked about limits, the owner chose "Remove total cap" (concurrency 3, 1 review, kept).
- Wrote `docs/adr/0027-sub-agents-by-default-no-total-cap.md` (`proposed`). It replaces
  ADR-0026 Decision 1 (in part) and Decisions 9.2 and 9.4. Every other decision of ADR-0026 stays.
  Edits on acceptance are written out. They are not applied.
- `docs/adr/README.md` row 0027; `docs/STATUS_NOTES.md` next step.

## Verified
- macOS 27.2, Rust 1.99.1: `cargo xtask ci` on `adr/0027-subagent-throughput` passes (11 steps).

## Decisions
- Accepted ADR-0026 is not edited (rule 8). The change is a new ADR.
- ADR-0026 Decision 5.1 is kept (a session never reviews its own tasks). Within one session,
  dependency chains therefore still advance only one link. The ADR states this so nobody expects more.
- This session wrote ADR-0027, so it does not review it. The ADR review is done by a later
  owner-started session (ADR-0026 Decision 5.4).

## Oracle changes
- None.

## Problems and traps
- The branch is cut from `main` after the ADR-0026 merge. `main` does not contain ADR-0027 until
  the review merges it.

## Next
- Start a new session on branch `adr/0027-subagent-throughput`. It reviews and accepts ADR-0027,
  then reviews M1-T002 and continues with task work, using sub-agents.
