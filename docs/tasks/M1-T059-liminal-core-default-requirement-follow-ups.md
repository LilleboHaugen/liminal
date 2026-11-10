---
id: M1-T059
title: liminal_core Default requirement follow-ups from the M1-T055 review
milestone: M1
status: done
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

- [x] `cargo xtask trace --crate liminal_core` lists at least one test for CORE-R041, and
      breaking `Default` for either type (for example `with_slot_limit(0)`) makes a test with
      that tag fail.
- [x] Spec §6 and the `requirements` frontmatter of M1-T013, M1-T014 and this task assign every
      CORE requirement to exactly one task.
- [x] CORE-R076 names, for each type-bound case, a behaviour a test can observe.
- [x] `spec_version` is bumped with a history row.
- [x] `cargo xtask ci` passes with no failed or skipped step.

## Test requirements

`Verifies: CORE-R041` tests as in item 1. CORE-R076 is tested by M1-T014.

## Handoff

**2026-11-10, implementation sub-agent (journal `2026-11-10-M1-T059.md`). Status: `review`.**

Done:
- Item 1: five tests with `/// Verifies: CORE-R041` in `crates/liminal_core/tests/handles.rs`:
  `default_allocator_is_in_the_state_of_new` (`live_count()` 0, `Debug` equal to `new()` and to
  `with_slot_limit(u32::MAX)`, different from `with_slot_limit(0)`),
  `default_allocator_issues_0v1_first`, `default_map_is_in_the_state_of_new`,
  `default_map_inserts_at_0v1_first`,
  `default_map_exists_for_a_value_type_without_default_or_debug` (`HandleMap<Opaque>` and
  `HandleMap<Large>`; neither type implements `Default` or `Debug`). No library code changed.
  The older assertions under CORE-R037 and CORE-R039 stay.
- Mutation check (acceptance criterion 1), both reverted: `HandleAllocator::default()` =
  `with_slot_limit(0)` fails 2 of the tagged tests (and the CORE-R037 test);
  `HandleMap::default()` = `with_slot_limit(0)` fails 3 of the tagged tests (and the CORE-R039
  test).
- Item 2: spec §6 table: M1-T013 has CORE-R030 to CORE-R040, new row M1-T059 with CORE-R041.
  CORE-R041 removed from M1-T013's `requirements`; dated line at the end of M1-T013's handoff.
- Item 3, **second option chosen**: in spec §5.4 `new`, `len` and `is_empty` of `DetHashMap` and
  `DetHashSet` are in `impl` blocks without trait bounds. Reason: the unbounded
  `impl Default { /* new() */ }` of §5.4 can only call `new()` if `new()` is unbounded too, and
  "`len()` 0" then holds and is testable for every type. CORE-R076 now has case (a) every `K`,
  `V`, `T` (`default()` and `new()` compile, `len()` 0, `is_empty()` true) and case (b)
  `Hash + Eq` types (iteration yields nothing; same results after the same operations). §10
  names the test for each case. `with_capacity`, `iter`, `clear` and the rest keep their bounds
  (not in the item's list).
- Item 4: `spec_version` 2 → 3 with a history row.

Check of acceptance criterion 2 (every CORE requirement in exactly one task): the union of the
`requirements` frontmatter of M1-T013 (R001–R013, R030–R040), M1-T014 (R050–R057, R060, R061,
R070–R076, R090–R098, R110–R114, R130–R133) and this task (R041) equals the ID column of spec
§6, with no ID twice; compared by script, see the journal.

Not done: nothing in scope. CORE-R076 has no test yet (M1-T014, as the task says).

Traps:
- M1-T014 must put `new`, `len`, `is_empty` in unbounded `impl` blocks; a single
  `impl<K: Hash + Eq, V>` block as in spec v2 no longer matches the spec.
- `Result<Handle<T>, CoreError>` is not `Copy`: bind the unwrapped handle before comparing twice.

Next step: a session reviews M1-T059. Then M1-T014 can start.

## Review

2026-11-10, review sub-agent of parent session `2026-11-10-session-m1-reviews.md`, fresh context
(journal `2026-11-10-M1-T059-review.md`). First review of this task. Rules read from `main` at
`5e0a1ea` (`AGENTS.md`, `docs/process/REVIEW.md` §1, ADR-0029). **Outcome: no blocking finding;
`done`.**

Checked:
- Criterion 1: `cargo xtask trace --crate liminal_core` lists CORE-R041 with
  `crates/liminal_core/tests/handles.rs`. Mutations run by the reviewer, both reverted:
  `HandleAllocator::default()` = `with_slot_limit(0)` fails 2 tagged tests
  (`default_allocator_is_in_the_state_of_new`, `default_allocator_issues_0v1_first`) and the
  CORE-R037 test; `HandleMap::default()` = `with_slot_limit(0)` fails 4 tagged tests (the
  handoff says 3; `default_map_exists_for_a_value_type_without_default_or_debug` fails too)
  and the CORE-R039 test.
- Criterion 2: reviewer's own script over spec §6 and the `requirements` frontmatter of every
  task file: 60 IDs in the spec's requirement tables, 60 in §6, 60 in the frontmatter of
  M1-T013 (24), M1-T014 (35), M1-T059 (1); the same set, no ID twice.
- Criterion 3: CORE-R076 case (a) (`default()`, `new()` compile; `len()` 0; `is_empty()`) is
  observable for every type because §5.4 now has `new`, `len`, `is_empty` in `impl` blocks
  without bounds; case (b) uses only methods bounded by `Hash + Eq`. §5.4, the paragraph below
  it, CORE-R076 and the §10 row agree. No other spec text states bounds for these methods.
  M1-T014's task file names no `impl` layout, so it needs no edit.
- Criterion 4: `spec_version` 3, history row present, the two sentences in §9 and §10 say 3.
- Criterion 5: `cargo xtask ci` on `2366b2a`: pass, 11 steps, none skipped; again on the merge
  of `main` (`5e0a1ea`) with the review edits: pass, 11 steps. macOS 27.2 (arm64). Branch CI
  run 42876422340 on `2366b2a`: success.
- Tests: each of the five tests asserts values named by CORE-R041 (count 0, `0v1`, `Debug`
  equal to `new()` and to `with_slot_limit(u32::MAX)`, the impl for `Opaque` / `Large`). No
  library code, oracle, floor or baseline changed; no test weakened or removed. No `unsafe`,
  no dependency, no workflow file touched. M1-T013: only the frontmatter list and one dated
  handoff line changed.
- Session boundaries: author is an implementation sub-agent of the same parent, reviewer a
  fresh review sub-agent (ADR-0029 Decision 1.2). The parent's journal entry is "in progress"
  and its sub-agent table is not written yet, so the listing of author and reviewer there
  could not be checked.

Not verified: Windows and Linux locally (the branch CI run covers them); the CI run of the
review commits was not waited for.

Backlog: 1 entry in `docs/BACKLOG.md` under "M1-T059 — review 2026-11-10" (the doc comment of
the CORE-R040 test now sits on the first CORE-R041 test). Follow-up task: none.
