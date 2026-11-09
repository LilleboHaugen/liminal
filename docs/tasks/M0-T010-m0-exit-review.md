---
id: M0-T010
title: M0 exit review
milestone: M0
status: review
crates: [xtask]
depends_on: [M0-T001, M0-T002, M0-T003, M0-T004, M0-T005, M0-T006, M0-T007, M0-T008, M0-T009, M0-T011]
requirements: []
---

# M0-T010: M0 exit review

## Goal

An independent session confirms that every M0 exit criterion holds and that `xtask` meets the `usable`
gates. After this, M1 can start on a trustworthy foundation.

## Context

`docs/ROADMAP.md` §M0 exit criteria. `docs/process/MATURITY.md` (`usable` gates).
`docs/process/REVIEW.md` §3. MUST be done by a session that implemented none of M0-T002 … M0-T009,
or if that is impossible, one that starts from a fresh context and states this in the review.

## Scope

**In:** verify each M0 exit criterion with evidence; seed one violation per check on a scratch branch
and confirm each is caught (then discard the branch); promote `xtask` to `usable` if every gate holds;
update the ROADMAP checkboxes.

**Out:** writing M1 tasks (M1-T001).

## Acceptance criteria

- [x] Every M0 exit criterion in `docs/ROADMAP.md` checked off, with evidence links in this file's
      Review section. (1–4 checked 2026-11-09 in session 1; 5 checked 2026-11-09 in session 2.)
- [x] Seeded-violation run: fmt, clippy, test failure, doc warning, layer violation, unapproved
      dependency, missing `forbid(unsafe_code)`, broken doc link, invalid frontmatter, untested MUST in
      a `usable` spec, untracked TODO, coverage drop, stale STATUS.md, GPL dependency. Each caught by
      the expected step. Results in a table in the journal.
- [x] `docs/specs/xtask.md` reviewed as a spec (REVIEW.md §2), then its maturity set to `usable` in
      one promotion (allowed by MATURITY.md when all cumulative gates hold; tool-crate gates apply).
      If any gate fails, list the failing gates and turn them into tasks.
- [x] XTASK-R013 (SHOULD): measure the native checks' total time on the reference machine and record
      it in the journal. This is not a test assertion (TESTING.md forbids wall-clock assertions).

## Scope change (2026-11-09)

The `usable` promotion cannot happen yet: the tool-crate gate "every command is covered by an
integration test that runs the real binary and checks its exit code and output" fails (details in
"Review"). As the task scope says, the failing gate became a task: M0-T011, added to
`depends_on`. The promotion and the last ROADMAP checkbox move to the session that continues this
task after M0-T011 is `done`. That session MUST NOT be one that implemented M0-T011 (the same
independence rule as for M0-T002 … M0-T009 in "Context").

## Handoff

2026-11-09, first session (it reviewed M0-T009 first, its journal
`2026-11-09-M0-T009-review-2.md`; it implemented none of M0-T001 … M0-T009). Status:
`in-progress`, waiting for M0-T011.

**Done:** exit criteria 1–4 verified and checked in `docs/ROADMAP.md`; seeded-violation run (14
seeds, all caught; table in journal `2026-11-09-M0-T010.md`); R013 measured; spec review of
`docs/specs/xtask.md` (REVIEW.md §2): passes, §13 updated with gate evidence (`spec_version` 15);
`usable` gates checked, one fails → M0-T011.

**Remaining (next session, after M0-T011 is `done`):**
1. Re-check the tool-crate gate against M0-T011's tests (every §5 command run through the real
   binary; doc examples for the main types).
2. Re-run `cargo xtask ci`, `cargo xtask trace --crate xtask` and `cargo xtask coverage` on the
   current `main`; confirm the other gates in spec §13 still hold.
3. If all hold: set `maturity: usable` in `docs/specs/xtask.md` (and `docs/specs/README.md`,
   R027), check the §13 boxes, regenerate STATUS.md, check the last ROADMAP M0 box, fill in
   "Review", set this task to `review`.

The seeded-violation run does not need repeating unless M0-T011 changes a check.

2026-11-09, second session (journal `2026-11-09-M0-T010-2.md`). It reviewed M0-T011 first
(journal `2026-11-09-M0-T011-review.md`) and implemented none of M0-T001 … M0-T009, M0-T011.
Remaining items 1–3 done; see "Review", part 2. Set to `review`.

M0-T011 did not change a check (only tests, doc examples, the CI workflow and the spec), so the
seeded-violation run was not repeated.

Next: review of this task in a new session (ADR-0020). It checks the evidence in "Review" and the
promotion diff (`docs/specs/xtask.md` frontmatter and §13, `docs/specs/README.md`,
`docs/ROADMAP.md`). After `done`, M0 is complete and M1 starts with M1-T001 (kickoff).

## Review

Exit review evidence, 2026-11-09, macOS 27.2 on the reference machine (Apple M5 Pro, 24 GB),
Rust 1.99.1, cargo-llvm-cov 0.9.1, cargo-deny 0.20.2.

| ROADMAP M0 exit criterion | Result | Evidence |
|---|---|---|
| Workspace builds on macOS, Windows, Linux in CI | holds | GitHub Actions run 42669108746 on `main` 6f485a4: `checks` on macos-latest, windows-latest, ubuntu-latest, `coverage`, `deny` all pass |
| `ci` runs the 11 steps and prints a JSON summary | holds | `cargo xtask ci`: fmt, clippy, test, doc, check-arch, check-docs, trace, hygiene, status-check, coverage, deny, all pass; `--json` and `target/xtask/ci-summary.json` (R005) |
| Each check catches a seeded violation | holds | 14 seeds, each caught by the expected step (journal `2026-11-09-M0-T010.md`); per-rule fixture tests: `cargo xtask trace --crate xtask` MUST 49/49 |
| STATUS.md generated by `cargo xtask status` | holds | `status-check` step passes; the stale-STATUS seed fails it (R072) |
| `xtask` reaches `usable` | **not yet** | Tool-crate gate fails, see below; M0-T011 |

`usable` gates for `xtask` (MATURITY.md, cumulative, tool-crate variants):

| Gate | Result | Evidence |
|---|---|---|
| Spec uses the template and is reviewed (`experimental`) | holds | Section headings match `_TEMPLATE.md`; spec review in this session (journal) |
| Compiles, lints pass, behaviours tested, in `ci/coverage.toml` (`experimental`) | holds | `cargo xtask ci` passes; `ci/coverage.toml` `xtask = 97.6` |
| Every MUST traced, `trace` gate on | holds | With `maturity: usable` set on a scratch copy: `trace: ok`, MUST 49/49 |
| Coverage ≥ 75 % (tools) | holds | 97.39 % in M0-T009 (`coverage` step passes against floor 97.6) |
| Every public item documented | holds | `missing_docs = "deny"` (workspace lints), `doc` step with `-D warnings` passes |
| Doc examples for main types | partly | One doc test (`xtask::run`); none for `COMMANDS`/`Command`, `Exit` → M0-T011 item 3 |
| Every command has a real-binary integration test (exit code and output) | **fails** | Only `help`, unknown command/option and `ci` usage errors (`xtask/tests/cli.rs`, `xtask/tests/ci.rs`); none for `check-arch`, `check-docs`, `trace`, `hygiene`, `coverage`, `status`, `doctor` → M0-T011 |
| `unsafe`, determinism gates | n/a | `unsafe = "forbid"`, `deterministic = false` |
| Used in CI or the project workflow | holds | `cargo xtask ci` runs in every CI job |

Outcome: not complete. M0-T010 stays `in-progress` until M0-T011 is `done`.

### Part 2: promotion review (2026-11-09, second session)

This session implemented none of M0-T001 … M0-T009 and not M0-T011; it reviewed M0-T011 and set it
to `done` first. macOS 27.2, reference machine, Rust 1.99.1, cargo-llvm-cov 0.9.1, cargo-deny
0.20.2, on `main` 8c238f5 (M0-T011 merged).

| Gate | Result | Evidence |
|---|---|---|
| Doc examples for main types | holds | `cargo test --doc -p xtask`: 4 pass (`run`, `Exit`, `Command`, `COMMANDS`, `xtask/src/cli.rs`) |
| Every command has a real-binary integration test (exit code and output) | holds | `xtask/tests/cli.rs` (`help`), `ci.rs` (`ci`), `commands.rs` (other 7); 4 wiring mutants killed (M0-T011 "Review") |
| Every MUST traced, `trace` gate on | holds | `cargo xtask trace --crate xtask` with `maturity: usable` set in the spec: MUST 49/49, `trace: ok` |
| Coverage ≥ 75 % (tools) | holds | `cargo xtask coverage`: 98.05 % (5050/5150), floor 97.6 |
| Compiles, lints, docs, all checks | holds | `cargo xtask ci`: 11 steps pass, with the promotion applied |
| Workspace builds on 3 OS in CI | holds | Run 42674386561 attempt 2 (957b646): `checks` macOS, Windows, Linux, `coverage`, `deny` pass |
| Other gates (spec review, documented items, used in CI, `unsafe`/determinism n/a) | unchanged | Part 1 table; M0-T011 changed no source outside doc comments |

ROADMAP M0 exit criterion 5 (`xtask` reaches `usable`): holds; checked. Outcome: `xtask` promoted
`planned` → `usable` (`docs/specs/xtask.md` `spec_version` 17, `docs/specs/README.md`). All five M0
exit criteria are checked. Task set to `review`.
