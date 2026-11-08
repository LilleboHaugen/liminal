---
id: M0-T007
title: GitHub Actions CI
milestone: M0
status: review
crates: [xtask]
depends_on: [M0-T002]
requirements: []
---

# M0-T007: GitHub Actions CI

## Goal

Every push and pull request runs the quality gates on macOS, Windows and Linux. This proves
Tier 1 builds natively on all three operating systems (ADR-0005).

## Context

- ADR-0005 (Tier 1 targets, native builds per OS). `docs/specs/xtask.md` (step names).
- Prerequisite handled by the project owner: the GitHub repository exists and the code is pushed. If
  not, set this task to `blocked`.

## Scope

**In:** `.github/workflows/ci.yml` with:
- job `checks`, matrix `macos-latest` (arm64), `windows-latest`, `ubuntu-latest`:
  `cargo xtask ci`, skipping the steps that run in their own jobs (`--skip coverage,deny`). `--skip`
  rejects unknown step names (XTASK-R008), so name only steps that already exist when this task runs.
  When M0-T006/M0-T009 land later, they update this workflow.
- job `coverage` (ubuntu): install `cargo-llvm-cov`, then `cargo xtask ci --only coverage` (once
  M0-T006 lands; until then, leave this job out).
- job `deny` (ubuntu): install `cargo-deny`, then `cargo xtask ci --only deny` (once M0-T009 lands;
  until then, leave this job out).
- Upload `target/xtask/ci-summary.json` as an artifact from every job, with `if: always()` (the
  summary matters most on failure) and a unique name per job and OS, e.g.
  `ci-summary-${{ github.job }}-${{ matrix.os }}` (artifact names must be unique within a run).
- Rust caching. All third-party actions pinned by full commit SHA, with the version in a comment.
- Toolchain installed from `rust-toolchain.toml` (rustup reads it automatically).

**Out:** Miri job (added in M1 when the first unsafe crate exists), GPU jobs (M3), release workflows (M7).

## Acceptance criteria

- [x] CI is green on all three operating systems on `main`.
- [x] A test branch with a deliberate formatting error shows the `checks` job failing on all three.
      Link the run in the journal, then delete the branch.
- [x] Workflow uses least-privilege `permissions:` (`contents: read`).
- [x] Average wall time per job recorded in the journal (baseline for later CI-time budgets).

## Test requirements

None in code. Verification is the CI runs linked in the journal.

## Notes

- Windows line endings: `.gitattributes` (`* text=auto eol=lf`) exists from planning. Verify that
  `cargo fmt --check` passes on the Windows runner.
- Windows cannot overwrite a running `.exe`. If `cargo xtask ci` running `cargo test`/`clippy` needs to
  relink `xtask.exe` while it runs, the step fails. If this happens, build xtask first and run the
  built binary from a copy, or build test targets into a separate `--target-dir`. Record the chosen fix.
- Set `CARGO_TERM_COLOR=always` for readable logs, and `RUST_BACKTRACE=1`.

## Handoff

2026-11-08, implementation session (after the review of M0-T006 in the same session). Status →
`review`.

**Done (all acceptance criteria met):**
- `.github/workflows/ci.yml`: job `checks` (matrix `macos-latest` arm64, `windows-latest`,
  `ubuntu-latest`; `fail-fast: false`; `cargo xtask ci --skip coverage`), job `coverage` (ubuntu;
  `cargo install cargo-llvm-cov --version 0.9.1 --locked`, then `cargo xtask ci --only
  coverage`). Triggers: every `push` and `pull_request`; concurrency cancels superseded runs of
  the same ref and event. `permissions: contents: read`; checkout with
  `persist-credentials: false`. `ci-summary.json` uploaded from every job with `if: always()`
  as `ci-summary-<job>-<os>`. Toolchain: `rustup toolchain install` (reads
  `rust-toolchain.toml`). Caching: `Swatinem/rust-cache` (also caches `~/.cargo/bin`, so the
  `cargo-llvm-cov` install is a no-op on a warm cache). Env: `CARGO_TERM_COLOR=always`,
  `RUST_BACKTRACE=1`. Pinned by commit SHA: `actions/checkout` v7.0.1,
  `actions/upload-artifact` v7.0.2, `Swatinem/rust-cache` v2.9.3 (annotated tag; peeled
  commit). How to re-pin is in the workflow header.
- Windows fix: `process::tests::native_checks_run_on_the_root_and_map_findings_and_errors`
  expected `/repo/a.md` while its check function builds the location with `Path::join`
  (`\` on Windows). The expected value is now built the same way; the test still verifies that
  the runner passes the root to the check. The only Windows failure; no product code changed.
- The `.exe` relink problem from the Notes did not occur: `cargo test` and `cargo clippy` in
  the running `xtask.exe` did not need to relink it. No workaround added.
- `.gitattributes` (`eol=lf`) works: `fmt` passes on the Windows runner.
- M0-T009 handoff: note to add the `deny` job and `--skip coverage,deny`.

**Merged to `main` before review:** acceptance criterion 1 ("green on `main`") needs the
workflow on `main`. Merged at 027dbcf, run below. If review requires changes, they follow on
the task branch and are merged again.

**Runs** (`https://github.com/LilleboHaugen/rust-game-engine/actions/runs/<id>`):

| Run | Ref | Result |
|---|---|---|
| 42529862659 | task branch, 0953417 | cancelled by the next push (concurrency) |
| 42529978816 | task branch, c1e6cc6 | `checks` windows failed (test above); others pass |
| 42530266416 | task branch, Windows fix | all 4 jobs pass |
| 42530471920 | `test/M0-T007-fmt-error` (deliberate fmt error in `xtask/src/lib.rs`) | `checks` fails on all three, `fmt: FAIL` on each; `coverage` passes (no fmt step). Branch deleted. |
| 42530730963 | `main`, 027dbcf | all 4 jobs pass |

**Wall time per job** (seconds, job start to end, the four completed runs above; every run ran
all steps, because `ci` continues after a failing step):

| Job | Runs | Mean | Range |
|---|---|---|---|
| checks (macos-latest) | 29, 24, 32, 41 | 32 | 24–41 |
| checks (ubuntu-latest) | 44, 38, 57, 59 | 50 | 38–59 |
| checks (windows-latest) | 79, 123, 105, 113 | 105 | 79–123 |
| coverage (ubuntu-latest) | 135, 35, 128, 125 | 106 | 35–135 |

`coverage` takes about 35 s with a warm cache of the same branch and about 130 s cold (new
branch: `cargo install cargo-llvm-cov` from source). Caches are per branch (GitHub scoping), with
fallback to `main`'s cache once `main` has one.

**Not verified:** a `pull_request` run (no pull request opened; the `push` trigger covers every
branch). GitHub announced that `ubuntu-latest` moves to Ubuntu 26 (run annotation); no action
needed now.

**Next:** review (fresh session). Then the first `todo` in M0 with all dependencies done:
M0-T008 (status generator).

## Review

_Empty._
