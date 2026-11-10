---
id: M1-T037
title: Write the spec for liminal_ecs
milestone: M1
status: review
crates: [liminal_ecs]
depends_on: [M1-T030, M1-T034]
requirements: []
---

# M1-T037: Write the spec for `liminal_ecs`

## Goal

`docs/specs/liminal_ecs.md` is a complete, reviewable contract for `liminal_ecs`, so the implementation tasks can be done without guessing.

## Context

- `docs/ARCHITECTURE.md` §2 (module map: archetype ECS: entities, components, resources, events, queries, commands, change detection, schedules with parallel execution).
- `docs/specs/_TEMPLATE.md`; `docs/specs/xtask.md` as the quality bar.
- `architecture.toml` entry for `liminal_ecs` (layer, `unsafe`, `deterministic`, allowed deps).
- ADR-0008 (archetype ECS); ADR-0009; `docs/specs/liminal_reflect.md`, `docs/specs/liminal_jobs.md`.
- PERFORMANCE.md competitive workloads (they shape the budgets section).
- PERFORMANCE.md "Reference implementations reachable from benchmarks" (M1-T054): the spec's API section lists the crate's `reference` items for each optimized path.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Spec for: entities (generational, from `liminal_core`), component registration via `liminal_reflect`, archetype tables and archetype moves, queries and filters, resources, events, deferred commands applied in a defined order, change detection, systems with declared access, schedules, conflict detection, single-threaded and parallel executors (on `liminal_jobs`), `World::state_hash()` (ADR-0009 item 6), and the naive reference world (for example a map from entity to component set) that differential tests compare against.

**Out:** Code. The implementation tasks (M1-T038, M1-T039, M1-T040, M1-T041, M1-T042, M1-T043) write it.

## Acceptance criteria

- [x] `docs/specs/liminal_ecs.md` exists, follows `docs/specs/_TEMPLATE.md` (all sections), with
      `maturity: planned`, `spec_version: 1` and `adrs` listing every ADR it implements.
- [x] Every requirement has an ID `ECS-Rnnn`, an RFC 2119 level and a testable statement with
      units, ranges and failure behaviour. No vague words ("fast", "robust", "efficient").
- [x] §5 Public API covers the scope below and is consistent with `docs/ARCHITECTURE.md` and the
      specs of the crates it depends on.
- [x] Error codes `LIM-ECS-NNNN` are listed with message and `help` (ADR-0017), or the spec says
      why the crate has no recoverable errors.
- [x] The test plan names the test kinds that apply (TESTING.md) and, for every optimized path, the
      reference implementation it is checked against (AGENTS.md rule 4).
- [x] Determinism requirements follow ADR-0024: the spec names the workloads of
      `tests/determinism.rs` and says whether cross-platform bit-identity is claimed for each.
- [x] Performance budgets name scenarios and machines; numbers that need a measurement say which
      task measures them (PERFORMANCE.md: budgets are never invented).
- [x] Every MUST requirement is assigned to exactly one implementation task (M1-T038, M1-T039, M1-T040, M1-T041, M1-T042, M1-T043): the spec
      lists the assignment, and this session writes the IDs into those tasks' `requirements`
      frontmatter (`check-docs` R028 validates them).
- [x] Open questions are listed in the spec instead of being decided silently.
- [x] `docs/specs/README.md` lists the spec (XTASK-R027); `cargo xtask ci` passes.

## Test requirements

None (documentation task). The review is a spec review (REVIEW.md §2).

## Notes

Requirement IDs leave gaps between groups (template §6). Keep the API small: every public item has to be documented, tested and later snapshotted.

## Handoff

2026-11-10, implementation sub-agent (parent: `docs/journal/2026-11-10-session-m1-reviews.md`).

**Done**
- `docs/specs/liminal_ecs.md`, `spec_version` 1, `maturity: planned`, prefix `ECS`, all 14
  template sections: 81 requirements (78 MUST, 3 SHOULD), 22 error codes (`LIM-ECS-0001` …
  `0041`), 10 invariants, 2 determinism workloads (both claim cross-platform bit-identity, not
  verified yet), 8 benchmark scenarios without numbers (measured by M1-T043, compared by
  M1-T052), 20 open questions and 6 answers to questions other specs addressed to this one.
- Every MUST is assigned to exactly one of M1-T038 … M1-T043 (spec §6 "Assignment"; checked by a
  script over the requirement tables: no MUST unassigned, none assigned twice). The IDs are in
  those tasks' `requirements` frontmatter and each task's Notes list what the spec sets for it.
- `docs/specs/README.md` index row; `docs/STATUS.md` regenerated.
- New task `M1-T061` (`Reflect` impls for `liminal_math` and `liminal_core` types), with its row
  in `docs/tasks/README.md`; `M1-T050` and `M1-T053` list it in `depends_on`.

**Design decisions in the spec (for the spec review to accept or reject)**
1. `Entity` uses `liminal_core::handle::HandleAllocator` unchanged (resolves the core spec's
   open question 5). It stores the handle bits as one `u64` and is reflected as
   `Primitive(U64)` under its own type path, so components with `Entity` fields can derive
   `Reflect` without a change to the reflect spec (ECS-R004, §12 item 2).
2. Every component and event type is `Typed` and must be registered; resources need neither,
   and only registered resources are hashed.
3. Columns are typed `Vec<T>`; `unsafe` is limited to four modules (§3). Byte-blob storage is an
   open question that needs a benchmark (§12 item 11).
4. Query iteration order is normative: archetype number, then row (append on entry, swap-remove
   on exit).
5. Conflicts are decided on component types plus `With`/`Without` disjointness, for parameters
   of one system and between systems (§4 "Conflict").
6. A schedule has named stages; commands are applied at the end of each stage in ascending
   `SystemId` index. `CommandBuffer::spawn` returns no `Entity` (§12 item 4).
7. Change ticks are `u64`, assigned from the single-threaded execution order also in
   `run_parallel`, so `Changed`/`Added` see each write exactly once (§4 "Run tick").
8. `Schedule::run` is the oracle of `Schedule::run_parallel`; the intended parallel design is one
   `JobGraph` per stage, which leaves the crate without lock-free state of its own (§10, §12
   item 16: this touches the wording of an M1 exit criterion).
9. Rejected commands (type errors) are reported as an error after the run completed; commands
   for entities that are gone are skipped and counted (ECS-R111, ECS-R112, ECS-R173).
10. The state hash covers entities, components, registered resources and buffered events, and
    leaves out freed slots, ticks and reader positions (ECS-R190, ECS-R192, §12 item 9).
11. `liminal_reflect.md` §12 item 1 is decided as option (a) (reflect depends on math); the
    change itself is task M1-T061, because it is outside `crates: [liminal_ecs]`.

**Verified**
- macOS 27.2 arm64, Rust 1.99.1: `cargo xtask ci` passes, 11 of 11 steps, none skipped
  (`check-docs` ok; `trace` lists the 81 ECS requirements as information at `planned`).
- The signatures of spec §5.1 to §5.7 (and three methods of §5.8) compile with stub bodies as
  one scratch program together with user code: the §5.7 example verbatim, systems of several
  arities, a capturing closure system, bundles of each form, the `Entity` `Reflect` impl against
  a stub of the reflect traits. The three `compile_fail` cases of ECS-R066 and ECS-R130 fail to
  compile for the intended reason (E0277, E0277, E0499). The scratch program is not committed.

**Not verified**
- That an implementation fits under the sealed traits as specified (only signatures and call
  sites were type-checked; `liminal_reflect` and `liminal_jobs` do not exist yet, so their
  types were stubs).
- That ECS-R068 and ECS-R175 (no heap allocation) can be met on all three platforms.
- Any performance property; cross-platform bit-identity of the two workloads.
- That `Primitive(U64)` for `Entity` is accepted by the future serializer (§12 item 2).

**Traps**
- `M1-T061` is the next free task ID on this branch and on every remote branch on 2026-11-10;
  another branch of the same parent session may take the same ID before the merge. If so,
  renumber one (file name, `docs/tasks/README.md`, `depends_on` of M1-T050 and M1-T053, the
  three mentions in `docs/specs/liminal_ecs.md` and in the notes of this task).
- `docs/specs/liminal_reflect.md` §12 items 1 and 2 and `docs/specs/liminal_math.md` §12 item 7
  still read as open; M1-T061 updates them (left untouched here because M1-T057 and M1-T060
  edit those files).
- Merging: `docs/specs/README.md` and `docs/tasks/README.md` (index rows) and `docs/STATUS.md`
  (regenerate) will differ from other branches.

**Next step:** spec review of this task (REVIEW.md §2), then M1-T038 once M1-T032 is `done`.

## Review

_Empty._
