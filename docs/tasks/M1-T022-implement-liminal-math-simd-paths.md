---
id: M1-T022
title: Implement liminal_math SIMD paths
milestone: M1
status: todo
crates: [liminal_math]
depends_on: [M1-T021, M1-T017, M1-T018]
requirements: []
---

# M1-T022: Implement `liminal_math` SIMD paths

## Goal

Hot math operations have SIMD paths that agree with the scalar reference and are faster on the reference machine.

## Context

- `docs/specs/liminal_math.md`; AGENTS.md rule 4; CODE_STYLE.md §unsafe (prefer safe until a benchmark proves the unsafe version is needed); ADR-0018.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** SIMD implementations for the operations the spec names; differential tests against scalar; benchmarks before and after.

**Out:** Comparison with `glam`: competitive benchmark task.

## Acceptance criteria

- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.
- [ ] Every SIMD path has a differential property test against the scalar reference, run on both x86_64 (CI) and aarch64 (reference machine).
- [ ] Every `unsafe` block has a `// SAFETY:` comment; Miri passes on the crate (or the exception is listed).
- [ ] Benchmarks in `liminal_benches` show each kept SIMD path faster than scalar on the reference machine; numbers in the journal; baseline updated with `cargo xtask bench`.

## Test requirements

Requirements in frontmatter; differential tests; Miri.

## Notes

_None._

## Handoff

_Empty._

## Review

_Empty._
