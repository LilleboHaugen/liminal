---
id: ADR-0003
title: Build from scratch; dependency policy
status: accepted
date: 2026-11-08
supersedes: []
superseded_by: []
---

# ADR-0003: Build from scratch; dependency policy

## Context

The project owner wants every engine system tailored and optimized for Liminal, built by AI agents
rather than adopted from existing engines. Rebuilding OS and driver boundary layers (window creation
on three operating systems, Objective-C interop, raw API bindings) costs a lot and gains no speed.
Some crates are needed only at compile time or in tests and never ship in a game.

## Decision

1. Engine systems MUST be implemented in Liminal: ECS, math, memory, jobs, telemetry, reflection,
   serialization (subject to OQ-002), assets, scene, input, the GPU abstraction (RHI), renderer,
   shaders pipeline integration, voxel, physics, audio mixing, text, UI, animation, probe, testkit.
2. External crates MAY be used only in the cases and scopes defined in `docs/process/DEPENDENCIES.md`:
   boundary code, compile-time only, testing/tooling, or "not yet worth replacing" behind a Liminal
   interface.
3. An external runtime crate MUST be replaced by our own implementation when it is shown to be a
   bottleneck, or when an agent shows with prototype, tests and benchmarks that our own version is
   better on a criterion that matters.
4. Every external crate MUST be registered in `architecture.toml` with its scope and the ADR approving
   it. This ADR approves the following:
   - `build` scope: `syn`, `quote`, `proc-macro2`, for `liminal_reflect_derive`. Writing a Rust parser
     is out of scope, and none of this code runs in games.
   - `tool` scope: `serde`, `serde_json`, `toml`, `cargo_metadata`, for `xtask`. These do not ship
     in games.
5. Comparison crates used in competitive benchmarks MUST NOT become dependencies of engine crates
   (OQ-004).

## Alternatives considered

- **Build on Bevy/wgpu/rapier:** fastest to a demo, but contradicts the goal of a tailored engine
  and caps optimization at those crates' designs.
- **Write absolutely everything (including windowing and API bindings):** large, platform-specific cost
  with no performance return. Remains possible later under rule 3.

## Consequences

Much more code to write and test. This is accepted, because the project's premise is that agents can
write and verify it. Boundary crates are hidden behind our own interfaces, so replacing them later is
local.

## Verification

`cargo xtask check-arch` rejects unregistered dependencies or dependencies used outside their scope.
`cargo deny` enforces licences.
