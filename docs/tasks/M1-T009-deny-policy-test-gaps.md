---
id: M1-T009
title: deny policy test gaps from review 2 of M0-T009
milestone: M1
status: review
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

_Empty._
