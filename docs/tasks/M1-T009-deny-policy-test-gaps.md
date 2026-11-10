---
id: M1-T009
title: deny policy test gaps from review 2 of M0-T009
milestone: M1
status: done
crates: [xtask]
depends_on: [M0-T009]
requirements: [XTASK-R083, XTASK-R084]
---

# M1-T009: `deny` policy test gaps from review 2 of M0-T009

## Goal

The two parts of the `deny.toml` sources and advisory policy that the R083/R084 tests do not yet
distinguish are either tested or recorded in the spec as unchecked, with the reason.

## Context

- Follow-up task under ADR-0021, written by review 2 of M0-T009 (see that task's "Review").
- `docs/specs/xtask.md` R083, R084 and "`doctor` and `deny` details" (`spec_version` 14),
  `deny.toml`, `xtask/tests/deny_policy.rs` (`git_dependencies_are_rejected`,
  `check_advisories`, `unmaintained_and_vulnerable_crates_are_rejected`).
- Related follow-ups of review 1: `docs/tasks/M1-T008-deny-doctor-follow-ups.md`. The two tasks
  touch the same test file; doing them in one session is fine.
- No M0 exit criterion needs this, so it is in M1. The M1 kickoff (M1-T001) places it in the M1
  order.

## Scope

**In:**
1. **Unmaintained direct dependencies.** Mutant `unmaintained = "transitive"` in `deny.toml`
   passes every test: both advised crates (`itoa`, `memchr`) are transitive in the fixture
   (app → `fixture_middle` → advised). With that mutant, a workspace crate that adds a direct
   dependency on an unmaintained crate is accepted. Add a case where the app itself depends
   directly on a crate with an `unmaintained` advisory and assert it is rejected (together with
   the existing transitive case this distinguishes `"all"` from `"workspace"` and
   `"transitive"`).
2. **Non-crates.io registry.** Mutant `unknown-registry = "allow"` passes every test. The spec
   (v14, R083 test approach) records that it is not tested because a local registry needs a
   registry server or index setup. Check whether cargo-deny 0.20.2 rejects a dependency from a
   local `file://` sparse or git index registry offline (cargo supports `registries.<name>.index =
   "file://…"` with a git index the test creates). If it works without network, add the case;
   otherwise keep the spec note and add the reason found.

**Out:** the items of M1-T008.

## Acceptance criteria

- [x] Item 1: a test case rejects a direct dependency with an `unmaintained` advisory; the mutant
      `unmaintained = "transitive"` fails a test (record in the journal).
- [x] Item 2: tested with mutant `unknown-registry = "allow"` killed, or the spec says why not.
- [x] Spec `spec_version` bumped if its text changes; `cargo xtask ci` passes; the `deny` CI job
      runs the changed tests.

## Test requirements

`Verifies: XTASK-R084` for item 1 and `Verifies: XTASK-R083` for item 2, if added.

## Handoff

2026-11-09, implementation sub-agent (parent session branch `session/2026-11-09-m1-subagents`).
Status `review`. Everything in scope is done; nothing is left open.

- Item 1 (`xtask/tests/deny_policy.rs`): `check_advisories` takes a `Graph` (which advised crates
  the app depends on directly, which only through `fixture_middle`).
  `unmaintained_and_vulnerable_crates_are_rejected` now also runs a passing control with `itoa`
  as a direct dependency and rejects an `unmaintained` advisory on that direct `itoa`
  (`RUSTSEC-2026-9003`). Mutants run by hand: `unmaintained = "transitive"` fails ("a direct
  unmaintained crate was accepted"), `unmaintained = "workspace"` fails ("a transitive
  unmaintained crate was accepted").
- Item 2: cargo-deny 0.20.2 does reject an offline local registry. New test
  `other_registry_dependencies_are_rejected` builds a git index at a `file://` URL (with
  `config.json` `dl` pointing at a `file://` directory) holding `itoa` at its locked version; the
  `.crate` file is copied from the local crates.io cache and its checksum taken from `Cargo.lock`.
  The registry is named through `CARGO_REGISTRIES_FIXTURE_INDEX` (cargo reads config files from
  the working directory, not the manifest directory), and `CARGO_HOME` is a temporary directory
  so the fixture registry does not land in the user's cache. Mutant `unknown-registry = "allow"`
  fails the test ("another registry was accepted").
- Spec `docs/specs/xtask.md` → `spec_version` 19 (R083/R084 test approach, §10 table, history).
  CI workflow comment above the `deny_policy` step updated (R082–R084). The `deny` CI job already
  runs every ignored test of `deny_policy.rs`, so it runs both changed tests.

Verified: `cargo xtask ci` passes on macOS (darwin 27.2, cargo-deny 0.20.2);
`cargo test -p xtask --test deny_policy -- --ignored` passes (5 tests). Branch CI run 42695532856
(commit b604012) passed every job, including `deny (ubuntu-latest)`, which runs the changed
tests. The registry test changes `CARGO_HOME`, so `cargo deny` must be found on `PATH` (it is on
GitHub runners and with Homebrew).

Trap: the existing `git_dependencies_are_rejected` test still writes a checkout of its fixture
repository into the user's `$CARGO_HOME/git/` on every run (not in scope; the same
`CARGO_HOME` override would fix it).

## Review

2026-11-10, review sub-agent (parent: the session on branch `session/2026-11-10-m1-t014-review`;
ADR-0026, ADR-0030), rules of `origin/main` (`043f3e2`). Outcome: **done**, no blocking finding.

Checked:
- Acceptance criteria, by running: `cargo test -p xtask --test deny_policy -- --ignored` passes
  (5 tests, cargo-deny 0.20.2). `deny.toml` mutants, each run and reverted:
  `unmaintained = "transitive"` fails ("a direct unmaintained crate was accepted"),
  `unmaintained = "workspace"` fails ("a transitive unmaintained crate was accepted"),
  `unknown-registry = "allow"` fails ("another registry was accepted"). The `deny` job of
  `.github/workflows/ci.yml` runs `--test deny_policy -- --ignored`, so both changed tests.
- `cargo xtask ci` on the task branch with `origin/main` (`043f3e2`) merged in: pass, 12 of 12
  steps, macOS 27.2 (arm64). Merge conflicts: `docs/STATUS.md` (regenerated) and
  `docs/specs/xtask.md`, where main had reached `spec_version` 24; this task's history row and
  the two "added in `spec_version`" notes in the R083/R084 test approach are now 26 (the
  handoff above says 19). A second merge of `origin/main` (`70dac69`, M1-T008 merged meanwhile
  and took 25) renumbered this change to 26, kept both §10 rows, and added the `env` argument
  (`&[]`) to M1-T008's new `run_deny` call in `check_licence_file`. On that tree:
  `deny_policy -- --ignored` 6 passed, the three mutants again each fail a test, `cargo xtask ci`
  passes (12 of 12).
- Tests: the registry test asserts `error[source-not-allowed]` and the crate name, so a failure
  of cargo metadata or of the fixture cannot pass it; each advisory case has a passing control.
  `Verifies:` tags match (R083 registry, R084 advisories).
- Spec agreement (R083/R084 test approach, §10 table), no oracle changed, no test weakened, no
  new dependency, no unsafe, CI workflow change is a comment only (ADR-0028 Decision 4 holds).
- Session boundaries: the implementation sub-agent is row 10 of
  `docs/journal/2026-11-09-session-m1-subagents.md`; it reviewed nothing. Its manual experiment
  in `~/.cargo/registry` was removed (checked: only `index.crates.io-*` there). The review
  prompt is the ADR-0030 Decision 7 template verbatim.

Backlog entries: 1 (`docs/BACKLOG.md`, "M1-T009 — review 2026-11-10"). Follow-up task: none.
