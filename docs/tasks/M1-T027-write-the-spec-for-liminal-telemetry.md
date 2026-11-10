---
id: M1-T027
title: Write the spec for liminal_telemetry
milestone: M1
status: done
crates: [liminal_telemetry]
depends_on: [M1-T001]
requirements: []
---

# M1-T027: Write the spec for `liminal_telemetry`

## Goal

`docs/specs/liminal_telemetry.md` is a complete, reviewable contract for `liminal_telemetry`, so the implementation tasks can be done without guessing.

## Context

- `docs/ARCHITECTURE.md` §2 (module map: structured logs, spans, counters, frame stats; JSON and Chrome-trace export).
- `docs/specs/_TEMPLATE.md`; `docs/specs/xtask.md` as the quality bar.
- `architecture.toml` entry for `liminal_telemetry` (layer, `unsafe`, `deterministic`, allowed deps).
- ADR-0016 (own telemetry); ADR-0018.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Spec for: log records, spans, counters and their macros or functions; the compile-out feature and what "near-zero cost when disabled" means in numbers; per-thread buffers and their lock-free hand-off; frame records; Chrome-trace JSON and compact JSON summary formats (written without `serde`, which is not approved for runtime crates, OQ-002); the overhead budget per span (measured by the second telemetry task).

**Out:** Code. The implementation tasks (M1-T028, M1-T029) write it.

## Acceptance criteria

- [x] `docs/specs/liminal_telemetry.md` exists, follows `docs/specs/_TEMPLATE.md` (all sections), with
      `maturity: planned`, `spec_version: 1` and `adrs` listing every ADR it implements.
- [x] Every requirement has an ID `TEL-Rnnn`, an RFC 2119 level and a testable statement with
      units, ranges and failure behaviour. No vague words ("fast", "robust", "efficient").
- [x] §5 Public API covers the scope below and is consistent with `docs/ARCHITECTURE.md` and the
      specs of the crates it depends on.
- [x] Error codes `LIM-TEL-NNNN` are listed with message and `help` (ADR-0017), or the spec says
      why the crate has no recoverable errors.
- [x] The test plan names the test kinds that apply (TESTING.md) and, for every optimized path, the
      reference implementation it is checked against (AGENTS.md rule 4).
- [x] Performance budgets name scenarios and machines; numbers that need a measurement say which
      task measures them (PERFORMANCE.md: budgets are never invented).
- [x] Every MUST requirement is assigned to exactly one implementation task (M1-T028, M1-T029): the spec
      lists the assignment, and this session writes the IDs into those tasks' `requirements`
      frontmatter (`check-docs` R028 validates them).
- [x] Open questions are listed in the spec instead of being decided silently.
- [x] `docs/specs/README.md` lists the spec (XTASK-R027); `cargo xtask ci` passes.

## Test requirements

None (documentation task). The review is a spec review (REVIEW.md §2).

## Notes

**Prefix (scope note, 2026-11-10):** the acceptance criteria below say `TEL-Rnnn` and
`LIM-TEL-NNNN`. The spec uses `TELE` instead: `docs/specs/README.md` reserves `TELE` for
`liminal_telemetry`, and CORE-R012 (`docs/specs/liminal_core.md`) requires the error area to equal
the spec's `req_prefix` (also `liminal_core` spec open question 4). The criteria are met with
`TELE` in place of `TEL`.

Requirement IDs leave gaps between groups (template §6). Keep the API small: every public item has to be documented, tested and later snapshotted.

## Handoff

**Done (2026-11-10, sub-agent session).**
- `docs/specs/liminal_telemetry.md`, `spec_version` 1, `maturity: planned`, prefix `TELE`:
  44 MUST + 1 SHOULD requirements (TELE-R001 … TELE-R100), one error code (LIM-TELE-0001),
  10 open questions, §9 scenarios with no numbers (M1-T029 measures span overhead).
- `requirements` frontmatter and Notes of M1-T028 (32 IDs) and M1-T029 (13 IDs). Every MUST is
  in exactly one task.
- `docs/specs/README.md` index row.
- `xtask/src/repo.rs` test `loads_the_real_repository`: same change as on the M1-T012 branch
  (byte-identical file, so the merge is clean). It asserted the real repository has exactly one
  spec, which fails as soon as any spec is added.

**Design decisions to check in review** (no ADR needed: no dependency, no layer or unsafe-policy
change; ADR-0016 delegates these to the spec):
- Feature `telemetry` off by default; it changes only the macro expansions and `SpanGuard`. The
  disabled expansions live in `__private::off` and are compiled in every build, so the
  compile-out test (TELE-R002) runs under `--all-features`. That is the "method chosen in the
  spec" of M1-T028's acceptance criteria.
- One `Collector` per process (LIM-TELE-0001); recording is active only while it exists.
- Per-thread SPSC ring of 16,384 fixed-size records (≤ 48 bytes), drop-newest on overflow with a
  count; `Mutex<VecDeque>` reference with a differential test; loom and Miri.
- Log messages are `&'static str` (no format strings); data goes in `ContextValue` fields.
- Exact Chrome-trace and summary JSON layouts (snapshot-testable), escaping and floats as in
  CORE-R009/R010.

**Not done / for others.**
- Spec open question 2: `cargo xtask ci` never builds with the feature off. A task that adds a
  feature-off build/test step to `cargo xtask ci` should be written (xtask crate) before M1-T029
  is reviewed. Not written here (outside this task's scope; reported to the parent).
- The spec references `docs/specs/liminal_core.md` (on branch `task/M1-T012-…`, reviewed `done`,
  not yet merged to `main` when this was written) and `docs/specs/liminal_memory.md` (branch
  `task/M1-T024-…`) in prose only, not as links, so `check-docs` passes before they merge.

**Next step:** a different session reviews M1-T027 (spec review, REVIEW.md §2).

## Review

2026-11-10, review sub-agent (ADR-0026/0027) of parent session `session/2026-11-10-m1-reviews`
(journal `2026-11-10-session-m1-reviews.md`), fresh context, prompt = the ADR-0026 review template.
This session implemented no part of M1-T027; the author was a sub-agent (no. 13, restarted as
no. 15) of the other parent session `2026-11-09-session-m1-subagents.md`. macOS 27.2 (Darwin),
Rust 1.99.1, worktree `liminal-worktrees/M1-T027-review`, branch at `a7accd2`. Task review
(REVIEW.md §1) plus spec review (REVIEW.md §2), as "Test requirements" says.

Outcome: **done**, no blocking findings. Follow-ups: M1-T056.

Checked:
- AC 1: all 14 template sections present in order; frontmatter `maturity: planned`,
  `spec_version: 1`, `adrs: [ADR-0016, ADR-0017, ADR-0018, ADR-0023]`.
- AC 2: 45 requirement rows (44 MUST, 1 SHOULD), IDs unique (script), prefix `TELE` per the
  scope note (reserved in `docs/specs/README.md`, CORE-R012). Search for vague words finds only
  the quoted "near-zero cost when disabled", which §4 defines as zero run-time work per call
  site. Counts, units and failure behaviour are stated (R002, R052, R054, R055, R056, R091).
- AC 3: §5 covers every scope item (log records, spans, counters, gauges, macros, feature,
  thread buffers and hand-off, frame records, both exporters without `serde`, span budget row).
  Compared with `docs/ARCHITECTURE.md` §2 row, §4 (frame record closed after render) and §5.3,
  ADR-0016 Decision, `architecture.toml` (layer 0, `unsafe = "allow"`, `deterministic = false`,
  dep `liminal_core`, dev `proptest`/`loom`), and `docs/specs/liminal_core.md` as merged on
  `main`: `EngineError`, `write_display`, `validate_error(&e, "TELE")`, `ContextValue` (derives
  `PartialEq`, so `FrameRecord: PartialEq` holds), `Tick` (`Copy`, `Eq`, `Default`, so
  `FrameInfo`'s derives hold), CORE-R009/R010 as cited by TELE-R082.
- AC 4: one code, LIM-TELE-0001, with message, help and "no context"; §8 says why recording and
  export have no errors.
- AC 5: test kinds per area are those of TESTING.md; the one optimized path (SPSC ring) has the
  `Mutex<VecDeque>` reference and a differential property test (R057), plus loom and Miri.
- AC 6: eight scenarios, no invented number; six name M1-T029 (one also M1-T033), two say "not
  scheduled" (follow-up item 9b).
- AC 7: script over the spec and both task files: the 44 MUST IDs are exactly M1-T028 (32) plus
  M1-T029 (12 MUST + TELE-R084 SHOULD), none in both, none unknown; the §6 table agrees.
- AC 8: ten open questions, each with a resolving task or milestone.
- AC 9: `docs/specs/README.md` row present. `cargo xtask ci` on `a7accd2`: pass, 11 of 11
  steps, none skipped. Branch CI run 42856209572 on `a7accd2`: success.
- Read for correctness: nearest-rank formula of R091 by hand for n = 1, 2, 20, 21 (the test
  plan's cases); `T(ns)` examples; Chrome trace event shapes (`M`, `X`, `i` with `"s":"t"`, `C`)
  against the trace event format from memory, not in a viewer (R084 is M1-T029's check);
  record sizes of R056 (span, counter, gauge fit 48 bytes; log: follow-up item 5); macro
  patterns of §5.1; `!Send` zero-sized guard without drop glue (R002, R022) is expressible.
- Oracle change: `xtask/src/repo.rs` `loads_the_real_repository`, exact spec list replaced by
  membership and exclusion checks. Justified in the journal; the file is byte-identical to the
  version already on `main` from M1-T012 (`git diff` empty), so nothing new is weakened.
- No fake progress: no code; scope note on the prefix is written in the task; nothing else
  reduced.
- Session boundaries (item 10): the author is listed in its parent's journal (rows 13 and 15;
  the restart deviates from ADR-0027 Decision 2.3 on the owner's quoted instruction 3). The
  author's journal names its parent as "the 2026-11-10 parent session journal" instead of the
  file `2026-11-09-session-m1-subagents.md` (ADR-0026 Decision 8.2); the parent's entry records
  that inconsistency. No install or nested spawn recorded.
- ADR-0028 (item 11): no workflow change; no credentials or personal data in the diff; no
  issue, pull request or outside code acted on.

Follow-ups (M1-T056): "provisional" frame fields allowed for M1-T028; per-simulation-step
records (ADR-0016); ADR-0018 benchmark clause and the `reference` module rule; span across two
collectors; log record size against 48 bytes; number-format wording (R082, R093); counter and
gauge of one name in the Chrome trace; summary `sim_steps` type; small text items; feature-off
build in `cargo xtask ci` (spec open question 2, which the handoff left without a task).
