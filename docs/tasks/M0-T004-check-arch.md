---
id: M0-T004
title: check-arch
milestone: M0
status: todo
crates: [xtask]
depends_on: [M0-T002]
requirements: [XTASK-R009, XTASK-R030, XTASK-R031, XTASK-R032, XTASK-R033, XTASK-R034, XTASK-R035, XTASK-R036, XTASK-R037]
---

# M0-T004: `check-arch`

## Goal

The workspace's real dependency graph is checked against `architecture.toml` on every CI run, so
layering, dependency approval and the unsafe policy hold without relying on review.

## Context

- ADR-0003 (dependency policy), ADR-0004 (layers), ADR-0018 (unsafe policy).
- `architecture.toml` (header comment lists the rules). `docs/specs/xtask.md` R030–R037.
- Approved `tool` crates for xtask: `cargo_metadata`, `toml`, `serde`, `serde_json`.

## Scope

**In:** full `architecture.toml` validation (R030); extending the shared repo-model loader (xtask spec
§3; created by M0-T003 if that task landed first, otherwise create it here, and M0-T003 reuses it) with
`cargo metadata` data (dependencies with kinds, manifest paths, targets); rules R031–R037;
`check-arch` command with `--json` (R009) and `ci` step; fixtures.

**Out:** R038 (maturity dependency rule). That starts in M2, and its task is created at M2 kickoff.

## Acceptance criteria

- [ ] `cargo xtask check-arch` passes on the real repository (only `xtask` exists, plus the planned
      registry).
- [ ] Rule logic runs on a repo model built from a saved `cargo metadata` JSON fixture plus an
      `architecture.toml` fixture, without spawning cargo in tests.
- [ ] One failing fixture per rule (R030–R036), each reporting exactly its rule ID with `help` (R037).
- [ ] Scope rules tested for every combination in R033/R034: runtime crate using `tool` scope fails;
      proc-macro using `build` passes; normal dependency on a `build`-scope crate from a non-proc-macro
      crate fails; dev-dependency not in `dev_external` fails.
- [ ] `check-arch` is a step in `ci`.

## Test requirements

`Verifies:` tests for R030–R037. Fixtures under `xtask/tests/fixtures/check_arch/<case>/`.

## Notes

- Dependency kinds from `cargo metadata`: `normal`, `build`, `dev` (`null` kind means normal).
  Target-specific dependencies (`cfg(...)`) count the same as unconditional ones.
- R035: detect `#![forbid(unsafe_code)]` by parsing the crate root file's inner attributes. Ignore
  occurrences in comments and strings (a simple token-level scan is enough; no `syn` in xtask).
- External crate names in `architecture.toml` use the crates.io name (`objc2-metal`, with hyphen),
  which `cargo metadata` also reports. Do not normalize hyphens/underscores.
- `cargo metadata` reports Windows paths with backslashes. Normalize to `/` before comparing with
  `path` (R031).

## Handoff

_Empty._

## Review

_Empty._
