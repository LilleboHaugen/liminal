---
id: M1-T011
title: check-arch — determinism lint configuration for deterministic crates
milestone: M1
status: todo
crates: [xtask]
depends_on: [M1-T001]
requirements: []
---

# M1-T011: check-arch: determinism lint configuration for deterministic crates

## Goal

A crate marked `deterministic = true` cannot be added without the clippy `disallowed-methods` configuration that ADR-0009 item 7 and CODE_STYLE.md require.

## Context

- ADR-0009 item 7; `docs/process/CODE_STYLE.md` §Lints (the banned list).
- `docs/specs/xtask.md` §check-arch; the root `clippy.toml`.
- Assumption to verify first: clippy reads the `clippy.toml` nearest to the crate's manifest directory and does not merge it with the root file. If so, a per-crate file must repeat the root settings.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** 1. Choose the mechanism (per-crate `clippy.toml` or another that clippy really applies) and record it in CODE_STYLE.md.
2. New XTASK requirement: `check-arch` reports a deterministic workspace crate whose effective clippy configuration does not ban every path on the CODE_STYLE.md list. Spec `spec_version` bump and history row.
3. Fixture tests for the passing and failing case.

**Out:** Writing the configuration for real crates: each crate's first implementation task does that, and this check then enforces it.

## Acceptance criteria

- [ ] The mechanism is shown to work: a fixture or scratch crate calling `f32::sin` fails `cargo clippy` with the configuration and passes without it (journal records the run).
- [ ] New requirement in `docs/specs/xtask.md` with `Verifies:` fixture tests for: missing file, file missing one banned path, complete file, non-deterministic crate without file.
- [ ] CODE_STYLE.md says where the configuration lives.
- [ ] `cargo xtask ci` passes.

## Test requirements

The new XTASK requirement (MUST), fixture tests in the style of the existing `check-arch` fixtures.

## Notes

_None._

## Handoff

_Empty._

## Review

_Empty._
