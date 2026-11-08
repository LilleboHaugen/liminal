---
id: M0-T009
title: cargo deny policy and doctor
milestone: M0
status: todo
crates: [xtask]
depends_on: [M0-T002]
requirements: [XTASK-R009, XTASK-R080, XTASK-R081]
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

- [ ] `cargo deny check` passes on the repository.
- [ ] A fixture or test demonstrates that a GPL-licensed dependency would be rejected by `deny.toml`
      (for example, by running `cargo deny check licenses` against a fixture manifest in a temp
      directory; mark the test `#[ignore = "M0-T009: needs network"]` if it needs the crates.io index,
      and run it once manually, recorded in the journal).
- [ ] `cargo xtask doctor` lists all tools in R080 with present/missing, version and install hint.
      JSON output with `--json`.
- [ ] `deny` is a step in `ci`.

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

_Empty._

## Review

_Empty._
