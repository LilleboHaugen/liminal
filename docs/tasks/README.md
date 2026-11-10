# Tasks

A task is one unit of work, sized so a single agent session can usually finish it. Not finishing is
normal: leave a precise handoff note instead of rushing (AGENTS.md rule 2).

- File name: `<ID>-<kebab-slug>.md`, ID `M<milestone>-T<NNN>`, e.g. `M1-T014-archetype-moves.md`.
- Template: `_TEMPLATE.md`. Frontmatter fields: `docs/process/DOCUMENTATION.md`.
- Status flow: `todo → in-progress → review → done`, with `blocked` when waiting on a decision or
  another task. A task is set to `done` only by a reviewing session (`docs/process/REVIEW.md` §1).
- Tasks are never deleted. Obsolete tasks are set to `done` with a note "Obsolete: <reason>".
- Detailed task files exist only for the current and next milestone. Later work lives in
  `docs/ROADMAP.md` until that milestone's kickoff.

## Current task files

| ID | Title |
|---|---|
| [M0-T001](M0-T001-workspace-bootstrap.md) | Bootstrap the Cargo workspace |
| [M0-T002](M0-T002-xtask-ci-command.md) | xtask `ci` command |
| [M0-T003](M0-T003-check-docs.md) | Frontmatter parser and `check-docs` |
| [M0-T004](M0-T004-check-arch.md) | `check-arch` |
| [M0-T005](M0-T005-trace-and-hygiene.md) | `trace` and `hygiene` |
| [M0-T006](M0-T006-coverage-floors.md) | Coverage with per-crate floors |
| [M0-T007](M0-T007-github-actions-ci.md) | GitHub Actions CI |
| [M0-T008](M0-T008-status-generator.md) | `status` generator |
| [M0-T009](M0-T009-deny-and-doctor.md) | `cargo deny` policy and `doctor` |
| [M0-T010](M0-T010-m0-exit-review.md) | M0 exit review |
| [M0-T011](M0-T011-xtask-binary-tests.md) | Real-binary integration tests for every xtask command |
| [M1-T001](M1-T001-m1-kickoff.md) | M1 kickoff |
| [M1-T002](M1-T002-check-docs-follow-ups.md) | `check-docs` follow-ups from the M0-T003 review |
| [M1-T003](M1-T003-check-arch-follow-ups.md) | `check-arch` follow-ups from the M0-T004 review |
| [M1-T004](M1-T004-trace-hygiene-follow-ups.md) | `trace` and `hygiene` follow-ups from the M0-T005 review |
| [M1-T005](M1-T005-coverage-follow-ups.md) | `coverage` follow-ups from the M0-T006 review |
| [M1-T006](M1-T006-ci-workflow-follow-ups.md) | CI workflow follow-ups from the M0-T007 review |
| [M1-T007](M1-T007-status-follow-ups.md) | `status` generator follow-ups from the M0-T008 review |
| [M1-T008](M1-T008-deny-doctor-follow-ups.md) | `deny` policy and `doctor` follow-ups from the M0-T009 review |
| [M1-T009](M1-T009-deny-policy-test-gaps.md) | `deny` policy test gaps from review 2 of M0-T009 |
| [M1-T010](M1-T010-binary-test-follow-ups.md) | Real-binary test and stale-root follow-ups from the review of M0-T011 |
| [M1-T011](M1-T011-check-arch-determinism-lint-configuration-for-de.md) | check-arch: determinism lint configuration for deterministic crates |
| [M1-T012](M1-T012-write-the-spec-for-liminal-core.md) | Write the spec for `liminal_core` |
| [M1-T013](M1-T013-implement-liminal-core-error-model-and-generatio.md) | Implement `liminal_core`: error model and generational handles |
| [M1-T014](M1-T014-implement-liminal-core-hashing-rng-collections-t.md) | Implement `liminal_core`: hashing, RNG, collections, time; reach `usable` |
| [M1-T015](M1-T015-write-the-spec-for-liminal-benches.md) | Write the spec for `liminal_benches` |
| [M1-T016](M1-T016-implement-the-liminal-benches-harness.md) | Implement the `liminal_benches` harness |
| [M1-T017](M1-T017-cargo-xtask-bench-and-the-first-wall-clock-basel.md) | `cargo xtask bench` and the first wall-clock baselines |
| [M1-T018](M1-T018-miri-and-loom-in-ci.md) | Miri and loom in CI |
| [M1-T019](M1-T019-write-the-spec-for-liminal-math.md) | Write the spec for `liminal_math` |
| [M1-T020](M1-T020-implement-liminal-math-scalar-vector-matrix-and.md) | Implement `liminal_math`: scalar vector, matrix and quaternion types |
| [M1-T021](M1-T021-implement-liminal-mathdet.md) | Implement `liminal_math::det` |
| [M1-T022](M1-T022-implement-liminal-math-simd-paths.md) | Implement `liminal_math` SIMD paths |
| [M1-T023](M1-T023-implement-liminal-math-geometry-reach-usable.md) | Implement `liminal_math` geometry; reach `usable` |
| [M1-T024](M1-T024-write-the-spec-for-liminal-memory.md) | Write the spec for `liminal_memory` |
| [M1-T025](M1-T025-implement-liminal-memory-arena-and-frame-allocat.md) | Implement `liminal_memory`: arena and frame allocators |
| [M1-T026](M1-T026-implement-liminal-memory-pool-aligned-tracking-r.md) | Implement `liminal_memory`: pool, aligned, tracking; reach `usable` |
| [M1-T027](M1-T027-write-the-spec-for-liminal-telemetry.md) | Write the spec for `liminal_telemetry` |
| [M1-T028](M1-T028-implement-liminal-telemetry-logs-spans-counters.md) | Implement `liminal_telemetry`: logs, spans, counters, per-thread buffers |
| [M1-T029](M1-T029-implement-liminal-telemetry-frame-stats-and-expo.md) | Implement `liminal_telemetry`: frame stats and export; reach `usable` |
| [M1-T030](M1-T030-write-the-spec-for-liminal-reflect.md) | Write the spec for `liminal_reflect` |
| [M1-T031](M1-T031-implement-liminal-reflect-type-info-stable-ids-f.md) | Implement `liminal_reflect`: type info, stable IDs, field access |
| [M1-T032](M1-T032-implement-the-reflect-derive-macro-reach-usable.md) | Implement the `Reflect` derive macro; reach `usable` for both reflect crates |
| [M1-T033](M1-T033-instruction-count-regression-gate-in-ci.md) | Instruction-count regression gate in CI |
| [M1-T034](M1-T034-write-the-spec-for-liminal-jobs.md) | Write the spec for `liminal_jobs` |
| [M1-T035](M1-T035-implement-liminal-jobs-pool-work-stealing-deques.md) | Implement `liminal_jobs`: pool, work-stealing deques, scoped tasks |
| [M1-T036](M1-T036-implement-liminal-jobs-parallel-for-and-job-grap.md) | Implement `liminal_jobs`: parallel-for and job graphs; reach `usable` |
| [M1-T037](M1-T037-write-the-spec-for-liminal-ecs.md) | Write the spec for `liminal_ecs` |
| [M1-T038](M1-T038-implement-liminal-ecs-entities-archetype-storage.md) | Implement `liminal_ecs`: entities, archetype storage, reference world |
| [M1-T039](M1-T039-implement-liminal-ecs-queries-and-archetype-move.md) | Implement `liminal_ecs`: queries and archetype moves |
| [M1-T040](M1-T040-implement-liminal-ecs-resources-events-commands.md) | Implement `liminal_ecs`: resources, events, commands, change detection |
| [M1-T041](M1-T041-implement-liminal-ecs-systems-schedules-single-t.md) | Implement `liminal_ecs`: systems, schedules, single-threaded executor |
| [M1-T042](M1-T042-implement-liminal-ecs-parallel-executor.md) | Implement `liminal_ecs`: parallel executor |
| [M1-T043](M1-T043-implement-worldstate-hash-ecs-benchmarks-reach-u.md) | Implement `World::state_hash`, ECS benchmarks; reach `usable` |
| [M1-T044](M1-T044-write-the-spec-for-liminal-app.md) | Write the spec for `liminal_app` |
| [M1-T045](M1-T045-implement-liminal-app-builder-modules-schedules.md) | Implement `liminal_app`: builder, modules, schedules |
| [M1-T046](M1-T046-implement-liminal-app-fixed-step-loop-and-headle.md) | Implement `liminal_app`: fixed-step loop and headless runner; reach `usable` |
| [M1-T047](M1-T047-write-the-spec-for-liminal-testkit.md) | Write the spec for `liminal_testkit` |
| [M1-T048](M1-T048-implement-liminal-testkit-harness-and-assert-det.md) | Implement `liminal_testkit`: harness and `assert_deterministic` |
| [M1-T049](M1-T049-implement-liminal-testkit-replay-reach-usable.md) | Implement `liminal_testkit` replay; reach `usable` |
| [M1-T050](M1-T050-headless-boids-example-100k-deterministic-agents.md) | `headless_boids` example: 100k deterministic agents |
| [M1-T051](M1-T051-competitive-benchmarks-workspace-math-vs-glam-jo.md) | Competitive benchmarks: workspace, math vs `glam`, jobs vs `rayon` |
| [M1-T052](M1-T052-competitive-benchmarks-ecs-vs-bevy-ecs-and-hecs.md) | Competitive benchmarks: ECS vs `bevy_ecs` and `hecs` |
| [M1-T053](M1-T053-m1-exit-review.md) | M1 exit review |
| [M1-T054](M1-T054-m1-kickoff-follow-ups.md) | M1 kickoff follow-ups from the M1-T001 review |
| [M2-T001](M2-T001-check-docs-markdown-follow-ups.md) | `check-docs` Markdown follow-ups from the M1-T002 review |
| [M2-T002](M2-T002-workspace-member-path-follow-ups.md) | Workspace member path follow-ups from the M1-T005 review |
| [M2-T003](M2-T003-status-requirement-tags.md) | `status` write-if-changed requirement tag follow-up from the M1-T007 review |
| [M2-T004](M2-T004-reference-module-rule-follow-ups.md) | Reference-module rule follow-ups from the M1-T054 review |
