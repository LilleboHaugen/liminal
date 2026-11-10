---
id: M1-T028
title: Implement liminal_telemetry — logs, spans, counters, per-thread buffers
milestone: M1
status: todo
crates: [liminal_telemetry]
depends_on: [M1-T027, M1-T014, M1-T018]
requirements: [TELE-R001, TELE-R002, TELE-R003, TELE-R004, TELE-R005, TELE-R006, TELE-R010, TELE-R011, TELE-R012, TELE-R020, TELE-R021, TELE-R022, TELE-R023, TELE-R030, TELE-R031, TELE-R040, TELE-R041, TELE-R042, TELE-R043, TELE-R050, TELE-R051, TELE-R052, TELE-R053, TELE-R054, TELE-R055, TELE-R056, TELE-R057, TELE-R060, TELE-R061, TELE-R062, TELE-R065, TELE-R100]
---

# M1-T028: Implement `liminal_telemetry`: logs, spans, counters, per-thread buffers

## Goal

Code can emit logs, spans and counters through `liminal_telemetry`, collected from several threads without locks on the hot path.

## Context

- `docs/specs/liminal_telemetry.md`; ADR-0016; ADR-0018.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Crate, recording API, compile-out feature, per-thread buffers with loom tests, Miri.

**Out:** Frame records and exporters: next task.

## Acceptance criteria

- [ ] The crate exists at the path in `architecture.toml`, is a workspace member, has
      `#![forbid(unsafe_code)]` if `unsafe = "forbid"`, and has a floor in `ci/coverage.toml`.
- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.
- [ ] loom tests cover the buffer hand-off; Miri passes; `// SAFETY:` comments on every `unsafe` block.
- [ ] A test shows that with the feature off the recording calls compile to nothing observable (method chosen in the spec).

## Test requirements

Requirements in frontmatter; loom; Miri.

## Notes

Requirements written by M1-T027 (spec §6 "Assignment"). This task also implements `Collector::end_frame` with the record contents (TELE-R062), because its tests read records only through it; frame timing and `FrameInfo` (TELE-R063, TELE-R064) are M1-T029's. Tests that install a `Collector` serialize on one lock per test binary (spec §10). Run `cargo test -p liminal_telemetry --no-default-features` by hand and record the result (TELE-R004; `cargo xtask ci` uses `--all-features` only, spec open question 2).

## Handoff

_Empty._

## Review

_Empty._
