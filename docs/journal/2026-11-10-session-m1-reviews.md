# 2026-11-10 — Parent session: M1 reviews of the m1-subagents wave

**Session type:** parent (owner-started 2026-11-10 about 17:52 CET; ADR-0026, ADR-0027)
**Session ID:** Claude Code session `f51f9eb3-5cd6-44cb-89e9-4a0bf0d07ef2`
**Session branch:** `session/2026-11-10-m1-reviews`, checked out in the primary checkout
(`/Users/lillebohaugen/Documents/dev/liminal`), so `main` is free for merging reviewers.
**Tasks touched:** none by the parent, except the merge of M1-T012 on the owner's instruction.
Sub-agents: see the table.

## Identity (facts for reviewers, ADR-0026 Decision 8.1)

- The project owner started this session on 2026-11-10 with the instruction "continue".
- This session is neither `2026-11-09-session-m1-subagents.md` (branch
  `session/2026-11-09-m1-subagents`, ended 2026-11-10 17:41 CET, merged to `main` as `bb94e3a`)
  nor `2026-11-09-session-m1-reviews.md` (branch `session/2026-11-09-m1-reviews`, entry still
  "in progress", not on `main`).
- At its start this session had implemented nothing. Every task in `review` at its start
  (M1-T004, T006, T008, T009, T010, T011, T024, T027, T030, T054) was written by sub-agents of
  `2026-11-09-session-m1-subagents.md`, so this session's review sub-agents may review them
  (ADR-0026 Decision 5.1).

- **Rule change during this session.** ADR-0029 was accepted on 2026-11-10 on the owner's
  instruction and is on `main` since `c75a54e`. Its Decision 1 replaces ADR-0026 Decision 5.1:
  a fresh review sub-agent MAY review a task that an implementation sub-agent of the same parent
  wrote. `AGENTS.md`, `docs/process/WORKFLOW.md` and `docs/process/REVIEW.md` on `main` carry
  the new wording. Task branches created before `c75a54e` still contain the old wording until
  `main` is merged into them; the text on `main` is the one in force.
- Tasks this session's implementation sub-agents wrote: M1-T013, T019, T034, T055, T056, T057,
  T058. The parent itself implemented nothing on any task.

## Owner instructions that limit sub-agents (ADR-0026 Decision 1)

- None at the start.
- "Stop as soon as you have a clean exit point" (about 20:10 CET). From then on nothing was
  spawned; the three running sub-agents (rows 21, 23, 26) ended on their own.

## Sub-agents

Worktrees under
`../liminal-worktrees/`: `<ID>` for implementations, `<ID>-review` for reviews. Prompt "review
template" / "impl template" = the ADR-0026 template verbatim. Status and commits are read from
the repository. Sub-agent journals: `2026-11-10-<ID>.md` (implementation) and
`2026-11-10-<ID>-review.md` (review).

| # | Kind | Task | Before → after | Last branch commit | Merge on `main` | Prompt |
|---|---|---|---|---|---|---|
| 1 | review | M1-T054 | review → done | `cb4e9fa` | `dde4005` | review template |
| 2 | impl | M1-T055 | todo → review | `6328305` | — | impl template |
| 3 | review | M1-T027 | review → done | `2729ef7` | `67efefb` | review template |
| 4 | impl | M1-T019 | todo → review | `bf8c884` | — | impl template |
| 5 | review | M1-T004 | review → done | `2d68a1f` | `02b19e7` | review template |
| 6 | impl | M1-T034 | todo → review | `1a06db9` | — | impl template |
| 7 | impl | M1-T056 | todo → review | `f61f9d6` | — | impl template |
| 8 | review | M1-T011 | review → done | `30325c0` | `cda3751` | review template |
| 9 | review | M1-T030 | review → done | `3810fca` | `ae6bdf7` | review template |
| 10 | impl | M1-T013 | todo → review | `38b43c1` | — | impl template |
| 11 | review | M1-T024 | review → done | `51f2421` | `3f1b06f` | review template |
| 12 | impl | M1-T057 | todo → review | `9903681` | — | impl template |
| 13 | review | M1-T006 | review → done | `2d44128` | `f0ca6f0` | review template |
| 14 | impl | M1-T058 | todo → review | `6209b31` | — | impl template |
| 15 | review | M1-T013 | review → done | `bd4933e` | `047574b` | review template; one further message (below) |
| 16 | review | M1-T055 | review → done | `0eed5dc` | `64bd047` | review template |
| 17 | impl | M1-T059 | todo → review | `2366b2a` | — | impl template |
| 18 | review | M1-T019 | review → done | `ceae17e` | `5e0a1ea` | review template |
| 19 | review | M1-T059 | review → done | `86ecb7b` | `0daa063` | review template |
| 20 | impl | M1-T060 | todo → review | `9042dd5` | — | impl template |
| 21 | impl | M1-T014 | todo → review | `fc7d13b` | — | impl template |
| 22 | review | M1-T034 | review → done | `3c3a551` | `bef9c95` | review template |
| 23 | impl | M1-T037 | todo → review | `d36b53c` | — | impl template |
| 24 | review | M1-T060 | review → done | `72f4102` | `4c02a19` | review template |
| 25 | review | M1-T056 | review → done | `3a70ac0` | `f8b9660` | review template |
| 26 | review | M1-T057 | review → done | `80bb6c8` | `63774cb` | review template |

Rows 15, 16, 18, 19, 22, 24, 25 and 26 are reviews of tasks that implementation sub-agents of this session
wrote, under ADR-0029 Decision 1 (in force on `main` from `c75a54e`).

Further message to sub-agent 15, after it first declined the review because its worktree's
branch predated ADR-0029 (verbatim):

> The rule you cited was replaced on `main` after the M1-T013 branch was created. `origin/main`
> (`c75a54e`) contains `docs/adr/0029-faster-loop.md` (accepted 2026-11-10, supersedes ADR-0021
> and ADR-0027; its Decision 1 replaces ADR-0026 Decision 5.1) and the matching wording in
> `AGENTS.md`, `docs/process/WORKFLOW.md` and `docs/process/REVIEW.md`. Your worktree's branch
> predates that commit, so it shows the old text. Read those files from `origin/main`, decide
> for yourself whether the review of M1-T013 is now yours to do, and if so complete it as those
> documents describe (review, and merge if you set `done`), then stop.

## Merges and rule changes by the parent, on the owner's instruction

1. M1-T012: its reviewer (a sub-agent of `2026-11-09-session-m1-reviews.md`) set `done` but did
   not merge. Owner, asked in this session: "Yes, merge it". Merged as `bbd28ee`; `cargo xtask
   ci` on the result: 11 steps pass. Deviation from ADR-0026 Decision 6.
2. Branch `session/2026-11-09-m1-reviews` (one unfinished journal entry). Owner: "Yes, merge as
   is". Merged as `b3feded`; CI on the result: pass. That entry lacks its M1-T012 review row and
   its end-of-session sections.
3. ADR-0029. Owner: "You are hereby allowed by me to change the ADR with those points directly,
   and merge to main". Written and accepted by this session with no separate ADR review; merged,
   `main` at `c75a54e`; CI on the result: pass.

## Done
- 26 sub-agents (table): 14 reviews, each ended in `done` and a merge to `main` by the reviewer;
  12 implementations, each ended in `review`. At most 3 ran at once, at most 1 review.
- `done` on `main` through this session: M1-T012 (merge only), T054, T027, T004, T011, T030,
  T024, T006, T013, T055, T019, T059, T034, T060, T056, T057.
- In `review` at the end, each on its own pushed task branch: M1-T014 and M1-T037 (written by
  this session, not reviewed), M1-T058 (written by this session), M1-T008, T009, T010 (written
  by `2026-11-09-session-m1-subagents.md`).
- ADR-0029 written, accepted and applied on the owner's instruction (item 3 above).
- All worktrees under `../liminal-worktrees/` removed; each branch was pushed and clean first.

## Verified
- For every sub-agent: task status and last commit read from the branch and from `origin/main`;
  worktree clean and branch equal to `origin` before removal.
- `cargo xtask ci` by the parent: on the M1-T012 merge, the `session/2026-11-09-m1-reviews`
  merge, the ADR-0029 merge, and this session branch after merging `main` (macOS 27.2 arm64,
  Rust 1.99.1), all steps pass.
- Not verified by the parent: the sub-agents' own `cargo xtask ci` claims, their branch CI runs,
  and the CI runs on `main` after each merge.

## Decisions
- ADR-0029 (owner instruction). Its fourth offered point, moving M1-T008 … T010 to M2, was left
  out: they are implemented and need only reviews.
- Deviations from "refill a free slot at once" (ADR-0027 Decision 2.2):
  - M1-T014 was held until the M1-T055 review ended, because M1-T055 changed M1-T014's
    requirement list. The review then created M1-T059, which M1-T014 depends on.
  - After the owner's stop instruction no slot was refilled.
- Review order was chosen to unblock implementation tasks first (M1-T054, T027, T004, T011
  before T030, T024, T006; later M1-T013, T055, T059, T060 before the others).

## Oracle changes
- None by the parent. Sub-agents' oracle changes are in their own journal entries (M1-T013 and
  M1-T014: two relaxed `xtask` tests, coverage floor for `liminal_core`, recorded literals).

## Problems and traps
- **Task ID collision M1-T061.** The M1-T057 review created
  `M1-T061-reflect-derive-path-check-definition.md` on `main`. The M1-T037 branch (in `review`,
  not merged) also creates an M1-T061 (`Reflect` impls for `liminal_math` and `liminal_core`
  types) and names it in `depends_on` of M1-T050 and M1-T053. Whoever merges M1-T037 renumbers
  its task to the next free ID; the M1-T037 handoff lists what to change.
- **Spec follow-up beside its implementation.** M1-T055 (spec change assigning CORE-R041 to
  M1-T013) and M1-T013 ran at the same time; the requirement landed on a `done` task untested
  and needed M1-T059. Do not run a contract-changing follow-up beside the task it changes
  (now ADR-0029 Decision 2.3: list it in `depends_on`).
- **Stale rules in review worktrees.** A review worktree is on the task branch. Branches created
  before `c75a54e` carry the pre-ADR-0029 `AGENTS.md`. Sub-agent 15 declined its review for
  that reason until told where the current text is; later reviewers read `main` themselves.
  This applies to M1-T008, T009, T010 and M1-T058, whose branches predate `c75a54e`.
- **`docs/specs/xtask.md` `spec_version`** is 24 on `main`. M1-T008, T009 and T010 each bump it
  from an older base and must renumber on merge, keeping all history rows.
- **M1-T014 oracles.** The reference C programs named in the core spec's test plan were not
  run (compiling downloaded code was denied). Published vectors and an independent Python
  implementation were used. The M1-T014 reviewer judges this.
- **Dependabot** is active since `.github/dependabot.yml` reached `main` (M1-T006). A pull
  request from it is outside code (ADR-0028 Decision 5).
- The `origin` URL of the owner's checkout still names the repository `rust-game-engine`;
  GitHub redirects it. Not changed.
- The harness flagged sub-agent 5 (M1-T004 review) with "modify shared resources". The parent
  checked the repository at that moment: a pushed review commit and a staged, unpushed merge in
  its own worktree, which is the reviewer procedure.

## Next
- Start a new session. It reviews M1-T014 first (task review and the promotion review of
  `liminal_core` to `usable`), then M1-T037, T058, T008, T009, T010, and assigns the eligible
  implementation tasks (M1-T061 now; M1-T020 and M1-T015 with M1-T016 once M1-T014 is `done`).
- Owner decisions open: see `docs/STATUS_NOTES.md` "Blockers needing an owner decision".
