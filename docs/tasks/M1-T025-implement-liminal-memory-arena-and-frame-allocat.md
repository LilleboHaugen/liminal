---
id: M1-T025
title: Implement liminal_memory — arena and frame allocators
milestone: M1
status: todo
crates: [liminal_memory]
depends_on: [M1-T024, M1-T014, M1-T018]
requirements: [MEM-R001, MEM-R002, MEM-R003, MEM-R010, MEM-R011, MEM-R012, MEM-R013, MEM-R014, MEM-R015, MEM-R016, MEM-R017, MEM-R018, MEM-R019, MEM-R030, MEM-R031, MEM-R032, MEM-R033]
---

# M1-T025: Implement `liminal_memory`: arena and frame allocators

## Goal

Arena and frame allocators exist with a safe API, checked by Miri.

## Context

- `docs/specs/liminal_memory.md`; ADR-0018.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Crate, arena and frame allocators, property tests against the reference allocator, Miri.

**Out:** Pool, aligned allocation, tracking: next task.

## Acceptance criteria

- [ ] The crate exists at the path in `architecture.toml`, is a workspace member, has
      `#![forbid(unsafe_code)]` if `unsafe = "forbid"`, and has a floor in `ci/coverage.toml`.
- [ ] The crate's clippy determinism configuration is in place.
- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.
- [ ] Miri passes on the crate's tests; every `unsafe` block has a `// SAFETY:` comment.

## Test requirements

Requirements in frontmatter; property tests (alignment, no overlap, reset semantics); Miri.

## Notes

- Scope clarification (M1-T024, 2026-11-09): `docs/specs/liminal_memory.md` assigns the arena's
  own aligned slice call (`Arena::alloc_slice_fill_aligned`, MEM-R003, MEM-R015) and the error
  codes LIM-MEM-0001 to LIM-MEM-0004 (MEM-R002) to this task, because they are part of the `Arena`
  type. "Aligned allocation" in **Out** means `AlignedBuf` (M1-T026). Allocation statistics
  (`Arena::stats`, MEM-R070) stay in M1-T026.
- ~~The spec's §5 uses a placeholder name for the `liminal_core` error trait (spec Q1); use the
  names from `docs/specs/liminal_core.md` and update the spec if they differ.~~ Done by M1-T058
  (2026-11-10, spec v2): §5 names `liminal_core::error::EngineError`, `write_display` and
  `validate_error`, and states the context key order per variant (MEM-R002).
- Spec v2 changes that affect this task (M1-T058, 2026-11-10): `FrameAllocator::begin_frame`
  returns `()` (MEM-R031, MEM-R032); zero-size requests have an address rule and still run check
  steps 1 to 3 (§4 "Zero-size request", MEM-R011); "does not compile" in MEM-R014 means "does
  not build" (`cargo check` does not report it); the §4 diagram now agrees with the placement
  rule.

## Handoff

_Empty._

## Review

_Empty._
