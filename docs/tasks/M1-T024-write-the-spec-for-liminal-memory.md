---
id: M1-T024
title: Write the spec for liminal_memory
milestone: M1
status: done
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
- PERFORMANCE.md "Reference implementations reachable from benchmarks" (M1-T054): the spec's API section lists the crate's `reference` items for each optimized path.
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

2026-11-10, review sub-agent (ADR-0026/0027) of parent session `session/2026-11-10-m1-reviews`
(journal `2026-11-10-session-m1-reviews.md`), fresh context, prompt = the ADR-0026 review template.
This session implemented no part of M1-T024; the author was sub-agent no. 5 of the other parent
session `2026-11-09-session-m1-subagents.md`. macOS 27.2 (Darwin), Rust 1.99.1, worktree
`liminal-worktrees/M1-T024-review`, branch at `f3d1af0`. Task review (REVIEW.md §1) plus spec
review (REVIEW.md §2), as "Test requirements" says.

Outcome: **done**, no blocking findings (B1–B5). Follow-ups: M1-T058.

Checked:
- AC 1: the spec has the 14 template sections in order; `maturity: planned`, `spec_version: 1`;
  `adrs` `[ADR-0003, ADR-0009, ADR-0010, ADR-0011, ADR-0017, ADR-0018, ADR-0023, ADR-0024]`.
- AC 2: 37 requirement rows (script): 36 MUST, 1 SHOULD, IDs unique, each with a level. Word
  search for "fast", "robust", "efficient", "seamless", "powerful", "quickly", "cheap",
  "lightweight": no hit. Inputs whose result is not stated are follow-up items 2 and 3.
- AC 3: §5 covers every scope item (arena, frame allocator, pool, aligned buffers, tracking,
  unsafe inventory with the safe API, reference models). Compared with `docs/ARCHITECTURE.md` §2
  row, `architecture.toml` (layer 0, `unsafe = "allow"`, `deterministic = true`, dep
  `liminal_core`, dev `proptest`), ADR-0017, ADR-0018, ADR-0024 and `docs/specs/liminal_core.md`
  v1 as on `main` (it did not exist when the spec was written; the spec's placeholder and Q1 say
  so). Nothing in §5 or §8 conflicts with the core error contract (CORE-R005, R006, R012): the
  context field names match `^[a-z][a-z0-9_]*$`, area `MEM`. Closing Q1 is follow-up item 7.
- AC 3, API shape compiled (rustc 1.99.1, a reduced copy in the session scratch directory, not
  committed): `Arena::alloc<T>(&self, T) -> Result<&mut T, _>` with
  `const { assert!(!needs_drop::<T>()) }`, `reset(&mut self)`, `begin_frame(&mut self) -> &Arena`.
  `compile_fail` doc tests on a private `#[cfg(doctest)]` function run under `cargo test --doc`
  and pass for: a type that needs drop, use of a reference after `reset`. The same rejection is
  reported by `cargo build` and not by `cargo check` (item 6). The returned borrow of
  `begin_frame` blocks every other method of the allocator while it is used (item 5).
- The handoff's three questions: (1) soundness table read row by row against the API: borrows
  are tied to `&self`/`PoolBox`, resets need `&mut self`, `Send` without `Sync` follows from
  `Cell`, `PoolBox` is neither `Send` nor `Sync` through `&Pool`, forgotten boxes only leak; no
  unsound row found. The pool's free-list storage is not stated (item 4). (2) Check order of
  errors: consistent between §5, MEM-R003, MEM-R015, MEM-R040 and MEM-R060, except the two
  `AlignedBuf` inputs of item 3. (3) `/// Verifies:` on a `#[cfg(doctest)]` function satisfies
  XTASK-R041 (the annotation attaches to the next `fn`), and the doc test is the real check:
  acceptable.
- The `isize::MAX / 2` assumption: on this machine `std::alloc::alloc` of `isize::MAX / 2` bytes
  with alignment 4096 returns null. Windows, Linux and Miri: not verified (M1-T025, as the spec
  says).
- AC 4: five codes with message and help; every context field appears in its message.
- AC 5: test kinds per area are those of TESTING.md (unit, property, differential, integration,
  doc, Miri, determinism); reference models named for arena, pool, `AlignedBuf` and tracking;
  the spec states why no loom tests apply. Rows missing for MEM-R031 and one half of MEM-R040:
  item 6.
- AC 6: W1–W3 with sizes, operation mixes and what is hashed; each "cross-platform identity
  claimed: yes", "not yet verified", recorded literals (ADR-0024 item 3).
- AC 7: four scenarios, no number, reference machine, baseline assigned to M1-T026
  (`depends_on` M1-T017).
- AC 8: script over the spec and both task files: 36 MUST IDs in exactly one of M1-T025 (17) and
  M1-T026 (19 + SHOULD MEM-R074), none unknown, no overlap; §13 agrees.
- AC 9: six open questions, each with the task or milestone that resolves it.
- AC 10: `docs/specs/README.md` row. `cargo xtask ci` on `f3d1af0`: pass, 11 of 11 steps, none
  skipped. Branch CI run 42686522939 on `f3d1af0`: success.
- Read for correctness: placement rule against the §4 diagram (they disagree: item 1); INV-1 to
  INV-6 against the requirements; MEM-R070 to MEM-R072 against `AllocStats`; slot order against
  the reference pool model.
- Oracle change: `xtask/src/repo.rs` `loads_the_real_repository`, exact spec list replaced by
  membership and exclusion checks. Justified in the journal ("Oracle changes"). `main` already
  carries an equivalent change from M1-T012; the merge keeps one copy of the assertions.
- No fake progress: no code; scope not reduced; the M1-T025 scope clarification is written into
  that task's "Notes" with the reason.
- Session boundaries (item 10): the author is row 5 of its parent's journal, "impl template".
  The author's journal names its parent as "`2026-11-09-m1-subagents` session journal", not the
  file name (ADR-0026 Decision 8.2); the parent's entry records that. No install or nested spawn
  recorded.
- ADR-0028: no workflow change; no credentials or personal data in the diff; no issue, pull
  request or outside code acted on.

Follow-ups (M1-T058): the §4 diagram contradicts the placement rule; address of zero-size
requests; two `AlignedBuf::new` inputs without a defined result and LIM-MEM-0004 fields for
`AlignedBuf` and `Pool`; storage of the pool free list; `begin_frame`'s returned borrow; test-plan
rows and "does not compile" meaning "does not build"; closing Q1 against the core spec; small
text items.

Whoever starts M1-T025 before M1-T058 is done: do not take offsets from the §4 diagram (item 1);
the placement rule in §4 and MEM-R011 is the contract.
