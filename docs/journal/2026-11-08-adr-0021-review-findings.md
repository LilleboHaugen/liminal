# 2026-11-08 — ADR-0021: Review findings and round limit

**Session type:** planning
**Tasks touched:** none. Same session as `2026-11-08-M0-T003-review-4.md` and `2026-11-08-M0-T003-5.md`.

## Done
- The owner asked whether the project was stuck, after M0-T003's fourth "changes required"
  review, and then asked for a rule change. Wrote ADR-0021 (accepted, owner-approved). Only
  findings of classes B1–B5 block. All other findings go into one follow-up task. Re-reviews are
  limited to the previous findings and the new diff. A task gets at most two "changes required"
  outcomes, after which a blocking finding goes to the owner as `blocked`.
- Updated `AGENTS.md` (session protocol 2.1, definition of done), `docs/process/REVIEW.md`
  (findings and outcome), `docs/process/WORKFLOW.md` (§2), `docs/adr/README.md`,
  `docs/STATUS.md`.

## Verified
- Docs only; no code changed. `cargo xtask check-docs`: ok (ADR frontmatter, index row, links).

## Decisions
- ADR-0021. Committed on `task/M0-T003-check-docs` (as with ADR-0020), because the next session
  starts on that branch. It reaches `main` when M0-T003 is merged.
- Under ADR-0021, review 4's U+FEFF finding would have been a follow-up, not blocking (not
  realistic input, B3). The fix is already made and tested, so it stays.

## Oracle changes
- None.

## Problems and traps
- M0-T003's next review falls under ADR-0021 item 4: it cannot set the task to `in-progress`.
- The task-file observation about `markdown.rs` trimming table cells with Unicode `trim` is a
  candidate for the follow-up task of that review.

## Next
- Project owner: start a new session. It will review M0-T003, then continue with M0-T004.
