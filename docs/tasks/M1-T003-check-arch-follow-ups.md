---
id: M1-T003
title: check-arch follow-ups from the M0-T004 review
milestone: M1
status: done
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

- [x] A path dependency on a root package (`manifest_dir = ""`) is a workspace dependency; a test
      shows an R032 finding (not R033) for a higher-layer root package.
- [x] The scope test covers all 72 combinations, each run at a layer of its group.
- [x] `cargo xtask ci` passes.

## Test requirements

`Verifies: XTASK-R032` on the root-package test, `Verifies: XTASK-R033, XTASK-R034` on the scope
test.

## Notes

- `relative()` is also used for target and manifest paths; check that a root-level `src/lib.rs`
  still becomes `src/lib.rs`.

## Handoff

2026-11-09, implementation sub-agent (ADR-0026/ADR-0027), branch
`task/M1-T003-check-arch-follow-ups`. Status `review`.

Done:
1. `xtask/src/repo/metadata.rs` `relative()` returns `Some("")` for the workspace root itself
   (with or without a trailing `/` on the root), so a path dependency on a root package gets
   `path: Some("")`, matching the root package's `manifest_dir = ""`. Tests:
   - `repo::metadata::tests::a_path_dependency_on_the_workspace_root_is_inside_with_an_empty_path`
     (`Verifies: XTASK-R032`): `relative()` on the root, below it and on a non-boundary prefix
     (`/w2`); `parse()` of a dependency on `/w` and on `C:\w` (Windows separators); a root
     package's root-level target is still `src/lib.rs` (the task note).
   - `check_arch::tests::r032_a_path_dependency_on_a_root_package_is_a_workspace_dependency`
     (`Verifies: XTASK-R032`): builds the packages through `metadata::parse()` from
     `cargo metadata`-shaped JSON (root package `high`, layer 2, registered with `path = ""`;
     `mid`, layer 1, depends on path `/w`) and expects exactly one R032 finding at
     `crates/mid/Cargo.toml`. Without the fix it gives R033 (checked by reverting `metadata.rs`).
2. `xtask/src/check_arch/tests.rs`: `SCOPE_TABLE` (47 hand-listed rows) is replaced by
   `SPEC_SCOPE_ROWS`, the six rows of the spec table copied as written, and `scope_table()`,
   which builds the full product 2 × 3 × 3 × 4 = 72 rows and asserts that exactly one spec row
   covers each (layer group, crate kind, dependency kind). `the_scope_table_has_every_combination_once`
   checks 72 distinct keys. The R033/R034 test runs layers 0–4 rows at layers 0 and 4 and
   layer 5 rows at layer 5, and asserts 108 runs.

Not changed: scope rules and `docs/specs/xtask.md` (the spec already says a path dependency on a
workspace crate's manifest directory is a workspace dependency; the code now matches it).

Verified: `cargo xtask ci` passes on macOS 27.2 (arm64), Rust 1.99.1, 11 steps, no skips. Not
verified on Windows or Linux locally (branch CI runs them).

Traps: none found. `architecture.toml` `path` is not validated, so `path = ""` registers a root
package; that is existing behaviour and out of scope.

## Review

Reviewer: review sub-agent under ADR-0026/ADR-0027, 2026-11-09, started fresh with the fixed
review prompt (template verbatim). It belongs to a different session from the one that wrote
M1-T003. Outcome: **done**, no blocking findings, no follow-up task.

Checked, macOS 27.2 (arm64), Rust 1.99.1:
1. Acceptance criteria:
   - Root-package path dependency: `relative()` returns `Some("")` for the root with or without a
     trailing `/`, keeps `None` for `/w2` and keeps `src/lib.rs` for a root-level target.
     `check_arch::tests::r032_a_path_dependency_on_a_root_package_is_a_workspace_dependency`
     goes through `metadata::parse()` and expects exactly one R032 finding at
     `crates/mid/Cargo.toml`. Met.
   - Scope test: `SPEC_SCOPE_ROWS` matches the six rows of the spec §6 scope table cell by
     cell. `scope_table()` asserts that exactly one spec row covers each (group, kind, dependency
     kind). The test asserts 72 distinct keys and 108 runs: layers 0–4 run at layers 0 and 4, layer 5
     runs at layer 5. Met.
   - `cargo xtask ci`: all 11 steps pass (the 4 ignored `deny_policy` tests carry M0-T009 and are
     unchanged). Met.
2. Mutation: I removed the `rest.is_empty()` branch of `relative()`. Then both new tests fail
   (`r032_a_path_dependency_on_a_root_package_is_a_workspace_dependency`,
   `a_path_dependency_on_the_workspace_root_is_inside_with_an_empty_path`). The scope rules in
   `check_arch/rules.rs` (scope check in `external()`: dev/tool, build/proc-macro, tool/layer 5) are each
   covered by rows whose expected value comes from the spec table.
3. No fake progress: no oracle changed. The 47 old scope rows are a subset of the 72 new ones and
   have the same values (checked against the spec table). No test was weakened.
4. Spec agreement: spec §6 defines a workspace dependency by the manifest directory. The code now
   matches it for the root package. No spec change needed. `Verifies:` tags are R032 on the
   root-package tests and R033/R034 on the scope test. They are correct.
5. Unsafe, determinism, architecture, dependencies: not affected (`xtask` test and path
   helper only).
6. Session boundaries: the implementation sub-agent's journal names its parent as
   `2026-11-09-m1-subagents.md`. That journal is not on any branch at review time, so the
   parent's sub-agent list could not be checked. Not a finding against this task: the parent
   session writes that journal at its handoff.
