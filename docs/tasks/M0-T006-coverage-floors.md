---
id: M0-T006
title: Coverage with per-crate floors
milestone: M0
status: done
crates: [xtask]
depends_on: [M0-T002]
requirements: [XTASK-R009, XTASK-R010, XTASK-R060, XTASK-R061, XTASK-R062, XTASK-R063, XTASK-R064]
---

# M0-T006: Coverage with per-crate floors

## Goal

Per-crate line coverage is measured and can only go up (ratchet), as defined in ADR-0010.

## Context

- `docs/process/TESTING.md` §Coverage. `docs/specs/xtask.md` R060–R064.
- External tool: `cargo-llvm-cov` (needs the `llvm-tools` component, formerly `llvm-tools-preview`, pinned in M0-T001).

## Scope

**In:** `coverage` command; parsing `cargo llvm-cov --json --summary-only` output; mapping files to
crates by manifest directory; `ci/coverage.toml` format and the initial file (an `xtask` entry);
`--update-floors`; `--json` (R009); `coverage` step in `ci`. If M0-T007 has landed, add the `coverage`
job to `.github/workflows/ci.yml` and add `--skip coverage` to the `checks` job. Otherwise leave a note
in M0-T007's handoff section.

**Out:** coverage for doc tests (llvm-cov support for doc tests needs nightly; revisit in M1).

## Acceptance criteria

- [x] `ci/coverage.toml` exists with a documented format (comment header) and a floor for `xtask`
      set to its measured coverage, rounded down to 0.1.
- [x] Parsing is tested on a saved llvm-cov JSON fixture (no tool needed in tests).
- [x] Ratchet logic tests: drop of 0.4 points passes, drop of 0.6 fails, a missing crate entry fails
      with a help line, `--update-floors` never lowers a floor.
- [x] Missing `cargo-llvm-cov` → `skipped` with install hint.
- [x] `coverage` is a step in `ci`.
- [x] xtask coverage measured and recorded in the journal. If it is below 75% (the `usable` target for
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

## Decisions that differ from the task text

Written here so the reviewer can judge them (AGENTS.md rule 2). All are in spec v11
("`coverage` details").
1. **R060 command gets `--locked`.** Without it `cargo llvm-cov` may rewrite `Cargo.lock`, a
   repository file, which R010 forbids for check commands. Every other cargo step has it (R012).
2. **R062 also reports stale entries** (a floor for a crate that is not a workspace crate), at
   the entry's line. Otherwise a renamed crate leaves a dead floor and the new name's R062 help
   is the only hint.
3. **`--update-floors` never adds a missing entry.** R063 defines the update as
   `max(current floor, measured)`, which needs a current floor; a missing entry stays an R062
   finding whose help gives the line to add. It writes the file only if a floor rises, and
   replaces only the number (byte span of the TOML value), so comments are kept.
4. **A crate with no measured lines** (no instrumented code) is shown as "no lines measured", is
   never below its floor, and keeps its floor on update. Guard against the opposite failure: if
   the export has files but none lies inside the repository root (a differently spelled root),
   that is LIM-XTASK-0002, so crates never silently look unmeasured.
5. **The standalone command with the tool missing** prints `coverage: skipped: …` and `help:`
   and exits 1 (JSON: `skipped` object, `ok: false`). The task only defines the `ci` step's skip.
6. **New error code LIM-XTASK-0006:** `cargo llvm-cov` ran and failed (e.g. a failing test). In
   `--json` mode of `ci` its last 50 stderr lines are in the message (`output_tail`).

## Handoff

2026-11-08, implementation session (after review 2 of M0-T005 in the same session). Status →
`review`.

**Done (all acceptance criteria met):**
- `xtask/src/repo/coverage.rs` (loader, INV-3): `ci/coverage.toml` parsing (floors in tenths,
  with line and value span; LIM-XTASK-0004), llvm-cov export parsing (per-crate sums,
  LIM-XTASK-0002), and `measure` (probe, run, LIM-XTASK-0006).
- `xtask/src/coverage.rs`: pure rules (`evaluate` R061/R062, `raised_floors` and `apply_raises`
  R063) in integer arithmetic, the `coverage` command (`--json`, `--update-floors`), and the `ci`
  step (`run_step` with an injected measurement; `run_real_step`).
- `repo/workspace.rs::owner`: innermost member owning a file, now shared by `repo/sources.rs`
  (refactored; its nested-member test still passes) and the coverage loader.
- `ci`: `Action::Coverage`, step `coverage` after `hygiene` (final §5 order). `cli`: command
  registered. `process::CARGO_HELP` shared.
- `ci/coverage.toml`: documented header, `xtask = 97.6`.
- Fixture `xtask/tests/fixtures/coverage/llvm-cov-summary.json` (real export of this repository,
  root replaced by `/work/liminal`; README beside it).
- Spec v11, `docs/process/TESTING.md` §Coverage, M0-T007 handoff note (CI workflow job; M0-T007
  has not landed).

**Measured (macOS 27.2, Apple M5 Pro, Rust 1.99.1, cargo-llvm-cov 0.9.1):** xtask line coverage
97.68 % (4590 of 4699 lines), above the 75 % `usable` target for tools. Before this task: 98.37 %
(4220 of 4290). Uncovered new lines: `coverage::command` and `cargo()` (the thin command wrapper,
run only by the real binary) and the success path of `repo::coverage::measure` after the tool
ran (exercised by the real `ci` step, not by unit tests: running `cargo llvm-cov` inside the
test run would recurse). Line coverage counts the inline `#[cfg(test)]` modules too, which
raises the number.

**Tests:** 227 lib tests (was 196). Verifies: R060 (fixture, mapping, Windows paths, command),
R061 (0.4 passes, 0.5 passes, 0.6 and 0.51 fail; message), R062 (missing with help line,
stale), R063 (raise, never lower, no add, file rewrite keeps comments), R064 (probe fails,
cargo cannot start, command and step skip), R009 (`--json`), R010 (no write without
`--update-floors`; `ci` step never writes; `--locked`).

**Verified:** `cargo xtask ci` passes (9 steps, coverage ≈ 4 s). Mutation run: 30 scripted
mutants of the new code, each reverted, full `cargo test -p xtask --lib`: 25 killed; 2
equivalent (a redundant `count > 0` guard, now removed; `span.end` vs `span.start` of a one-line
TOML number); 2 test gaps, fixed (integer floor `100`; `78.44` rejected) and then killed; 1 not
applied at first (pattern typo; wrong skip reason), then killed.
Faults injected into the real repository, each reverted: floor 99.9 → `ci --json --only
coverage` fails with R061 at `ci/coverage.toml:14`; entry renamed to `gone` → two R062 findings;
`~/.cargo/bin/cargo-llvm-cov` renamed → `ci` step `skipped` with the install hint, command exits
1; a failing test file → step `fail` with LIM-XTASK-0006 and the stderr tail. Not verified:
Windows, Linux (M0-T007); the Windows path handling is unit-tested only.

**Traps:**
- `cargo llvm-cov` builds into `target/llvm-cov-target` (second instrumented build; slow once).
- A test that runs `cargo llvm-cov` must use a directory outside the repository (system temp):
  cargo walks up to the workspace `Cargo.toml` and would run the real measurement recursively.
- `cargo-llvm-cov` must be installed on every agent machine (`cargo install cargo-llvm-cov
  --locked`), or `ci` reports `coverage` as skipped and the task cannot go to `review`.

**Next:** review (fresh session). Then the first `todo` in M0 with all dependencies done:
M0-T007 (GitHub Actions CI).

## Review

2026-11-08, new owner-started session (fresh context; did not write M0-T006). Outcome:
**done**, no blocking findings. Follow-ups in M1-T005.

Checked, macOS 27.2, Apple M5 Pro, Rust 1.99.1, cargo-llvm-cov 0.9.1:
1. `cargo xtask ci`: pass, 9 steps, `coverage` 4.2 s. Working tree unchanged afterwards.
2. Acceptance criteria, each by reading or running: `ci/coverage.toml` has a format header and
   `xtask = 97.6`; `cargo xtask coverage` on the repository reproduces 97.68 % (4590 of 4699
   lines), so 97.6 is the measured value rounded down to 0.1. Fixture test parses the saved
   export (26 files, 4220 of 4290) without the tool. Ratchet tests: 0.4 and exactly 0.5 below
   pass, 0.6 and 0.51 below fail; missing entry fails with the `add `<crate> = …`` help;
   `--update-floors` never lowers (unit and file-level tests). Missing tool → `skipped` with
   `cargo install cargo-llvm-cov --locked`. `coverage` is the last `ci` step. Coverage is in the
   journal (above the 75 % target).
3. Code read in full: `xtask/src/coverage.rs`, `xtask/src/repo/coverage.rs`, both test files,
   the `ci.rs`, `ci/tests.rs`, `cli.rs`, `process.rs`, `repo/workspace.rs::owner` and
   `repo/sources.rs` diffs. Checked against spec v11 "`coverage` details" paragraph by
   paragraph: agrees. `cargo xtask coverage --update-floors` on the repository writes nothing
   (no floor rises).
4. Own mutants (4, each reverted, `cargo test -p xtask --lib`): tolerance 5 → 6, innermost →
   outermost owner, `>` → `!=` in `raised_floors`, removing the "no file inside the root" guard.
   All killed.
5. The six "decisions that differ from the task text" are accepted: each is in spec v11, and
   each is stricter than or a necessary completion of the task text (`--locked` follows R010).
6. No placeholders, no weakened tests. Oracle changes (`ci/coverage.toml` new; `ci/tests.rs`
   step list) are justified in the journal. No `unwrap`/`expect` in non-test code except
   `expect("invariant: …")`. No unsafe. Determinism: n/a. No new dependency.
7. Session boundaries: the M0-T006 journal shows an owner-started session (after review 2 of
   M0-T005), no sub-agents.
8. Note, not a finding: the implementation session installed `cargo-llvm-cov` without asking.
   ADR-0022 (owner approval before installs) was committed on `main` while that session ran and
   is not on the task branch, so the rule was not in force for it.

Follow-ups (non-blocking, ADR-0021) in `docs/tasks/M1-T005-coverage-follow-ups.md`:
1. A crate with a floor above 0 whose files all drop out of the export (e.g. a member path the
   owner lookup does not match) passes silently ("no lines measured" is never below a floor).
2. A member path spelled `./name` in the root `members` list is kept as `./name` by
   `repo/workspace.rs::expand` (only a trailing `/` is trimmed), so `owner` matches no file of
   that crate (affects `coverage` and the `trace`/`hygiene` source collection). Together with
   item 1 this disables the crate's ratchet silently.
