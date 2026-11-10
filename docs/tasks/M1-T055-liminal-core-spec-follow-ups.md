---
id: M1-T055
title: liminal_core spec follow-ups from the M1-T012 review
milestone: M1
status: done
crates: [liminal_core]
depends_on: [M1-T012]
requirements: []
---

# M1-T055: `liminal_core` spec follow-ups from the M1-T012 review

## Goal

The `liminal_core` spec leaves no `Default` behaviour or test oracle open to a wrong reading, and the
M1 spec tasks use the reserved requirement prefixes.

## Context

- Follow-up task under ADR-0021, written by the review of M1-T012 (see that task's "Review").
- `docs/specs/liminal_core.md` v1 §5.2, §5.3, §6 (CORE-R060), §9, §10, §12 item 4.
- In M1, not M2: the items amend contracts that M1 tasks implement (M1-T013, M1-T014, M1-T027,
  M1-T047). Best done before M1-T013 starts; if M1-T013 is already done, item 1 also checks the
  code.

## Scope

**In:**
1. §5.2 shows `HandleAllocator` with `#[derive(Clone, Debug, Default)] // Default = new()`.
   A derived `Default` gives every field its default, so a straightforward implementation has a
   slot limit of 0 (every `allocate` fails with LIM-CORE-0011) instead of `new()`'s `u32::MAX`.
   Remove `Default` from the derive list and write `impl Default for HandleAllocator { /* new() */ }`
   as for `HandleMap`.
2. No requirement says `Default::default()` equals `new()` for `HandleAllocator`, `HandleMap`,
   `StateHasher`, `DetHashMap`, `DetHashSet` (only `TickRate`, CORE-R110). Add the statement to
   existing requirements or a new one (bump `spec_version`, history row, assign to M1-T013 or
   M1-T014 and write the ID into that task's `requirements`).
3. CORE-R060 / §10: the "recorded hash literal" for `FixedState` covers `hash_one(&v)` of a
   `v: Hash`, whose `Hash` impl is std's and, as §5.3 itself says, not guaranteed stable across
   Rust releases. Restrict the recorded literal to values whose byte sequence the test fixes
   (calls on `DetHasher` directly, or integer types), or state that a toolchain bump may change
   it and is then an intended oracle change.
4. §9: two budget rows say "Not scheduled". Name the task (or the promotion review) that adds
   their measurement, or drop the rows until it exists.
5. M1-T027 and M1-T047 name the areas `TEL`/`TK` (`TEL-Rnnn`, `LIM-TEL-NNNN`, `TK-Rnnn`,
   `LIM-TK-NNNN`); `docs/specs/README.md` reserves `TELE` and `TKIT`, and CORE-R012 requires the
   code area to equal the spec's `req_prefix`. Correct both task files (if they are still `todo`)
   and close spec §12 item 4.

**Out:** Any other change to the `liminal_core` design.

## Acceptance criteria

- [x] Items 1 to 5 are done, or a declined item is written into this task with the reason.
- [x] Spec changes bump `spec_version` with a history row.
- [x] `cargo xtask ci` passes.

## Test requirements

None beyond the requirement(s) added by item 2, which their implementation task tests.

## Handoff

2026-11-10, implementation sub-agent (ADR-0026/0027), branch
`task/M1-T055-liminal-core-spec-follow-ups`, worktree `../liminal-worktrees/M1-T055`. Journal:
`docs/journal/2026-11-10-M1-T055.md`.

Done (`docs/specs/liminal_core.md`, `spec_version` 1 → 2, history row added):

| Item | Change |
|---|---|
| 1 | §5.2: `Default` removed from the `HandleAllocator` derive list; `impl Default for HandleAllocator { /* new() */ }` added, as for `HandleMap`. No code to check: M1-T013 is `todo` and `crates/` does not exist. |
| 2 | Three new MUST requirements, one per requirement group so each has one implementing task: CORE-R041 (`HandleAllocator`, `HandleMap`; M1-T013), CORE-R057 (`StateHasher`; M1-T014), CORE-R076 (`DetHashMap`, `DetHashSet`; M1-T014). IDs written into the `requirements` frontmatter of M1-T013 and M1-T014 and into the §6 assignment table; §10 row ranges updated. |
| 3 | CORE-R060 reworded and the §10 `FixedState` row changed: recorded literals only for `Hasher` call sequences the test makes on a `DetHasher` directly; `hash_one(&v)` is compared in-process only and its value is not part of the contract across Rust releases. The first option of the task item was taken, without the "integer types" variant: that `u64::hash` calls `write_u64` exactly once is std behaviour, not a documented guarantee. |
| 4 | §9: the two "Not scheduled" rows (`HandleAllocator`, `DetHashMap`) dropped. A paragraph under the table and open question 6 say that the task preparing the `stable` promotion adds them. No such task exists, so no task could be named. |
| 5 | M1-T047 (`todo`): `TK-Rnnn` → `TKIT-Rnnn`, `LIM-TK-NNNN` → `LIM-TKIT-NNNN`. Spec §12 item 4 marked resolved (kept in place, because other text refers to open questions by number). M1-T027: see "Declined". |

Declined (one half of item 5): the M1-T027 task file is not edited. M1-T027 is no longer `todo`:
on its pushed branch `task/M1-T027-write-the-spec-for-liminal-telemetry` (`a7accd2`, not merged
to `main` when this task ran) it is `review`, its spec uses `req_prefix: TELE` and
`LIM-TELE-NNNN`, and the task file carries a scope note explaining the difference from its
acceptance-criteria wording. The item's condition ("if they are still `todo`") is not met, and
an edit of the same lines here would conflict with that branch for no gain.

Not done, outside the scope (five named types) and left for the reviewer to judge: `ErrorContext`,
`Tick` and `TickDuration` derive `Default`, and no requirement states their default value
(empty; count 0). The derived values are the obvious ones, unlike `HandleAllocator`.

Verified: `cargo xtask ci` on macOS 27.2: pass, 11 of 11 steps, none skipped. Windows and Linux: not run locally;
branch CI result not checked by this session.

Traps: `docs/STATUS.md` is regenerated in this branch; on merge regenerate it, do not resolve by
side. The M1-T027 branch predates the `liminal_core` spec on `main`; its merge is unaffected by
this change (it does not touch `docs/specs/liminal_core.md`).

Next step: review (REVIEW.md §1 and §2, spec change) by a session that did not write this task.

## Review

2026-11-10, review sub-agent of parent session `2026-11-10-session-m1-reviews.md`, fresh context
(journal `2026-11-10-M1-T055-review.md`). This agent wrote no part of M1-T055. The author was an
implementation sub-agent of the same parent; ADR-0029 Decision 1 allows that (see "Session
boundaries"). macOS 27.2 (arm64), Rust 1.99.1, worktree `liminal-worktrees/M1-T055-review`,
branch at `6328305` before the review commits. Rules read from `main` at `047574b` (ADR-0029
wording of `AGENTS.md` and `REVIEW.md`); the branch itself predates them.

Outcome: **done**, no blocking finding. Backlog entries: 2 (`docs/BACKLOG.md`, heading
"M1-T055 — review 2026-11-10"). Follow-up task: **M1-T059** (contract-changing, 2 items), added
to `depends_on` of M1-T014.

Checked (REVIEW.md §1; the changed spec text also against §2):
1. Acceptance criteria, each item read in the diff `b3feded..6328305`:
   - Item 1: §5.2 `HandleAllocator` derives `Clone, Debug` only and has a hand-written
     `impl Default`. M1-T013 became `done` on `main` after the author's session, so I did the
     code check the task context asks for: `crates/liminal_core/src/handle/allocator.rs` and
     `map.rs` implement `Default` by hand as `new()`; `tests/handles.rs` compares the `Debug`
     form of `HandleAllocator::default()` with `with_slot_limit(u32::MAX)`. Code and spec v2
     agree.
   - Item 2: CORE-R041, CORE-R057, CORE-R076 exist, are MUST, are in the §6 table, the §10
     ranges and the `requirements` of M1-T013 / M1-T014. `0v1` agrees with CORE-R033; the
     `impl` headers in §5.2 to §5.4 carry no bounds, as the requirements say. See follow-up
     items 1 and 2.
   - Item 3: CORE-R060 and the §10 row allow recorded literals only for `Hasher` call sequences
     made on a `DetHasher`; `hash_one` is compared in-process. No other requirement relies on a
     `hash_one` value across processes (CORE-R073 makes iteration order independent of hash
     values; CORE-R130 to R133 hash through `StateHasher`).
   - Item 4: no "Not scheduled" row is left; the paragraph under the §9 table and open question
     6 name who adds the rows. M1-T016's scope ("hasher and RNG") matches the claim.
   - Item 5: M1-T047 (`todo`) says `TKIT-Rnnn` / `LIM-TKIT-NNNN`; no `TK`/`TEL` area is left
     outside M1-T027's criteria text, M1-T012's history and this task. The declined half is
     justified: M1-T027 is `done` on `main` with `req_prefix: TELE`, `LIM-TELE-0001` and a scope
     note, as the handoff says. §12 item 4 is marked resolved.
   - `spec_version` 2 with a history row. `cargo xtask ci`: below.
2. `cargo xtask ci` on `6328305`: pass, 11 steps, none skipped. On the merge of `main`
   (`047574b`, which brings `crates/liminal_core`) with the review edits: pass, 11 steps, none
   skipped. Branch CI run 42860367806 on `6328305`: success.
3. Tests: the task adds none and needs none (documentation only). The new requirements are
   testable as written, except the point in follow-up item 2.
4. No fake progress: no scope cut beyond the declined half of item 5, which is recorded with
   its reason; dropping the two §9 rows is an option the task names. No oracle changed.
5. Spec agreement: the only code for this spec is M1-T013's; checked for item 1 and CORE-R041
   (behaviour agrees).
6. Unsafe, 7. determinism, 9. architecture: no code, no dependency change.
8. Docs: `docs/STATUS.md` regenerated on the merge. No stale reference found.
10. Session boundaries: below. 11. Public repository: no workflow change, no issue or pull
    request acted on.

Findings (none is B1 to B5):
- Follow-up task M1-T059 item 1: CORE-R041 has no `Verifies:` test and no pending task that
  writes one. M1-T055 assigned it to M1-T013 while that task was `todo` (allowed by item 2);
  M1-T013 was implemented in parallel against spec v1 and is `done`. Its code meets CORE-R041
  and its tests cover part of it under CORE-R037 / CORE-R039, so this is not B5. Without the
  tag, the trace gate of M1-T014's `usable` evidence fails.
- Follow-up task M1-T059 item 2: CORE-R076 says "same state as `new()`" and "`len()` 0" also
  for `K` / `T` without `Hash`, but `new`, `len` and `iter` exist only for `K: Hash + Eq`
  (§5.4). For such types only "`Default::default()` compiles" can be observed.
- Backlog: `ErrorContext`, `Tick`, `TickDuration` derive `Default` with no requirement for the
  value (the point the handoff left to the reviewer); `last_reviewed` has no defined meaning.

Session boundaries (item 10), stated as facts:
- The author's journal names its parent `2026-11-10-session-m1-reviews.md`. That entry (on
  branch `session/2026-11-10-m1-reviews`, "in progress") lists M1-T055 among the tasks its
  implementation sub-agents wrote; its sub-agent table is not filled in yet, so "listed in its
  parent's journal" can be checked fully only when the parent completes it.
- This review has the same parent. My prompt was the ADR-0026 review template verbatim, with
  nothing added. The worktree's `AGENTS.md` (ADR-0026 Decision 5.1) forbids a review by a
  sibling; `main` carries ADR-0029, which allows it, and the text on `main` is in force. I read
  that from the repository, not from a message.
- ADR-0029 was written and merged by this parent session on an owner instruction that the ADR
  quotes. I cannot check the quote. If the owner did not give it, this review is void under
  ADR-0026 Decision 5.1 and the task needs a review by another session.
- M1-T055 had one implementation sub-agent and this one review sub-agent. Neither ran an
  install; I spawned no agent.
