# 2026-11-10 — Parent session: open M1 reviews, then M1-T020, T015+T016, T031

**Session type:** parent (owner-started 2026-11-10 with "Continue"; ADR-0026, ADR-0027, ADR-0030)
**Session ID:** Claude Code session `ac6793a5-fd40-4d6b-ae5c-076eded5b59e`
**Session branch:** `session/2026-11-10-m1-t014-review`, checked out in the primary checkout
(`/Users/lillebohaugen/Documents/dev/liminal`).
**Start / end:** 22:15:47 CET to about 22:56 CET. Budget 45 minutes (ADR-0030 default, the owner
stated no other): end 23:00, cutoff for new starts 22:40. Last spawn 22:36.
**Tasks touched:** none by the parent. Sub-agents: see the table.

## Identity (facts for reviewers, ADR-0026 Decision 8.1)

- The owner started this session with "Continue". It is not `2026-11-10-session-m1-reviews.md`
  nor `2026-11-10-session-adr-0030.md` (both merged to `main` before this session started).
- At its start this session had implemented nothing. Every task in `review` at its start
  (M1-T014, T037, T058, T008, T009, T010) was written by sub-agents of earlier sessions.
- Tasks this session's implementation sub-agents wrote: M1-T061, M1-T020, M1-T015 with M1-T016,
  M1-T031. M1-T061 was reviewed by a fresh review sub-agent of this session (ADR-0030
  Decision 3 allows it). The parent implemented and reviewed nothing.

## Start check (ADR-0030 Decision 2)

- 22:15: one worktree (primary checkout on `main`, clean), no unmerged `session/*` branch, six
  unmerged `task/*` branches (the six tasks in review), no build running.
- One other live Claude session (`liminal-c0`, VS Code, idle, started about 20 s before this
  one). Reported to the owner; no task work. Owner: "Close all running sessions". The parent
  identified its own process through its parent-process chain (PID 17931) and sent SIGTERM to
  the other one (PID 17788); afterwards only PID 17931 ran and no peer session was listed.
  Owner then: "Go".

## Owner instructions that limit sub-agents (ADR-0026 Decision 1)

- None.

## Sub-agents

Worktrees under `../liminal-worktrees/`: `<ID>` for implementations, `<ID>-review` for reviews.
"review template" = ADR-0030 Decision 7 verbatim; "impl template" = ADR-0026 Decision 5
implementation template verbatim. For the pair, `<task ID>` was filled with `M1-T015 and
M1-T016` (ADR-0030 Decision 5.1). Status and commits read from the repository. Times CET.

| # | Kind | Task | Spawned | Before → after | Last branch commit | Merge on `main` | Prompt |
|---|---|---|---|---|---|---|---|
| 1 | review | M1-T014 | 22:20 | review → done | `d7d7acc` | `35a8597` | review template |
| 2 | review | M1-T037 | 22:20 | review → done | `14d8df3` | `c891029` | review template |
| 3 | impl | M1-T061 | 22:20 | todo → review | `ef7ba98` | — | impl template |
| 4 | review | M1-T058 | 22:24 | review → done | `7c1d7f8` | `7c1d7f8` | review template |
| 5 | impl | M1-T020 | 22:26 | todo → review | `6cb91b7` | — | impl template |
| 6 | review | M1-T061 | 22:26 | review → done | `ef3d586` | `043f3e2` | review template |
| 7 | review | M1-T008 | 22:28 | review → done | `70dac69` | `70dac69` | review template |
| 8 | review | M1-T009 | 22:30 | review → done | `e652b0c` | `e652b0c` | review template |
| 9 | review | M1-T010 | 22:32 | review → done | `20d96bb` | `a1cf50e` | review template |
| 10 | impl | M1-T015 + M1-T016 | 22:35 | todo → review | `12543d4` | — | impl template (pair) |
| 11 | impl | M1-T031 | 22:36 | todo → review | `2a12e48` | — | impl template |

- At most 3 sub-agents ran at once and at most 2 of them were reviews.
- Concurrent implementations shared no crate: M1-T020 `liminal_math`, M1-T015/T016
  `liminal_benches` (plus `xtask`, see Problems), M1-T031 `liminal_reflect`. M1-T061 (spec of
  `liminal_reflect_derive`) had ended before M1-T031 started.
- The parent pushed the branches of sub-agents 3 and 10, which had committed but not pushed.
  It made no other commit on any task branch.
- All worktrees removed; each was clean and its branch equal to `origin` (or, for the detached
  M1-T061 review worktree, its HEAD contained in `origin/main`) before removal.

## Verified

- Outcomes above read from `origin/main` and the task branches (`status:` lines, `git log`).
- GitHub CI on `main` (Decision 4.4), read once at 22:54: run `42906566265` on `a1cf50e` (head
  of `main` before this entry) `success`. Earlier runs this session: `35a8597`, `c891029`,
  `7c1d7f8`, `70dac69` success; `043f3e2` and `e652b0c` cancelled by the next push
  (`cancel-in-progress`).
- `cargo xtask ci` on this branch before merging: see the merge commit.

## Decisions

- Slot order: reviews first (AGENTS.md step 2), then the first eligible `todo` tasks by ID.
  M1-T018 was not started: it needs the owner's yes to an exact install command (nightly +
  Miri, STATUS_NOTES blocker 1), which a sub-agent cannot obtain.
- No GitHub action other than pushes by sub-agents and the parent.

## Oracle changes

- None by the parent. Sub-agent 10 raised the `xtask` coverage floor 97.6 → 98.2 and changed two
  `xtask` tests; its journal `2026-11-10-M1-T015-M1-T016.md` gives the reasons. For its reviewer.

## Problems and traps

- **`xtask` touched by M1-T016** although `xtask` is frozen (ADR-0030 Decision 9) except for
  tooling that blocks engine work: a new check-arch rule XTASK-R090 and a coverage file filter
  (cargo-llvm-cov skips every `benches/` directory by default, so the new crate measured no
  lines). The reviewer of M1-T015/T016 judges whether both fall under the exception.
- **Renumbering on merge** happened three times this session (M1-T037's M1-T061 → M1-T062;
  XTASK-R014 → R015 in M1-T010; `xtask` `spec_version` 25, 26, 27 for T008, T009, T010). The
  M1-T015/T016 branch also sets `xtask` `spec_version` 27 from an older base: its merge must
  renumber to 28.
- **M1-T009 merge** needed a code fix: M1-T008's new `check_licence_file` called the old
  four-argument `run_deny`. Fixed by the M1-T009 reviewer.
- Several reviewers report that `origin` names the old repository URL; GitHub redirects it.
  Not changed (owner may run `git remote set-url origin https://github.com/LilleboHaugen/liminal.git`).

## Next

- Start a new session. It runs the start check, then reviews M1-T020 and M1-T031 in parallel
  (two review slots), then M1-T015 with M1-T016 (one review of both). M1-T021 follows M1-T020;
  M1-T032 follows M1-T031.
- M1-T018 needs the owner's yes to the exact install command before it can start.
