---
id: M1-T024
title: Write the spec for liminal_memory
milestone: M1
status: todo
crates: [liminal_memory]
depends_on: [M1-T001]
requirements: []
---

# M1-T024: Write the spec for `liminal_memory`

## Goal

`docs/specs/liminal_memory.md` is a complete, reviewable contract for `liminal_memory`, so the implementation tasks can be done without guessing.

## Context

- `docs/ARCHITECTURE.md` §2 (module map: arena, frame (linear), pool and aligned allocators; allocation tracking).
- `docs/specs/_TEMPLATE.md`; `docs/specs/xtask.md` as the quality bar.
- `architecture.toml` entry for `liminal_memory` (layer, `unsafe`, `deterministic`, allowed deps).
- ADR-0018 (`unsafe`); TESTING.md (allocators need property tests).
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Spec for: arena and frame (linear, reset per frame) allocators, pool allocator for fixed-size blocks, aligned allocation, allocation tracking (counts, bytes, high-water mark per allocator), what is `unsafe` inside and the safe API around it, and the simple reference allocator each is checked against.

**Out:** Code. The implementation tasks (M1-T025, M1-T026) write it.

## Acceptance criteria

- [ ] `docs/specs/liminal_memory.md` exists, follows `docs/specs/_TEMPLATE.md` (all sections), with
      `maturity: planned`, `spec_version: 1` and `adrs` listing every ADR it implements.
- [ ] Every requirement has an ID `MEM-Rnnn`, an RFC 2119 level and a testable statement with
      units, ranges and failure behaviour. No vague words ("fast", "robust", "efficient").
- [ ] §5 Public API covers the scope below and is consistent with `docs/ARCHITECTURE.md` and the
      specs of the crates it depends on.
- [ ] Error codes `LIM-MEM-NNNN` are listed with message and `help` (ADR-0017), or the spec says
      why the crate has no recoverable errors.
- [ ] The test plan names the test kinds that apply (TESTING.md) and, for every optimized path, the
      reference implementation it is checked against (AGENTS.md rule 4).
- [ ] Determinism requirements follow ADR-0024: the spec names the workloads of
      `tests/determinism.rs` and says whether cross-platform bit-identity is claimed for each.
- [ ] Performance budgets name scenarios and machines; numbers that need a measurement say which
      task measures them (PERFORMANCE.md: budgets are never invented).
- [ ] Every MUST requirement is assigned to exactly one implementation task (M1-T025, M1-T026): the spec
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
