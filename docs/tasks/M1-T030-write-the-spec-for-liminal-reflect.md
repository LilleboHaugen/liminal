---
id: M1-T030
title: Write the spec for liminal_reflect
milestone: M1
status: todo
crates: [liminal_reflect]
depends_on: [M1-T001]
requirements: []
---

# M1-T030: Write the spec for `liminal_reflect`

## Goal

`docs/specs/liminal_reflect.md` is a complete, reviewable contract for `liminal_reflect`, so the implementation tasks can be done without guessing.

## Context

- `docs/ARCHITECTURE.md` §2 (module map: runtime type information, field access by name, stable type IDs; `#[derive(Reflect)]`).
- `docs/specs/_TEMPLATE.md`; `docs/specs/xtask.md` as the quality bar.
- `architecture.toml` entry for `liminal_reflect` (layer, `unsafe`, `deterministic`, allowed deps).
- ADR-0009 item 6; ADR-0003 (`syn`, `quote`, `proc-macro2` approved for the derive).
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Spec for `liminal_reflect` **and** `liminal_reflect_derive` (one spec file each, or one file covering both if `check-docs` R022/R026 allow it; decide and record): type descriptors, stable type IDs (how they stay stable across builds and platforms), field access by name, supported shapes (structs, tuple structs, enums, generics), how state hashing (ADR-0009 item 6) walks reflected values in canonical order, and the derive macro's accepted input and compile errors.

**Out:** Code. The implementation tasks (M1-T031, M1-T032) write it.

## Acceptance criteria

- [ ] `docs/specs/liminal_reflect.md` exists, follows `docs/specs/_TEMPLATE.md` (all sections), with
      `maturity: planned`, `spec_version: 1` and `adrs` listing every ADR it implements.
- [ ] Every requirement has an ID `REFL-Rnnn`, an RFC 2119 level and a testable statement with
      units, ranges and failure behaviour. No vague words ("fast", "robust", "efficient").
- [ ] §5 Public API covers the scope below and is consistent with `docs/ARCHITECTURE.md` and the
      specs of the crates it depends on.
- [ ] Error codes `LIM-REFL-NNNN` are listed with message and `help` (ADR-0017), or the spec says
      why the crate has no recoverable errors.
- [ ] The test plan names the test kinds that apply (TESTING.md) and, for every optimized path, the
      reference implementation it is checked against (AGENTS.md rule 4).
- [ ] Determinism requirements follow ADR-0024: the spec names the workloads of
      `tests/determinism.rs` and says whether cross-platform bit-identity is claimed for each.
- [ ] Performance budgets name scenarios and machines; numbers that need a measurement say which
      task measures them (PERFORMANCE.md: budgets are never invented).
- [ ] Every MUST requirement is assigned to exactly one implementation task (M1-T031, M1-T032): the spec
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
