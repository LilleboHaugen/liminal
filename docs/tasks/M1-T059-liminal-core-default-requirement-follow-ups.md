---
id: M1-T059
title: liminal_core Default requirement follow-ups from the M1-T055 review
milestone: M1
status: in-progress
crates: [liminal_core]
depends_on: [M1-T055, M1-T013]
requirements: [CORE-R041]
---

# M1-T059: `liminal_core` `Default` requirement follow-ups from the M1-T055 review

## Goal

CORE-R041 has a `Verifies:` test, and CORE-R076 states only what a test can observe, before
M1-T014 implements the rest of `liminal_core` and collects the `usable` evidence.

## Context

- Follow-up task under ADR-0021 / ADR-0029 Decision 2, written by the review of M1-T055 (see that
  task's "Review"). Both items are contract-changing: M1-T014 (`todo`) needs every MUST of the
  spec traced (its acceptance criterion on the `usable` gates) and implements CORE-R076.
- `docs/specs/liminal_core.md` v2: CORE-R041, CORE-R076, §5.4, §6 assignment table.
- M1-T055 added CORE-R041 (`HandleAllocator::default()` and `HandleMap::<T>::default()` equal
  `new()`) and assigned it to M1-T013 while that task was `todo`. M1-T013 was implemented in
  parallel against spec v1 and is `done`. Its code meets CORE-R041
  (`crates/liminal_core/src/handle/allocator.rs`, `map.rs`: hand-written `Default` calling
  `new()`), but no test carries `/// Verifies: CORE-R041`, and no pending task writes one.
  `cargo xtask trace --crate liminal_core` lists CORE-R041 as untested (information only while
  the crate is `planned`; a gate failure at `usable`).

## Scope

**In:**
1. Tests with `/// Verifies: CORE-R041` in `crates/liminal_core/tests/handles.rs`, one clause of
   the requirement each: for `HandleAllocator::default()` and `HandleMap::<T>::default()`,
   `live_count()` / `len()` is 0, the first `allocate` / `insert` returns `Ok` with `0v1`, the
   `Debug` output equals that of `new()`, and `HandleMap::<T>::default()` compiles for a `T`
   that implements neither `Default` nor `Debug`. Existing assertions under CORE-R037
   (`slot_limit_bounds_the_indices_and_exhaustion_changes_nothing`) and CORE-R039
   (`map_stores_one_value_per_live_handle`) cover part of this; they stay. No library code
   change is expected; if a test fails, fix the code.
2. Assignment: spec §6 table names this task for CORE-R041 (M1-T013: CORE-R030 to CORE-R040).
   Remove CORE-R041 from the `requirements` frontmatter of M1-T013, which M1-T055 added after
   M1-T013's implementation, and say so in a dated line of M1-T013's handoff.
3. CORE-R076 says `DetHashMap::<K, V>::default()` and `DetHashSet::<T>::default()` are "in the
   same state as `new()`" with "`len()` 0" for every `K`, `V`, `T`, "including types that
   implement neither `Default` nor `Hash`". In §5.4 `new`, `len`, `iter` and every other method
   exist only for `K: Hash + Eq` / `T: Hash + Eq`, so for a type without `Hash` nothing but
   "`Default::default()` compiles" can be observed. Reword the requirement into the two cases
   (with `Hash + Eq`: the stated equalities; without: the impl exists, checked by a test that
   compiles), or move `new`, `len` and `is_empty` into an `impl` block without bounds in §5.4
   and say which was chosen.
4. Items 2 and 3 bump `spec_version` with a history row.

**Out:** Any other change to the `liminal_core` design. Code for `hash`, `collections`, `rng`,
`time` (M1-T014).

## Acceptance criteria

- [ ] `cargo xtask trace --crate liminal_core` lists at least one test for CORE-R041, and
      breaking `Default` for either type (for example `with_slot_limit(0)`) makes a test with
      that tag fail.
- [ ] Spec §6 and the `requirements` frontmatter of M1-T013, M1-T014 and this task assign every
      CORE requirement to exactly one task.
- [ ] CORE-R076 names, for each type-bound case, a behaviour a test can observe.
- [ ] `spec_version` is bumped with a history row.
- [ ] `cargo xtask ci` passes with no failed or skipped step.

## Test requirements

`Verifies: CORE-R041` tests as in item 1. CORE-R076 is tested by M1-T014.

## Handoff

_Empty._

## Review

_Empty._
