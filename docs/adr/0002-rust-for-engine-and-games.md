---
id: ADR-0002
title: Rust for engine and game code; no scripting language
status: accepted
date: 2026-11-08
supersedes: []
superseded_by: []
---

# ADR-0002: Rust for engine and game code; no scripting language

## Context

The engine must produce native binaries for macOS, Windows and Linux with performance close to the
hardware's limits. Its users are AI agents, which write Rust well and benefit from a compiler that
rejects many mistakes before runtime. Most engines add a scripting layer (Lua, C#, GDScript) so that
humans can iterate without recompiling.

## Decision

- The engine MUST be written in Rust (stable toolchain, edition 2024).
- Game logic MUST be written in Rust against the `liminal` facade. Liminal MUST NOT embed a scripting
  language runtime.
- Fast iteration is provided by (a) data files that hot-reload without recompiling (ADR-0013),
  (b) the probe protocol for live inspection and editing (ADR-0012), and (c) treating incremental
  compile time as a tracked performance budget (`docs/process/PERFORMANCE.md`).

## Alternatives considered

- **Embedded Lua / Wasm scripting:** faster iteration without recompiling, but a second language,
  marshalling overhead, weaker static checking, and two places for game logic. Agents gain little,
  because they do not mind recompiling as long as it is fast.
- **C++:** mature game ecosystem, but weaker memory safety, and agents produce more subtle UB bugs.
- **Zig / other systems languages:** smaller ecosystems and less agent familiarity.

## Consequences

- Compile times are the main cost. They must be managed actively (crate splitting, careful generics,
  possibly dynamic linking in dev builds).
- Modding by players (who cannot recompile) is not supported. It could be revisited with a sandboxed
  Wasm layer later. That would need a superseding ADR.

## Verification

Architecture review. `architecture.toml` lists no scripting runtime dependency.
