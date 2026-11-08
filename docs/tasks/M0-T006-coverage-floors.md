---
id: M0-T006
title: Coverage with per-crate floors
milestone: M0
status: todo
crates: [xtask]
depends_on: [M0-T002]
requirements: [XTASK-R009, XTASK-R010, XTASK-R060, XTASK-R061, XTASK-R062, XTASK-R063, XTASK-R064]
---

# M0-T006: Coverage with per-crate floors

## Goal

Per-crate line coverage is measured and can only go up (ratchet), as defined in ADR-0010.

## Context

- `docs/process/TESTING.md` §Coverage. `docs/specs/xtask.md` R060–R064.
- External tool: `cargo-llvm-cov` (needs the `llvm-tools-preview` component, pinned in M0-T001).

## Scope

**In:** `coverage` command; parsing `cargo llvm-cov --json --summary-only` output; mapping files to
crates by manifest directory; `ci/coverage.toml` format and the initial file (an `xtask` entry);
`--update-floors`; `--json` (R009); `coverage` step in `ci`. If M0-T007 has landed, add the `coverage`
job to `.github/workflows/ci.yml` and add `--skip coverage` to the `checks` job. Otherwise leave a note
in M0-T007's handoff section.

**Out:** coverage for doc tests (llvm-cov support for doc tests needs nightly; revisit in M1).

## Acceptance criteria

- [ ] `ci/coverage.toml` exists with a documented format (comment header) and a floor for `xtask`
      set to its measured coverage, rounded down to 0.1.
- [ ] Parsing is tested on a saved llvm-cov JSON fixture (no tool needed in tests).
- [ ] Ratchet logic tests: drop of 0.4 points passes, drop of 0.6 fails, a missing crate entry fails
      with a help line, `--update-floors` never lowers a floor.
- [ ] Missing `cargo-llvm-cov` → `skipped` with install hint.
- [ ] `coverage` is a step in `ci`.
- [ ] xtask coverage measured and recorded in the journal. If it is below 75% (the `usable` target for
      tools), say what is uncovered.

## Test requirements

`Verifies:` tests for R060–R064.

## Notes

`ci/coverage.toml` format suggestion:
```toml
# Per-crate line-coverage floors (percent). Managed by `cargo xtask coverage --update-floors`.
# Lowering a floor requires a journal entry (docs/process/TESTING.md).
[floors]
xtask = 78.4
```

## Handoff

_Empty._

## Review

_Empty._
