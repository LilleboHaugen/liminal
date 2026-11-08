---
id: M0-T007
title: GitHub Actions CI
milestone: M0
status: todo
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

- [ ] CI is green on all three operating systems on `main`.
- [ ] A test branch with a deliberate formatting error shows the `checks` job failing on all three.
      Link the run in the journal, then delete the branch.
- [ ] Workflow uses least-privilege `permissions:` (`contents: read`).
- [ ] Average wall time per job recorded in the journal (baseline for later CI-time budgets).

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

_Empty._

## Review

_Empty._
