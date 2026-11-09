---
id: M1-T011
title: check-arch — determinism lint configuration for deterministic crates
milestone: M1
status: review
crates: [xtask]
depends_on: [M1-T001]
requirements: [XTASK-R039]
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

- [x] The mechanism is shown to work: a fixture or scratch crate calling `f32::sin` fails `cargo clippy` with the configuration and passes without it (journal records the run).
- [x] New requirement in `docs/specs/xtask.md` with `Verifies:` fixture tests for: missing file, file missing one banned path, complete file, non-deterministic crate without file.
- [x] CODE_STYLE.md says where the configuration lives.
- [x] `cargo xtask ci` passes.

## Test requirements

The new XTASK requirement (MUST), fixture tests in the style of the existing `check-arch` fixtures.

## Notes

_None._

## Handoff

Done (2026-11-09, implementation sub-agent):

- Assumption verified (clippy 0.1.99, scratch workspaces outside the repository; journal
  `docs/journal/2026-11-09-M1-T011.md`): clippy uses the nearest `.clippy.toml` or `clippy.toml`
  (`.clippy.toml` wins in one directory) and does not merge it with a parent's file. Mechanism:
  each deterministic crate has its own `<manifest dir>/clippy.toml` that repeats the root
  settings and adds `disallowed-methods`. Recorded in `docs/process/CODE_STYLE.md` §Lints and a
  comment in the root `clippy.toml`.
- New XTASK-R039 (MUST) in `docs/specs/xtask.md`, `spec_version` 21, with details (lookup,
  entry forms, LIM-XTASK-0004, locations).
- Code: `xtask/src/repo/clippy.rs` (lookup and parsing, loader side), `ArchModel::clippy`
  filled by `repo::load_arch` for packages registered `deterministic = true`,
  `xtask/src/check_arch/determinism.rs` (rule and `BANNED_METHODS`, with a test that the constant
  equals the CODE_STYLE.md list). `CrateEntry::deterministic` lost its `expect(dead_code)`.
- Tests: unit tests in `determinism.rs` (complete, missing paths, no file, non-deterministic crate)
  and `repo/clippy.rs`; fixtures `valid/` (deterministic `runtime` with a complete file, root
  `clippy.toml` banning nothing), `r039_missing_clippy_config`, `r039_incomplete_clippy_config`,
  plus a no-file-at-all variant and a LIM-XTASK-0004 case in `fixture_tests.rs`.
- `cargo xtask ci` passes (11 steps, no skips), macOS 27.2.

Not done / traps:

- No real deterministic crate is in the workspace yet, so R039 judges nothing on the real
  repository today. Each deterministic crate's first implementation task (M1-T013 for
  `liminal_core`, etc.) must add the crate's `clippy.toml`; check-arch then fails until it does.
- `disallowed-types` (ADR-0009 item 7 mentions it) is not on the CODE_STYLE.md list and is not
  checked. `HashMap::default`/`with_capacity` are not banned by the list either; changing the list
  is a CODE_STYLE.md change plus an update of `BANNED_METHODS` (the test catches a mismatch).
- Not verified on Windows or Linux (path lookup uses `/`-joined repository-relative paths, as the
  rest of the loader does).

## Review

_Empty._
