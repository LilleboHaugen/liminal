---
id: M1-T003
title: check-arch follow-ups from the M0-T004 review
milestone: M1
status: todo
crates: [xtask]
depends_on: [M0-T004]
requirements: [XTASK-R032, XTASK-R033, XTASK-R034]
---

# M1-T003: `check-arch` follow-ups from the M0-T004 review

## Goal

The non-blocking findings of the M0-T004 review are fixed: a path dependency on the workspace's
root package is classified as a workspace dependency, and the R033/R034 scope test covers the
full table it claims to cover.

## Context

- Follow-up task under ADR-0021, written by the first review of M0-T004 (see that task's
  "Review").
- `docs/specs/xtask.md` §6 "`check-arch` details" (model, R033/R034 scope table).
- No M0 exit criterion needs this, so it is in M1. The M1 kickoff (M1-T001) places it in the M1
  order; it touches only `xtask`.

## Scope

**In:**
1. `xtask/src/repo/metadata.rs` `relative()`: for a path equal to the workspace root it returns
   `None` (the `strip_prefix('/')` fails on the empty rest). A root package has
   `manifest_dir = ""`, so a path dependency on it gets `path: None` and `check-arch` treats it as
   an external dependency (an R033/R034 finding "not in its `external` list" instead of R032).
   Not realistic today: the root `Cargo.toml` is a virtual manifest. Return `Some("")` for the
   root itself, and test it in `metadata.rs` and in `check_arch/tests.rs`.
2. `xtask/src/check_arch/tests.rs` `SCOPE_TABLE`: the M0-T004 handoff says the table covers
   "every layer group × crate kind × dependency kind × scope". It has 47 rows. Missing: layers
   0–4 `bin` with build/{runtime, dev, tool} and dev/{runtime, build}; layer 5 `bin` with 8 of
   12 combinations; layer 5 `proc-macro` entirely. Generate the full product (layer group ×
   kind × dependency kind × scope = 2 × 3 × 3 × 4 = 72 rows) from the spec table, or list all
   rows. The expected values come from the spec text, not from the implementation.

**Out:** behaviour changes to the scope rules.

## Acceptance criteria

- [ ] A path dependency on a root package (`manifest_dir = ""`) is a workspace dependency; a test
      shows an R032 finding (not R033) for a higher-layer root package.
- [ ] The scope test covers all 72 combinations, each run at a layer of its group.
- [ ] `cargo xtask ci` passes.

## Test requirements

`Verifies: XTASK-R032` on the root-package test, `Verifies: XTASK-R033, XTASK-R034` on the scope
test.

## Notes

- `relative()` is also used for target and manifest paths; check that a root-level `src/lib.rs`
  still becomes `src/lib.rs`.

## Handoff

_Filled in by the implementing session(s)._

## Review

_Filled in by the reviewing session._
