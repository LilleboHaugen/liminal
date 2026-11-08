---
id: ADR-0004
title: Layered crate architecture, machine-checked
status: accepted
date: 2026-11-08
supersedes: []
superseded_by: []
---

# ADR-0004: Layered crate architecture, machine-checked

## Context

The project owner requires that parts can be added, changed or removed without breaking others. Many
agents working independently will be tempted to add a "quick" dependency between crates that should
not know each other. Over time this tangles the graph and makes every change risky. Rules that only
live in prose are not followed reliably.

## Decision

- The engine is a Cargo workspace of many small crates in six layers: 0 foundation, 1 runtime,
  2 platform/GPU, 3 subsystems, 4 agent interface/integration, 5 tools (`docs/ARCHITECTURE.md` §1).
- A crate MAY depend only on crates in strictly lower layers, plus same-layer crates listed in
  `same_layer_deps`.
- `architecture.toml` is the normative registry: every crate, its layer, kind, unsafe policy, whether
  it is deterministic, and its allowed external dependencies.
- `cargo xtask check-arch` MUST enforce the registry in CI.
- Subsystems in layer 3 SHOULD communicate through ECS data (components, resources, events) rather than
  direct calls, so they can be removed independently.
- Games depend only on the facade crate `liminal_engine`, whose Cargo features turn subsystems on and off.

## Alternatives considered

- **Single crate with modules:** simplest, but poor compile parallelism, no enforceable boundaries,
  and everything recompiles together.
- **Layering by convention only:** proven to erode. Rejected.

## Consequences

- More crates means more `Cargo.toml` files and some re-export ceremony.
- Better incremental compile times and parallel builds.
- Moving a crate between layers is an architectural change and needs an ADR.

## Verification

`cargo xtask check-arch` (spec `docs/specs/xtask.md`, requirements XTASK-R030 to XTASK-R037).
