---
id: M1-T044
title: Write the spec for liminal_app
milestone: M1
status: todo
crates: [liminal_app]
depends_on: [M1-T037]
requirements: []
---

# M1-T044: Write the spec for `liminal_app`

## Goal

`docs/specs/liminal_app.md` is a complete, reviewable contract for `liminal_app`, so the implementation tasks can be done without guessing.

## Context

- `docs/ARCHITECTURE.md` §2 (module map: app builder, module registration, main loop (fixed-step simulation and interpolated rendering), runners).
- `docs/specs/_TEMPLATE.md`; `docs/specs/xtask.md` as the quality bar.
- `architecture.toml` entry for `liminal_app` (layer, `unsafe`, `deterministic`, allowed deps).
- ARCHITECTURE.md §4 (frame lifecycle); ADR-0009 item 1; `docs/specs/liminal_ecs.md`.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Spec for: app builder, modules and their `build`, standard schedules (ARCHITECTURE.md §4), fixed-step accumulator and interpolation alpha, headless runner and `step(n)`, how telemetry frame records are opened and closed, error reporting during build.

**Out:** Code. The implementation tasks (M1-T045, M1-T046) write it.

## Acceptance criteria

- [ ] `docs/specs/liminal_app.md` exists, follows `docs/specs/_TEMPLATE.md` (all sections), with
      `maturity: planned`, `spec_version: 1` and `adrs` listing every ADR it implements.
- [ ] Every requirement has an ID `APP-Rnnn`, an RFC 2119 level and a testable statement with
      units, ranges and failure behaviour. No vague words ("fast", "robust", "efficient").
- [ ] §5 Public API covers the scope below and is consistent with `docs/ARCHITECTURE.md` and the
      specs of the crates it depends on.
- [ ] Error codes `LIM-APP-NNNN` are listed with message and `help` (ADR-0017), or the spec says
      why the crate has no recoverable errors.
- [ ] The test plan names the test kinds that apply (TESTING.md) and, for every optimized path, the
      reference implementation it is checked against (AGENTS.md rule 4).
- [ ] Determinism requirements follow ADR-0024: the spec names the workloads of
      `tests/determinism.rs` and says whether cross-platform bit-identity is claimed for each.
- [ ] Performance budgets name scenarios and machines; numbers that need a measurement say which
      task measures them (PERFORMANCE.md: budgets are never invented).
- [ ] Every MUST requirement is assigned to exactly one implementation task (M1-T045, M1-T046): the spec
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
