---
id: M0-T011
title: Real-binary integration tests for every xtask command
milestone: M0
status: done
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

- [x] Every command of §5 has at least one real-binary test checking exit code and output; the
      list is in the handoff, with the test name per command.
- [x] Doc examples for `COMMANDS`/`Command` and `Exit` run in `cargo test --doc -p xtask`.
- [x] Tests do not modify the repository (`git status` clean after `cargo test -p xtask`) and
      pass on the macOS, Windows and Linux `checks` jobs of CI.
- [x] Spec §13 updated; `cargo xtask ci` passes.

## Test requirements

`Verifies:` tags only where a test verifies a requirement's behaviour (R002, R007, R009, and
command-specific ones such as R072 for `status --check`), not on every binary test.

## Handoff

2026-11-09, one session (journal `2026-11-09-M0-T011.md`). Done; set to `review`.

Real-binary tests per command (all run `env!("CARGO_BIN_EXE_xtask")` on this repository):

| Command | Tests (file) | Runs covered |
|---|---|---|
| `help` (and no arguments) | `help_and_no_arguments_print_the_command_list_and_exit_0`, `unknown_command_and_unknown_option_print_usage_to_stderr_and_exit_2`, `non_utf8_argument_is_a_usage_error` (`cli.rs`, unchanged) | ok, exit 2 |
| `ci` | `ci_reports_passing_and_skipped_steps_in_json_human_output_and_the_summary_file`, `unknown_step_names_exit_2_without_running_anything` (`ci.rs`) | ok `--json` and human, summary file = stdout, skipped step exit 1, `--allow-skip` exit 0, exit 2 |
| `check-arch` | `check_arch_passes_on_the_repository`, `check_arch_rejects_unknown_options`, `check_arch_reports_a_cargo_that_cannot_start_as_internal_error_on_stderr` (`commands.rs`) | ok human and `--json`, exit 2, LIM-XTASK-0005 exit 1 |
| `check-docs` | `check_docs_passes_on_the_repository`, `check_docs_rejects_unexpected_arguments` | ok human and `--json`, exit 2 |
| `trace` | `trace_passes_and_prints_the_matrix`, `trace_json_with_crate_lists_only_that_crate`, `trace_rejects_a_crate_without_spec_and_a_missing_crate_name` | ok human, `--json`, `--crate`, `--crate=`, exit 2 |
| `hygiene` | `hygiene_passes_on_the_repository`, `hygiene_rejects_unknown_options` | ok human and `--json`, exit 2 |
| `status` | `status_check_passes_and_status_leaves_an_up_to_date_file_unchanged`, `status_has_no_json_option` | `--check` ok, `status` up to date (file bytes unchanged), exit 2 |
| `doctor` | `doctor_reports_every_tool_whatever_is_installed`, `doctor_reports_cargo_tools_missing_when_cargo_cannot_start`, `doctor_rejects_unknown_options` | human and `--json` vs. installed tools, missing tools exit 1, exit 2 |
| `coverage` | `coverage_is_skipped_and_fails_when_cargo_cannot_start`, `coverage_rejects_unknown_options`, `coverage_measures_every_crate_against_its_floor` (ignored; `coverage` CI job) | skipped exit 1 human and `--json`, exit 2, real measurement ok |

How the traps were handled:
- Repository root: fixed at compile time (`repo::root()`, `CARGO_MANIFEST_DIR/..`); no override was
  added. Failure paths use usage errors, or `$CARGO` set to a missing program
  (`CARGO_TARGET_TMPDIR/xtask-test-no-such-cargo`). Every command already reads `$CARGO` (spec), so
  this is no test-only mechanism. Error-*finding* runs (exit 1 with findings) are not run through the
  binary: that needs a broken repository; the library fixture tests cover them.
- Writes: `status` without `--check` runs only after `status --check` passed and the test compares
  `docs/STATUS.md` bytes before and after. `ci` runs write `target/xtask/ci-summary.json` (allowed by
  R010); they are in one test so no parallel test overwrites the file between write and read.
- Recursion: `ci` tests select native steps and `deny` (with the missing `$CARGO`, so it skips);
  `coverage` real run is `#[ignore = "M0-T011: …"]` and run by a new step of the `coverage` CI job
  (`cargo test --package xtask --all-features --locked --test commands -- --ignored`).
- `doctor` assertions derive the expected output from its own `--json` report (present ⇒ no help,
  no finding; missing ⇒ `null` version, help, R080 finding, `error` iff `needed_by` is non-empty;
  result line counts all findings, warnings included).

Doc examples: `Exit`, `Command`, `COMMANDS` (plus the existing `run`) in `xtask/src/cli.rs`.
The `COMMANDS` example lists all names in order, so a task adding a command updates it.

Evidence: two mutants of `xtask/src/main.rs` (exit code always 0; stdout routed to stderr), each
killed (mutant 1 failed 13 binary tests in 3 files). `cargo xtask ci` passes (11 steps) on macOS;
CI run results in the journal. Coverage 98.05 % (5050/5150), floor 97.6 left unchanged.

Next: review (new session, ADR-0020). After `done`, M0-T010 continues in a session that did not
implement M0-T011.

## Review

Reviewer: new session, 2026-11-09 (did not implement M0-T011). Outcome: **done**, no blocking
findings. Follow-up task: `docs/tasks/M1-T010-binary-test-follow-ups.md`.

Checked, macOS 27.2, Rust 1.99.1, cargo-llvm-cov 0.9.1, cargo-deny 0.20.2:
1. Acceptance criteria:
   - Every §5 command has a real-binary test that checks exit code and output: the handoff table
     matches `xtask/tests/cli.rs`, `ci.rs`, `commands.rs` (read in full). Met.
   - `cargo test --doc -p xtask`: 4 doc tests pass (`run`, `Exit`, `Command`, `COMMANDS`). Met.
   - `git status` after `cargo xtask ci`, after the ignored tests and after the mutants: only
     the owner's unrelated uncommitted edits (repository URL rename), nothing from tests. CI: run
     42672908688 (ba58a81), attempt 1: `checks (macos-latest)`, `checks (windows-latest)`,
     `coverage` pass; attempt 2: `checks (ubuntu-latest)`, `checks (windows-latest)`, `coverage`
     pass. All three `checks` jobs passed. Met. The `deny` job was never created in either
     attempt (GitHub start-up problem, also seen in run 42672235596); the run is therefore
     marked failed. The diff changes nothing the `deny` job runs; locally `deny` passes and the
     4 ignored `deny_policy` tests pass.
   - Spec §3, §10, §13 updated (`spec_version` 16); `cargo xtask ci` passes (11 steps). Met.
2. Ignored test `coverage_measures_every_crate_against_its_floor`: passes locally (5 s).
3. Mutants against the binary tests (`--test ci --test cli --test commands`), each reverted:
   M1 internal error → exit 2 (`cli.rs` `run`): killed. M2 internal error printed to stdout:
   killed. M3 `trace` wired to `hygiene::command`: killed (3 tests). M4 `main.rs` exit code
   capped at 1: killed (`unknown_step_names_exit_2_without_running_anything`).
4. `ci` tests overwrite `target/xtask/ci-summary.json` while an outer `cargo xtask ci` runs its
   `test` step; the outer run writes last (summary after the full run lists all 11 steps).
5. `Verifies:` tags agree with the requirements, except the partial R072 tag (follow-up item 2).
   `#[ignore]` names the task. No placeholders, no oracle changed (floor 97.6 unchanged). No new
   dependency (`serde_json` is a dependency of `xtask` since M0-T002).
6. Session boundaries: implemented in one owner-started session (journal
   `2026-11-09-M0-T011.md`); no sub-agents.

Non-blocking findings (follow-up M1-T010):
- A moved checkout leaves a stale `xtask` binary whose compiled-in root does not exist; every
  command then fails with misleading messages (observed in this session). In unchanged code
  (R001, M0-T002/T003), not behaviour this task claims.
- `Verifies: XTASK-R072` on a test that runs only the passing side of `status --check`.
- `doctor` human-mode test does not check present tools' versions.
