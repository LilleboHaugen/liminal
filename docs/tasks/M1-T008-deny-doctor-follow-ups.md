---
id: M1-T008
title: deny policy and doctor follow-ups from the M0-T009 review
milestone: M1
status: in-progress
crates: [xtask]
depends_on: [M0-T009]
requirements: [XTASK-R080, XTASK-R081, XTASK-R082, XTASK-R085]
---

# M1-T008: `deny` policy and `doctor` follow-ups from the M0-T009 review

## Goal

The non-blocking findings of the M0-T009 review are resolved: the remaining untested parts of
`deny.toml` and of the `doctor` probes are either tested or recorded as unchecked, and `doctor`
has been run on every CI platform.

## Context

- Follow-up task under ADR-0021, written by review 1 of M0-T009 (see that task's "Review").
- `docs/specs/xtask.md` "`doctor` and `deny` details" (`spec_version` 13), `deny.toml`,
  `xtask/src/doctor.rs`, `xtask/tests/deny_policy.rs`.
- No M0 exit criterion needs this, so it is in M1. The M1 kickoff (M1-T001) places it in the M1
  order.

## Scope

**In:**
1. **Probe stdin and stderr.** The spec says probes run "with stdin closed". `RealProber::command`
   sets `Stdio::null()` for stdin and stderr, but no test inspects either; removing them passes
   every test. Outcome changes only if a tool's `--version` reads stdin (none does today). Either
   assert it (e.g. a probe of a program that reads stdin returns instead of blocking) or record in
   the spec that it is not tested and why.
2. **`doctor` on Windows and Linux.** No CI job runs `doctor`, so its output there is unverified
   (handoff "Not verified"). Add `cargo xtask doctor --json` to the `checks` job (it exits 0 when
   only Miri is missing) or record a manual run on each OS.
3. **`confidence-threshold`.** Lowering it from 0.93 to 0.5 passes every test: the fixture uses
   SPDX `license` expressions, for which the threshold does not apply. It matters for crates that
   ship only a `license-file`. Add a fixture case with a `license-file` (e.g. a GPL text) or record
   that it is unchecked.
4. **Expiry of ignored advisories.** `deny.toml` requires each `[advisories] ignore` entry to carry
   `expires YYYY-MM-DD (<task ID>)` in its `reason`, but only review checks it. AGENTS.md: every
   claim checkable by a machine. Consider a native check (in `hygiene` or a new rule) that fails on
   a malformed or expired entry; spec it with a requirement ID if added.
5. **Coverage measurement drift.** The M0-T009 journal measured `main` before the task at 97.35 %
   (4888/5021), while the M0-T008 handoff reported 97.56 % (4879/5001) for nearly the same code.
   Find out why the totals differ (line counting, toolchain, test set) so that floor comparisons
   are trustworthy.

**Out:** auditing transitive `unsafe` (`cargo-geiger`, M3, as in M0-T009).

## Acceptance criteria

- [ ] Items 1–4 each tested, or the decision not to test recorded in the spec with the reason.
- [ ] `doctor` output recorded for macOS, Windows and Linux (CI log or journal).
- [ ] Item 5 explained in the journal; a floor or tool change, if any, justified there.
- [ ] Spec `spec_version` bumped if its text changes; `cargo xtask ci` passes.

## Test requirements

`Verifies:` tests for each behaviour added (R080 for probes, R082 for licence fixtures, a new ID
for an advisory-expiry check if one is added).

## Handoff

_No session has worked on this task yet._

## Review

_Empty._
