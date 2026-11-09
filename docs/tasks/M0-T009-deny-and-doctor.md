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
`review`.

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

**CI:** run 42662952356 on `main` (564be70): all 5 jobs pass (checks on macOS, Windows,
Linux; coverage; deny). The `deny` job log shows `deny  pass` and both policy tests
(`copyleft_dependencies_are_rejected`, `the_documented_allow_list_is_exactly_what_deny_toml_accepts`)
`ok`. `deny` job time 3 min 52 s (first run: `cargo install cargo-deny` compiles; later runs use
the cache).

**Next:** review (fresh session). Then M0-T010 (M0 exit review).

**Not verified:** Windows and Linux `doctor` output (CI runs no `doctor` step; the `deny` job is
Linux only).

Note from M0-T007 (2026-11-08): `.github/workflows/ci.yml` exists. Add a `deny` job modelled on
the `coverage` job (ubuntu; install `cargo-deny` pinned with `--version … --locked`, then
`cargo xtask ci --only deny`, summary artifact `ci-summary-deny-ubuntu-latest`), and change the
`checks` job to `--skip coverage,deny`.

## Review

2026-11-09, new owner-started session (fresh context; did not write M0-T009). Outcome:
**changes required** (first "changes required" outcome of this task). Two blocking findings,
both B4. Follow-ups in `docs/tasks/M1-T008-deny-doctor-follow-ups.md`.

Checked, macOS 27.2, Apple M5 Pro, Rust 1.99.1, cargo-llvm-cov 0.9.1, cargo-deny 0.20.2:
1. `cargo xtask ci`: pass, 11 steps (`deny` 1.3 s). `cargo xtask doctor`: 6 tools present,
   `miri` missing (warning), exit 0. Policy tests with `--include-ignored`: 3 passed. Working
   tree unchanged afterwards.
2. Acceptance criteria, each by running or reading: `cargo deny check` passes on the
   repository; the GPL fixture is rejected (test run, and mutants below); `doctor` lists the 7
   tools of R080 with present/missing, version and hint, `--json` matches the spec shape;
   `deny` is the last `ci` step with the spec command line. The deviation (path-crate fixture
   with `--offline`, run on every push by the `deny` job instead of once by hand) is accepted:
   it is stronger than the task text and recorded in the spec.
3. Code read in full: `deny.toml`, `xtask/src/doctor.rs` and its tests, `xtask/tests/deny_policy.rs`
   and fixture, the `ci.rs`, `ci/tests.rs`, `cli.rs`, `finding.rs` and workflow diffs. Checked
   against spec v13 "`doctor` and `deny` details": agrees. `Severity::Warning` is only emitted by
   `doctor`, whose exit code ignores warnings as specified.
4. Own mutants (each reverted, `cargo test -p xtask --all-features -- --include-ignored`):
   `deny.toml` drops `Unicode-3.0`: killed. Adds `LGPL-2.1-or-later`: killed.
   `[licenses.private] ignore = true`: killed (the fixture crates are private).
   `confidence-threshold = 0.5`: survived (follow-up 3). `unknown-git = "allow"`: survived
   (finding 1). `unknown-registry = "allow"`: survived (finding 1). `unmaintained = "none"`:
   survived (finding 2). In each surviving case `cargo deny check` on the repository also passes.
5. No placeholders, no weakened tests; the `ci/tests.rs` step-list change is justified in the
   journal. No `unwrap`/`expect` outside tests except `expect("invariant: …")`. No unsafe.
   Determinism: n/a. Architecture: no new crate dependency (cargo-deny is an external tool,
   listed in DEPENDENCIES.md §External tools). cargo-deny install recorded with owner approval
   (ADR-0022). Session boundaries: the M0-T009 journal shows an owner-started session after the
   M0-T008 review, no sub-agents.

Required changes:
1. **B4: the sources policy is untested.** Scope and handoff claim "sources: crates.io only".
   Mutants `unknown-git = "allow"` and `unknown-registry = "allow"` pass every test. Realistic
   input: an agent adds `foo = { git = "https://github.com/…" }` to a crate; with either mutant
   `cargo deny check` accepts it. Add a policy test that a git dependency is rejected by the
   repository `deny.toml`. Verified feasible without network: a fixture crate depending on
   `{ git = "file://<temp>/dep" }` (a `git init` + commit in a temp directory) makes
   `cargo deny --config deny.toml --manifest-path … check sources` report `sources FAILED`.
   (A non-crates.io registry test is welcome but not required if it needs network; say so.)
2. **B4: the advisory policy is untested.** Goal: "security-advisory policy is enforced
   automatically"; scope and handoff claim unmaintained crates and vulnerabilities are errors.
   Mutant `unmaintained = "none"` passes every test. Add a policy test with a fixture advisory
   database that makes the repository `deny.toml` reject (a) a crate with an `unmaintained`
   advisory and (b) a crate with a vulnerability advisory. Verified feasible offline with
   cargo-deny 0.20.2: a copy of `deny.toml` with only `db-path` and `db-urls` added under
   `[advisories]` (`db-urls` must be `https://`; `file://` is rejected), the fixture database
   committed into a git repository at `<db-path>/advisory-db-<hash>` (the directory name
   cargo-deny derives from the URL; it prints it on a miss), `cargo deny --offline … check
   advisories`, and a crates.io dependency already in `Cargo.lock` (e.g. `itoa`, from the local
   registry cache). With `informational = "unmaintained"` the check reports
   `advisories FAILED`; with the mutant it reports `advisories ok`. Document the URL-hash
   dependency in the test (it may change with the cargo-deny version).
