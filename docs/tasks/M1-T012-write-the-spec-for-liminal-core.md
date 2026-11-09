---
id: M1-T012
title: Write the spec for liminal_core
milestone: M1
status: review
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

- [x] `docs/specs/liminal_core.md` exists, follows `docs/specs/_TEMPLATE.md` (all sections), with
      `maturity: planned`, `spec_version: 1` and `adrs` listing every ADR it implements.
- [x] Every requirement has an ID `CORE-Rnnn`, an RFC 2119 level and a testable statement with
      units, ranges and failure behaviour. No vague words ("fast", "robust", "efficient").
- [x] §5 Public API covers the scope below and is consistent with `docs/ARCHITECTURE.md` and the
      specs of the crates it depends on.
- [x] Error codes `LIM-CORE-NNNN` are listed with message and `help` (ADR-0017), or the spec says
      why the crate has no recoverable errors.
- [x] The test plan names the test kinds that apply (TESTING.md) and, for every optimized path, the
      reference implementation it is checked against (AGENTS.md rule 4).
- [x] Determinism requirements follow ADR-0024: the spec names the workloads of
      `tests/determinism.rs` and says whether cross-platform bit-identity is claimed for each.
- [x] Performance budgets name scenarios and machines; numbers that need a measurement say which
      task measures them (PERFORMANCE.md: budgets are never invented).
- [x] Every MUST requirement is assigned to exactly one implementation task (M1-T013, M1-T014): the spec
      lists the assignment, and this session writes the IDs into those tasks' `requirements`
      frontmatter (`check-docs` R028 validates them).
- [x] Open questions are listed in the spec instead of being decided silently.
- [x] `docs/specs/README.md` lists the spec (XTASK-R027); `cargo xtask ci` passes.

## Test requirements

None (documentation task). The review is a spec review (REVIEW.md §2).

## Notes

Requirement IDs leave gaps between groups (template §6). Keep the API small: every public item has to be documented, tested and later snapshotted.

## Handoff

2026-11-09, implementation sub-agent (ADR-0026/0027) on branch
`task/M1-T012-write-the-spec-for-liminal-core`. Journal: `2026-11-09-M1-T012.md`. Status: `review`.

**Done:**
1. `docs/specs/liminal_core.md` (`spec_version` 1, `maturity: planned`, all template sections).
   57 requirements: error model R001–R013, handles R030–R040, hashing R050–R061, collections
   R070–R075, RNG R090–R098, time R110–R114, determinism workloads R130–R133. 11 error codes
   LIM-CORE-0001 … 0030 with message, help and context fields.
2. Requirement IDs written into M1-T013 (R001–R013, R030–R040) and M1-T014 (all others)
   `requirements` frontmatter; the assignment table is in spec §6.
3. `docs/specs/README.md` index row.
4. `xtask/src/repo.rs` test `loads_the_real_repository` no longer hard-codes the spec list (it
   failed as soon as a second spec existed; reason in the journal). This touches crate `xtask`.
5. `cargo xtask ci`: 11 of 11 steps pass (macOS 27.2, Rust 1.99.1).

**Decisions taken in the spec (reviewer, check these):**
- State hasher: XXH64, seed 0, little-endian canonical encoding through an own `StateHash` trait
  (not std `Hash`, whose impls are not guaranteed stable). 64-bit output.
- RNG: xoshiro256** seeded by SplitMix64 from XXH64(world seed LE ‖ stream key LE); bounded ints
  by Lemire's nearly-divisionless method, specified step by step so the number of draws is fixed.
- Collections: insertion-ordered `DetHashMap`/`DetHashSet` (order independent of hash values);
  order-preserving O(n) `remove`. `FixedState` for std maps where order does not matter.
- Handles: LIFO slot reuse, generation starts at 1, slots retired at `u32::MAX` (no ABA).
- Time: `TickRate` 1..=1000 Hz, integer `Tick`/`TickDuration`; the frame-time accumulator is left
  to `liminal_app`.
- No root re-exports; module paths only.

**Not done / for others:**
- ADR-0017 "Verification" (check-docs compares codes used in code with spec tables) has no task.
  Spec §12 item 3. Someone with authority to create tasks should add one (not created here to
  avoid task-ID collisions with parallel sub-agents).
- M1-T027 and M1-T047 name error areas `TEL` and `TK`; the reserved prefixes are `TELE` and
  `TKIT` (spec §12 item 4).
- Usable gate "one example or dependent crate" may not hold at the end of M1-T014 (spec §13).

**Traps:** do not put `|` in requirement table cells (breaks R040 row parsing); the word
"withdrawn" in parentheses marks a requirement withdrawn.

## Review

_Empty._
