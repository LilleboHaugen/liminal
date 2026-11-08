---
id: ADR-0015
title: Naming and licensing
status: accepted
date: 2026-11-08
supersedes: []
superseded_by: []
---

# ADR-0015: Naming and licensing

## Context

The project is named Liminal and is open source. On 2026-11-08 the crate name `liminal` on crates.io
was already taken by an unrelated project (v0.1.4, last updated 2024-05-06). The names `liminal_core`,
`liminal_engine` and `liminal-engine` were free.

## Decision

- Engine crates are named `liminal_<area>` with underscores (package name equals crate name), in
  `crates/liminal_<area>/`.
- The facade package is `liminal_engine`, with `[lib] name = "liminal"`, so game code writes
  `use liminal::prelude::*`.
- The CLI binary is named `liminal`.
- License: **MIT OR Apache-2.0** (the Rust ecosystem convention). Files `LICENSE-MIT` and
  `LICENSE-APACHE` are at the repository root. The copyright holder line is "The Liminal Contributors".
  Contributions are accepted under the same dual license (README).
- Crate publishing to crates.io is not planned before the first `stable` crates exist. Names are
  re-checked at that time.

## Alternatives considered

- **Hyphenated package names (`liminal-core`):** common, but the package name and crate name then
  differ, which confuses agents in `use` paths.
- **GPL or MPL:** would limit use in commercial games.

## Consequences

Dependency licences must be compatible (`docs/process/DEPENDENCIES.md`, enforced by `cargo deny`).

## Verification

`cargo deny check licenses`. `check-arch` checks crate names against `architecture.toml`.
