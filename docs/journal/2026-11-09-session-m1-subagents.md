# 2026-11-09 — Parent session: M1 follow-ups and specs through sub-agents

**Session type:** parent (owner-started, 2026-11-09 to 2026-11-10; ADR-0026, ADR-0027)
**Tasks touched:** none by the parent. Sub-agents: see the table.

Before acting as parent, this session did the ADR review of ADR-0027 and accepted it
(`2026-11-09-adr-0027-review.md`). Session branch: `session/2026-11-09-m1-subagents`.

## Owner instructions in this session (ADR-0026 Decision 8.1)

1. Approved acceptance of ADR-0027 with the two review clarifications (answer "Approve").
2. "You are allowed by me to merge it" — about the M1-T002 review commit `c8dd306`, which the
   review sub-agent set to `done` but did not push or merge. The parent merged it (below). This
   deviates from ADR-0026 Decision 6 (a parent never merges a task branch), on the owner's
   explicit instruction.
3. "The agents stopped. Start them again" (2026-11-10) — after the M1-T027 sub-agent died and the
   owner interrupted the session. The parent restarted M1-T027 and M1-T030. This deviates from
   ADR-0027 Decision 2.3 (no restart or replacement for the same task), on the owner's explicit
   instruction. The quote was added to both prompts as permitted by ADR-0026 Decision 5.

## Sub-agents

All worktrees under `../liminal-worktrees/<ID>`. Prompt "impl template" / "review template" =
the ADR-0026 template verbatim. Status before → after is read from the task frontmatter on the
branch, last commit from `git log` / `git ls-remote`.

| # | Kind | Task | Branch | Before → after | Last commit | Sub-agent journal | Prompt |
|---|---|---|---|---|---|---|---|
| 1 | review | M1-T002 | `task/M1-T002-check-docs-follow-ups` (wt `M1-T002-review`) | review → done | `c8dd306` | `2026-11-09-M1-T002-review.md` | review template |
| 2 | impl | M1-T003 | `task/M1-T003-check-arch-follow-ups` | todo → review | `1c98c66` | `2026-11-09-M1-T003.md` | impl template |
| 3 | impl | M1-T012 | `task/M1-T012-write-the-spec-for-liminal-core` | todo → review | `886ed0c` | `2026-11-09-M1-T012.md` | impl template |
| 4 | impl | M1-T004 | `task/M1-T004-trace-hygiene-follow-ups` | todo → review | `b8031d0` | `2026-11-09-M1-T004.md` | impl template |
| 5 | impl | M1-T024 | `task/M1-T024-write-the-spec-for-liminal-memory` | todo → review | `f3d1af0` | `2026-11-09-M1-T024.md` | impl template |
| 6 | impl | M1-T005 | `task/M1-T005-coverage-follow-ups` | todo → review | `4af8ec6` | `2026-11-09-M1-T005.md` | impl template |
| 7 | impl | M1-T006 | `task/M1-T006-ci-workflow-follow-ups` | todo → review | `efe662f` | `2026-11-09-M1-T006.md` | impl template |
| 8 | impl | M1-T007 | `task/M1-T007-status-follow-ups` | todo → review | `6da51b8` | `2026-11-09-M1-T007.md` | impl template |
| 9 | impl | M1-T008 | `task/M1-T008-deny-doctor-follow-ups` | todo → review | `e13eebe` | `2026-11-09-M1-T008.md` | impl template |
| 10 | impl | M1-T009 | `task/M1-T009-deny-policy-test-gaps` | todo → review | `08e5a16` | `2026-11-09-M1-T009.md` | impl template |
| 11 | impl | M1-T010 | `task/M1-T010-binary-test-follow-ups` | todo → review | `c83a2c2` | `2026-11-09-M1-T010.md` | impl template |
| 12 | impl | M1-T011 | `task/M1-T011-check-arch-determinism-lint-configuration-for-de` | todo → review | `ecf7f24` | `2026-11-09-M1-T011.md` | impl template |
| 13 | impl | M1-T027 | `task/M1-T027-write-the-spec-for-liminal-telemetry` | todo → in-progress (uncommitted) | none | none | impl template |
| 14 | impl | M1-T030 | `task/M1-T030-write-the-spec-for-liminal-reflect` | todo → review | `576eb66` | `2026-11-10-M1-T030.md` | impl template |
| 15 | impl | M1-T027 (restart) | same as 13 | in-progress (uncommitted) → review | `a7accd2` | `2026-11-10-M1-T027.md` | impl template + owner quote 3 |
| 16 | impl | M1-T030 (restart) | same as 14 | — (stopped by parent, no own commit) | — | none | impl template + owner quote 3 |
| 17 | impl | M1-T054 | `task/M1-T054-m1-kickoff-follow-ups` | todo → review | `97e89b2` | `2026-11-10-M1-T054.md` | impl template |

Sub-agent journals name this entry inconsistently (the prompt did not give its file name):
`2026-11-09-m1-subagents.md`, the session branch name, or a description. They all mean this file.

## Done
- Spawned the 17 sub-agents above, at most 3 at once and at most 1 review. Merges by the parent:
  M1-T002 only (owner instruction 2). Branch pushed by the parent: M1-T004 (`b8031d0`, the
  sub-agent committed but did not push). No other task, status or merge change by the parent.
- **M1-T002 merge:** pushed the branch, branch CI run 42686720480 green; merged on `main` in
  worktree `M1-T002-review` as `5383696`. `docs/STATUS_NOTES.md` conflict combined (ADR-0027
  paragraph from `main`, "M1-T002 done; follow-ups in M2-T001" from the branch; both stale
  next-step sentences dropped). `docs/STATUS.md` regenerated. `cargo xtask ci` on the merge
  result: 11 steps pass. Pushed `main`.
- **Skips (ADR-0026 Decision 10):**
  - M1-T004 … M1-T011 waited while another `crates: [xtask]` task ran (Decision 4.2).
  - M1-T054 (`crates: []`) ran only when no other implementation ran (Decision 4.2).
  - M1-T027 and M1-T030 were held while an `xtask` task ran: every new spec makes the `xtask`
    test `loads_the_real_repository` fail on `main`, so a spec task also changes
    `xtask/src/repo.rs` (as M1-T012 and M1-T024 did). Applied as the intent of Decision 4.2;
    the frontmatter check alone allowed it.
  - No review slot was used after M1-T002: every task in `review` was written by this session
    (Decision 5.1), and no other session left one.

## Verified
- `cargo xtask ci` on `5383696` (M1-T002 merge): pass, 11 steps, macOS 27.2, Rust 1.99.1.
- For every sub-agent: task status and last commit read from the branch; worktree clean before
  removal; branch on `origin` via `git ls-remote`, except as noted.
- Not verified by the parent: the sub-agents' own `cargo xtask ci` claims and branch CI runs.

## Decisions
- ADR-0027 accepted (separate entry).
- Spec tasks held while an `xtask` task runs, until M1-T012's `repo.rs` fix is on `main` (above).

## Oracle changes
- None by the parent.

## Problems and traps
- **M1-T002 reviewer stopped before push and merge** (ADR-0026 Decision 6). Resolved by owner
  instruction 2.
- **Sub-agents 13 and 16.** No. 13 died on an API error ("computer went to sleep"); it left only
  an uncommitted status change. After the owner's interrupt, the parent wrongly concluded that
  no. 14 (M1-T030) had stopped, because its worktree was still clean, and started no. 16 in the
  same worktree. For some minutes two agents worked in one worktree. No. 16 set the status to
  `in-progress` and applied the `repo.rs` test fix (identical to M1-T012's). No. 14 noticed the
  foreign edit, kept it and recorded it in its handoff and commit `576eb66`. The parent stopped
  no. 16 on no. 14's report; it left no other change. Trap: an interrupt does not reliably stop
  background sub-agents. Wait for each agent's notification before reusing its task.
- **Parallel owner-started sessions** reviewed and merged M1-T003, M1-T005 and M1-T007, and
  accepted ADR-0028 (AGENTS.md rule 17), while this session ran. One of them switched the
  primary checkout from this session's branch to `main` (ADR-0026 4.1 assumes one parent per
  checkout). This entry was written in its own worktree `../liminal-worktrees/session-m1-subagents`.
- **M1-T006** opened and closed draft PR #1 and added `.github/dependabot.yml`. ADR-0028 Decision 6
  now needs the owner's yes for opening PRs. Its reviewer checks REVIEW.md item 11.
- **M1-T009** wrote registry entries into the owner's `~/.cargo/registry` during an experiment
  and reports removing them (not checked by the parent).
- **Merge conflicts ahead:**
  - `docs/specs/xtask.md` `spec_version` and history: M1-T004, T006, T008, T009, T010, T011 and
    T054 each bump it (to 18–21) from different bases.
  - `xtask/src/repo.rs`: M1-T012, T024, T027 and T030 change the same test identically.
  - `docs/specs/README.md`: one index row each from M1-T012, T024, T027, T030.
  - Renumber spec versions and keep both sides (ADR-0026 4.3).
- **Gaps reported by sub-agents, no task yet:**
  1. No task for the ADR-0017 check that error codes in code and spec agree (M1-T012).
  2. M1-T047 names its error area `TK`; the reserved prefix is `TKIT` (M1-T012).
  3. `liminal_core` `usable` needs a user crate or example; may be unmet after M1-T014 (M1-T012).
  4. `architecture.toml` has no `liminal_reflect` → `liminal_math` edge; decide by M1-T037 (M1-T030).
  5. `cargo xtask ci` never builds with the `telemetry` feature off; needed before M1-T029 is
     reviewed (M1-T027).
  6. The first implementation task of each deterministic crate must add its `clippy.toml`
     (XTASK-R039, M1-T011).

## Next
- Start a new session. It will review whichever of M1-T004, M1-T006, M1-T008 … M1-T012, M1-T024,
  M1-T027, M1-T030 and M1-T054 (all written in this session) are still in `review`, with review
  sub-agents, one at a time. The parallel session `session/2026-11-09-m1-reviews` may already have
  reviewed some (an M1-T012 review worktree existed at handoff).
