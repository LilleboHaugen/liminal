---
id: M1-T055
title: liminal_core spec follow-ups from the M1-T012 review
milestone: M1
status: todo
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

- [ ] Items 1 to 5 are done, or a declined item is written into this task with the reason.
- [ ] Spec changes bump `spec_version` with a history row.
- [ ] `cargo xtask ci` passes.

## Test requirements

None beyond the requirement(s) added by item 2, which their implementation task tests.

## Handoff

_Empty._

## Review

_Empty._
