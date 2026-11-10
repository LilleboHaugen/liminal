---
id: M1-T056
title: liminal_telemetry spec follow-ups from the M1-T027 review
milestone: M1
status: done
crates: [liminal_telemetry, xtask]
depends_on: [M1-T027]
requirements: []
---

# M1-T056: `liminal_telemetry` spec follow-ups from the M1-T027 review

## Goal

The `liminal_telemetry` spec leaves no point open to a wrong reading by its implementation tasks,
and `cargo xtask ci` runs the tests that exist only with the `telemetry` feature off.

## Context

- Follow-up task under ADR-0021, written by the review of M1-T027 (see that task's "Review").
- `docs/specs/liminal_telemetry.md` v1; ADR-0016; ADR-0018; `docs/process/PERFORMANCE.md`
  "Reference implementations reachable from benchmarks" (rule set by M1-T054, merged after the
  spec was written); `docs/specs/xtask.md` XTASK-R012; CODE_STYLE.md "Every feature must build
  alone".
- In M1, not M2: items 1 to 9 amend a contract that M1-T028 and M1-T029 implement, and item 10
  is needed before M1-T029 is reviewed (the ROADMAP M1 exit criteria need `liminal_telemetry` at
  `usable`). Best done before M1-T028 starts. If M1-T028 is already in progress or done, each
  spec item also checks the code.

## Scope

**In:**

1. **Provisional frame fields (§6 "Assignment").** The spec lets M1-T028 "fill `index`,
   `start_ns`, `end_ns`, `sim_steps` and `last_tick` provisionally". A task in `review` with
   provisional return values breaks AGENTS.md rule 1 (placeholder return values). Move TELE-R063
   and TELE-R064 to M1-T028 (spec table and both tasks' `requirements` frontmatter), or state
   that M1-T028 implements both fully, with their tests.
2. **Records per simulation step (ADR-0016 Decision).** ADR-0016 says frame records aggregate
   spans and counters "for each simulation step and rendered frame". The spec has one record per
   `end_frame` call and reports simulation steps only as a count. State how a per-step record is
   obtained (for example: the caller calls `end_frame` after each step with `sim_steps = 1`, and
   after rendering with `sim_steps = 0`), or list it as an open question for the `liminal_app`
   spec (M1-T044).
3. **ADR-0018 benchmark clause and the `reference` module.** ADR-0018: "Unsafe for performance
   requires a benchmark showing the safe version is insufficient, referenced in the commit or
   spec." The spec keeps the `Mutex<VecDeque>` reference in test code and lists no such
   benchmark. Either state why the clause does not apply (ADR-0016 requires lock-free buffers),
   or add a §9 case that compares the ring with the reference; in that case the reference is a
   `liminal_telemetry::reference` item listed in §5 (PERFORMANCE.md items 1 to 4).
4. **Span across two collectors (TELE-R011).** A span entered under collector A whose guard is
   dropped after A is dropped and collector B is installed was "active" at both moments, so by
   the letter it records into B with a `start_ns` before B's frame 0. TELE-R061 does not cover
   it (the record is not yet in a buffer). State the behaviour (for example: the guard remembers
   the collector it started under and records nothing under another), with a test.
5. **Size of a log record (TELE-R056, §4).** §4 says a log record "keeps its fields on the
   heap". `target` and `message` (2 × 16 bytes), `time_ns` (8), `level`, the enum tag and one
   pointer already exceed 48 bytes with 8-byte alignment. Say that the log payload as a whole is
   boxed (or whatever layout meets 48 bytes), and that a dropped or discarded log record frees
   its heap data (TELE-R052, TELE-R061, TELE-R065).
6. **Number formats (TELE-R082, TELE-R093).** "Integers are decimal without sign" reads as if it
   also covered `ContextValue::Int` field values, which CORE-R010 writes with a leading `-`.
   Limit the sentence to this crate's unsigned integers. In TELE-R093, an infinite gauge `min`
   or `max` is written as `null` by TELE-R082, the same output as "all values are NaN": state
   that this is intended, or take `min` and `max` over finite values only.
7. **Counter and gauge with the same name in the Chrome trace (TELE-R031, TELE-R081).** Both are
   written as `"ph":"C"` events with the same `pid` and `name`. Assumption (not checked in a
   viewer): the trace event format keys counter tracks by process, name and `id`, not by `cat`,
   so the two series share one track. Check it in Perfetto with TELE-R084, together with a
   `"value":null` counter event (NaN gauge); if confirmed, add an `id` or a name prefix.
8. **Summary `sim_steps` (TELE-R090).** State the type and overflow rule of the sum of the `u32`
   values (for example `u64`, cannot overflow for fewer than 2^32 frames, saturating beyond).
9. **Small text items.** (a) §10: the collector lock "in `tests/common/`" covers integration
   tests only; say what unit tests in `src/` do (and that other crates' test binaries that
   install a `Collector` need their own lock). (b) §9: two rows say "Not scheduled"; name the
   task or promotion review that adds them, or drop the rows (same finding as M1-T055 item 4).
   (c) `docs/specs/liminal_core.md` is on `main` now: turn the prose references into links;
   same for `docs/specs/liminal_memory.md` once it is merged.
10. **Feature-off build in `cargo xtask ci` (spec open question 2; `xtask`).** `cargo xtask ci`
    builds and tests with `--all-features` only (XTASK-R012). The test of TELE-R004 is compiled
    only under `cfg(not(feature = "telemetry"))`, so CI never runs it, while `cargo xtask trace`
    still counts TELE-R004 as tested. Add a build and test of crates with features, with the
    features off, to `cargo xtask ci` (new or changed XTASK requirement with a test, `xtask`
    `spec_version` bump), then close open question 2. If this part is too large for one
    session, split it into its own task and record that here.

**Out:** Any other change to the `liminal_telemetry` design. Code in `crates/liminal_telemetry`
(M1-T028, M1-T029), unless one of those tasks is already done and an item changes its contract.

## Acceptance criteria

- [x] Items 1 to 10 are done, or a declined or split-off item is written into this task with the
      reason. (Item 7: the viewer check is not done, see "Handoff"; item 9c: `liminal_memory`
      links not done, its spec is not on `main`.)
- [x] Spec changes bump `spec_version` with a history row; moved requirement IDs are in exactly
      one task's `requirements` frontmatter.
- [x] `cargo xtask ci` passes.

## Test requirements

Item 10: tests for the new or changed XTASK requirement. Items 1 to 9 change the spec only; the
requirements they touch are tested by M1-T028 and M1-T029.

## Handoff

Session 2026-11-10 (implementation sub-agent). Status `review`. M1-T028 and M1-T029 were both
`todo`, so no code was checked. `docs/specs/liminal_telemetry.md` is at `spec_version` 2,
`docs/specs/xtask.md` at 23.

| Item | Outcome |
|---|---|
| 1 | Done. TELE-R063 and TELE-R064 moved to M1-T028 (spec §6 "Assignment", both tasks' `requirements`, scope and notes). The word "provisionally" is gone; M1-T028 implements `end_frame` completely. |
| 2 | Done. Spec §4 "Frame" states both ways of calling `end_frame` (per main-loop iteration; per simulation step with `sim_steps: 1` plus once after rendering with `sim_steps: 0`). Which one `liminal_app` uses is open question 11 (M1-T044). No requirement changed: TELE-R063 and TELE-R064 already hold for every call. |
| 3 | Done, first option: the clause is stated not to apply (spec §10 "Ring buffer and its reference", pointers in §3 and §9). The reference stays private test code (PERFORMANCE.md item 3), so §5 has no `reference` module and §9 no comparison case. See "For the reviewer" 1. |
| 4 | Done. TELE-R011: a span records only into the collector it was entered under; mechanism "collector identity" in §4 (the value the one relaxed load reads); test described in the requirement. The requirement also states what happens to the open-span count. |
| 5 | Done. §4 "Record": the log payload is one `Box`; TELE-R056 says the payload is outside the 48 bytes; TELE-R052 (with a test), TELE-R061 and TELE-R065 say when a discarded log record's payload is freed. |
| 6 | Done. TELE-R082: the unsigned rule covers only this crate's own integers; field values follow CORE-R010 (leading `-`). TELE-R093: `min` and `max` over finite values only, `null` exactly when there is none; `last` is `null` when non-finite. §10 snapshot input gained a negative `Int`, an infinite gauge and a counter and gauge with one name. |
| 7 | **Partly done.** Not checked in Perfetto: no code writes a trace yet and this agent had no trace viewer. The spec keeps the format, states the assumption and both checks in open question 12 and TELE-R084, and names the change M1-T029 makes if the check confirms it (also in M1-T029 "Notes"). |
| 8 | Done. TELE-R090: the sum is `u64`, saturating at `u64::MAX`. |
| 9a | Done. §10 table: lock per test binary for integration tests, unit tests, doc tests and other crates' test binaries; poisoned-lock rule. |
| 9b | Done. The two "Not scheduled" rows are removed; §9 says the task that prepares the `stable` promotion adds them (same wording as M1-T055 used for `liminal_core`). |
| 9c | Done for `liminal_core` (three links). **Not done for `liminal_memory`:** `docs/specs/liminal_memory.md` is not on `main` (M1-T024 not merged), and a link to a missing file fails `check-docs`. The two "on its branch" references (§2, open question 7) stay; whoever next edits this spec after M1-T024 is merged turns them into links. |
| 10 | Done, not split off. New step `test-features-off` in `cargo xtask ci`, directly after `test` (XTASK-R014, `xtask/src/features_off.rs`, runner in `xtask/src/process.rs`). Spec open question 2 is closed; TELE-R004 and §13 name the step. |

Item 10 design (details in `docs/specs/xtask.md` "`test-features-off` details"):

- One `cargo test --package <name> --no-default-features --locked` per workspace package whose
  `features` object in `cargo metadata` is not empty (this includes implicit features of
  optional dependencies). Not one `--workspace` command: feature unification would let another
  member turn the feature on again.
- No package with features (the repository today): the step passes without running a test
  command. The real process path is still tested: a unit test writes a two-package workspace to
  a temporary directory and runs the real step on it (real `cargo metadata`, real `cargo test`).
- `repo::metadata::Package` gained `features`; `cargo metadata` output without a `features`
  member is LIM-XTASK-0002, never read as "no features".
- `doctor` lists the step among those that need `rustc` and `cargo`.
- `.github/workflows/ci.yml` is unchanged: the `checks` jobs run `cargo xtask ci --skip
  coverage,deny`, which includes the new step.

Verified: `cargo xtask ci` on the merge with `main` (02b19e7): pass, 12 steps, none skipped,
macOS 27.2 arm64, Rust 1.99.1; `xtask` coverage 98.1 % (floor 97.6, unchanged). Two seeded
mutants in `features_off.rs` (flag `--no-default-features` removed; filter "has features"
removed) each failed three tests, then were reverted. Windows and Linux: not run locally;
branch CI run 42865804835 on `39189ec` passed all 5 jobs (journal entry `2026-11-10-M1-T056.md`).

For the reviewer:

1. Item 3 is a judgment. The spec argues that the ring's `unsafe` implements ADR-0016's
   "lock-free per-thread buffers" and is not an optimization of a safe buffer that meets the
   contract. Two parts of the argument are marked in the spec as not proven: that no safe
   single-producer single-consumer ring is possible with `std`, and that `sync_channel` is
   excluded only by INV-5's "no read-modify-write atomic" rule, which is this spec's choice.
   The alternative (second option of item 3) needs the ring and its reference to be public.
2. Not covered by any command: each feature built alone (`--no-default-features --features x`,
   CODE_STYLE.md). Stated in the xtask spec and CODE_STYLE.md. No task was written for it
   (several sub-agents create tasks at once; a new task ID from here could collide). The
   follow-up task of this review can carry it.
3. `cargo llvm-cov` runs the new real-workspace test too, so a nested `cargo test` runs inside
   the coverage step. It passed here and the `xtask` percentage is computed as before; the
   `coverage` CI job on Linux is the second data point.
4. The `test-features-off` step stops at the first failing package (spec details). `ci` itself
   still runs the later steps (XTASK-R004).
5. M1-T029's title still says "frame stats"; only its scope and `requirements` changed, to
   keep the task index and file name stable.

## Review

2026-11-10, review sub-agent of parent session `2026-11-10-session-m1-reviews.md`, fresh context
(journal `2026-11-10-M1-T056-review.md`). This agent wrote no part of M1-T056. The author was an
implementation sub-agent of the same parent; ADR-0029 Decision 1 allows that (see "Session
boundaries"). macOS 27.2 (arm64), Rust 1.99.1, worktree `liminal-worktrees/M1-T056-review`,
branch at `f61f9d6` before the review commits. Rules read from `main` at `4c02a19` (ADR-0029
wording of `AGENTS.md` and `REVIEW.md`); the branch itself predated them.

Outcome: **done**, no blocking finding. Backlog entries: 6 (`docs/BACKLOG.md`, heading
"M1-T056 — review 2026-11-10"). Follow-up task: none (no finding is contract-changing).

Checked (REVIEW.md §1; the changed spec text also against §2):
1. Acceptance criteria, each item read in the diff `02b19e7..f61f9d6`:
   - Item 1: TELE-R063 and TELE-R064 are in the `requirements` of M1-T028 only (grep over
     `docs/tasks/`: besides M1-T028 they occur only in prose of M1-T029 and this task); spec §6
     "Assignment" agrees; no "provisional" permission is left.
   - Item 2: §4 "Frame" states both call patterns; open question 11 names M1-T044.
   - Item 3: first option of the task (clause stated not to apply), §10. The spec marks its two
     unproven parts as assumptions. Accepted as meeting the item; one backlog entry on the
     stated reason.
   - Item 4: TELE-R011 with the collector identity of §4 and a described test; consistent with
     TELE-R021 (the count is restored whether or not the guard records). One backlog entry.
   - Item 5: §4 "Record", TELE-R052 (with a test), R056, R061, R065.
   - Item 6: TELE-R082 and TELE-R093; no "non-NaN" wording is left; §10 snapshot input extended.
   - Item 7: not checked in a viewer, recorded with the reason in the acceptance criterion, the
     handoff, open question 12, TELE-R084 and M1-T029 "Notes". The item itself ties the check
     to TELE-R084, which is M1-T029's. Accepted as a recorded split-off.
   - Item 8: TELE-R090. Item 9a, 9b: §10 table, §9 paragraph. Item 9c: three `liminal_core`
     links; `liminal_memory` was not on `main` then (it is now: backlog entry).
   - Item 10: step `test-features-off`, XTASK-R014, details, `doctor`, CODE_STYLE.md; spec open
     question 2 closed.
   - `spec_version` bumps with history rows: telemetry 2; xtask 23, renumbered to 24 in the
     merge of `main` (M1-T011 took 23 there). The handoff's "at 23" predates that merge.
2. `cargo xtask ci` on `f61f9d6`: pass, 12 steps, none skipped. On the merge of `main`
   (`4c02a19`) with the review edits: pass, 12 steps, none skipped. Branch CI runs 42865804835
   (`39189ec`) and 42866951448 (`f61f9d6`): success (read with `gh run list`).
3. Tests prove behaviour. Four mutants seeded by this review, each killed and reverted: a
   skipped command treated as a pass (`a_command_that_cannot_run_skips_the_step` fails); a
   `cargo metadata` error treated as a pass
   (`real_step_without_cargo_is_skipped_and_a_metadata_error_fails_it`); `#[serde(default)]` on
   the `features` member (`unexpected_output_is_lim_xtask_0002`); no stop at the first failing
   package (`the_first_failure_ends_the_step_and_names_its_command`). The author's two mutants
   (flag removed, filter removed) were not repeated. The `Verifies:` tags name requirements the
   tests check; the real-workspace test runs real `cargo metadata` and `cargo test`.
4. No fake progress: no placeholder; the two open parts (item 7, item 9c) are recorded with
   reasons; no oracle changed (`ci/coverage.toml` and `xtask/tests/snapshots/` untouched; the
   changed test expectations follow the new step).
5. Spec agreement: XTASK-R014 and its details against `xtask/src/features_off.rs`,
   `process.rs` (`run_features_off`), `ci.rs`, `repo/metadata.rs`: command line, package
   selection, order (ascending name, from `parse`), stop at the first failure, `output_tail`
   prefix, skip without `cargo`, failure on a metadata error, human progress line. They agree.
6. Unsafe: none added. 7. Determinism: not a deterministic crate. 9. Architecture: no new
   dependency; `check-arch` passes.
8. Docs: no other list of `ci` steps is stale (searched `docs/`, `AGENTS.md`, `.github/`).
10. Session boundaries: below. 11. Public repository: `.github/workflows/` unchanged; no issue
    or pull request acted on.

Findings (none is B1 to B5; all in `docs/BACKLOG.md`):
- The two "on its branch" references to the `liminal_memory` spec can now be links.
- XTASK-R014 details claim more than `--package` gives: a dev-dependency of the tested package
  that enables its feature turns it on again (by Cargo's unification rules; not run).
- No command builds each feature alone; XTASK-R014 has no "revisit at M3" note like XTASK-R012.
- Spec §10: the reason given for "no safe ring" (atomics hold one word) is not the obstacle.
- TELE-R011: the "no span before `install`" consequence holds only without a concurrent
  collector change.
- M1-T029's title and file name still say "frame stats".

Merge edits by this review (ADR-0026 Decision 6, no task work): `docs/specs/xtask.md` version
renumbered as above, both history rows kept; `docs/STATUS.md` regenerated; one line in
`xtask/src/check_arch/determinism.rs` (a test helper new on `main` builds a
`metadata::Package`, which this branch gave the field `features`; without `features:
Vec::new()` the merge result does not compile).

Session boundaries (item 10), stated as facts:
- The author's journal names its parent `2026-11-10-session-m1-reviews.md`. That entry (branch
  `session/2026-11-10-m1-reviews`, "in progress", read at `b32bbab`) lists the author as row 7,
  "impl template". It does not list this review yet.
- This review has the same parent. My prompt was the ADR-0026 review template verbatim, with
  nothing added. The worktree's `AGENTS.md` forbade a review by a sibling; `main` carries
  ADR-0029, which allows it, and the text on `main` is in force. I read that from the
  repository.
- ADR-0029 was written and merged by this parent session on an owner instruction that the ADR
  quotes. I cannot check the quote. If the owner did not give it, this review is void under
  ADR-0026 Decision 5.1 and the task needs a review by another session.
- M1-T056 had one implementation sub-agent and this one review sub-agent. Neither ran an
  install; I spawned no agent.
