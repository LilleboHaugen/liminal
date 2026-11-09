---
id: M1-T012
title: Write the spec for liminal_core
milestone: M1
status: todo
crates: [liminal_core]
depends_on: [M1-T001]
requirements: []
---

# M1-T012: Write the spec for `liminal_core`

## Goal

`docs/specs/liminal_core.md` is a complete, reviewable contract for `liminal_core`, so the implementation tasks can be done without guessing.

## Context

- `docs/ARCHITECTURE.md` §2 (module map: error model with stable codes, generational IDs/handles, deterministic hashing and RNG, time types).
- `docs/specs/_TEMPLATE.md`; `docs/specs/xtask.md` as the quality bar.
- `architecture.toml` entry for `liminal_core` (layer, `unsafe`, `deterministic`, allowed deps).
- ADR-0009 (determinism), ADR-0017 (errors), ADR-0024 (state hasher in `liminal_core`).
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Spec for: the error model shared by all crates (code type `LIM-<AREA>-NNNN`, message, structured context, `help`, and how an error renders as JSON before `liminal_serial` exists); generational IDs and handles; the state hasher (ADR-0024: algorithm, seed, byte order, what it guarantees across platforms); the fixed-seed hasher and collections used by simulation state (ADR-0009 item 3: iteration order defined); RNG streams derived from world seed and stream key (ADR-0009 item 2, algorithm named); time types (fixed `dt`, ticks, durations).

**Out:** Code. The implementation tasks (M1-T013, M1-T014) write it.

## Acceptance criteria

- [ ] `docs/specs/liminal_core.md` exists, follows `docs/specs/_TEMPLATE.md` (all sections), with
      `maturity: planned`, `spec_version: 1` and `adrs` listing every ADR it implements.
- [ ] Every requirement has an ID `CORE-Rnnn`, an RFC 2119 level and a testable statement with
      units, ranges and failure behaviour. No vague words ("fast", "robust", "efficient").
- [ ] §5 Public API covers the scope below and is consistent with `docs/ARCHITECTURE.md` and the
      specs of the crates it depends on.
- [ ] Error codes `LIM-CORE-NNNN` are listed with message and `help` (ADR-0017), or the spec says
      why the crate has no recoverable errors.
- [ ] The test plan names the test kinds that apply (TESTING.md) and, for every optimized path, the
      reference implementation it is checked against (AGENTS.md rule 4).
- [ ] Determinism requirements follow ADR-0024: the spec names the workloads of
      `tests/determinism.rs` and says whether cross-platform bit-identity is claimed for each.
- [ ] Performance budgets name scenarios and machines; numbers that need a measurement say which
      task measures them (PERFORMANCE.md: budgets are never invented).
- [ ] Every MUST requirement is assigned to exactly one implementation task (M1-T013, M1-T014): the spec
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
