---
id: ADR-0019
title: Agent interface (revised) — probe protocol, CLI, MCP server and build gating
status: accepted
date: 2026-11-08
supersedes: [ADR-0012]
superseded_by: []
---

# ADR-0019: Agent interface (revised) — probe protocol, CLI, MCP server and build gating

## Context

ADR-0012 said the probe's Cargo feature would be "on in dev profiles and off in release builds". Cargo
features cannot depend on the build profile, so that rule was impossible to implement as written. An
independent review of the planning documents also found that the method list in
`docs/ARCHITECTURE.md` §5.4 and the one in ADR-0012 differed. This ADR replaces ADR-0012 in full. Only
the build-gating rule and the method list change; everything else is carried over.

## Decision

- `liminal_probe` is an in-process server speaking **JSON-RPC 2.0** over stdio (for child processes)
  or a TCP socket bound to localhost only.
- This is the authoritative method list. `docs/ARCHITECTURE.md` §5.4 and `docs/ROADMAP.md` refer here:

  | Method | Milestone |
  |---|---|
  | `probe.version` | M2 |
  | `sim.pause`, `sim.resume`, `sim.step` | M2 |
  | `world.query`, `entity.get`, `entity.set_component`, `entity.spawn`, `entity.despawn` | M2 |
  | `input.inject` | M2 |
  | `state.hash` | M2 |
  | `replay.record`, `replay.play` | M2 |
  | `telemetry.frames`, `log.tail` | M2 |
  | `capture.screenshot` | M3 |

- Every method has a JSON Schema for parameters and results. Errors use stable codes (ADR-0017).
  The protocol is versioned through `probe.version` from the first release.
- Probe commands are applied between frames, so they do not break determinism. Injected input is
  recorded like real input.
- `liminal_cli` (the `liminal` binary) is a probe client for shell-based agents. `liminal_mcp` exposes
  the same methods as MCP tools through an own minimal MCP implementation over stdio. The MCP protocol
  revision is pinned at M2 kickoff.
- **Build gating (changed from ADR-0012):** the probe is compiled in when the `probe` Cargo feature of
  `liminal_engine` is enabled. Game templates (`liminal new`) enable it only for development builds,
  through a game-level feature that the dev workflow (`liminal run`) turns on and `liminal package`
  does not. Separately, a probe compiled into a build without `debug_assertions` refuses to start
  unless the game also sets an explicit opt-in (`ProbeConfig::allow_in_release(true)`). The probe
  never binds to non-loopback interfaces.

## Alternatives considered

- **gRPC / custom binary protocol:** heavier tooling, less readable for agents. Bulk data (screenshots)
  is written to files and returned as paths instead.
- **CLI only:** MCP clients get typed tools, which they use more reliably.
- **Gating on `cfg(debug_assertions)` alone:** ties the probe to unoptimized builds, but agents need
  the probe when profiling optimized builds. Hence feature + opt-in.

## Consequences

Subsystems should expose state through reflection so the probe can reach it. Protocol compatibility
must be managed through versioning.

## Verification

M2 exit criteria: scripted end-to-end tests through both CLI and MCP. Snapshot tests of schemas. A test
that a release-mode build without opt-in refuses to start the probe.
