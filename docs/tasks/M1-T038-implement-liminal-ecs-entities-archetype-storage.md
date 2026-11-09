---
id: M1-T038
title: Implement liminal_ecs — entities, archetype storage, reference world
milestone: M1
status: todo
crates: [liminal_ecs]
depends_on: [M1-T037, M1-T032]
requirements: []
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

_None._

## Handoff

_Empty._

## Review

_Empty._
