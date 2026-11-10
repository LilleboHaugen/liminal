---
crate: liminal_memory
req_prefix: MEM
layer: 0
maturity: planned
spec_version: 2
last_reviewed: 2026-11-09
adrs: [ADR-0003, ADR-0009, ADR-0010, ADR-0011, ADR-0017, ADR-0018, ADR-0023, ADR-0024]
---

# Spec: `liminal_memory`

## 1. Purpose

`liminal_memory` provides the engine's explicit allocators: a bump arena, a frame allocator built on
it (reset once per frame), a typed pool of fixed-size blocks, and heap buffers with a chosen
alignment. Each allocator reserves its memory from the system allocator once, at construction, and
then serves allocations without further system calls, so per-frame hot paths can run without heap
allocation (`docs/process/CODE_STYLE.md`, "Performance conventions"). Each allocator reports its own
usage (counts, bytes, high-water mark). The crate is separate because it is the one place in layer 0
where allocator `unsafe` lives (ADR-0018): every other crate uses the safe API and needs no `unsafe`
for scratch memory.

## 2. Scope

**In scope:**
- `Arena`: fixed-capacity bump allocator; typed values, slices, slices with a requested alignment;
  `reset`.
- `FrameAllocator`: an `Arena` that is reset at each frame boundary, with frame counting.
- `Pool<T>`: fixed number of blocks of one type, RAII handles (`PoolBox`), deterministic slot order.
- `AlignedBuf<T>`: owned heap buffer of `Copy` elements with a chosen alignment (SIMD, cache line,
  GPU upload staging).
- Allocation tracking (`AllocStats`) for `Arena`, `FrameAllocator` and `Pool<T>`. `AlignedBuf<T>`
  is not tracked and has no `stats()`: it is one reservation of a fixed size made by `new`, with
  no allocation call afterwards, so `len() × size_of::<T>()` and `align()` already state
  everything a counter could.
- The error type `MemoryError` with codes `LIM-MEM-0001` … `LIM-MEM-0005`.
- The `unsafe` inside these types and the safe API around it (§5 "Unsafe inside the crate").

**Out of scope / non-goals:**
- Replacing or wrapping the process-wide `#[global_allocator]`, and counting global heap allocations
  (open question Q3).
- Growable arenas (chunk chaining) and arena-backed collections (`Vec` in an arena; the
  `allocator_api` is unstable): open question Q2.
- Allocators that keep data alive for more than one frame (double-buffered frame data): Q4.
- Thread-local or per-worker allocators: decided by the `liminal_jobs` spec (M1-T034), Q5.
- GPU memory (`liminal_gpu`), ECS column storage (`liminal_ecs` owns its storage), asset memory and
  streaming (`liminal_assets`).
- Reporting allocator statistics to telemetry: `liminal_telemetry` is layer 0 with only
  `liminal_core` as same-layer dependency, so it cannot read `AllocStats`. A higher-layer crate
  (`liminal_app` or `liminal_engine`) copies the numbers into telemetry counters.
- Thread-safe (`Sync`) allocators. Every allocator here is used from one thread at a time.

## 3. Dependencies

- Engine crates: `liminal_core` (same layer, `architecture.toml` `same_layer_deps`), for
  `liminal_core::error::{EngineError, ErrorCode, ErrorContext, write_display}` (ADR-0017). Used by
  tests only: `liminal_core::error::validate_error`, and in `tests/determinism.rs` (ADR-0024)
  `liminal_core::rng::{Rng, StreamKey}` and `liminal_core::hash::StateHasher`.
- External crates: none at runtime. Dev: `proptest` (`architecture.toml` `dev_external`,
  approved for property tests).
- `std` items used: `std::alloc::{alloc, dealloc, Layout}`, `core::ptr` (including
  `core::ptr::NonNull`), `core::slice::{from_raw_parts, from_raw_parts_mut}`, `core::cell::Cell`,
  `core::mem::{needs_drop, size_of, align_of}`. `alloc_zeroed` is not used: no requirement needs
  zeroed memory, because every byte handed out is written by the copy, fill or move of the call
  that hands it out.
- `architecture.toml`: layer 0, `unsafe = "allow"`, `deterministic = true`.

## 4. Concepts

- **Base:** the one block an allocator reserves from the system allocator at construction. Every
  allocator has at most one. An `Arena` base is aligned to `MAX_ALIGN`. A capacity of 0 reserves
  nothing. The pool's base also holds its free list (**Pool layout** below).
- **Request:** one allocation call, described by a size `s` in bytes and an alignment `a` in bytes
  (a power of two, `1 ≤ a ≤ MAX_ALIGN`). For a value of type `T`: `s = size_of::<T>()`,
  `a = align_of::<T>()`. For a slice of `len` elements: `s = len × size_of::<T>()`.
- **Effective alignment:** the `a` of a request. `align_of::<T>()` for calls without an `align`
  argument; `max(align, align_of::<T>())` for `Arena::alloc_slice_fill_aligned` and
  `AlignedBuf::new`.
- **Used (`used()`):** for an `Arena`, the offset in bytes from the base to the end of the last
  successful non-empty request, padding included. Starts at 0; `reset` sets it to 0.
- **Placement rule (bump):** a request `(s, a)` with `s > 0` starts at
  `start = round_up(used, a)`. It succeeds if and only if `start + s ≤ capacity`; then
  `used = start + s`. Since the base is aligned to `MAX_ALIGN ≥ a`, the address is a multiple of `a`.
  The placement rule does not apply to a request with `s = 0` (next item).
- **Zero-size request:** a request with `s = 0`: a value of a zero-sized type, a slice with
  `len == 0`, or a slice of any `len` whose element type is zero-sized. This item is the one
  statement of its rules; MEM-R011, MEM-R063 and MEM-R070 refer to it.
  - It passes through check steps 1 to 3 of §5 like every other request, so
    `alloc_slice_fill_aligned(0, v, 3)` returns LIM-MEM-0002 and
    `alloc_slice_fill_aligned(0, v, 8192)` returns LIM-MEM-0003. Step 3 always passes (the total
    size is 0, also for a zero-sized element type with `len == usize::MAX`).
  - If the checks pass it succeeds, whatever `used()` and `capacity()` are, also in an arena of
    capacity 0 (which has no base). `used()` is unchanged: no padding is added.
  - The returned `&mut T` or `&mut [T]` (of the requested `len`) has an address that is non-null
    and a multiple of the effective alignment `a`. The address need not lie inside the base and is
    otherwise unspecified (the implementation uses the dangling address `a`); two zero-size
    requests may return the same address. `base + used()` is not used, since it is not a multiple
    of `a` in general and does not exist for capacity 0.
  - It counts as one successful call in `AllocStats::allocations`; `live_bytes` and `peak_bytes`
    do not change.
  - The same address rule holds for `AlignedBuf::new` when `len × size_of::<T>() == 0`
    (MEM-R063).
- **Frame:** the interval between two calls of `FrameAllocator::begin_frame`. Frame 0 is the
  interval before the first call. `frame_index()` is the number of `begin_frame` calls so far.
- **Slot:** one block of a `Pool<T>`, numbered `0 … capacity − 1`. A slot is either *live* (owned
  by exactly one `PoolBox`) or *free*.
- **Slot order:** `Pool::alloc` takes the most recently freed slot (LIFO). If no freed slot exists,
  it takes the lowest slot that has never been used. This order depends only on the sequence of
  `alloc` and `PoolBox` drop calls.
- **Pool layout:** a `Pool<T>` with `blocks > 0` makes one reservation at construction that holds
  two areas. With `S = size_of::<T>()`:
  - the block area, bytes `[0, blocks × S)`: slot `i` is at offset `i × S`;
  - the free list, bytes `[list_offset, list_offset + blocks × 4)` with
    `list_offset = round_up(blocks × S, 4)`: a stack of up to `blocks` freed slot numbers, each a
    `u32`. A separate array is needed because a list threaded through the free blocks would need
    `S ≥ 4` and `align_of::<T>() ≥ 4`, which `Pool<u8>` does not have.

  `pool_size = list_offset + blocks × 4` and `pool_align = max(align_of::<T>(), 4)` are the size
  and alignment of the reservation. The number of freed slots on the stack and the number of slots
  ever used are kept in `Cell`s inside the `Pool` value. `blocks == 0` reserves nothing.
- **Live bytes / peak bytes:** see `AllocStats` (§5). Peak bytes is the high-water mark since
  construction.

```
Arena (capacity 64, MAX_ALIGN-aligned base) after alloc(1u64), alloc(2u32), alloc(3u64)
offset: 0         8    12   16        24                                64
        [u64 ....][u32][pad][u64 ....][             free               ]
                                      ^ used() = 24
```

The second `u64` has alignment 8, so it starts at `round_up(12, 8) = 16` after 4 bytes of padding.
A `[u8; 16]` in its place has alignment 1: it starts at 12 with no padding, and `used()` is 28.

## 5. Public API

All items are at the crate root. `liminal_core` names are those of `docs/specs/liminal_core.md`
`spec_version` 1 (§5.1 there).

```rust
/// Largest alignment any allocator in this crate accepts, in bytes. Value: 4096.
/// Chosen as the x86-64 page size; it covers SIMD (16–64), cache lines (64/128) and
/// GPU staging offsets (256).
pub const MAX_ALIGN: usize = 4096;

/// Usage counters of one allocator. All counters start at 0 at construction.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AllocStats {
    /// Successful allocation calls (zero-size requests included, §4).
    pub allocations: u64,
    /// Allocation calls that returned an error (any code).
    pub failed_allocations: u64,
    /// Bytes currently in use (Arena: `used()`, padding included; Pool: live slots × size_of::<T>()).
    pub live_bytes: usize,
    /// Largest `live_bytes` value since construction (not lowered by reset).
    pub peak_bytes: usize,
    /// Capacity in bytes (Arena: capacity; Pool: capacity × size_of::<T>(), the block area
    /// only: the free list of the pool layout (§4) is not counted).
    pub capacity_bytes: usize,
    /// Number of resets (Arena: `reset` calls; FrameAllocator: `begin_frame` calls; Pool: 0).
    pub resets: u64,
}

/// Fixed-capacity bump allocator. `Send`, not `Sync`. Never runs destructors.
pub struct Arena { /* private */ }

impl Arena {
    /// Reserves `capacity` bytes, aligned to MAX_ALIGN. `capacity == 0` reserves nothing.
    /// Errors: LIM-MEM-0002 if `capacity` rounded up to MAX_ALIGN exceeds isize::MAX;
    /// LIM-MEM-0004 if the system allocator returns null.
    pub fn with_capacity(capacity: usize) -> Result<Arena, MemoryError>;
    pub fn capacity(&self) -> usize;
    pub fn used(&self) -> usize;
    /// Moves `value` into the arena. Build error if `core::mem::needs_drop::<T>()`
    /// (see "Compile-time rejections" below).
    /// Errors: LIM-MEM-0003 if align_of::<T>() > MAX_ALIGN; LIM-MEM-0001 if it does not fit.
    pub fn alloc<T>(&self, value: T) -> Result<&mut T, MemoryError>;
    /// Copies `src` into the arena.
    pub fn alloc_slice_copy<T: Copy>(&self, src: &[T]) -> Result<&mut [T], MemoryError>;
    /// `len` copies of `value`.
    pub fn alloc_slice_fill<T: Copy>(&self, len: usize, value: T) -> Result<&mut [T], MemoryError>;
    /// Like `alloc_slice_fill`, with alignment max(align, align_of::<T>()).
    pub fn alloc_slice_fill_aligned<T: Copy>(&self, len: usize, value: T, align: usize)
        -> Result<&mut [T], MemoryError>;
    /// Sets `used()` to 0. Taking `&mut self` ends every borrow handed out before.
    pub fn reset(&mut self);
    pub fn stats(&self) -> AllocStats;
}
impl core::fmt::Debug for Arena; // capacity, used, stats; never an address

/// An Arena reset at every frame boundary. `Send`, not `Sync`.
pub struct FrameAllocator { /* private */ }

impl FrameAllocator {
    /// Same rules and errors as Arena::with_capacity. frame_index() == 0, last_frame_bytes() == 0.
    pub fn with_capacity(capacity: usize) -> Result<FrameAllocator, MemoryError>;
    /// Ends the current frame and starts the next: records used() as last_frame_bytes,
    /// resets the arena and increments frame_index by 1. Returns nothing; get the arena of
    /// the new frame with `arena()` (see "`begin_frame` returns nothing" below).
    pub fn begin_frame(&mut self);
    /// The arena of the current frame.
    pub fn arena(&self) -> &Arena;
    pub fn frame_index(&self) -> u64;
    /// used() of the arena at the end of the previous frame, in bytes. 0 before the first begin_frame.
    pub fn last_frame_bytes(&self) -> usize;
    pub fn stats(&self) -> AllocStats;
}
impl core::fmt::Debug for FrameAllocator;

/// `capacity` blocks of `T`. `Send` if `T: Send`; never `Sync`.
pub struct Pool<T> { /* private */ }

impl<T> Pool<T> {
    /// Build error if size_of::<T>() == 0 (see "Compile-time rejections" below).
    /// Errors: LIM-MEM-0003 if align_of::<T>() > MAX_ALIGN; LIM-MEM-0002 if blocks > u32::MAX
    /// or the size of the pool layout (§4) exceeds isize::MAX; LIM-MEM-0004 if the system
    /// allocator returns null. `blocks == 0` reserves nothing.
    pub fn with_capacity(blocks: usize) -> Result<Pool<T>, MemoryError>;
    pub fn capacity(&self) -> usize;
    /// Number of live slots.
    pub fn live(&self) -> usize;
    /// Moves `value` into a slot chosen by the slot order (§4).
    /// Errors: LIM-MEM-0005 if every slot is live; `value` is then dropped before returning.
    pub fn alloc(&self, value: T) -> Result<PoolBox<'_, T>, MemoryError>;
    pub fn stats(&self) -> AllocStats;
}
impl<T> core::fmt::Debug for Pool<T>; // capacity, live, stats

/// Owner of one live slot. Dropping it drops the value and frees the slot.
/// Neither `Send` nor `Sync` (it holds a `&Pool<T>`, and `Pool<T>` is not `Sync`).
pub struct PoolBox<'p, T> { /* private */ }
impl<'p, T> PoolBox<'p, T> {
    /// Slot number, 0 ≤ slot < capacity.
    pub fn slot(&self) -> u32;
}
impl<T> core::ops::Deref for PoolBox<'_, T> { type Target = T; }
impl<T> core::ops::DerefMut for PoolBox<'_, T>;
impl<T> Drop for PoolBox<'_, T>;
impl<T: core::fmt::Debug> core::fmt::Debug for PoolBox<'_, T>;

/// Owned heap buffer of `len` elements whose first element is aligned to `align()`.
/// `Send`/`Sync` exactly when `T` is.
pub struct AlignedBuf<T: Copy> { /* private */ }
impl<T: Copy> AlignedBuf<T> {
    /// Errors: LIM-MEM-0002 if `align` is not a power of two, or if len × size_of::<T>()
    /// rounded up to a multiple of the effective alignment exceeds isize::MAX; LIM-MEM-0003
    /// if the effective alignment max(align, align_of::<T>()) > MAX_ALIGN; LIM-MEM-0004 if
    /// the system allocator returns null.
    pub fn new(len: usize, value: T, align: usize) -> Result<AlignedBuf<T>, MemoryError>;
    /// Effective alignment: max(align, align_of::<T>()).
    pub fn align(&self) -> usize;
}
impl<T: Copy> core::ops::Deref for AlignedBuf<T> { type Target = [T]; }
impl<T: Copy> core::ops::DerefMut for AlignedBuf<T>;
impl<T: Copy> Drop for AlignedBuf<T>;
impl<T: Copy + core::fmt::Debug> core::fmt::Debug for AlignedBuf<T>;

/// Errors of this crate. Codes in §8.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemoryError {
    CapacityExhausted { requested: usize, align: usize, available: usize }, // LIM-MEM-0001
    InvalidLayout { elem_size: usize, len: usize, align: usize },          // LIM-MEM-0002
    UnsupportedAlignment { align: usize, max: usize },                      // LIM-MEM-0003
    SystemAllocationFailed { size: usize, align: usize },                   // LIM-MEM-0004
    PoolExhausted { capacity: usize },                                      // LIM-MEM-0005
}
impl liminal_core::error::EngineError for MemoryError; // code, message, help, context (below)
impl core::fmt::Display for MemoryError;               // liminal_core::error::write_display
impl std::error::Error for MemoryError;
```

Notes on the API:
- **`MemoryError` as an `EngineError`** (`docs/specs/liminal_core.md` §5.1). `code()` returns
  `ErrorCode::new("MEM", n)` with `n` from §8. `message()` returns the §8 message with the field
  values in decimal. `help()` returns `Some` of the §8 help text for every variant, never `None`.
  `context()` holds exactly the variant's fields, each as `ContextValue::UInt`, with the field
  name as key, in the order of the enum declaration above:

  | Variant | Context keys, in order |
  |---|---|
  | `CapacityExhausted` | `requested`, `align`, `available` |
  | `InvalidLayout` | `elem_size`, `len`, `align` |
  | `UnsupportedAlignment` | `align`, `max` |
  | `SystemAllocationFailed` | `size`, `align` |
  | `PoolExhausted` | `capacity` |

  `Display` calls `liminal_core::error::write_display`, so the text form is
  `{code}: {message} help: {help}`. Every variant value passes
  `liminal_core::error::validate_error(&e, "MEM")` (CORE-R012; MEM-R002, MEM-R042).
- **Context fields of `InvalidLayout`.** `with_capacity(n)` of `Arena`/`FrameAllocator` reports
  `elem_size: 1, len: n, align: MAX_ALIGN`. Slice calls and `AlignedBuf::new` report
  `size_of::<T>()`, `len`, and as `align` the `align` argument if the call has one, otherwise
  `align_of::<T>()`. `Pool::with_capacity` reports `size_of::<T>()`, `blocks`, `align_of::<T>()`.
- **Context fields of `UnsupportedAlignment`.** `align` is the effective alignment (§4) of the
  call (`align_of::<T>()` for `Pool::with_capacity`); `max` is `MAX_ALIGN`.
- **Context fields of `SystemAllocationFailed`.** `size` and `align` are the size and alignment
  of the `Layout` passed to the system allocator: `Arena`/`FrameAllocator::with_capacity(n)`:
  `n` and `MAX_ALIGN`; `Pool::with_capacity`: `pool_size` and `pool_align` (§4);
  `AlignedBuf::new`: `len × size_of::<T>()` and the effective alignment.
- **Check order** (so the error for a request with several faults is defined): (1) the `align`
  argument, if the call has one, is a power of two (else 0002), (2) the effective alignment is
  ≤ `MAX_ALIGN` (else 0003), (3) total size fits (else 0002), (4) capacity (0001 or 0005),
  (5) system allocation (0004, constructors only). "Total size fits" in step 3 means:
  - `Arena` slice calls: `len × size_of::<T>()` does not overflow and is ≤ `isize::MAX`;
  - `Arena`/`FrameAllocator::with_capacity(n)`: `n` rounded up to a multiple of `MAX_ALIGN` is
    ≤ `isize::MAX`;
  - `AlignedBuf::new`: `len × size_of::<T>()` does not overflow and, rounded up to a multiple of
    the effective alignment, is ≤ `isize::MAX`;
  - `Pool::with_capacity`: `blocks ≤ u32::MAX`, and `pool_size` (§4) is computed without
    overflow and, rounded up to a multiple of `pool_align`, is ≤ `isize::MAX`.

  The rounded forms are the condition of `core::alloc::Layout::from_size_align`, so a request
  that passes step 3 always has a valid `Layout` and construction cannot panic there (measured by
  the M1-T024 review, rustc 1.99.1: `Layout::from_size_align(isize::MAX as usize - 10, 64)` is
  `Err`, with alignment 1 it is `Ok`).
- **`CapacityExhausted` fields:** `requested = s`, `align = a` (effective alignment),
  `available = capacity − used()` before the call (padding not subtracted).
- **`begin_frame` returns nothing.** A `begin_frame(&mut self) -> &Arena` would keep the
  `FrameAllocator` mutably borrowed for as long as the returned reference is used, so
  `frame_index()`, `last_frame_bytes()`, `stats()` and `arena()` could not be called during the
  frame (measured by the M1-T024 review on a reduced copy of the API, rustc 1.99.1: the borrow
  checker rejects it). The one pattern is therefore:

  ```rust
  frame.begin_frame();          // &mut borrow ends here
  let arena = frame.arena();    // shared borrow for the frame
  let xs = arena.alloc_slice_fill(16, 0u32)?;
  let n = frame.frame_index();  // allowed: shared borrows only
  ```

  A reference from `arena()` or from an allocation is a shared borrow of the `FrameAllocator`,
  so the next `begin_frame` does not compile while one is still used (MEM-R032).
- **Compile-time rejections** (`Arena::alloc::<T>` of a type that needs drop;
  `Pool::<T>::with_capacity` of a zero-sized `T`) use an inline `const { assert!(…) }` in that
  generic function, so they fail when the function is monomorphized. "Does not compile" in
  MEM-R014 and MEM-R040 therefore means **does not build**: the error (E0080) is reported by
  `cargo build`, `cargo test` and rustdoc `compile_fail` doc tests, and **not** by `cargo check`
  or clippy, which do not monomorphize. The type `Pool<T>` of a zero-sized `T` can be named; the
  first use of its constructor fails the build. Measured by the M1-T024 review on a reduced copy
  of this API (rustc 1.99.1): `cargo build` fails, `cargo check` reports nothing, and a
  `compile_fail` doc test on a private `#[cfg(doctest)]` function runs under `cargo test --doc`
  and passes for the arena case; a `/// Verifies:` line there attaches to that `fn`
  (XTASK-R041). The pool case uses the same mechanism and was not measured; M1-T026 verifies it.

### Unsafe inside the crate (ADR-0018)

The public API has no `unsafe fn`, no `unsafe trait` and no function whose correct use needs
`unsafe` by the caller. `unsafe` is used only for:

| Operation | Where | Why it is sound |
|---|---|---|
| `std::alloc::alloc` / `dealloc` of the base | constructors, `Drop` of `Arena`, `Pool`, `AlignedBuf` | Each value makes at most one reservation. The same `Layout` is stored and used for `dealloc`; size 0 never calls the system allocator. |
| A dangling, aligned, non-null pointer turned into `&mut T` or `&mut [T]` | zero-size requests of `Arena` (§4), `AlignedBuf` with `len × size_of::<T>() == 0` | A reference to zero bytes is valid for any non-null address that is a multiple of the type's alignment; no byte is read or written through it. |
| Read and write of `u32` free-list entries at `base + list_offset + 4 × i` | `Pool::alloc`, `Drop for PoolBox` | `list_offset` is a multiple of 4 and the base is aligned to `pool_align ≥ 4`; `i < blocks`, so the entry lies inside the reservation and outside the block area (pool layout, §4); an entry is read only below the stack length, where it was written before. |
| `base.add(start)`, `ptr::write(value)` and `&mut *ptr` | `Arena::alloc*`, `Pool::alloc` | `start + s ≤ capacity` (placement rule); regions of live allocations are disjoint (INV-2, INV-3); the returned borrow is tied to `&self` (Arena) or to the `PoolBox`, and `reset`/`begin_frame` need `&mut self`, so no reference survives a reset. |
| `ptr::drop_in_place` and slot release | `Drop for PoolBox` | The `PoolBox` is the only owner of its live slot; the slot is pushed to the free list after the drop. |
| `slice::from_raw_parts(_mut)` | `Arena` slice calls, `AlignedBuf` deref | Elements were fully initialised by the copy or fill before the slice is created. |
| `unsafe impl Send` for `Arena`, `FrameAllocator`, `Pool<T: Send>`; `Send`/`Sync` for `AlignedBuf<T>` | type definitions | Each type owns its base exclusively; interior mutability is `Cell` only, so none is `Sync` except `AlignedBuf`, which has no interior mutability. |

The `unsafe` is required for function, not only for speed: handing out `&mut T` of different
types from one shared buffer is not expressible in safe Rust. The benchmark against the system
allocator (§9, M1-T026) records what the crate gains over `Box`.

## 6. Requirements

### General (R001–R009)

| ID | Level | Requirement |
|---|---|---|
| MEM-R001 | MUST | No public function or trait method of `Arena`, `FrameAllocator` or `MemoryError` panics, for any argument values and any call sequence the safe API allows. Every failure is returned as a `MemoryError` (verified by property tests over generated call sequences). |
| MEM-R002 | MUST | `MemoryError` implements `liminal_core::error::EngineError` (ADR-0017). For the variants with codes LIM-MEM-0001 to LIM-MEM-0004: `code()` returns the code in §8; `message()` is the §8 message and contains every context field value of the variant; `help()` returns `Some` of the §8 text; `context()` holds the variant's fields in the key order of §5; `Display` is `liminal_core::error::write_display`; and a value of each variant passes `liminal_core::error::validate_error(&e, "MEM")` (CORE-R012). |
| MEM-R003 | MUST | Arena alignment rules: `MAX_ALIGN` is 4096. `Arena::alloc_slice_fill_aligned` with an `align` that is not a power of two (including 0) returns LIM-MEM-0002; with a power of two above `MAX_ALIGN` it returns LIM-MEM-0003. `Arena::alloc*` with `align_of::<T>() > MAX_ALIGN` returns LIM-MEM-0003. Checks follow the order in §5. |

### Arena (R010–R029)

| ID | Level | Requirement |
|---|---|---|
| MEM-R010 | MUST | `Arena::with_capacity(n)` returns an arena with `capacity() == n` and `used() == 0`. If `n` rounded up to a multiple of 4096 exceeds `isize::MAX`, it returns LIM-MEM-0002 with `elem_size: 1, len: n, align: 4096`. If the system allocator returns null, it returns LIM-MEM-0004 with `size: n, align: 4096` and does not abort the process. `n == 0` makes no call to the global allocator. |
| MEM-R011 | MUST | Every successful `Arena` allocation follows the placement rule (§4): for a request `(s, a)` with `s > 0`, `used()` afterwards equals `round_up(used_before, a) + s`, and the returned address is a multiple of `a`. A zero-size request (`s == 0`, §4) that passes check steps 1 to 3 of §5 succeeds for every `used()` and `capacity()`, including capacity 0, leaves `used()` unchanged, and returns a reference or slice (of the requested `len`) whose address is non-null and a multiple of `a`. A zero-size request that fails step 1 or 2 returns that step's error (`alloc_slice_fill_aligned(0, v, 3)`: LIM-MEM-0002). |
| MEM-R012 | MUST | A request that does not fit (`s > 0` and `round_up(used, a) + s > capacity`; the sum cannot overflow `usize`, because `used ≤ capacity ≤ isize::MAX`, `a ≤ 4096` and `s ≤ isize::MAX` after check step 3) returns LIM-MEM-0001 with `requested: s`, `align: a`, `available: capacity − used()`, and leaves `used()` and the contents of all earlier allocations unchanged. |
| MEM-R013 | MUST | Live allocations from one arena never overlap, and writing through one allocation never changes the value read through another. Checked by a differential property test against the reference arena model (§10) over generated sequences of at least 256 operations per case. |
| MEM-R014 | MUST | A crate that calls `Arena::alloc::<T>` with `core::mem::needs_drop::<T>()` true does not build (§5 "Compile-time rejections": reported by `cargo build`, `cargo test` and `compile_fail` doc tests, not by `cargo check`). The arena never runs a destructor. |
| MEM-R015 | MUST | `alloc_slice_copy(src)` returns a slice equal to `src`; `alloc_slice_fill(len, v)` and `alloc_slice_fill_aligned(len, v, align)` return a slice of `len` elements, each equal to `v`. `alloc_slice_fill_aligned` aligns to `max(align, align_of::<T>())`. If `len × size_of::<T>()` overflows or exceeds `isize::MAX`, the call returns LIM-MEM-0002 and `used()` is unchanged. |
| MEM-R016 | MUST | `Arena::reset` sets `used()` to 0 and keeps `capacity()`. After `reset`, the full capacity is available again (an allocation of `capacity()` bytes with alignment 1 succeeds). A reference obtained before `reset` cannot be used after it (`compile_fail` test). |
| MEM-R017 | MUST | `Arena` and `FrameAllocator` are `Send` and not `Sync`. |
| MEM-R018 | MUST | Dropping an `Arena` or `FrameAllocator` returns its base to the system allocator exactly once (Miri: no leak, no double free) and runs no destructor of values stored in it. |
| MEM-R019 | MUST | After construction, `Arena::alloc*`, `Arena::reset`, `Arena::used`, `Arena::capacity` make no call to the global allocator, on success and on failure (checked with a per-thread counting `#[global_allocator]` in a dedicated test binary). |

### Frame allocator (R030–R039)

| ID | Level | Requirement |
|---|---|---|
| MEM-R030 | MUST | `FrameAllocator::with_capacity(n)` follows the rules and errors of MEM-R010; the new allocator has `frame_index() == 0`, `last_frame_bytes() == 0` and `arena().capacity() == n`. |
| MEM-R031 | MUST | `begin_frame` sets `last_frame_bytes()` to `arena().used()` as it was before the call, resets the arena (`arena().used() == 0` afterwards, `arena().capacity()` unchanged) and increments `frame_index()` by exactly 1. It returns `()`; `arena()` called afterwards is the arena of the new frame, and `frame_index()`, `last_frame_bytes()`, `stats()` and `arena()` can be called while a reference from `arena()` is in use. |
| MEM-R032 | MUST | A reference obtained during a frame (from `arena()` or from an allocation in it) cannot be used after the next `begin_frame` call (`compile_fail` test: `let a = f.arena(); let x = a.alloc(1u32)?; f.begin_frame(); *x += 1;`. Its compiling counterpart omits the use after `begin_frame`). |
| MEM-R033 | MUST | `begin_frame`, `arena`, `frame_index` and `last_frame_bytes` make no call to the global allocator (same check as MEM-R019). |

### Pool (R040–R059)

| ID | Level | Requirement |
|---|---|---|
| MEM-R040 | MUST | `Pool::<T>::with_capacity(blocks)` returns a pool with `capacity() == blocks` and `live() == 0`. It returns LIM-MEM-0003 if `align_of::<T>() > 4096` (`align: align_of::<T>()`, `max: 4096`); LIM-MEM-0002 (`elem_size: size_of::<T>()`, `len: blocks`, `align: align_of::<T>()`) if `blocks > u32::MAX`, or if `pool_size` of the pool layout (§4) overflows or, rounded up to a multiple of `pool_align`, exceeds `isize::MAX`; and LIM-MEM-0004 with `size: pool_size, align: pool_align` (no abort) if the system allocator returns null. With `blocks > 0` it makes exactly one reservation, of `pool_size` bytes aligned to `pool_align`, holding the blocks and the free list; `blocks == 0` makes no call to the global allocator. A crate that calls `Pool::<T>::with_capacity` with `size_of::<T>() == 0` does not build (§5 "Compile-time rejections"). |
| MEM-R041 | MUST | `Pool::alloc` follows the slot order (§4): it returns a `PoolBox` whose `slot()` is the most recently freed slot, or, if none is free, the lowest never-used slot; the box dereferences to the moved-in value; `live()` increases by 1. |
| MEM-R042 | MUST | `Pool::alloc` with every slot live returns LIM-MEM-0005 with `capacity: capacity()`, drops the passed value exactly once before returning, and leaves `live()` and every live value unchanged. For LIM-MEM-0005, `code()`, `message()` (containing `capacity`) and `help()` (`Some` of the text) match §8, `context()` holds the one key `capacity`, and the value passes `liminal_core::error::validate_error(&e, "MEM")` (CORE-R012). |
| MEM-R043 | MUST | Dropping a `PoolBox` drops its value exactly once and frees its slot (`live()` decreases by 1). A `PoolBox` passed to `core::mem::forget` keeps its slot live until the pool is dropped; its value is never dropped, and dropping the pool is still free of undefined behaviour (Miri). |
| MEM-R044 | MUST | Values in live slots never overlap; each `PoolBox` address is a multiple of `align_of::<T>()`; writing through one box never changes the value of another. Checked by a differential property test against the reference pool model (§10) comparing slot numbers, `live()` and all live values after every operation, over at least 256 operations per case. |
| MEM-R045 | MUST | After construction, `Pool::alloc` (success and failure), `PoolBox` drop, `Pool::live` and `Pool::capacity` make no call to the global allocator (same check as MEM-R019). The free list lives in the reservation made at construction (pool layout, §4) and never grows. |
| MEM-R046 | MUST | `Pool<T>` is `Send` when `T: Send` and is never `Sync`. `PoolBox` is neither `Send` nor `Sync`. Dropping a `Pool` frees its base, the one reservation that holds the blocks and the free list, exactly once (Miri). |
| MEM-R047 | MUST | No public function or trait method of `Pool` or `PoolBox` panics for any argument values and any call sequence the safe API allows, except by propagating a panic from `T`'s own `Drop`. |

### Aligned buffers (R060–R069)

| ID | Level | Requirement |
|---|---|---|
| MEM-R060 | MUST | `AlignedBuf::new(len, value, align)` validates in the order of §5, with `e = max(align, align_of::<T>())`: `align` not a power of two (including 0) → LIM-MEM-0002; `e > 4096` (also when `align ≤ 4096 < align_of::<T>()`) → LIM-MEM-0003 with `align: e, max: 4096`; `len × size_of::<T>()` overflows, or rounded up to a multiple of `e` exceeds `isize::MAX` → LIM-MEM-0002; system allocator returns null → LIM-MEM-0004 with `size: len × size_of::<T>(), align: e` (no abort). LIM-MEM-0002 carries `elem_size: size_of::<T>(), len, align` (the argument). It never panics. |
| MEM-R061 | MUST | A successful `AlignedBuf::new(len, value, align)` dereferences to a slice of `len` elements, each equal to `value`; `align()` returns `max(align, align_of::<T>())`; the address of the first element is a multiple of `align()`. Writes through `DerefMut` are read back unchanged. |
| MEM-R062 | MUST | Dropping an `AlignedBuf` returns its memory to the system allocator exactly once (Miri). |
| MEM-R063 | MUST | When `len × size_of::<T>() == 0`, `AlignedBuf::new` makes no call to the global allocator, the buffer dereferences to a slice of `len` elements (`len > 0` only for a zero-sized `T`), and the slice pointer is non-null and a multiple of `align()` (the address rule of zero-size requests, §4). The checks of MEM-R060 still run first. |
| MEM-R064 | MUST | `AlignedBuf<T>` is `Send` exactly when `T: Send` and `Sync` exactly when `T: Sync`. |

### Allocation tracking (R070–R079)

| ID | Level | Requirement |
|---|---|---|
| MEM-R070 | MUST | `Arena::stats()`: `allocations` counts successful `alloc*` calls, zero-size requests (§4) included; `failed_allocations` counts `alloc*` calls that returned an error; `live_bytes == used()`; `peak_bytes` is the maximum `used()` since construction and is not lowered by `reset`; `capacity_bytes == capacity()`; `resets` counts `reset` calls. All fields are 0 at construction except `capacity_bytes`. |
| MEM-R071 | MUST | `FrameAllocator::stats()` equals `arena().stats()`, where `begin_frame` counts as one reset; so `peak_bytes` is the largest per-frame use since construction. |
| MEM-R072 | MUST | `Pool::stats()`: `allocations` and `failed_allocations` count `Pool::alloc` results; `live_bytes == live() × size_of::<T>()`; `peak_bytes` is the maximum `live_bytes` since construction; `capacity_bytes == capacity() × size_of::<T>()`, which is the block area only and excludes the free list of the pool layout (§4); `resets == 0`. |
| MEM-R073 | MUST | `stats()` of every allocator makes no call to the global allocator, and maintaining the counters adds no call to any allocation path (same check as MEM-R019). |
| MEM-R074 | SHOULD | The `Debug` output of `Arena`, `FrameAllocator` and `Pool` contains their capacity, use and stats and no memory address, so it is identical across runs for the same call sequence. |

### Determinism (R080–R089)

| ID | Level | Requirement |
|---|---|---|
| MEM-R080 | MUST | For the same sequence of public API calls with the same arguments, every observable result other than memory addresses (`Ok`/`Err`, error codes and context fields, `used()`, `slot()`, `live()`, `frame_index()`, `last_frame_bytes()`, `stats()`, values read back) is identical between runs in one process and between processes. Verified by workloads W1–W3 (§11), each run twice with state hashes compared after every step. |
| MEM-R081 | MUST | The results of workloads W1–W3 are identical on macOS arm64, Windows x86_64 and Linux x86_64: `tests/determinism.rs` compares each workload's final hash with a hash literal recorded in the test, and CI runs it on all three (ADR-0024 item 3). |

## 7. Invariants

- INV-1: An `Arena` base address is a multiple of `MAX_ALIGN`, and `used() ≤ capacity()` at all
  times (`debug_assert!` after every allocation).
- INV-2: Live arena allocations lie within `[base, base + used())` and are pairwise disjoint
  (property test MEM-R013).
- INV-3: Every pool slot is either live (owned by exactly one `PoolBox`) or free (in the free list
  exactly once, or never used). `live() ≤ capacity()` (`debug_assert!` in `alloc` and `PoolBox`
  drop; property test MEM-R044).
- INV-4: For every allocator, `live_bytes ≤ peak_bytes ≤ capacity_bytes`; `allocations`,
  `failed_allocations`, `resets` and `peak_bytes` never decrease (property tests of MEM-R070,
  MEM-R072).
- INV-5: The public API has no `unsafe fn` or `unsafe trait`; every `unsafe` block has a
  `// SAFETY:` comment (clippy `undocumented_unsafe_blocks`, review per ADR-0018).
- INV-6: No observable result depends on a memory address except the address itself (MEM-R080).

## 8. Error codes

Every code is returned through `MemoryError` and implements the `liminal_core` error trait
(MEM-R002, MEM-R042). `{field}` is the context field of the same name.

| Code | Meaning | Help |
|---|---|---|
| LIM-MEM-0001 | `CapacityExhausted`: "allocation of {requested} bytes with alignment {align} does not fit: {available} bytes left in the arena" | Create the arena with a larger capacity, or reset it more often. `stats().peak_bytes` is the most the arena has held. |
| LIM-MEM-0002 | `InvalidLayout`: "invalid layout: {len} elements of {elem_size} bytes with alignment {align}; the alignment is not a power of two or the size exceeds isize::MAX" | Use a power-of-two alignment and a total size of at most isize::MAX bytes. |
| LIM-MEM-0003 | `UnsupportedAlignment`: "alignment {align} exceeds the maximum supported alignment {max}" | Use an alignment of at most `liminal_memory::MAX_ALIGN` (4096) bytes. |
| LIM-MEM-0004 | `SystemAllocationFailed`: "the system allocator could not provide {size} bytes with alignment {align}" | Request less memory. The process is out of memory or the request exceeds what the OS provides. |
| LIM-MEM-0005 | `PoolExhausted`: "all {capacity} blocks of the pool are in use" | Create the pool with more blocks, or drop `PoolBox` values sooner. `stats().peak_bytes` divided by the block size is the most blocks held at once. |

Panics are reserved for violated internal invariants (`expect("invariant: …")`, ADR-0017).

## 9. Performance budgets

None yet: set at `stable` from baselines (`docs/process/PERFORMANCE.md`). M1-T026 writes these
benchmarks in `liminal_benches` and records the first wall-clock baseline in
`ci/perf/<reference machine id>/liminal_memory.json` with `cargo xtask bench` (ADR-0023).

| Scenario | Budget | Machine | Source of number |
|---|---|---|---|
| B1: 1,000,000 × `Arena::alloc::<u64>` into an 8 MiB arena, then `reset`; compared with 1,000,000 × `Box::new(0u64)` and drop | None yet | Reference, single thread | Baseline from M1-T026 |
| B2: 1,000 frames × (`begin_frame` + 1,000 × `alloc_slice_fill::<u8>(64, 0)`) | None yet | Reference, single thread | Baseline from M1-T026 |
| B3: 1,000,000 × (`Pool::<[u8; 64]>::alloc` + drop), capacity 1,024; compared with `Box::new([0u8; 64])` and drop | None yet | Reference, single thread | Baseline from M1-T026 |
| B4: `AlignedBuf::<f32>::new(1_048_576, 0.0, 64)`; compared with `vec![0.0f32; 1_048_576]` | None yet | Reference, single thread | Baseline from M1-T026 |

Whether B1–B3 also become instruction-count cases of the CI gate is decided by the instruction-count
task (M1-T033).

## 10. Test plan

| Area | Test kinds | Oracle |
|---|---|---|
| Arena placement, failure, contents (R011–R013, R015, R016) | unit, property, differential | Reference arena model `tests/common/reference.rs`: plain integer arithmetic for `used`/padding and one `Vec<T>` per allocation for contents. Same generated operation sequence on both; compare `Ok`/`Err` (code and fields), `used()`, and every live allocation's values after each operation. |
| Frame allocator (R030, R031) | unit, property | The reference arena model plus two integers (`frame_index`, `last_frame_bytes`). Generated sequences of allocations and `begin_frame` calls; compare `frame_index()`, `last_frame_bytes()`, `arena().used()`, `arena().capacity()` after each operation. A unit test calls `frame_index()`, `last_frame_bytes()` and `arena()` while a reference from `arena()` is in use (MEM-R031). |
| Pool slot order and values (R041–R044) | unit, property, differential | Reference pool model: `Vec<Option<T>>` slots plus a `Vec<u32>` free stack. Compare `slot()`, `live()`, every live value after each operation. |
| `AlignedBuf` (R060–R064) | unit, property, differential | `vec![value; len]` (contents); address modulo `align()` (alignment). |
| Tracking (R070–R073) | property | Counters recomputed by the reference models above. |
| `Debug` output (R074) | unit | Two allocators of each kind (`Arena`, `FrameAllocator`, `Pool`) driven by the same call sequence give equal `format!("{:?}")` strings; the string contains the decimal capacity and use and does not contain `0x`. |
| Constructor validation, error codes (R002, R003, R010, R030, R040, R042, R060) | unit | Hand-written expected codes and context fields (values and key order); `liminal_core::error::validate_error(&e, "MEM")` on one value of every variant. Zero-size requests (R011, R063): address non-null and a multiple of the effective alignment, for capacity 0 and for a full arena. System-allocation failure: a request of `isize::MAX / 2` bytes, which no supported OS can satisfy (assumption; M1-T025 confirms on all three CI platforms). |
| No panic (R001, R047, R060) | property | Generated call sequences with arbitrary sizes (including `usize::MAX`), alignments (0, non-powers of two, up to `2^63`) and lengths; the test passes if no panic occurs and every error code is one of §8. |
| No global allocation (R019, R033, R045, R063, R073) | integration, own test binary `tests/no_global_alloc.rs` | A counting `#[global_allocator]` wrapping `std::alloc::System`, counting per thread (`thread_local!` with `const` initialiser), asserted to count 0 across the measured calls. |
| Compile-time rejections and borrow rules (R014, R016, R032, R040 zero-sized `T`; R017, R046, R064 negative halves: `Arena`, `FrameAllocator`, `Pool` not `Sync`; `Pool<T>` not `Send` for a `T` that is not `Send`; `PoolBox` not `Send` and not `Sync`; `AlignedBuf<T>` not `Send`/`Sync` for a `T` that is not) | doc tests (`compile_fail`) | Each `compile_fail` test sits in the docs of a `#[cfg(doctest)]` function in `src/compile_tests.rs` that carries the `/// Verifies:` line, and has a compiling counterpart that differs only in the checked property, so it fails for that reason and no other. R014 and R040 fail at monomorphization (§5 "Compile-time rejections"); doc tests build, so they see the error. |
| `Send`/`Sync` (R017, R046, R064 positive halves: `Arena`, `FrameAllocator` `Send`; `Pool<T: Send>` `Send`; `AlignedBuf<T>` `Send`/`Sync` with `T`) | unit | `fn assert_send<T: Send>()` / `assert_sync` instantiations. `PoolBox` has no positive half. |
| Drop exactly once (R018, R042, R043, R046, R062) | unit + Miri | Drop-counting test type; Miri reports leaks and double frees. |
| Determinism (R080, R081) | determinism (`tests/determinism.rs`, ADR-0024) | `liminal_core` state hasher; two runs per workload; recorded literals (§11). |
| Public API (primary entry points) | doc tests (`# Examples`) | `Arena`, `FrameAllocator`, `Pool`, `AlignedBuf` each have a runnable example. |
| Undefined behaviour | Miri (`cargo +nightly miri test -p liminal_memory`, CI job from M1-T018) | Miri. Property-test case counts are reduced under `cfg(miri)` (at least 16 cases). Exceptions: the system-allocation-failure tests, if Miri cannot model a failed `isize::MAX / 2` request (unverified); M1-T025 lists them with `#[cfg_attr(miri, ignore = "M1-T025: …")]` and records the reason here. |
| Concurrency | none | No atomics, no lock-free code, no `Sync` allocator, so no loom tests (ADR-0018 loom rule does not apply). |

Optimized paths: the crate has no SIMD, parallel or lock-free code. The allocators themselves are
the optimized path relative to the system allocator; their oracles are the reference models above
(AGENTS.md rule 4).

## 11. Determinism

`deterministic = true`. The crate reads no clock, no randomness and no randomly seeded hash map,
and no observable result depends on an address (INV-6). Placement is a pure function of
`used()` and the request because the arena base is aligned to `MAX_ALIGN` and every accepted
alignment is at most `MAX_ALIGN`. Pool slot order is defined in §4.

`tests/determinism.rs` (ADR-0024 item 2): each workload draws its operations from one
`liminal_core::rng::Rng`, created by `Rng::new(world_seed, StreamKey::from_name(name))` with the
seed and the name recorded in the test (draws by `Rng::below_u32` and `Rng::next_u64`), runs twice
in the same process, and feeds a `liminal_core::hash::StateHasher` with the observable results
after every step. Encoding: every `usize` by `StateHasher::write_usize` (which writes it as
`u64`, CORE-R053); `u64` counters by `write_u64`; `slot()` by `write_u32`; a result code by
`write_u16` of `code().number()`, 0 for `Ok`; context fields by `write_usize` in the key order of
§5; `stats()` as its six fields in declaration order. `StateHasher::finish` of the two runs is
compared after every step.

| Workload | Description | Cross-platform bit-identity claimed |
|---|---|---|
| W1 `arena_ops` | `Arena` of 65,536 bytes; 10,000 operations: 98 % `alloc_slice_fill_aligned` of `u8`, `u32`, `u64` or `f32` with `len` in 0..=512 and `align` in {1, 2, 4, 8, 16, 64, 256, 4096}, 1 % `reset`, 1 % a request larger than the capacity (`alloc_slice_fill::<u8>(65_537, 0)`, which returns LIM-MEM-0001). Hashed per step: result code and context fields, `used()`, `stats()`. | Yes: recorded hash literal (MEM-R081) |
| W2 `frame_ops` | `FrameAllocator` of 16,384 bytes; 600 frames, each `begin_frame` plus 0..=64 allocations drawn as in W1. Hashed per frame: `frame_index()`, `last_frame_bytes()`, `stats()`, result codes. | Yes: recorded hash literal (MEM-R081) |
| W3 `pool_ops` | `Pool<u64>` with 256 blocks; 10,000 operations: 55 % `alloc` of a random `u64`, 45 % drop of a live box chosen by index into the list of live boxes. A drawn drop when no box is live makes no pool call; the step is hashed like every other. An `alloc` on a full pool hashes the error code (LIM-MEM-0005) and its `capacity` field in place of `slot()`. Hashed per step: the operation kind (`write_u8`: 0 alloc, 1 drop, 2 drop with no live box), `slot()` (of the new or the dropped box) or the error, `live()`, `stats()`, wrapping sum of live values. | Yes: recorded hash literal (MEM-R081) |

The claim rests on: only primitive element types whose size and alignment are equal on the three
64-bit targets (`docs/adr/0005-target-platforms-and-toolchain.md`); no floating-point arithmetic
(`f32` values are only copied); and the `liminal_core` RNG and hasher being platform-independent
(their spec). Cross-platform identity is not yet verified; it is verified when M1-T026's
determinism test passes in CI on all three platforms. A change of the `liminal_core` RNG or hasher
changes the literals; that is an oracle change (AGENTS.md rule 5).

## 12. Open questions

| ID | Question | Resolved by |
|---|---|---|
| Q1 | Names of the `liminal_core` error trait and code type, the RNG stream API and the state hasher API. | Resolved in `spec_version` 2 (M1-T058) against `docs/specs/liminal_core.md` `spec_version` 1: `liminal_core::error::EngineError`, `ErrorCode`, `write_display`, `validate_error` (§5); `liminal_core::rng::{Rng, StreamKey}` and `liminal_core::hash::StateHasher` (§11). If a later core `spec_version` renames one of them, the task that makes the change updates this spec. |
| Q2 | Growable arenas (chain a new block when full) and arena-backed collections. Not needed by any M1 crate; `allocator_api` is unstable. | First consumer that needs it (expected M2 kickoff, scene/assets). |
| Q3 | A counting `#[global_allocator]` for "no allocation in hot paths after warm-up" tests in other crates (CODE_STYLE.md): which crate owns it (`liminal_memory`, `liminal_telemetry` or `liminal_testkit`), and whether it is per thread. This spec uses one only inside its own tests. | `liminal_telemetry` spec (M1-T027) or `liminal_testkit` spec (M1-T047), whichever comes first. |
| Q4 | Frame data that must live into the next frame (render submission). Not provided by `FrameAllocator`. | M3 kickoff (rendering). |
| Q5 | Per-thread frame allocators for job workers. | `liminal_jobs` spec (M1-T034). |
| Q6 | Whether `MAX_ALIGN = 4096` suffices (some GPU APIs want 64 KiB buffer alignment for placed resources). | M3 kickoff (`liminal_gpu` spec). |

## 13. Maturity

Current level: `planned`.

Assignment of MUST requirements to implementation tasks (each MUST in exactly one task):

| Task | Requirements |
|---|---|
| M1-T025 (arena, frame allocator) | MEM-R001, MEM-R002, MEM-R003, MEM-R010–MEM-R019, MEM-R030–MEM-R033 |
| M1-T026 (pool, aligned buffers, tracking, determinism) | MEM-R040–MEM-R047, MEM-R060–MEM-R064, MEM-R070–MEM-R073, MEM-R080, MEM-R081 |

MEM-R074 (SHOULD) is implemented by M1-T026 with the tracking work.

M1-T025 implements `MemoryError` and MEM-R002, which need the `liminal_core` error model
(`EngineError`, `ErrorCode`, `validate_error`; implemented by M1-T013). M1-T025 does not list
M1-T013 in `depends_on`; it depends on it through M1-T014, whose `depends_on` is M1-T013. If that
link is ever removed, M1-T013 MUST be added to `depends_on` of M1-T025.

Promotion to `experimental`:
- [ ] Spec review (REVIEW.md §2) passed (evidence: review of M1-T024)
- [ ] Crate compiles, lints pass, implemented behaviour tested (evidence: M1-T025)
- [ ] Registered in `ci/coverage.toml` (evidence: M1-T025)

Promotion to `usable` (filled in by M1-T026):
- [ ] Every MUST traced to tests (`cargo xtask trace` with `maturity: usable` on a scratch copy)
- [ ] Line coverage ≥ 85 % (pure logic, layer 0)
- [ ] Every public item documented; `# Examples` for `Arena`, `FrameAllocator`, `Pool`, `AlignedBuf`
- [ ] Integration tests use the public API as a dependent crate would
- [ ] Miri passes; exceptions listed in §10 with reasons; `SAFETY:` comments reviewed
- [ ] `tests/determinism.rs` with W1–W3 and recorded literals, green on macOS, Windows, Linux
- [ ] At least one example or dependent crate uses it

## 14. History

| spec_version | Date | Change | By task |
|---|---|---|---|
| 1 | 2026-11-09 | Initial spec | M1-T024 |
| 2 | 2026-11-10 | Follow-ups of the M1-T024 review; no requirement ID added, moved or withdrawn. Behaviour changes: `FrameAllocator::begin_frame` returns `()` instead of `&Arena` (MEM-R031, MEM-R032); address rule and check steps for zero-size requests (§4, MEM-R011, MEM-R063); `AlignedBuf::new` returns LIM-MEM-0003 on the effective alignment and LIM-MEM-0002 on the rounded size (MEM-R060); the pool is one reservation holding blocks and a `u32` free list, with the size check and LIM-MEM-0004 fields derived from it (§4 pool layout, MEM-R040, MEM-R045, MEM-R046, MEM-R072); context fields of LIM-MEM-0003 and LIM-MEM-0004 for every producer (§5). Clarifications: §4 diagram redrawn to agree with the placement rule; "does not compile" means "does not build" (MEM-R014, MEM-R040); Q1 closed with the `liminal_core` v1 names, `validate_error` and context key order added (MEM-R002, MEM-R042); `PoolBox` not `Sync`; overflow wording of MEM-R012; `AlignedBuf` not tracked (§2); `alloc_zeroed` removed (§3, §5); test-plan rows for MEM-R031, MEM-R040 and MEM-R074 (§10); W1 and W3 details and hash encoding (§11); note on M1-T013 (§13). | M1-T058 |
