---
id: M1-T037
title: Write the spec for liminal_ecs
milestone: M1
status: done
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
- New task `M1-T062` (`Reflect` impls for `liminal_math` and `liminal_core` types), with its row
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
    change itself is task M1-T062, because it is outside `crates: [liminal_ecs]`.

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
- `M1-T062` is the next free task ID on this branch and on every remote branch on 2026-11-10;
  another branch of the same parent session may take the same ID before the merge. If so,
  renumber one (file name, `docs/tasks/README.md`, `depends_on` of M1-T050 and M1-T053, the
  three mentions in `docs/specs/liminal_ecs.md` and in the notes of this task).
- `docs/specs/liminal_reflect.md` §12 items 1 and 2 and `docs/specs/liminal_math.md` §12 item 7
  still read as open; M1-T062 updates them (left untouched here because M1-T057 and M1-T060
  edit those files).
- Merging: `docs/specs/README.md` and `docs/tasks/README.md` (index rows) and `docs/STATUS.md`
  (regenerate) will differ from other branches.

**Next step:** spec review of this task (REVIEW.md §2), then M1-T038 once M1-T032 is `done`.

## Review

2026-11-10, review sub-agent (ADR-0026, ADR-0030) of the parent session on branch
`session/2026-11-10-m1-t014-review`, fresh context, review prompt with worktree and task ID.
This agent did no work on M1-T037. macOS 27.2 (Darwin, arm64), Rust 1.99.1, worktree
`liminal-worktrees/M1-T037-review`, reviewed commit `d36b53c`. Rules read from `origin/main`
(`AGENTS.md`, `docs/process/REVIEW.md`). Task review (REVIEW.md §1) plus spec review (§2).

Outcome: **done**, no blocking finding (B1–B5). No contract-changing follow-up, so no follow-up
task. Backlog: 6 entries in `docs/BACKLOG.md` under "M1-T037 — review 2026-11-10".

Merge work done by this review (not a change of the reviewed content):
- Task ID collision, as the handoff predicted: `main` has `M1-T061` (reflect-derive path check,
  from the M1-T057 review). This branch's `Reflect`-impls task is renumbered **M1-T062** (file
  name, `id`, title line, `docs/tasks/README.md` row, `depends_on` of M1-T050 and M1-T053, the
  two mentions in `docs/specs/liminal_ecs.md` §12 and the mentions in this task's handoff).
  M1-T062 was free on every local and remote branch when taken. The M1-T037 journal entry keeps
  the old number (it is a log).
- `origin/main` merged twice (`d873165`, then `35a8597` with M1-T014 `done` and
  `liminal_core` `usable`); conflicts in `docs/STATUS.md` (regenerated) and
  `docs/specs/README.md` (both rows kept, core `usable`).

Checked:
- AC 1: 14 template sections in template order; `maturity: planned`, `spec_version: 1`;
  `adrs` lists ADR-0008, 0009, 0010, 0016, 0017, 0018, 0023, 0024 (each is applied in the text;
  ADR-0005 and ADR-0019 are only cited).
- AC 2: 81 requirement rows, IDs unique, 78 MUST, 3 SHOULD; every `ECS-Rnnn` referenced in the
  spec is defined, every `LIM-ECS-NNNN` used is in §8 (script). Word search for "fast",
  "robust", "efficient" and similar: only "faster" (§3, a condition on a measurement) and
  "not the fast path" (§5.8). Spot checks of worked examples by hand: the §4 run-tick table
  (T = 7 gives 8 … 12, then 13 = T + 1 + 3 systems + 2 stages), ECS-R170 (orders (3,0), (2,1)
  give 2, 1, 3, 0, identical to JOBS-R085), ECS-R121 tick sequence, ECS-R102 (a, b, c gives
  b, c), ECS-R125 consequences derived from the baseline rule (each holds).
- AC 3: §5 covers every scope item. Consistent with `docs/ARCHITECTURE.md` §2,
  `architecture.toml` (layer 1, `same_layer_deps = ["liminal_jobs"]`, `external = []`, dev
  `proptest` and `loom`, `unsafe = "allow"`, `deterministic = true`) and the dependency specs
  as on `main` after the merge: `liminal_core` `spec_version` 4 (`RawHandle` bit layout and
  `Display`, `HandleAllocator::with_slot_limit`/`allocate`/`free`, CORE-R033 to R037,
  `StateHasher::new`/`write_u64`/`finish`, `state_hash_of`, `validate_error` checks);
  `liminal_reflect` `spec_version` 2 (`ReflectMut::Primitive(PrimitiveKind)`,
  `downcast_mut` by `Any` with LIM-REFL-0011 (REFL-R044), `validate_reflect`, `write_reflect`,
  `reflect_state_hash`, `TypeRegistry::register`/`get`, LIM-REFL-0001 for two types with one
  path (REFL-R062), `StableTypeId(u64)`); `liminal_jobs` (`ThreadPool::threads`,
  `PoolConfig::threads: Option<usize>`, `JobGraph::add_node(&'static str, FnMut + Send)`,
  JOBS-R085 sequential order, JOBS-R104 panic semantics, which ECS-R183 mirrors);
  `liminal_telemetry` `spec_version` 2 (`span!`/`counter!` take `&'static str`, feature
  unification as §3 describes, `Collector`, `FrameRecord`, `CounterTotal`). The signature
  type-check claimed in the handoff was not repeated.
- AC 4: 22 codes with variant, message, help and context; groups and free numbers stated;
  context keys match `^[a-z][a-z0-9_]*$` (CORE-R012).
- AC 5: test kinds per area use TESTING.md names; the four optimized paths of §5.8 name their
  reference and test file; `reference::World` contains no `unsafe` and is public (PERFORMANCE.md
  items 1 to 4).
- AC 6: two workloads, each with "bit-identity claimed, not verified yet" stated.
- AC 7: §9 has 8 scenarios on the reference machine, no numbers, measured by M1-T043 and
  M1-T052 (both exist).
- AC 8: the 78 MUST IDs equal the union of the `requirements` frontmatter of M1-T038 (16),
  M1-T039 (16), M1-T040 (13), M1-T041 (22), M1-T042 (4), M1-T043 (7), no ID twice, no SHOULD in
  them (script).
- AC 9: 20 open questions and 6 answers to other specs' questions; the reflect spec's §12
  item 1 is decided (option (a)) and item 2 is handed to M1-T062 by name.
- AC 10: `docs/specs/README.md` row present. `cargo xtask ci` on the final tree (this branch with
  `origin/main` `35a8597` merged, review edits included): pass, 12 of 12 steps, none skipped.
- Spec review §2: purpose, scope, non-goals clear; requirements testable; invariants
  assertable; open questions listed. Determinism (§11): ordered iteration table, no random
  hashers, `TypeId`/`type_name` kept out of the hash; agrees with ADR-0008 and ADR-0009.
- Session boundaries: the author is sub-agent row 23 of `2026-11-10-session-m1-reviews.md`
  (owner-started); no other session worked on the task; this review is its first.
- Public repository: no workflow change; no outside issue or pull request acted on.
