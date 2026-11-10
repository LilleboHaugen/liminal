---
id: M1-T030
title: Write the spec for liminal_reflect
milestone: M1
status: review
crates: [liminal_reflect, liminal_reflect_derive]
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

- [x] `docs/specs/liminal_reflect.md` exists, follows `docs/specs/_TEMPLATE.md` (all sections), with
      `maturity: planned`, `spec_version: 1` and `adrs` listing every ADR it implements.
- [x] Every requirement has an ID `REFL-Rnnn`, an RFC 2119 level and a testable statement with
      units, ranges and failure behaviour. No vague words ("fast", "robust", "efficient").
- [x] §5 Public API covers the scope below and is consistent with `docs/ARCHITECTURE.md` and the
      specs of the crates it depends on.
- [x] Error codes `LIM-REFL-NNNN` are listed with message and `help` (ADR-0017), or the spec says
      why the crate has no recoverable errors.
- [x] The test plan names the test kinds that apply (TESTING.md) and, for every optimized path, the
      reference implementation it is checked against (AGENTS.md rule 4).
- [x] Determinism requirements follow ADR-0024: the spec names the workloads of
      `tests/determinism.rs` and says whether cross-platform bit-identity is claimed for each.
- [x] Performance budgets name scenarios and machines; numbers that need a measurement say which
      task measures them (PERFORMANCE.md: budgets are never invented).
- [x] Every MUST requirement is assigned to exactly one implementation task (M1-T031, M1-T032): the spec
      lists the assignment, and this session writes the IDs into those tasks' `requirements`
      frontmatter (`check-docs` R028 validates them).
- [x] Open questions are listed in the spec instead of being decided silently.
- [x] `docs/specs/README.md` lists the spec (XTASK-R027); `cargo xtask ci` passes.

## Test requirements

None (documentation task). The review is a spec review (REVIEW.md §2).

## Notes

Requirement IDs leave gaps between groups (template §6). Keep the API small: every public item has to be documented, tested and later snapshotted.

## Handoff

**Done (2026-11-10, implementation sub-agent).** Status `review`.

- Decision (Scope: one file or two): **two spec files.** XTASK-R026 requires a spec at
  `docs/specs/<crate>.md` for every workspace member, and the file-name rule of XTASK-R022 makes a
  spec's `crate` equal its file name, so one file cannot cover both crates once
  `liminal_reflect_derive` is a member. `req_prefix` must be unique (XTASK-R022), so the derive has
  the new prefix `REFLD`; `docs/specs/README.md` now reserves `REFL` for `liminal_reflect` only and
  `REFLD` for `liminal_reflect_derive`.
- `docs/specs/liminal_reflect.md` (`REFL`, 33 requirements: 32 MUST, 1 SHOULD; error codes
  LIM-REFL-0001 to 0005 and 0010 to 0013; three determinism workloads, all claiming
  cross-platform bit-identity).
- `docs/specs/liminal_reflect_derive.md` (`REFLD`, 14 requirements: 12 MUST, 2 SHOULD; compile
  error codes LIM-REFLD-0001 to 0009, no runtime errors).
- Key design choices (reasons in the specs): stable type ID = XXH64 seed 0 of the type path
  (`module_path!()` + identifier + generic arguments, overridable with
  `#[reflect(type_path = "…")]`), never `core::any::TypeId`; descriptors reference field types
  by `TypeRef` (path + ID), so recursive types work; `Option<T>` is reflected as an enum; the
  state encoding writes values only (field order, `u32` variant index, `u64` list length) through
  `liminal_core::hash::StateHasher`; `TypeRegistry` iterates in ascending ID order.
- `requirements` frontmatter of M1-T031 (25 IDs) and M1-T032 (22 IDs): every spec ID is in
  exactly one task (checked by script: 47 spec IDs, 47 task entries, no overlap).
- `xtask/src/repo.rs` test `loads_the_real_repository` asserted that the repository has exactly
  one spec. The change is byte-identical to M1-T012's fix (not merged to `main` yet); it was
  already in this worktree when this session reached that step (modification time 17:30, before
  any edit of this session; presumably the parent). Identical changes merge without conflict.
- This task's `crates` now lists `liminal_reflect_derive` too (the task wrote its spec).

**Not verified:** the specs depend on `docs/specs/liminal_core.md` (`StateHasher`, `StateHash`,
`EngineError`, `validate_error`), which is reviewed (`done`) on branch
`task/M1-T012-write-the-spec-for-liminal-core` but not merged to `main`. The specs therefore name
it in code spans, not links (a link would fail XTASK-R024 on this branch).

**For the reviewer and the next tasks:**
- Spec §12 item 1 (`Reflect` for `liminal_math` types) must be resolved before ECS components hold
  math vectors (M1-T037 at the latest, before M1-T038); it needs an `architecture.toml` edge.
- M1-T031 builds the crate without the derive dependency; M1-T032 adds it and the re-export
  (REFL-R120).

## Review

_Empty._
