---
id: M1-T008
title: deny policy and doctor follow-ups from the M0-T009 review
milestone: M1
status: done
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

- [x] Items 1–4 each tested, or the decision not to test recorded in the spec with the reason.
- [x] `doctor` output recorded for macOS, Windows and Linux (CI log or journal).
- [x] Item 5 explained in the journal; a floor or tool change, if any, justified there.
- [x] Spec `spec_version` bumped if its text changes; `cargo xtask ci` passes.

## Test requirements

`Verifies:` tests for each behaviour added (R080 for probes, R082 for licence fixtures, a new ID
for an advisory-expiry check if one is added).

## Handoff

2026-11-09, implementation sub-agent (journal `docs/journal/2026-11-09-M1-T008.md`). Status:
`review`.

**Done** (spec `docs/specs/xtask.md` v19, renumbered v25 at the review merge, "`doctor` and `deny` details"):
1. Probe stdin/stderr: `a_probe_that_reads_stdin_sees_it_closed` (R080,
   `xtask/src/doctor/tests.rs`) probes the test binary itself, which reads stdin to its end; it
   must return at once with 0 bytes and without its stderr in stdout. Inheriting stdin with an
   open pipe is killed (timeout). Removing `.stdin(Stdio::null())` or `.stderr(Stdio::null())`
   is not observable (`Command::output` closes stdin; stderr is unread); the spec says so.
2. `doctor` on every OS: `checks` jobs run `cargo xtask doctor --json` (exit 1 accepted, ≥ 2
   fails) and upload `doctor.json`. Output for macOS (local), Windows and Linux (CI run
   42692578387) is in the journal.
3. `confidence-threshold`: `a_licence_file_below_the_confidence_threshold_is_rejected` (R082,
   `xtask/tests/deny_policy.rs`): exact MIT `license-file` accepted; altered MIT (score 0.88)
   rejected; asserts 0.8 < score < policy threshold. Kills 0.5, 0.85 and removing the key.
4. Advisory expiry: new XTASK-R085 in `hygiene` (`xtask/src/hygiene/advisories.rs`): every
   `[advisories] ignore` entry of `deny.toml` needs a `reason` ending with
   `expires YYYY-MM-DD (<task ID>)`, a valid date after today (UTC), and an existing, not `done`
   task. Unit tests with a fixed date and fixture `r085_advisory_ignore`.
5. Coverage drift: explained in the journal (M0-T008's 4879/5001 is not reproducible from any
   commit; both commits measure 4888/5021 deterministically). No floor or tool change.

**Verified:** `cargo xtask ci` passes on macOS (11 steps); `deny_policy --ignored`: 5 passed;
coverage 98.14 % (5235/5334). CI run 42692578387 on this branch: `coverage`, `deny`, `checks` on Windows and Linux pass;
`checks (macos-latest)` was still queued for a runner at handoff (unverified; the reviewer should
check it with `gh run view 42692578387`).

**Traps:** branches M1-T004/T005/T006 also bump `docs/specs/xtask.md` `spec_version` and §14,
and M1-T004 likely touches `xtask/src/hygiene.rs`; renumber and keep both sides at merge.

## Review

2026-11-10, review sub-agent (ADR-0026, ADR-0030; journal `docs/journal/2026-11-10-M1-T008-review.md`).
Outcome: **done**, no blocking finding.

Checked:
- Acceptance criteria 1–4 by reading and running: `a_probe_that_reads_stdin_sees_it_closed`
  passes; `deny_policy -- --ignored` 5 passed; mutant `confidence-threshold = 0.85` fails the
  new test; R085 unit and fixture tests cover `<=` today, missing/bare reason, malformed suffix,
  invalid dates, unknown and `done` tasks; `run_check` integration is exercised by the fixture
  test. Spec v19 text matches `xtask/src/hygiene/advisories.rs`.
- `doctor` output: Windows log of CI run 42692578387 read (cargo-deny, cargo-llvm-cov missing,
  Miri warning, as in the journal); run 42694808041 on `e13eebe` green on all three OS, including
  `checks (macos-latest)`, which the handoff left unverified.
- Item 5 explanation in the journal; no floor or tool change.
- `.github/workflows/ci.yml`: no new action, no permission or secret change (ADR-0028).
- Session boundaries: implementation sub-agent #9 of `2026-11-09-session-m1-subagents.md`.
- Merge of `origin/main` (`7c1d7f8`): `docs/specs/xtask.md` conflicted; this task's version
  renumbered 19 → 25 (main reached 24), LIM-XTASK-0004 row lists both R039's clippy
  configuration and `deny.toml`; `docs/STATUS.md` regenerated. `cargo xtask ci` on the merged
  tree: 12 steps pass, macOS 27.2 arm64, Rust 1.99.1.

Backlog: 2 entries in `docs/BACKLOG.md` ("M1-T008 — review 2026-11-10"). Follow-up task: none.
