---
id: M1-T009
title: deny policy test gaps from review 2 of M0-T009
milestone: M1
status: in-progress
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

- [ ] Item 1: a test case rejects a direct dependency with an `unmaintained` advisory; the mutant
      `unmaintained = "transitive"` fails a test (record in the journal).
- [ ] Item 2: tested with mutant `unknown-registry = "allow"` killed, or the spec says why not.
- [ ] Spec `spec_version` bumped if its text changes; `cargo xtask ci` passes; the `deny` CI job
      runs the changed tests.

## Test requirements

`Verifies: XTASK-R084` for item 1 and `Verifies: XTASK-R083` for item 2, if added.

## Handoff

_No session has worked on this task yet._

## Review

_Empty._
