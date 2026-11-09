---
id: M1-T024
title: Write the spec for liminal_memory
milestone: M1
status: review
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

- [x] `docs/specs/liminal_memory.md` exists, follows `docs/specs/_TEMPLATE.md` (all sections), with
      `maturity: planned`, `spec_version: 1` and `adrs` listing every ADR it implements.
- [x] Every requirement has an ID `MEM-Rnnn`, an RFC 2119 level and a testable statement with
      units, ranges and failure behaviour. No vague words ("fast", "robust", "efficient").
- [x] §5 Public API covers the scope below and is consistent with `docs/ARCHITECTURE.md` and the
      specs of the crates it depends on.
- [x] Error codes `LIM-MEM-NNNN` are listed with message and `help` (ADR-0017), or the spec says
      why the crate has no recoverable errors.
- [x] The test plan names the test kinds that apply (TESTING.md) and, for every optimized path, the
      reference implementation it is checked against (AGENTS.md rule 4).
- [x] Determinism requirements follow ADR-0024: the spec names the workloads of
      `tests/determinism.rs` and says whether cross-platform bit-identity is claimed for each.
- [x] Performance budgets name scenarios and machines; numbers that need a measurement say which
      task measures them (PERFORMANCE.md: budgets are never invented).
- [x] Every MUST requirement is assigned to exactly one implementation task (M1-T025, M1-T026): the spec
      lists the assignment, and this session writes the IDs into those tasks' `requirements`
      frontmatter (`check-docs` R028 validates them).
- [x] Open questions are listed in the spec instead of being decided silently.
- [x] `docs/specs/README.md` lists the spec (XTASK-R027); `cargo xtask ci` passes.

## Test requirements

None (documentation task). The review is a spec review (REVIEW.md §2).

## Notes

Requirement IDs leave gaps between groups (template §6). Keep the API small: every public item has to be documented, tested and later snapshotted.

## Handoff

2026-11-09, implementation sub-agent (parent session `session/2026-11-09-m1-subagents`).
Journal: `docs/journal/2026-11-09-M1-T024.md`. Status: `review`.

**Done:**
- `docs/specs/liminal_memory.md` (`spec_version: 1`, `maturity: planned`): `Arena` (fixed
  capacity, `MAX_ALIGN = 4096`-aligned base, bump placement rule), `FrameAllocator` (arena reset
  by `begin_frame`, frame counting), `Pool<T>` + `PoolBox` (fixed blocks, LIFO slot order),
  `AlignedBuf<T: Copy>`, `AllocStats`, `MemoryError` (LIM-MEM-0001 … 0005). Requirements
  MEM-R001 … R081 (36 MUST, 1 SHOULD), invariants, unsafe inventory with soundness reasons,
  reference models as oracles, determinism workloads W1–W3 (cross-platform identity claimed,
  recorded literals), benchmark scenarios B1–B4 (no budgets; M1-T026 measures), open questions
  Q1–Q6.
- MUST assignment written into `requirements` of M1-T025 (17 IDs) and M1-T026 (19 IDs + SHOULD
  MEM-R074). M1-T025 "Notes" records a scope clarification: the arena's own aligned slice call
  and codes 0001–0004 belong to M1-T025.
- `docs/specs/README.md` index row.
- Fix outside the task, because it blocked `cargo xtask ci`: `xtask/src/repo.rs` test
  `loads_the_real_repository` asserted that `docs/specs/xtask.md` is the only spec. It now
  asserts that xtask's spec is found and that `_TEMPLATE.md` and `README.md` are skipped.

**For the reviewer (spec review, REVIEW.md §2):**
- `docs/specs/liminal_core.md` did not exist when this was written (M1-T012 runs in parallel).
  §5 uses a placeholder for the `liminal_core` error trait name (spec Q1); M1-T025 aligns it.
- Check the soundness table in §5 "Unsafe inside the crate", the check order of errors, and
  whether the `compile_fail` doc test approach on `#[cfg(doctest)]` functions (§10) is an
  acceptable `Verifies:` carrier.
- The assumption that a request of `isize::MAX / 2` bytes fails on all three OSes (test of
  LIM-MEM-0004) is unverified; M1-T025 confirms it.

**Traps:** any branch that adds a spec needs the `repo.rs` test fix above (M1-T012 will hit the
same failure); when merging, combine both versions.

## Review

_Empty._
