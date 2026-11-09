---
id: M1-T046
title: Implement liminal_app — fixed-step loop and headless runner; reach usable
milestone: M1
status: todo
crates: [liminal_app]
depends_on: [M1-T045]
requirements: []
---

# M1-T046: Implement `liminal_app`: fixed-step loop and headless runner; reach `usable`

## Goal

Apps run headless with a fixed timestep and `step(n)`, and `liminal_app` is ready for the `usable` promotion review.

## Context

- `docs/specs/liminal_app.md`; ADR-0009 item 1.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Fixed-step accumulator, interpolation alpha, headless runner, `step(n)`, telemetry frame records, `tests/determinism.rs` (single vs multithreaded), the `usable` evidence.

**Out:** Windowed runner (M3).

## Acceptance criteria

- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.
- [ ] Accumulator tests with a fake clock: number of simulation steps and alpha for given frame times match hand-computed values.
- [ ] The spec's §Maturity checklist holds the evidence for every `usable` gate in
      `docs/process/MATURITY.md` (trace with `maturity: usable` applied on a scratch copy, coverage
      against the 85 % target for its crate type, examples, integration tests).
- [ ] The task is set to `review` with `maturity` still unchanged. The reviewing session performs
      the promotion review (REVIEW.md §3) and, if every gate holds, sets `maturity: usable` in the
      spec and `docs/specs/README.md` and regenerates STATUS.md (MATURITY.md "Promotion
      procedure": a different session changes `maturity`).

## Test requirements

Requirements in frontmatter; determinism tests.

## Notes

_None._

## Handoff

_Empty._

## Review

_Empty._
