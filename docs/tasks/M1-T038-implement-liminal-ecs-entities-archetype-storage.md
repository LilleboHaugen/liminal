---
id: M1-T038
title: Implement liminal_ecs — entities, archetype storage, reference world
milestone: M1
status: todo
crates: [liminal_ecs]
depends_on: [M1-T037, M1-T032]
requirements: [ECS-R001, ECS-R002, ECS-R003, ECS-R004, ECS-R005, ECS-R006, ECS-R007, ECS-R008, ECS-R009, ECS-R020, ECS-R021, ECS-R022, ECS-R023, ECS-R200, ECS-R201, ECS-R230]
---

# M1-T038: Implement `liminal_ecs`: entities, archetype storage, reference world

## Goal

Entities can be spawned and despawned with components stored in archetype tables, and a naive reference world exists for differential tests.

## Context

- `docs/specs/liminal_ecs.md`; ADR-0008.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Crate, entity allocator, component registration, archetype tables, spawn/despawn/get, the reference world, differential tests.

**Out:** Queries and archetype moves: next task.

## Acceptance criteria

- [ ] The crate exists at the path in `architecture.toml`, is a workspace member, has
      `#![forbid(unsafe_code)]` if `unsafe = "forbid"`, and has a floor in `ci/coverage.toml`.
- [ ] The crate's clippy determinism configuration is in place.
- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.
- [ ] Differential property test: random spawn/despawn/get sequences give the same observable state in the archetype world and the reference world.

## Test requirements

Requirements in frontmatter; differential property tests; Miri if `unsafe` is used.

## Notes

Requirements written by M1-T037 (spec §6 "Assignment"). Points set by the spec: (1) `Entity` stores `RawHandle::to_bits()` as one `u64` and is reflected as `Primitive(U64)` (ECS-R004); entity IDs come from `liminal_core::handle::HandleAllocator` unchanged (ECS-R002). (2) This task implements all of `EcsError` (ECS-R230), including variants whose producers come later. (3) Components must be registered before use (`World::register_component`, LIM-ECS-0003); `World::with_entity_limit` exists so that LIM-ECS-0002 is testable. (4) `Mut` exists from this task without tick stamping; ticks come with M1-T040. (5) The reference world is the public module `liminal_ecs::reference` (spec §5.8); the differential test is `tests/world_differential.rs` (ECS-R201). (6) `unsafe` only in the modules of spec §3; the storage module and `reference` forbid it (typed `Vec<T>` columns). (7) This task writes the crate's `clippy.toml` (CODE_STYLE.md, XTASK-R039).

## Handoff

_Empty._

## Review

_Empty._
