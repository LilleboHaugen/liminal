---
id: M1-T047
title: Write the spec for liminal_testkit
milestone: M1
status: todo
crates: [liminal_testkit]
depends_on: [M1-T044]
requirements: []
---

# M1-T047: Write the spec for `liminal_testkit`

## Goal

`docs/specs/liminal_testkit.md` is a complete, reviewable contract for `liminal_testkit`, so the implementation tasks can be done without guessing.

## Context

- `docs/ARCHITECTURE.md` §2 (module map: headless app harness, deterministic stepping, replay, state-hash assertions (golden-image comparison is M3)).
- `docs/specs/_TEMPLATE.md`; `docs/specs/xtask.md` as the quality bar.
- `architecture.toml` entry for `liminal_testkit` (layer, `unsafe`, `deterministic`, allowed deps).
- TESTING.md §Determinism tests; ADR-0024 item 4; OQ-003 (text format, M2).
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Spec for the headless subset: app harness, `step(n)`, `assert_deterministic` (TESTING.md: first diverging step and differing components), state-hash checkpoints, replay files (recorded inputs and expected hashes; what an "input" is before `liminal_input` exists in M3) and their text format (strict JSON written without `serde`, or deferred: decide against OQ-003 and record). Golden images are out of scope until M3.

**Out:** Code. The implementation tasks (M1-T048, M1-T049) write it.

## Acceptance criteria

- [ ] `docs/specs/liminal_testkit.md` exists, follows `docs/specs/_TEMPLATE.md` (all sections), with
      `maturity: planned`, `spec_version: 1` and `adrs` listing every ADR it implements.
- [ ] Every requirement has an ID `TK-Rnnn`, an RFC 2119 level and a testable statement with
      units, ranges and failure behaviour. No vague words ("fast", "robust", "efficient").
- [ ] §5 Public API covers the scope below and is consistent with `docs/ARCHITECTURE.md` and the
      specs of the crates it depends on.
- [ ] Error codes `LIM-TK-NNNN` are listed with message and `help` (ADR-0017), or the spec says
      why the crate has no recoverable errors.
- [ ] The test plan names the test kinds that apply (TESTING.md) and, for every optimized path, the
      reference implementation it is checked against (AGENTS.md rule 4).
- [ ] Performance budgets name scenarios and machines; numbers that need a measurement say which
      task measures them (PERFORMANCE.md: budgets are never invented).
- [ ] Every MUST requirement is assigned to exactly one implementation task (M1-T048, M1-T049): the spec
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
