---
id: M1-T014
title: Implement liminal_core — hashing, RNG, collections, time; reach usable
milestone: M1
status: todo
crates: [liminal_core]
depends_on: [M1-T013]
requirements: []
---

# M1-T014: Implement `liminal_core`: hashing, RNG, collections, time; reach `usable`

## Goal

`liminal_core` is complete per its spec and ready for the `usable` promotion review.

## Context

- `docs/specs/liminal_core.md`; ADR-0009; ADR-0024.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** State hasher, fixed-seed hasher and collections, RNG streams, time types; `tests/determinism.rs`; the `usable` evidence.

**Out:** Benchmarks of the hasher and RNG: the `liminal_benches` tasks.

## Acceptance criteria

- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.
- [ ] RNG and hasher outputs are checked against published reference vectors of the chosen algorithms, or against values computed by an independent implementation named in the test.
- [ ] `tests/determinism.rs` per ADR-0024, with recorded hash literals for the workloads the spec claims to be cross-platform.
- [ ] The spec's §Maturity checklist holds the evidence for every `usable` gate in
      `docs/process/MATURITY.md` (trace with `maturity: usable` applied on a scratch copy, coverage
      against the 85 % target for its crate type, examples, integration tests).
- [ ] The task is set to `review` with `maturity` still unchanged. The reviewing session performs
      the promotion review (REVIEW.md §3) and, if every gate holds, sets `maturity: usable` in the
      spec and `docs/specs/README.md` and regenerates STATUS.md (MATURITY.md "Promotion
      procedure": a different session changes `maturity`).

## Test requirements

Requirements in frontmatter. Known-answer tests for hash and RNG; determinism tests.

## Notes

_None._

## Handoff

_Empty._

## Review

_Empty._
