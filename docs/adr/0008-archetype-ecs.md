---
id: ADR-0008
title: Own archetype-based ECS with explicit schedules
status: accepted
date: 2026-11-08
supersedes: []
superseded_by: []
---

# ADR-0008: Own archetype-based ECS with explicit schedules

## Context

The ECS is the central data model: game state, engine state, introspection by the probe, state hashing
for determinism, and parallel execution all go through it. Its design decides iteration speed, how
easy it is to parallelize work, and how understandable game code is for agents.

## Decision

- `liminal_ecs` is our own implementation.
- **Storage:** archetype tables with one contiguous column per component type (structure of arrays).
  Entities are generational IDs (index + generation). Moving an entity between archetypes on component
  add/remove is cached through an archetype graph. A sparse-set storage option for components that change
  often MAY be added if benchmarks justify it (decided in the ECS spec).
- **Systems** are Rust functions whose parameters (queries, resources, events, commands) declare their
  data access. Access sets are computed at schedule build time.
- **Schedules** are explicit: named stages and explicitly declared ordering constraints. Systems with
  non-conflicting access run in parallel on `liminal_jobs`. Conflicting systems without an ordering
  constraint are a **schedule build error**, never resolved by guessing. This keeps execution
  deterministic (ADR-0009).
- **Structural changes** (spawn, despawn, add/remove component) during a parallel stage go through
  command buffers, applied at defined sync points in a deterministic order.
- **Change detection** through per-component change ticks.
- Components that take part in simulation state MUST implement reflection (`liminal_reflect`) so the
  probe can inspect them and `World::state_hash()` can include them.

## Alternatives considered

- **Sparse-set ECS (EnTT-style) as the primary storage:** cheap add/remove, slower multi-component
  iteration. Archetypes favour iteration, which dominates in games.
- **Implicit, inferred system ordering:** convenient, but nondeterministic or surprising. Rejected for an
  agent-first engine.
- **Adopt `bevy_ecs` / `hecs`:** contradicts ADR-0003. They are kept as competitive benchmark baselines.

## Consequences

Agents must declare ordering explicitly, which is more verbose and also more predictable. The ECS
spec must define performance budgets against `bevy_ecs` and `hecs` (`docs/process/PERFORMANCE.md`).

## Verification

ECS spec requirements and tests. Schedule-conflict detection tests. Competitive benchmarks. Miri.
