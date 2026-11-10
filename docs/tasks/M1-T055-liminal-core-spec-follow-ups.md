---
id: M1-T055
title: liminal_core spec follow-ups from the M1-T012 review
milestone: M1
status: review
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

_Empty._
