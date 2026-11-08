---
id: ADR-0012
title: Agent interface — probe protocol, CLI and MCP server
status: superseded
date: 2026-11-08
supersedes: []
superseded_by: [ADR-0019]
---

# ADR-0012: Agent interface — probe protocol, CLI and MCP server

## Context

What sets Liminal apart is that agents can verify what they build without a human. That needs a way
to inspect and drive a running game: pause, step, query state, change state, inject input, capture
frames, read performance data. Agents reach tools in two common ways: running shell commands and calling
MCP (Model Context Protocol) tools.

## Decision

- `liminal_probe` is an in-process server speaking **JSON-RPC 2.0**, over stdio (for child processes) or
  a TCP socket bound to localhost only.
- Initial method set (M2; capture in M3): `sim.pause`, `sim.resume`, `sim.step`, `world.query`,
  `entity.get`, `entity.set_component`, `entity.spawn`, `entity.despawn`, `input.inject`,
  `state.hash`, `replay.record`, `replay.play`, `telemetry.frames`, `log.tail`, `capture.screenshot`.
- Every method has a JSON Schema for parameters and results. Errors use stable codes (ADR-0017).
- Probe commands are applied at a defined point in the frame (between frames), so they do not break
  determinism. Injected input is recorded like real input.
- `liminal_cli` (the `liminal` binary) is a client of the probe for shell-based agents.
  `liminal_mcp` exposes the same methods as MCP tools. The MCP server is our own minimal
  implementation over stdio. The exact MCP protocol revision is pinned at M2 kickoff.
- **Security:** the probe is compiled only with the `probe` Cargo feature, which is on in dev profiles
  and off in release builds by default. It never binds to non-loopback interfaces.

## Alternatives considered

- **gRPC / custom binary protocol:** faster, but heavier tooling and less readable for agents. JSON is
  fast enough for control traffic. Bulk data (screenshots) is written to files and returned as paths.
- **CLI only:** shell agents are covered, but MCP clients get typed tools, which they use more
  reliably.
- **An in-engine scripting console:** contradicts ADR-0002.

## Consequences

Every engine subsystem should expose its state through reflection so the probe can reach it. Protocol
compatibility becomes something to manage. It is versioned (`probe.version` method) from the first
release.

## Verification

M2 exit criteria: scripted end-to-end tests through both CLI and MCP. Snapshot tests of schemas.
