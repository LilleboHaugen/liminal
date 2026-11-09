---
id: M1-T016
title: Implement the liminal_benches harness
milestone: M1
status: todo
crates: [liminal_benches]
depends_on: [M1-T015]
requirements: []
---

# M1-T016: Implement the `liminal_benches` harness

## Goal

Benchmarks can be written and produce the JSON result files ADR-0023 defines.

## Context

- `docs/specs/liminal_benches.md`; ADR-0023 items 1–2.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** The crate at `benches/`, the harness, one bench target measuring the `liminal_core` hasher and RNG; `check-arch` rule that engine crates have no `benches/` targets (ADR-0023 "Verification"; XTASK spec change).

**Out:** Comparison with baselines (`cargo xtask bench`): next task.

## Acceptance criteria

- [ ] The crate exists at the path in `architecture.toml`, is a workspace member, has
      `#![forbid(unsafe_code)]` if `unsafe = "forbid"`, and has a floor in `ci/coverage.toml`.
- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.
- [ ] Statistics functions have unit tests with hand-computed expected values (no wall-clock assertions, TESTING.md).
- [ ] Running the bench target writes a JSON file that matches the spec's format (integration test on a trivial case with a fixed iteration count).

## Test requirements

Requirements in frontmatter; the new XTASK requirement for engine `benches/` targets.

## Notes

_None._

## Handoff

_Empty._

## Review

_Empty._
