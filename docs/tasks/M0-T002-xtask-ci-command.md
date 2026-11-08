---
id: M0-T002
title: xtask ci command
milestone: M0
status: todo
crates: [xtask]
depends_on: [M0-T001]
requirements: [XTASK-R003, XTASK-R004, XTASK-R005, XTASK-R006, XTASK-R007, XTASK-R008, XTASK-R010, XTASK-R012]
---

# M0-T002: xtask `ci` command

## Goal

`cargo xtask ci` runs the cargo-based quality steps (`fmt`, `clippy`, `test`, `doc`) with correct
pass/fail/skip semantics and a machine-readable JSON summary. Later tasks add their steps to it.

## Context

- `docs/specs/xtask.md` §4 (steps, findings), §5 (JSON schema), R003–R010, R012.
- Spec test plan: orchestration is tested with a fake step runner. Real steps run in CI.

## Scope

**In:**
- A `Step` abstraction (name, run → outcome) and a runner that executes selected steps in order, timing
  each one.
- Real steps: `fmt`, `clippy`, `test`, `doc` with the exact commands in R012.
- Options `--json`, `--fail-fast`, `--only`, `--skip`, `--allow-skip`.
- JSON summary to `target/xtask/ci-summary.json` (always) and stdout (`--json`). Findings structure
  shared with later checks (§4 Finding) defined in a module the other checks will reuse.
- Exit codes per R007.

**Out:** checks implemented in M0-T003 … M0-T009 (each adds its own step).

## Acceptance criteria

- [ ] On a clean repository, `cargo xtask ci` passes on macOS and writes a valid summary.
- [ ] Introducing a formatting error makes `fmt` fail while later steps still run (without
      `--fail-fast`). Exit code 1. Verified manually once and covered by orchestration tests.
- [ ] A step whose tool is missing reports `skipped` with `reason` and `help`, and `ok` is false.
      Covered by unit test with fake runner.
- [ ] `--only`/`--skip` with an unknown step name exits 2.
- [ ] The JSON output is checked by a snapshot test against a fixture run with fake steps
      (durations replaced by a placeholder before comparison).

## Test requirements

`Verifies:` tests for R003–R008, R010 and R012 (R012: assert the exact argument vectors the real steps
build, without running them). R010 here covers `ci`. Later tasks extend R010's tests to their own
commands. R009 (`--json` for check commands) is delivered by each check's task.

## Notes

- Run child processes with inherited stdout/stderr in human mode, and capture them in `--json` mode
  (attach the last ~50 lines of output to failed steps as `output_tail`). This adds an optional field to
  the §5 schema. Update the spec (`spec_version` 2) if you do it.
- `doc` step: set `RUSTDOCFLAGS` only for the child process, never globally.

## Handoff

_Empty._

## Review

_Empty._
