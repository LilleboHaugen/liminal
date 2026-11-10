# 2026-11-10 — Parent session: reviews of M1-T020, T031, T015/T016; M1-T032; review fixes

**Session type:** parent (owner-started 2026-11-10 with "Continue"; ADR-0026, ADR-0027, ADR-0030)
**Session branch:** `session/2026-11-10-m1-t020-t031-review`, checked out in the primary checkout
(`/Users/lillebohaugen/Documents/dev/liminal`).
**Start / end:** 22:57:21 CET to about 23:30 CET. Budget 45 minutes (ADR-0030 default; the
owner stated no other): end 23:42, cutoff for new starts 23:22. Last spawn 23:18.
**Tasks touched:** none by the parent. Sub-agents: see the table.

## Identity (facts for reviewers, ADR-0026 Decision 8.1)

- The owner started this session with "Continue". It is not
  `2026-11-10-session-m1-t014-review.md` (merged to `main` as `115b556` before this session
  started).
- At its start this session had implemented nothing. The tasks in `review` at its start
  (M1-T020, M1-T031, M1-T015 with M1-T016) were written by sub-agents of
  `2026-11-10-session-m1-t014-review.md`.
- Tasks this session's implementation sub-agents worked on: M1-T032 (new), M1-T020 and M1-T016
  (changes after their first review in this session). M1-T032 was reviewed by a fresh review
  sub-agent of this session (ADR-0030 Decision 3). M1-T020 and M1-T016 each already had their
  one review sub-agent this session and are left in `review` for the next session.

## Start check (ADR-0030 Decision 2)

- 22:57: one worktree (primary checkout, clean, on the merged branch
  `session/2026-11-10-m1-t014-review`, one commit behind `origin/main`); no unmerged `session/*`
  branch; unmerged `task/*` branches only of merged or in-review tasks; no build running.
- Live Claude processes: PID 17931 (VS Code, started 22:15, the previous Liminal parent session,
  its handoff already merged); PID 65862 (Claude desktop, working in another repository,
  `claude-session-viz`); this session. Reported to the owner; no task work.
- Owner: "Close the old session". The parent sent SIGTERM to PID 17931 (gone afterwards), left
  PID 65862 alone, switched the checkout to `main` (`115b556`) and created the session branch.

## Owner instructions that limit sub-agents (ADR-0026 Decision 1)

- None.

## Sub-agents

Worktrees under `../liminal-worktrees/`: `<ID>` for implementations, `<ID>-review` for reviews.
"review template" = ADR-0030 Decision 7 verbatim; "impl template" = ADR-0026 Decision 5
implementation template verbatim. For the pair review, `<task ID>` was filled with
`M1-T015 and M1-T016` (ADR-0030 Decision 5.1). Status and commits read from the repository.
Times CET.

| # | Kind | Task | Spawned | Before → after | Last branch commit | Merge on `main` | Prompt |
|---|---|---|---|---|---|---|---|
| 1 | review | M1-T020 | 22:58 | review → in-progress (changes required, round 1) | `229b73e` | — | review template |
| 2 | review | M1-T031 | 22:58 | review → done | `5850496` | `483f277` | review template |
| 3 | review | M1-T015 + M1-T016 | 23:02 | T015 review → done; T016 review → in-progress (changes required, round 1) | `638ff2c` | — | review template (pair) |
| 4 | impl | M1-T032 | 23:02 | todo → review | `5580675` | — | impl template |
| 5 | impl | M1-T020 | 23:04 | in-progress → review | `bfa5236` | — | impl template |
| 6 | impl | M1-T016 | 23:07 | in-progress → review | `e610504` | — | impl template |
| 7 | review | M1-T032 | 23:18 | review → in-progress (changes required, round 1) | `65d0c19` | — | review template |

- At most 3 sub-agents ran at once and at most 2 of them were reviews.
- Concurrent implementations shared no crate: M1-T032 `liminal_reflect` + `liminal_reflect_derive`,
  M1-T020 `liminal_math`, M1-T016 `liminal_benches`.
- Each task got at most one implementation and one review sub-agent in this session.
- M1-T016's implementation sub-agent got the impl template with `<task ID>` = `M1-T016` on the
  shared branch `task/M1-T015-M1-T016-liminal-benches` (M1-T015 is already `done` there).
- The parent created branch `task/M1-T032-implement-the-reflect-derive-macro` from `origin/main`;
  `git branch` set its upstream to `origin/main`, which the parent removed before the sub-agent
  pushed anything (a plain `git push` would otherwise have targeted `main`). The parent pushed
  that branch after sub-agent 4 ended (it had committed but not pushed). No other commit by
  the parent on any task branch.
- All worktrees removed; each was clean and its branch equal to `origin` before removal.

## Verified

- Outcomes above read from `origin/main` and the task branches (`status:` lines, `git log`).
- GitHub CI on `main` (Decision 4.4), read once at 23:23: run `42910729047` on `483f277` (head of `main` before this entry) `success`.
  Earlier run this session: `42909604262` on `115b556` `success`.
- GitHub run `42913316674` on the M1-T032 branch (`5580675`): `checks (windows-latest)` failed;
  this is the M1-T032 reviewer's blocking finding.
- `cargo xtask ci` on this branch before merging: see the merge commit.

## Decisions

- Slot order: reviews first (AGENTS.md step 2), then continue `in-progress` tasks with a
  handoff (M1-T020, M1-T016 after their reviews), then the first eligible `todo` (M1-T032,
  eligible once M1-T031 was `done`).
- Not started: M1-T018 (needs the owner's yes to the nightly + Miri install command,
  STATUS_NOTES blocker), M1-T044 (spec; waits for the first `liminal_app` implementation task,
  ADR-0030 Decision 5), M1-T021 (needs M1-T020 `done`), M2 tasks (later milestone).

## Oracle changes

- None by the parent. Sub-agent 5 committed a proptest regression file
  (`crates/liminal_math/tests/matrices.proptest-regressions`); sub-agent 4 added a coverage
  floor for `liminal_reflect_derive` and determinism hash literals; sub-agent 6 changed
  BENCH-R024 in the spec (`spec_version` 2). Their journals give the reasons.

## Problems and traps

- M1-T020 review found a second flaky property test beyond the one reported
  (`determinant_is_exact_for_small_integers`); fixed by sub-agent 5.
- `git branch <new> origin/main` sets upstream `origin/main`. Parents creating task branches
  should use `--no-track`.
- M1-T032 adds external dependencies `syn`, `quote`, `proc-macro2` (rule 7); its reviewer
  judges whether the ADR and `architecture.toml` entries cover them.

## Next

- Start a new session. It reviews M1-T020 and M1-T016 in parallel (second and last round each,
  ADR-0021; M1-T016's review covers the M1-T015/T016 branch, whose merge brings both to `main`).
  Then M1-T032 changes (Windows CI failure: `cargo xtask ci` on Windows cannot replace the
  running `xtask.exe`, because the derive's `syn`/`proc-macro2` features change the `xtask`
  build under `--workspace`; see the M1-T032 Review note). Done when a GitHub run on the task
  branch is green on all three platforms.
- M1-T021 follows M1-T020. M1-T018 needs the owner's yes to the exact install command.
