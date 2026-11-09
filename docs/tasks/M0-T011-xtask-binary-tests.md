---
id: M0-T011
title: Real-binary integration tests for every xtask command
milestone: M0
status: in-progress
crates: [xtask]
depends_on: [M0-T009]
requirements: [XTASK-R002, XTASK-R007, XTASK-R009]
---

# M0-T011: Real-binary integration tests for every `xtask` command

## Goal

`xtask` meets the tool-crate `usable` gate it fails today (found by the M0 exit review, M0-T010):
every command is covered by an integration test that runs the real binary and checks its exit code
and output, and the library's main types have runnable doc examples.

## Context

- `docs/process/MATURITY.md` §"Tool crates (layer 5)": for tool crates, "Primary entry points have
  runnable examples" is replaced by "the crate's library target has runnable doc examples for its
  main types, **and** every command is covered by an integration test that runs the real binary
  and checks its exit code and output".
- M0-T010 "Review" (gate evidence). State found on `main` 6f485a4: only `xtask/tests/cli.rs`
  (`help`, no arguments, unknown command/option, non-UTF-8 argument) and `xtask/tests/ci.rs` (`ci`
  usage errors) run `env!("CARGO_BIN_EXE_xtask")`. `check-arch`, `check-docs`, `trace`,
  `hygiene`, `coverage`, `status` and `doctor` are tested only through library functions, so a
  defect in `main.rs` wiring, argument forwarding, stdout/stderr routing or exit-code mapping for
  them is not caught by a binary test.
- Doc tests: one (`xtask::run`, `xtask/src/cli.rs`). The public API is `COMMANDS`, `Command`,
  `CommandError`, `CommandFn`, `Exit`, `run` (`xtask/src/lib.rs`).
- `docs/specs/xtask.md` §5, §6 "Check command output", R002, R007, R009, R010.
- M0 exit criterion "`xtask` itself reaches maturity `usable`" needs this, so it is in M0.

## Scope

**In:**
1. For each of `check-arch`, `check-docs`, `trace`, `hygiene`, `status` (with and without
   `--check`), `doctor`, `coverage` and `ci`: at least one integration test in `xtask/tests/` that
   runs the real binary and asserts the exit code and the output (stdout/stderr content, and for
   commands with `--json` that the JSON parses with `"version": 1` and an `ok` matching the exit
   code). Where the command accepts it, cover both an `ok` run (exit 0) and a run with an error
   finding or usage error (exit 1 or 2).
2. Traps to handle, not work around silently:
   - **Recursion.** `ci` (with the `test` or `coverage` step) and `coverage` run `cargo test`,
     which would run these tests again. Test them with step selections or arguments that do not
     spawn the test suite (e.g. `ci --only check-docs,status-check --json`, usage errors), and
     write in the test module doc why the full run is covered by CI instead (spec §10 already
     says the real repository is run in CI).
   - **Repository root and writes.** Find out how the binary locates the repository root. A test
     must not modify the repository (R010): `status` without `--check` writes `docs/STATUS.md`
     only if it differs, so run it where it is up to date, or against a fixture copy if the
     binary can be pointed at one. Do not add a root-override option only for tests without
     specifying it in the spec.
   - **External tools.** `doctor` and `coverage` depend on installed tools; assertions must hold
     whether or not `cargo-llvm-cov`, `cargo-deny` and Miri are installed (CI `checks` jobs have
     none of them), or the test is `#[ignore = "M0-T011: …"]` and run by a CI job that has the
     tool.
3. Runnable doc examples (`# Examples`) for `COMMANDS`/`Command` and `Exit`, in addition to the
   existing one on `run`.
4. Fill the evidence for this gate in `docs/specs/xtask.md` §13 (bump `spec_version`).

**Out:** promoting `xtask`; that is done by M0-T010 in a session that did not implement this task.

## Acceptance criteria

- [ ] Every command of §5 has at least one real-binary test checking exit code and output; the
      list is in the handoff, with the test name per command.
- [ ] Doc examples for `COMMANDS`/`Command` and `Exit` run in `cargo test --doc -p xtask`.
- [ ] Tests do not modify the repository (`git status` clean after `cargo test -p xtask`) and
      pass on the macOS, Windows and Linux `checks` jobs of CI.
- [ ] Spec §13 updated; `cargo xtask ci` passes.

## Test requirements

`Verifies:` tags only where a test verifies a requirement's behaviour (R002, R007, R009, and
command-specific ones such as R072 for `status --check`), not on every binary test.

## Handoff

_No session has worked on this task yet._

## Review

_Empty._
