---
id: M1-T029
title: Implement liminal_telemetry — frame stats and export; reach usable
milestone: M1
status: todo
crates: [liminal_telemetry]
depends_on: [M1-T028, M1-T017]
requirements: [TELE-R063, TELE-R064, TELE-R080, TELE-R081, TELE-R082, TELE-R083, TELE-R084, TELE-R090, TELE-R091, TELE-R092, TELE-R093, TELE-R094, TELE-R095]
---

# M1-T029: Implement `liminal_telemetry`: frame stats and export; reach `usable`

## Goal

Telemetry produces frame records and exports them as Chrome-trace and JSON summaries, and the crate is ready for the `usable` promotion review.

## Context

- `docs/specs/liminal_telemetry.md`; ADR-0016.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Frame records, Chrome-trace and JSON exporters with snapshot tests, span overhead benchmark, the `usable` evidence.

**Out:** Integration into `liminal_app`'s loop: the `liminal_app` tasks.

## Acceptance criteria

- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.
- [ ] Snapshot tests of both export formats (`tests/snapshots/`); one Chrome-trace file opened in Perfetto and the result noted in the journal.
- [ ] Span overhead measured in `liminal_benches` and written into the spec's budget.
- [ ] The spec's §Maturity checklist holds the evidence for every `usable` gate in
      `docs/process/MATURITY.md` (trace with `maturity: usable` applied on a scratch copy, coverage
      against the 85 % target for its crate type, examples, integration tests, Miri passes on the test suite (exceptions listed with reasons), loom tests pass for the lock-free structures).
- [ ] The task is set to `review` with `maturity` still unchanged. The reviewing session performs
      the promotion review (REVIEW.md §3) and, if every gate holds, sets `maturity: usable` in the
      spec and `docs/specs/README.md` and regenerates STATUS.md (MATURITY.md "Promotion
      procedure": a different session changes `maturity`).

## Test requirements

Requirements in frontmatter; snapshot tests; benchmark.

## Notes

Requirements written by M1-T027 (spec §6 "Assignment"). TELE-R084 (SHOULD) is the Perfetto check of the acceptance criteria. Benchmark case IDs and scenarios: spec §9.

## Handoff

_Empty._

## Review

_Empty._
