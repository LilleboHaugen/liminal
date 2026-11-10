---
id: M1-T026
title: Implement liminal_memory — pool, aligned, tracking; reach usable
milestone: M1
status: todo
crates: [liminal_memory]
depends_on: [M1-T025, M1-T017]
requirements: [MEM-R040, MEM-R041, MEM-R042, MEM-R043, MEM-R044, MEM-R045, MEM-R046, MEM-R047, MEM-R060, MEM-R061, MEM-R062, MEM-R063, MEM-R064, MEM-R070, MEM-R071, MEM-R072, MEM-R073, MEM-R074, MEM-R080, MEM-R081]
---

# M1-T026: Implement `liminal_memory`: pool, aligned, tracking; reach `usable`

## Goal

`liminal_memory` is complete per its spec and ready for the `usable` promotion review.

## Context

- `docs/specs/liminal_memory.md`.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Pool and aligned allocators, allocation tracking, `tests/determinism.rs`, benchmarks against the system allocator, the `usable` evidence.

**Out:** Use of the allocators by other crates.

## Acceptance criteria

- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.
- [ ] Benchmarks in `liminal_benches` against the system allocator, baseline recorded.
- [ ] The spec's §Maturity checklist holds the evidence for every `usable` gate in
      `docs/process/MATURITY.md` (trace with `maturity: usable` applied on a scratch copy, coverage
      against the 85 % target for its crate type, examples, integration tests, Miri passes on the test suite (exceptions listed with reasons)).
- [ ] The task is set to `review` with `maturity` still unchanged. The reviewing session performs
      the promotion review (REVIEW.md §3) and, if every gate holds, sets `maturity: usable` in the
      spec and `docs/specs/README.md` and regenerates STATUS.md (MATURITY.md "Promotion
      procedure": a different session changes `maturity`).

## Test requirements

Requirements in frontmatter; property tests; Miri.

## Notes

_None._

## Handoff

_Empty._

## Review

_Empty._
