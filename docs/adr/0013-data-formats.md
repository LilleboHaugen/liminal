---
id: ADR-0013
title: Authored data as text, runtime data as cooked binary
status: accepted
date: 2026-11-08
supersedes: []
superseded_by: []
---

# ADR-0013: Authored data as text, runtime data as cooked binary

## Context

Agents author scenes, prefabs, tuning values, input maps and materials. They need a text format they
produce reliably and that can be validated before it reaches the engine. Shipped games need fast
loading, ideally zero-copy, and should not carry parsers for many source formats.

## Decision

- **Authored data** (scenes, prefabs, configs, input maps, materials) is stored as text. The text format
  is decided in OQ-003 (default candidate: strict JSON). Schemas are generated from reflected Rust
  types, so the Rust type is the single source of truth. `liminal validate` checks files against
  schemas with errors that give file, JSON path, expected and actual values, and an error code.
- **Source assets** (glTF meshes, PNG images, audio, fonts) are converted by `liminal_cook` into
  **engine-native binary formats** designed for direct loading (aligned, little-endian, versioned
  headers, checksums). Cooking is reproducible: same input and same cooker version give byte-identical
  output.
- The **runtime never parses source asset formats.** Dev builds may cook on demand when a source file
  changes (hot reload). Shipped builds contain only cooked data.
- Authored text data MAY be loaded directly in dev builds for hot reload. Shipped builds load a binary
  form of it.

## Alternatives considered

- **Runtime parsing of PNG/glTF:** simpler pipeline, but slower loading, larger binaries, and more
  attack surface.
- **RON / TOML for authored data:** RON is Rust-specific and less familiar to agents. TOML handles
  deep nesting poorly. Both remain options for configuration files in tools.

## Consequences

Cooked formats must be versioned and their readers fuzzed (`stable` gate). Importers in
`liminal_cook` may use external crates in the `tool` scope, each with an ADR (M4).

## Verification

Cook reproducibility tests (byte-identical output). Schema snapshot tests. Fuzzing of cooked-format
readers.
