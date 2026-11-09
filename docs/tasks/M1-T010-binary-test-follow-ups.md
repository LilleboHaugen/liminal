---
id: M1-T010
title: Real-binary test and stale-root follow-ups from the review of M0-T011
milestone: M1
status: review
crates: [xtask]
depends_on: [M0-T011]
requirements: [XTASK-R001, XTASK-R014, XTASK-R072, XTASK-R080]
---

# M1-T010: Real-binary test and stale-root follow-ups from the review of M0-T011

## Goal

The non-blocking findings of the M0-T011 review are fixed or recorded: a moved checkout gives
one clear error instead of misleading ones, and two real-binary tests check what their names and
tags claim.

## Context

- Follow-up task under ADR-0021, written by the review of M0-T011 (see that task's "Review").
- `xtask/src/repo.rs` `root()`: the repository root is `CARGO_MANIFEST_DIR/..`, fixed at compile
  time (`env!`). `xtask/tests/commands.rs`, `docs/specs/xtask.md` R001, R072, R080, §8.
- No M0 exit criterion needs this, so it is in M1. The M1 kickoff (M1-T001) places it in the M1
  order.

## Scope

**In:**
1. **Moved checkout (observed in the review session, 2026-11-09).** The owner renamed the
   checkout directory (`…/dev/rust-game-engine` → `…/dev/liminal`). `cargo xtask ci` did not
   rebuild `xtask` (sources unchanged), so the binary's compiled-in root named a directory that no
   longer existed. Every step failed or skipped, with misleading messages:
   - `could not start `…/toolchains/1.99.1-aarch64-apple-darwin/bin/cargo`: No such file or
     directory (os error 2)`. The cargo binary exists; the missing path is the working directory
     the process is spawned in.
   - `LIM-XTASK-0001: : could not be read: No such file or directory`: the path is empty.
   `cargo clean -p xtask` fixed it. Required: when the compiled-in root does not exist (or has
   no `architecture.toml`), every command reports one internal error that names the root and the
   fix (`cargo clean -p xtask`, or rebuild), exit 1. Alternatively, find the root at run time
   (e.g. from the current directory, as `cargo` does) if the spec owner prefers that; either
   way, specify it under R001 or §8 first. Also fix the empty path in the LIM-XTASK-0001 message,
   and do not report a missing working directory as "could not start <program>".
2. **R072 tag.** `status_check_passes_and_status_leaves_an_up_to_date_file_unchanged` carries
   `Verifies: XTASK-R072` but runs only the passing side of `status --check`. The failing side
   (stale `docs/STATUS.md` → exit 1) is tested only through the library. Either drop the tag
   from the binary test, or add a binary run of the failing side if one is possible without
   modifying the repository (R010).
3. **`doctor` human output.** `doctor_reports_every_tool_whatever_is_installed` checks only
   the first word of each line, and `missing` for missing tools. For present tools, also assert
   that the line contains the version from the `--json` report, so a human-mode line that drops
   or swaps the version fails a test.

**Out:** anything already in M1-T002 … M1-T009.

## Acceptance criteria

- [x] Item 1: specified in `docs/specs/xtask.md` (`spec_version` bumped); a test runs a command
      with a root that does not exist and asserts the single error and exit code; the empty-path
      message is fixed.
- [x] Item 2: the tag matches what the test checks.
- [x] Item 3: a mutant that prints a present tool's line without its version fails a test
      (record in the journal).
- [x] `cargo xtask ci` passes.

## Test requirements

`Verifies: XTASK-R001` (or the new requirement ID) for item 1; `Verifies: XTASK-R080` stays on
the `doctor` test.

## Handoff

Implementation sub-agent session, 2026-11-09. Status `review`.

Done:
- **Item 1.** Kept the compile-time root (R001) and added a check before it is used: new
  **XTASK-R014** and **LIM-XTASK-0007** in `docs/specs/xtask.md` (`spec_version` 20, §6, §8,
  §10, §14). `xtask::run` now calls `cli::run_in(&repo::root(), …)`, which for every command
  except `help` runs `repo::check_root` first (root is a directory with `architecture.toml`);
  on failure it prints one line `error: LIM-XTASK-0007: <root>: … (help: … run `cargo clean -p
  xtask` …)`, nothing on stdout, exit 1. Run-time root discovery from the current directory was
  not chosen: it changes which repository a command checks and needs a rule for nested
  fixtures; the check is smaller and keeps R001 as is.
  - Empty path: `repo::walk` names the root itself by its absolute path (`xtask/src/repo.rs`).
  - Missing working directory: `process::start_error` reports `working directory `<path>` does
    not exist` instead of `could not start <program>`; used by `RealRunner`, the `cargo metadata`
    loader and the `cargo llvm-cov` loader.
  - Tests: `cli::tests::every_command_but_help_reports_one_error_for_a_missing_root`,
    `cli::tests::a_root_without_architecture_toml_is_rejected_and_help_still_works`,
    `repo::tests::check_root_accepts_the_real_root_and_names_a_missing_one` (all
    `Verifies: XTASK-R014`), `repo::tests::an_unreadable_root_is_named_by_its_absolute_path`,
    `process::tests::a_missing_working_directory_is_not_reported_as_a_missing_program`.
  - The R014 tests are library tests through `run_in`: the binary's compiled-in root cannot be
    changed without rebuilding it. Checked by hand once: built xtask in a copy of the
    repository under the scratchpad, renamed the copy, ran the binary: `check-docs` printed the
    single LIM-XTASK-0007 line and exited 1; `help` exited 0.
- **Item 2.** Dropped `Verifies: XTASK-R072` from the binary test
  `status_check_passes_and_status_leaves_an_up_to_date_file_unchanged` and wrote why in its doc
  comment: the binary cannot see a stale `docs/STATUS.md` without changing the repository
  (R010). R072 stays verified on both sides by `src/status/tests.rs`.
- **Item 3.** `doctor_reports_every_tool_whatever_is_installed` now asserts the whole state after
  the tool name: `missing`, the `--json` `version`, or `present (no version output)`.

Mutants (all restored afterwards):
- `tool_lines` printing `present` instead of the version: the binary `doctor` test fails.
- `run_in` checking the compiled-in root instead of the given one: both R014 cli tests fail.

Trap: the R014 cli tests pass an option no command accepts (`--xtask-test-bogus`), so if the
root check is ever removed, the commands fail with a usage error instead of really running
`ci`/`coverage` (an earlier version of the test used `--json` and, under the mutant, started a
nested `ci` run).

`cargo xtask ci`: pass on macOS (all 11 steps, none skipped); `xtask` coverage 98.13 %.

## Review

_Empty._
