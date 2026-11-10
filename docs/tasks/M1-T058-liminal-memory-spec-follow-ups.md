---
id: M1-T058
title: liminal_memory spec follow-ups from the M1-T024 review
milestone: M1
status: todo
crates: [liminal_memory]
depends_on: [M1-T024]
requirements: []
---

# M1-T058: `liminal_memory` spec follow-ups from the M1-T024 review

## Goal

`docs/specs/liminal_memory.md` leaves no point open to a wrong reading by its implementation
tasks (M1-T025, M1-T026), and its one diagram agrees with its placement rule.

## Context

- Follow-up task under ADR-0021, written by the review of M1-T024 (see that task's "Review").
- `docs/specs/liminal_memory.md` v1; `docs/specs/liminal_core.md` (on `main` since the M1-T012
  merge).
- In M1, not M2: every item amends a contract that M1-T025 and M1-T026 implement, and the ROADMAP
  M1 exit criteria need the crate at `usable`. Best done before M1-T025 starts. If M1-T025 or
  M1-T026 is already in progress or done, each item also checks the code.

## Scope

**In:**

1. **The §4 diagram contradicts the placement rule.** It shows `u64` at 0–8, `u32` at 8–12,
   padding 12–16, `[u8; 16]` at 16–32 and `used() = 32`. `[u8; 16]` has alignment 1, so by the
   placement rule (`start = round_up(used, a)`) it starts at 12 and `used()` is 28. Either change
   the third allocation to one that has alignment 8 or 16 (for example a `u64`, or
   `alloc_slice_fill_aligned::<u8>(16, 0, 16)`, and say so in the diagram), or redraw the offsets.
2. **Zero-size requests (MEM-R011, MEM-R003).** The spec says a request with `s == 0` succeeds and
   leaves `used()` unchanged, but not which address it returns. A `&mut T` of a zero-sized `T`
   and an empty `&mut [T]` must be non-null and aligned to `align_of::<T>()`, and
   `base + used()` is not aligned in general. State the returned address rule (as MEM-R063 does
   for `AlignedBuf`), including for `alloc_slice_fill_aligned(0, v, align)`, and for an arena of
   capacity 0. State that a zero-size request still runs check steps 1 to 3 of §5 (so
   `alloc_slice_fill_aligned(0, v, 3)` returns LIM-MEM-0002) and whether it counts in
   `allocations` (MEM-R070 says yes; keep the two in one place).
3. **`AlignedBuf::new` cases without a defined result (MEM-R060, MEM-R061).**
   - `align_of::<T>() > MAX_ALIGN` with a small `align` argument: §5 and MEM-R060 test only the
     argument against `MAX_ALIGN`, while the effective alignment is `max(align, align_of::<T>())`.
     Say whether this returns LIM-MEM-0003 (as `Arena::alloc*` and `Pool` do) or succeeds.
   - `len × size_of::<T>() ≤ isize::MAX`, but the size rounded up to `align()` exceeds
     `isize::MAX`: `Layout::from_size_align` rejects it (review probe, rustc 1.99.1:
     `Layout::from_size_align(isize::MAX as usize - 10, 64)` is `Err`, with alignment 1 it is
     `Ok`). MEM-R060 requires "never panics" and names no code for this input. MEM-R010 has the
     rounded rule for `Arena`; use the same wording here (LIM-MEM-0002).
   - The context fields of LIM-MEM-0004 (`size`, `align`) are stated for `Arena` (MEM-R010) but
     not for `AlignedBuf::new` and `Pool::with_capacity`.
4. **Where the pool's free list lives (MEM-R040, MEM-R045, MEM-R046, §5 unsafe table).** The slot
   order needs a stack of up to `capacity` freed slot numbers. An intrusive list inside the blocks
   needs `size_of::<T>() ≥ 4` and enough alignment for a `u32`, which `Pool<u8>` does not have; a
   separate array is a second reservation. The spec speaks of one base of
   `capacity × size_of::<T>()` bytes. State the storage (one reservation with a computed layout,
   or two reservations made at construction), and from it: the size checked against `isize::MAX`
   in MEM-R040, the `size` reported by LIM-MEM-0004, what "frees its base exactly once" covers in
   MEM-R046, and the unsafe-table row if a second block exists. `capacity_bytes` (MEM-R072) can
   stay `capacity × size_of::<T>()`; say that it excludes the free list.
5. **`FrameAllocator::begin_frame(&mut self) -> &Arena` keeps the allocator mutably borrowed.**
   While the returned `&Arena` is in use, `frame_index()`, `last_frame_bytes()`, `stats()` and
   `arena()` cannot be called (review probe on a reduced copy of the API, rustc 1.99.1: the call
   is rejected by the borrow checker; calling `begin_frame()` and then `arena()` compiles). Decide
   and write down one of: `begin_frame` returns `()`; or it keeps the return value and §5 states
   the borrow consequence and the pattern to use. MEM-R031 and the `compile_fail` test of MEM-R032
   follow the decision.
6. **Test plan gaps (§10).** MEM-R031 is in no row. The compile-time half of MEM-R040 (`Pool<T>`
   of a zero-sized `T` does not compile) is not in the `compile_fail` row. MEM-R074 (SHOULD) has
   no row. Add them. Also record in the §5 note on compile-time rejections what the review
   measured (rustc 1.99.1, reduced copy): a `const { assert!(…) }` in the generic function is
   reported by `cargo build` and by `rustdoc --test` `compile_fail` tests (also on a private
   `#[cfg(doctest)]` function, where the tests run and `/// Verifies:` attaches to the `fn` as
   XTASK-R041 requires), but **not** by `cargo check` or clippy, which do not monomorphize. So
   "does not compile" in MEM-R014 and MEM-R040 means "does not build"; say so.
7. **Open question Q1 can be closed against `docs/specs/liminal_core.md` v1.** Replace the
   placeholder `liminal_core::error::Error` by `liminal_core::error::EngineError`; `help()`
   returns `Option<String>`; `Display` is `liminal_core::error::write_display`; add to MEM-R002
   and MEM-R042 that every variant passes `validate_error(&e, "MEM")` (CORE-R012) and state the
   context key order per variant (the field names in §5 already match `^[a-z][a-z0-9_]*$`).
   Name the hasher and RNG items §11 uses (`StateHasher::write_usize` hashes `usize` as `u64`,
   which is what §11 asks for). If M1-T025 has done this already, check it and drop the item.
8. **Small text items.**
   - §2 and §5: allocation tracking covers `Arena`, `FrameAllocator` and `Pool`; `AlignedBuf`
     has no `stats()`. The M1-T024 scope says "per allocator"; say in §2 that `AlignedBuf` is not
     tracked, and why.
   - MEM-R046: `PoolBox` is "not `Send`"; say also that it is not `Sync` (it holds `&Pool`, which
     is not `Sync`), and add it to the `Send`/`Sync` rows of §10.
   - MEM-R012: "including arithmetic overflow" cannot happen once MEM-R015 has bounded `s` by
     `isize::MAX` and `used ≤ capacity ≤ isize::MAX`; either remove it or say that the
     implementation uses checked arithmetic anyway.
   - §11 W3: say what a drawn "drop" does when no box is live, and that an `alloc` on a full pool
     hashes the error code. W1: name the call used for "a request larger than the capacity".
   - §3 "`std` items used": add `core::slice::from_raw_parts(_mut)` and `core::ptr::NonNull` if
     used; `alloc_zeroed` is listed (also in the unsafe table) but no requirement needs zeroed
     memory. Remove it or say where it is used.
   - §13: M1-T025 implements `MemoryError` and MEM-R002, which need the `liminal_core` error model
     (M1-T013). M1-T025 depends on it only through M1-T014. Say so, or add M1-T013 to
     `depends_on` of M1-T025.

**Out:** Code (M1-T025, M1-T026). New features (growable arenas, a public counting allocator: spec
Q2, Q3).

## Acceptance criteria

- [ ] Items 1 to 8 are done, or a declined item is written into this task with the reason.
- [ ] Spec changes bump `spec_version` with a history row; requirement IDs added or moved are in
      exactly one task's `requirements` frontmatter (M1-T025 or M1-T026).
- [ ] `cargo xtask ci` passes.

## Test requirements

None (documentation task). The requirements it touches are tested by M1-T025 and M1-T026.

## Notes

The review probes (reduced copies, not committed) are described in
`docs/journal/2026-11-10-M1-T024-review.md`.

## Handoff

_Empty._

## Review

_Empty._
