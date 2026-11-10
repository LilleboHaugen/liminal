---
id: M1-T056
title: liminal_telemetry spec follow-ups from the M1-T027 review
milestone: M1
status: in-progress
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

- [ ] Items 1 to 10 are done, or a declined or split-off item is written into this task with the
      reason.
- [ ] Spec changes bump `spec_version` with a history row; moved requirement IDs are in exactly
      one task's `requirements` frontmatter.
- [ ] `cargo xtask ci` passes.

## Test requirements

Item 10: tests for the new or changed XTASK requirement. Items 1 to 9 change the spec only; the
requirements they touch are tested by M1-T028 and M1-T029.

## Handoff

_Empty._

## Review

_Empty._
