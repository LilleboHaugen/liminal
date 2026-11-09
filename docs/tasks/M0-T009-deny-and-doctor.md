---
id: M0-T009
title: cargo deny policy and doctor
milestone: M0
status: in-progress
crates: [xtask]
depends_on: [M0-T002]
requirements: [XTASK-R009, XTASK-R080, XTASK-R081, XTASK-R082]
---

# M0-T009: `cargo deny` policy and `doctor`

## Goal

Licence and security-advisory policy is enforced automatically, and agents can see in one command which
tools they are missing.

## Context

- `docs/process/DEPENDENCIES.md` §Licence policy, §External tools. ADR-0015.
- `docs/specs/xtask.md` R080, R081.

## Scope

**In:** `deny.toml` (licences per DEPENDENCIES.md; advisories: deny vulnerabilities and unmaintained
crates, with ignored advisories listed with reason and expiry; bans: warn on multiple versions;
sources: crates.io only); `deny` step in `ci`; `doctor` command; adding the `deny` job to CI
(M0-T007's workflow) if M0-T007 is done, otherwise add a note to M0-T007.

**Out:** auditing transitive `unsafe` (`cargo-geiger`); may be added for runtime dependencies at M3.

## Acceptance criteria

- [x] `cargo deny check` passes on the repository.
- [x] A fixture or test demonstrates that a GPL-licensed dependency would be rejected by `deny.toml`
      (for example, by running `cargo deny check licenses` against a fixture manifest in a temp
      directory; mark the test `#[ignore = "M0-T009: needs network"]` if it needs the crates.io index,
      and run it once manually, recorded in the journal).
- [x] `cargo xtask doctor` lists all tools in R080 with present/missing, version and install hint.
      JSON output with `--json`.
- [x] `deny` is a step in `ci`.

## Notes

- Use cargo-deny's **current** config schema (check its documentation at the time of the task; the
  `[advisories]` section has changed across releases, and older keys were removed). Do not copy old
  examples blindly.
- MPL-2.0 is allowed only for tool and dev scopes (DEPENDENCIES.md). cargo-deny cannot express scopes,
  so express it as per-crate `exceptions` entries, each with a comment naming the scope.
- Duplicate versions are `warn` (warnings do not fail `cargo deny check`).

## Test requirements

`Verifies:` tests for R080 (tool detection logic with a fake command runner) and R081 (argument vector
and skip behaviour).

## Handoff

2026-11-09, implementation session (after the review of M0-T008 in the same session). Status →
`review` once the CI run on `main` is recorded below.

**Done (all acceptance criteria met):**
- `deny.toml` for cargo-deny 0.20.2 (schema checked against `cargo deny init` of that version):
  licences per DEPENDENCIES.md (MPL-2.0 only through per-crate `exceptions`, none yet);
  `unmaintained = "all"`; vulnerabilities are always errors (the removed `vulnerability` key is
  rejected, verified); `yanked = "warn"`; ignored advisories need reason + expiry date in the
  `reason` (format in a comment; cargo-deny has no expiry field); `multiple-versions = "warn"`;
  sources: crates.io only. Unknown keys and invalid values fail the check (verified).
  `cargo deny --locked check` on the repository: advisories, bans, licenses, sources ok.
- `deny` step in `ci` (last, spec §5 order): `cargo deny --locked --config deny.toml check`,
  skipped with the doctor hint if `cargo deny --version` fails (`xtask/src/ci.rs`).
- `cargo xtask doctor [--json]` (`xtask/src/doctor.rs`): the 7 tools of R080 with present /
  missing, version, install hint. Missing tool needed by a `ci` step: error; not needed yet
  (Miri): warning, exit 0. Probes run with `RUSTUP_AUTO_INSTALL=0` so they never install
  (ADR-0022). `Severity::Warning` added (first check that reports one).
- R082 (new): licence policy tests `xtask/tests/deny_policy.rs` on fixture
  `xtask/tests/fixtures/deny/`: 6 copyleft licences rejected; every DEPENDENCIES.md licence
  accepted except the scoped MPL-2.0 (read from the doc, so doc and `deny.toml` cannot drift).
- CI workflow: `deny` job (ubuntu, `cargo install cargo-deny --version 0.20.2 --locked`, `ci
  --only deny`, then the policy tests with `--ignored`); `checks` uses `--skip coverage,deny`.
- Spec `docs/specs/xtask.md` v13: "`doctor` and `deny` details", R082, test plan rows.

**Deviation from the task text:** the criterion suggests `#[ignore = "M0-T009: needs network"]`
and one manual run. The fixture uses path crates only, so the tests need no network, only
`cargo-deny`. They are ignored in the `test` step (machines without cargo-deny) and run on every
push by the `deny` CI job, which is stronger than one manual run. Manual run on macOS recorded
in the journal.

**Tests:** 13 `doctor` tests (fake prober; real command; probe command line with
`RUSTUP_AUTO_INSTALL=0` in the root), 1 `ci` test for the deny step and its skip, 3 policy
tests. Own mutants (each reverted), all killed: `deny.toml` allows GPL-3.0-only; drops Zlib;
allows MPL-2.0 globally; doctor: every missing tool a warning; no `RUSTUP_AUTO_INSTALL`;
version = last line; warnings fail `doctor`.

**Coverage:** `xtask` 97.39 % (5016 of 5150) against floor 97.6: passes within the 0.5-point
tolerance; floor unchanged. `main` before this task measures 97.35 % (4888 of 5021) on the same
machine, so this task does not lower it.

**Installed (owner approval, ADR-0022):** cargo-deny 0.20.2, installed by the owner with
Homebrew (`/opt/homebrew/bin/cargo-deny`).

**Not verified:** Windows and Linux `doctor` output (CI runs no `doctor` step; the `deny` job is
Linux only).

Note from M0-T007 (2026-11-08): `.github/workflows/ci.yml` exists. Add a `deny` job modelled on
the `coverage` job (ubuntu; install `cargo-deny` pinned with `--version … --locked`, then
`cargo xtask ci --only deny`, summary artifact `ci-summary-deny-ubuntu-latest`), and change the
`checks` job to `--skip coverage,deny`.

## Review

_Empty._
