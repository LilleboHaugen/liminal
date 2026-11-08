---
id: M0-T002
title: xtask ci command
milestone: M0
status: done
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

- [x] On a clean repository, `cargo xtask ci` passes on macOS and writes a valid summary.
- [x] Introducing a formatting error makes `fmt` fail while later steps still run (without
      `--fail-fast`). Exit code 1. Verified manually once and covered by orchestration tests.
- [x] A step whose tool is missing reports `skipped` with `reason` and `help`, and `ok` is false.
      Covered by unit test with fake runner.
- [x] `--only`/`--skip` with an unknown step name exits 2.
- [x] The JSON output is checked by a snapshot test against a fixture run with fake steps
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

2026-11-08, implementation session. Status → `review`.

**Done (all acceptance criteria met):**
- `xtask/src/ci.rs`: `STEPS` table (`fmt`, `clippy`, `test`, `doc` with the exact R012 commands;
  `doc` sets `RUSTDOCFLAGS=-D warnings` on the child only), option parsing (`--json`,
  `--fail-fast`, `--only`, `--skip`, `--allow-skip`; `--opt list` and `--opt=list`), the
  `StepRunner` trait, orchestration (`run_steps`), the summary (`Summary`/`StepReport`, serde
  derive, field order as in spec §5), the human report, and `execute`, which writes
  `target/xtask/ci-summary.json` under the repository root.
- `xtask/src/process.rs`: `ProcessRunner` spawns `$CARGO` (else `cargo`) in the repository root.
  A step may declare a `Tool` probe (`cargo fmt --version`, `cargo clippy --version`). A failing
  probe → `skipped` with `reason`/`help`, and so does a `cargo` that cannot start. `--json` mode
  captures stdout and stderr through one `std::io::pipe` (interleaved), and failed steps get
  `output_tail` (last 50 lines).
- `xtask/src/finding.rs`: `Finding`/`Severity` (spec §4) for M0-T003 onward. `Severity::Error`
  carries `#[cfg_attr(not(test), expect(dead_code))]` until the first check constructs it; the
  `expect` then fails the build and must be removed.
- `xtask/src/repo.rs`: `root()` (from `CARGO_MANIFEST_DIR`). The repo-model loader grows here.
- `cli.rs`: `ci` registered; new `CommandError::File` (exit 1) for the summary file.
- Spec `docs/specs/xtask.md` → `spec_version` 2: §5 "Summary details" (`output_tail`, fail-fast
  stopping rule, list option forms, empty selection = usage error).
- serde `derive` feature enabled in `[workspace.dependencies]`.

**Tests:** 47 in total (was 16). `src/ci/tests.rs` covers orchestration with a fake runner:
R003 (order + `STEPS` is a subsequence of the spec's final order, read from the spec file), R004,
R005 (incl. snapshot `xtask/tests/snapshots/ci-summary.json`, written by hand from §5), R006, R007,
R008, R010 (only `target/` is written) and R012 (exact argv/env). `process.rs` tests the real
runner against real `cargo` (pass, fail with tail, missing tool probe, missing program).
`tests/ci.rs` runs the binary for exit-2 cases.

**Verified (macOS 27.2, Apple M5 Pro, Rust 1.99.1):** `cargo xtask ci` passes on the clean
repository, also from `docs/specs/` (exit 0, summary written). With a formatting error added:
`cargo xtask ci --json --skip doc` → `fmt` fail (with diff in `output_tail`), `clippy` and `test`
still ran, exit 1, and stdout equal to the summary file. 12 single-line mutations of
`ci.rs`/`process.rs` (fail-fast logic, skip/allow-skip, selection, name validation, doc env, JSON
output, capture mode, probe, tail, stderr capture): the suite failed for each one.
Not verified: Windows and Linux (CI arrives in M0-T007). On Windows, `cargo xtask ci`'s `test` step
could need to relink the running `target/debug/xtask.exe`, which Windows forbids. This does not
happen on macOS because the binary is up to date; watch for it in M0-T007.

**Traps:**
- `serde_json::Value` sorts object keys (no `preserve_order`), so compare summary *text* for field
  order, not a re-serialized `Value`.
- `process.rs`'s inherit-mode test prints `cargo --version` into test output; that is expected.

**Next:** review (fresh session). After `done`: M0-T003, M0-T004, M0-T006, M0-T007 and M0-T009 are
unblocked; the first `todo` in order is M0-T003. Each adds its step to `STEPS` at its §5 position.

## Review

2026-11-08, review session (owner-started, fresh context). Outcome: **done**.

Checked (REVIEW.md §1):
1. Acceptance criteria, each verified by running on macOS 27.2 (Apple M5 Pro, Rust 1.99.1):
   `cargo xtask ci` exits 0 and writes a valid summary, also from `docs/specs/`. With a formatting
   error added to `xtask/src/repo.rs`: `cargo xtask ci --json --only fmt,doc` → `fmt` fail with
   `output_tail`, `doc` still ran and passed, exit 1, stdout byte-identical to
   `target/xtask/ci-summary.json`; `--fail-fast` stopped after `fmt` (summary lists only `fmt`).
   Unknown step names → exit 2 (`tests/ci.rs`). Missing tool → `skipped` with `reason`/`help`
   (fake-runner tests and the real-probe test in `process.rs`). Snapshot test compares exact text.
2. `cargo xtask ci`: pass (fmt, clippy, test, doc).
3. Tests: 7 own single-line mutations (different from the author's 12): 5 killed (`ok` any/all,
   fail-fast ignoring skips, skip counted as ok, stderr not captured, progress line in `--json`
   mode). 2 survived, both non-blocking: the human report's "N skipped without --allow-skip" count
   ignoring `--allow-skip` (display text only, no requirement), and removing `current_dir` from
   child commands (no observable effect: cargo finds the workspace from any subdirectory, and the
   alias only runs inside the repository). `Verifies:` tags match the requirements tested.
4. No placeholders, no weakened tests. The one new oracle (the snapshot) is justified in the journal.
5. Code matches spec v2 (§5 summary details, R003–R008, R010, R012). The history row is present.
6. Unsafe: none (`#![forbid(unsafe_code)]`).
7. Determinism: not applicable (tool crate). Summary field order is fixed.
8. Docs: items documented, `cargo doc` with `-D warnings` passes, STATUS updated.
9. Architecture: only `serde` (with `derive`) and `serde_json`, both approved for `xtask` with scope
   `tool` (ADR-0003). `serde_derive`/`syn` are transitive dependencies; for `tool` scope,
   DEPENDENCIES.md does not require a transitive dependency audit.
10. Session boundaries: implemented in an owner-started session. Per
    `2026-11-08-adr-0020-session-boundaries.md`, a review sub-agent was started and then stopped
    before it changed any file. This review is the first and only review.

Not verified: Windows and Linux (M0-T007). The Windows relink risk in the handoff still applies.
