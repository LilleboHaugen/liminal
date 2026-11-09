---
id: M1-T015
title: Write the spec for liminal_benches
milestone: M1
status: todo
crates: [liminal_benches]
depends_on: [M1-T014]
requirements: []
---

# M1-T015: Write the spec for `liminal_benches`

## Goal

`docs/specs/liminal_benches.md` is a complete, reviewable contract for `liminal_benches`, so the implementation tasks can be done without guessing.

## Context

- `docs/ARCHITECTURE.md` §2 (module map: benchmark harness and engine benchmarks (ADR-0023)).
- `docs/specs/_TEMPLATE.md`; `docs/specs/xtask.md` as the quality bar.
- `architecture.toml` entry for `liminal_benches` (layer, `unsafe`, `deterministic`, allowed deps).
- ADR-0023 (all of it); `docs/process/PERFORMANCE.md`.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Spec for: the `std`-only harness (warm-up, calibration, samples, median/MAD/min/max, throughput), case IDs `<crate>/<group>/<case>`, the JSON result format (`version: 1`, machine metadata), the single-threaded marker for the instruction-count gate, and how bench targets are laid out. This is a tool crate (layer 5): tool-crate gates apply.

**Out:** Code. The implementation tasks (M1-T016, M1-T017) write it.

## Acceptance criteria

- [ ] `docs/specs/liminal_benches.md` exists, follows `docs/specs/_TEMPLATE.md` (all sections), with
      `maturity: planned`, `spec_version: 1` and `adrs` listing every ADR it implements.
- [ ] Every requirement has an ID `BENCH-Rnnn`, an RFC 2119 level and a testable statement with
      units, ranges and failure behaviour. No vague words ("fast", "robust", "efficient").
- [ ] §5 Public API covers the scope below and is consistent with `docs/ARCHITECTURE.md` and the
      specs of the crates it depends on.
- [ ] Error codes `LIM-BENCH-NNNN` are listed with message and `help` (ADR-0017), or the spec says
      why the crate has no recoverable errors.
- [ ] The test plan names the test kinds that apply (TESTING.md) and, for every optimized path, the
      reference implementation it is checked against (AGENTS.md rule 4).
- [ ] Performance budgets name scenarios and machines; numbers that need a measurement say which
      task measures them (PERFORMANCE.md: budgets are never invented).
- [ ] Every MUST requirement is assigned to exactly one implementation task (M1-T016, M1-T017): the spec
      lists the assignment, and this session writes the IDs into those tasks' `requirements`
      frontmatter (`check-docs` R028 validates them).
- [ ] Open questions are listed in the spec instead of being decided silently.
- [ ] `docs/specs/README.md` lists the spec (XTASK-R027); `cargo xtask ci` passes.

## Test requirements

None (documentation task). The review is a spec review (REVIEW.md §2).

## Notes

Requirement IDs leave gaps between groups (template §6). Keep the API small: every public item has to be documented, tested and later snapshotted.

## Handoff

_Empty._

## Review

_Empty._
