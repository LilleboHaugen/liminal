---
crate: liminal_memory
req_prefix: MEM
layer: 0
maturity: planned
spec_version: 1
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
- Allocation tracking per allocator (`AllocStats`).
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

- Engine crates: `liminal_core` (same layer, `architecture.toml` `same_layer_deps`), for the shared
  error trait and error code type (ADR-0017). Dev only: the `liminal_core` RNG and state hasher, used
  by `tests/determinism.rs` (ADR-0024).
- External crates: none at runtime. Dev: `proptest` (`architecture.toml` `dev_external`,
  approved for property tests).
- `std` items used: `std::alloc::{alloc, alloc_zeroed, dealloc, Layout}`, `core::ptr`,
  `core::cell::Cell`, `core::mem::{needs_drop, size_of, align_of}`.
- `architecture.toml`: layer 0, `unsafe = "allow"`, `deterministic = true`.

## 4. Concepts

- **Base:** the one block an allocator reserves from the system allocator at construction. An
  `Arena` base is aligned to `MAX_ALIGN`. A capacity of 0 reserves nothing.
- **Request:** one allocation call, described by a size `s` in bytes and an alignment `a` in bytes
  (a power of two, `1 ≤ a ≤ MAX_ALIGN`). For a value of type `T`: `s = size_of::<T>()`,
  `a = align_of::<T>()`. For a slice of `len` elements: `s = len × size_of::<T>()`.
- **Used (`used()`):** for an `Arena`, the offset in bytes from the base to the end of the last
  successful non-empty request, padding included. Starts at 0; `reset` sets it to 0.
- **Placement rule (bump):** a request `(s, a)` with `s > 0` starts at
  `start = round_up(used, a)`. It succeeds if and only if `start + s ≤ capacity`; then
  `used = start + s`. Since the base is aligned to `MAX_ALIGN ≥ a`, the address is a multiple of `a`.
  A request with `s = 0` always succeeds and leaves `used` unchanged.
- **Frame:** the interval between two calls of `FrameAllocator::begin_frame`. Frame 0 is the
  interval before the first call. `frame_index()` is the number of `begin_frame` calls so far.
- **Slot:** one block of a `Pool<T>`, numbered `0 … capacity − 1`. A slot is either *live* (owned
  by exactly one `PoolBox`) or *free*.
- **Slot order:** `Pool::alloc` takes the most recently freed slot (LIFO). If no freed slot exists,
  it takes the lowest slot that has never been used. This order depends only on the sequence of
  `alloc` and `PoolBox` drop calls.
- **Live bytes / peak bytes:** see `AllocStats` (§5). Peak bytes is the high-water mark since
  construction.

```
Arena (capacity 64, MAX_ALIGN-aligned base)
offset: 0        8  12 16                32                 64
        [u64 ....][u32][pad][ [u8;16] ....][      free        ]
                                          ^ used() = 32
```

## 5. Public API

All items are at the crate root. Names marked *placeholder* come from `liminal_core`, whose spec
(M1-T012) did not exist when this spec was written (Q1).

```rust
/// Largest alignment any allocator in this crate accepts, in bytes. Value: 4096.
/// Chosen as the x86-64 page size; it covers SIMD (16–64), cache lines (64/128) and
/// GPU staging offsets (256).
pub const MAX_ALIGN: usize = 4096;

/// Usage counters of one allocator. All counters start at 0 at construction.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AllocStats {
    /// Successful allocation calls (zero-size requests included).
    pub allocations: u64,
    /// Allocation calls that returned an error (any code).
    pub failed_allocations: u64,
    /// Bytes currently in use (Arena: `used()`, padding included; Pool: live slots × size_of::<T>()).
    pub live_bytes: usize,
    /// Largest `live_bytes` value since construction (not lowered by reset).
    pub peak_bytes: usize,
    /// Capacity in bytes (Arena: capacity; Pool: capacity × size_of::<T>()).
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
    /// Moves `value` into the arena. Compile-time error if `core::mem::needs_drop::<T>()`.
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
    /// resets the arena, increments frame_index by 1, and returns the arena for the new frame.
    pub fn begin_frame(&mut self) -> &Arena;
    /// The arena of the current frame.
    pub fn arena(&self) -> &Arena;
    pub fn frame_index(&self) -> u64;
    /// used() of the arena at the end of the previous frame, in bytes. 0 before the first begin_frame.
    pub fn last_frame_bytes(&self) -> usize;
    pub fn stats(&self) -> AllocStats;
}
impl core::fmt::Debug for FrameAllocator;

/// `capacity` blocks of `T`. Compile-time error if size_of::<T>() == 0.
/// `Send` if `T: Send`; never `Sync`.
pub struct Pool<T> { /* private */ }

impl<T> Pool<T> {
    /// Errors: LIM-MEM-0003 if align_of::<T>() > MAX_ALIGN; LIM-MEM-0002 if blocks > u32::MAX
    /// or blocks × size_of::<T>() exceeds isize::MAX; LIM-MEM-0004 if the system allocator
    /// returns null. `blocks == 0` reserves nothing.
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

/// Owner of one live slot. Dropping it drops the value and frees the slot. Not `Send`.
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
    /// Errors: LIM-MEM-0002 if `align` is not a power of two or len × size_of::<T>() exceeds
    /// isize::MAX; LIM-MEM-0003 if align > MAX_ALIGN; LIM-MEM-0004 if the system allocator
    /// returns null.
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
impl liminal_core::error::Error for MemoryError;   // placeholder name (Q1): code, message, help, context
impl core::fmt::Display for MemoryError;           // format defined by the liminal_core spec
impl std::error::Error for MemoryError;
```

Notes on the API:
- **Context fields of `InvalidLayout`.** `with_capacity(n)` of `Arena`/`FrameAllocator` reports
  `elem_size: 1, len: n, align: MAX_ALIGN`. Slice calls and `AlignedBuf::new` report
  `size_of::<T>()`, `len` and the requested alignment. `Pool::with_capacity` reports
  `size_of::<T>()`, `blocks`, `align_of::<T>()`.
- **Check order** (so the error for a request with several faults is defined): (1) alignment is a
  power of two (else 0002), (2) alignment ≤ `MAX_ALIGN` (else 0003), (3) total size fits (else
  0002), (4) capacity (0001 or 0005), (5) system allocation (0004, constructors only).
- **`CapacityExhausted` fields:** `requested = s`, `align = a` (effective alignment),
  `available = capacity − used()` before the call (padding not subtracted).
- **Compile-time rejections** (`Arena::alloc` of a type that needs drop, `Pool` of a zero-sized
  type) use an inline `const { assert!(…) }` in the generic function, so they fail at
  monomorphization (assumption: rustdoc `compile_fail` tests build far enough to see these errors;
  M1-T025 verifies it for the arena case).

### Unsafe inside the crate (ADR-0018)

The public API has no `unsafe fn`, no `unsafe trait` and no function whose correct use needs
`unsafe` by the caller. `unsafe` is used only for:

| Operation | Where | Why it is sound |
|---|---|---|
| `std::alloc::alloc` / `alloc_zeroed` / `dealloc` of the base | constructors, `Drop` of `Arena`, `Pool`, `AlignedBuf` | The same `Layout` is stored and used for `dealloc`; size 0 never calls the system allocator. |
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
| MEM-R002 | MUST | `MemoryError` implements the `liminal_core` error trait (ADR-0017). For the variants with codes LIM-MEM-0001 to LIM-MEM-0004, `code()` returns the code in §8, the message contains every context field value of the variant, and `help` returns the text in §8. |
| MEM-R003 | MUST | Arena alignment rules: `MAX_ALIGN` is 4096. `Arena::alloc_slice_fill_aligned` with an `align` that is not a power of two (including 0) returns LIM-MEM-0002; with a power of two above `MAX_ALIGN` it returns LIM-MEM-0003. `Arena::alloc*` with `align_of::<T>() > MAX_ALIGN` returns LIM-MEM-0003. Checks follow the order in §5. |

### Arena (R010–R029)

| ID | Level | Requirement |
|---|---|---|
| MEM-R010 | MUST | `Arena::with_capacity(n)` returns an arena with `capacity() == n` and `used() == 0`. If `n` rounded up to a multiple of 4096 exceeds `isize::MAX`, it returns LIM-MEM-0002 with `elem_size: 1, len: n, align: 4096`. If the system allocator returns null, it returns LIM-MEM-0004 with `size: n, align: 4096` and does not abort the process. `n == 0` makes no call to the global allocator. |
| MEM-R011 | MUST | Every successful `Arena` allocation follows the placement rule (§4): for a request `(s, a)` with `s > 0`, `used()` afterwards equals `round_up(used_before, a) + s`, and the returned address is a multiple of `a`. A request with `s == 0` succeeds and leaves `used()` unchanged. |
| MEM-R012 | MUST | A request that does not fit (`round_up(used, a) + s > capacity`, including arithmetic overflow) returns LIM-MEM-0001 with `requested: s`, `align: a`, `available: capacity − used()`, and leaves `used()` and the contents of all earlier allocations unchanged. |
| MEM-R013 | MUST | Live allocations from one arena never overlap, and writing through one allocation never changes the value read through another. Checked by a differential property test against the reference arena model (§10) over generated sequences of at least 256 operations per case. |
| MEM-R014 | MUST | `Arena::alloc::<T>` does not compile when `core::mem::needs_drop::<T>()` is true. The arena never runs a destructor. |
| MEM-R015 | MUST | `alloc_slice_copy(src)` returns a slice equal to `src`; `alloc_slice_fill(len, v)` and `alloc_slice_fill_aligned(len, v, align)` return a slice of `len` elements, each equal to `v`. `alloc_slice_fill_aligned` aligns to `max(align, align_of::<T>())`. If `len × size_of::<T>()` overflows or exceeds `isize::MAX`, the call returns LIM-MEM-0002 and `used()` is unchanged. |
| MEM-R016 | MUST | `Arena::reset` sets `used()` to 0 and keeps `capacity()`. After `reset`, the full capacity is available again (an allocation of `capacity()` bytes with alignment 1 succeeds). A reference obtained before `reset` cannot be used after it (`compile_fail` test). |
| MEM-R017 | MUST | `Arena` and `FrameAllocator` are `Send` and not `Sync`. |
| MEM-R018 | MUST | Dropping an `Arena` or `FrameAllocator` returns its base to the system allocator exactly once (Miri: no leak, no double free) and runs no destructor of values stored in it. |
| MEM-R019 | MUST | After construction, `Arena::alloc*`, `Arena::reset`, `Arena::used`, `Arena::capacity` make no call to the global allocator, on success and on failure (checked with a per-thread counting `#[global_allocator]` in a dedicated test binary). |

### Frame allocator (R030–R039)

| ID | Level | Requirement |
|---|---|---|
| MEM-R030 | MUST | `FrameAllocator::with_capacity(n)` follows the rules and errors of MEM-R010; the new allocator has `frame_index() == 0`, `last_frame_bytes() == 0` and `arena().capacity() == n`. |
| MEM-R031 | MUST | `begin_frame` sets `last_frame_bytes()` to `arena().used()` as it was before the call, resets the arena (`used() == 0` afterwards), increments `frame_index()` by exactly 1, and returns the arena of the new frame. |
| MEM-R032 | MUST | A reference allocated during a frame cannot be used after the next `begin_frame` call (`compile_fail` test with a compiling counterpart). |
| MEM-R033 | MUST | `begin_frame`, `arena`, `frame_index` and `last_frame_bytes` make no call to the global allocator (same check as MEM-R019). |

### Pool (R040–R059)

| ID | Level | Requirement |
|---|---|---|
| MEM-R040 | MUST | `Pool::<T>::with_capacity(blocks)` returns a pool with `capacity() == blocks` and `live() == 0`. It returns LIM-MEM-0003 if `align_of::<T>() > 4096`, LIM-MEM-0002 if `blocks > u32::MAX` or `blocks × size_of::<T>() > isize::MAX`, and LIM-MEM-0004 (no abort) if the system allocator returns null. `blocks == 0` makes no call to the global allocator. `Pool<T>` with `size_of::<T>() == 0` does not compile. |
| MEM-R041 | MUST | `Pool::alloc` follows the slot order (§4): it returns a `PoolBox` whose `slot()` is the most recently freed slot, or, if none is free, the lowest never-used slot; the box dereferences to the moved-in value; `live()` increases by 1. |
| MEM-R042 | MUST | `Pool::alloc` with every slot live returns LIM-MEM-0005 with `capacity: capacity()`, drops the passed value exactly once before returning, and leaves `live()` and every live value unchanged. For LIM-MEM-0005, `code()`, message (containing `capacity`) and `help` match §8. |
| MEM-R043 | MUST | Dropping a `PoolBox` drops its value exactly once and frees its slot (`live()` decreases by 1). A `PoolBox` passed to `core::mem::forget` keeps its slot live until the pool is dropped; its value is never dropped, and dropping the pool is still free of undefined behaviour (Miri). |
| MEM-R044 | MUST | Values in live slots never overlap; each `PoolBox` address is a multiple of `align_of::<T>()`; writing through one box never changes the value of another. Checked by a differential property test against the reference pool model (§10) comparing slot numbers, `live()` and all live values after every operation, over at least 256 operations per case. |
| MEM-R045 | MUST | After construction, `Pool::alloc` (success and failure), `PoolBox` drop, `Pool::live` and `Pool::capacity` make no call to the global allocator (same check as MEM-R019). |
| MEM-R046 | MUST | `Pool<T>` is `Send` when `T: Send` and is never `Sync`. `PoolBox` is not `Send`. Dropping a `Pool` frees its base exactly once (Miri). |
| MEM-R047 | MUST | No public function or trait method of `Pool` or `PoolBox` panics for any argument values and any call sequence the safe API allows, except by propagating a panic from `T`'s own `Drop`. |

### Aligned buffers (R060–R069)

| ID | Level | Requirement |
|---|---|---|
| MEM-R060 | MUST | `AlignedBuf::new(len, value, align)` validates in the order of §5: `align` not a power of two (including 0) → LIM-MEM-0002; `align > 4096` → LIM-MEM-0003; `len × size_of::<T>()` overflows or exceeds `isize::MAX` → LIM-MEM-0002; system allocator returns null → LIM-MEM-0004 (no abort). It never panics. |
| MEM-R061 | MUST | A successful `AlignedBuf::new(len, value, align)` dereferences to a slice of `len` elements, each equal to `value`; `align()` returns `max(align, align_of::<T>())`; the address of the first element is a multiple of `align()`. Writes through `DerefMut` are read back unchanged. |
| MEM-R062 | MUST | Dropping an `AlignedBuf` returns its memory to the system allocator exactly once (Miri). |
| MEM-R063 | MUST | When `len × size_of::<T>() == 0`, `AlignedBuf::new` makes no call to the global allocator and the (empty) slice pointer is still a multiple of `align()`. |
| MEM-R064 | MUST | `AlignedBuf<T>` is `Send` exactly when `T: Send` and `Sync` exactly when `T: Sync`. |

### Allocation tracking (R070–R079)

| ID | Level | Requirement |
|---|---|---|
| MEM-R070 | MUST | `Arena::stats()`: `allocations` counts successful `alloc*` calls (zero-size included); `failed_allocations` counts `alloc*` calls that returned an error; `live_bytes == used()`; `peak_bytes` is the maximum `used()` since construction and is not lowered by `reset`; `capacity_bytes == capacity()`; `resets` counts `reset` calls. All fields are 0 at construction except `capacity_bytes`. |
| MEM-R071 | MUST | `FrameAllocator::stats()` equals `arena().stats()`, where `begin_frame` counts as one reset; so `peak_bytes` is the largest per-frame use since construction. |
| MEM-R072 | MUST | `Pool::stats()`: `allocations` and `failed_allocations` count `Pool::alloc` results; `live_bytes == live() × size_of::<T>()`; `peak_bytes` is the maximum `live_bytes` since construction; `capacity_bytes == capacity() × size_of::<T>()`; `resets == 0`. |
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
| Pool slot order and values (R041–R044) | unit, property, differential | Reference pool model: `Vec<Option<T>>` slots plus a `Vec<u32>` free stack. Compare `slot()`, `live()`, every live value after each operation. |
| `AlignedBuf` (R060–R064) | unit, property, differential | `vec![value; len]` (contents); address modulo `align()` (alignment). |
| Tracking (R070–R073) | property | Counters recomputed by the reference models above. |
| Constructor validation, error codes (R002, R003, R010, R030, R040, R042, R060) | unit | Hand-written expected codes and context fields. System-allocation failure: a request of `isize::MAX / 2` bytes, which no supported OS can satisfy (assumption; M1-T025 confirms on all three CI platforms). |
| No panic (R001, R047, R060) | property | Generated call sequences with arbitrary sizes (including `usize::MAX`), alignments (0, non-powers of two, up to `2^63`) and lengths; the test passes if no panic occurs and every error code is one of §8. |
| No global allocation (R019, R033, R045, R063, R073) | integration, own test binary `tests/no_global_alloc.rs` | A counting `#[global_allocator]` wrapping `std::alloc::System`, counting per thread (`thread_local!` with `const` initialiser), asserted to count 0 across the measured calls. |
| Compile-time rejections and borrow rules (R014, R016, R032, R017, R046 negative halves) | doc tests (`compile_fail`) | Each `compile_fail` test sits in the docs of a `#[cfg(doctest)]` function in `src/compile_tests.rs` that carries the `/// Verifies:` line, and has a compiling counterpart that differs only in the checked property, so it fails for that reason and no other. |
| `Send`/`Sync` (R017, R046, R064 positive halves) | unit | `fn assert_send<T: Send>()` / `assert_sync` instantiations. |
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

`tests/determinism.rs` (ADR-0024 item 2): each workload draws its operations from a
`liminal_core` RNG stream with a seed recorded in the test, runs twice in the same process, and
feeds the state hasher with the observable results after every step (every `usize` hashed as
`u64`). Hashes of the two runs are compared after every step.

| Workload | Description | Cross-platform bit-identity claimed |
|---|---|---|
| W1 `arena_ops` | `Arena` of 65,536 bytes; 10,000 operations: 98 % `alloc_slice_fill_aligned` of `u8`, `u32`, `u64` or `f32` with `len` in 0..=512 and `align` in {1, 2, 4, 8, 16, 64, 256, 4096}, 1 % `reset`, 1 % a request larger than the capacity. Hashed per step: result code and context fields, `used()`, `stats()`. | Yes: recorded hash literal (MEM-R081) |
| W2 `frame_ops` | `FrameAllocator` of 16,384 bytes; 600 frames, each `begin_frame` plus 0..=64 allocations drawn as in W1. Hashed per frame: `frame_index()`, `last_frame_bytes()`, `stats()`, result codes. | Yes: recorded hash literal (MEM-R081) |
| W3 `pool_ops` | `Pool<u64>` with 256 blocks; 10,000 operations: 55 % `alloc` of a random `u64`, 45 % drop of a live box chosen by index into the list of live boxes. Hashed per step: `slot()` or error code, `live()`, `stats()`, wrapping sum of live values. | Yes: recorded hash literal (MEM-R081) |

The claim rests on: only primitive element types whose size and alignment are equal on the three
64-bit targets (`docs/adr/0005-target-platforms-and-toolchain.md`); no floating-point arithmetic
(`f32` values are only copied); and the `liminal_core` RNG and hasher being platform-independent
(their spec). Cross-platform identity is not yet verified; it is verified when M1-T026's
determinism test passes in CI on all three platforms. A change of the `liminal_core` RNG or hasher
changes the literals; that is an oracle change (AGENTS.md rule 5).

## 12. Open questions

| ID | Question | Resolved by |
|---|---|---|
| Q1 | Names of the `liminal_core` error trait and code type (written here as the placeholder `liminal_core::error::Error`), the RNG stream API and the state hasher API. `docs/specs/liminal_core.md` did not exist when this spec was written. | M1-T025: uses the names from the accepted core spec and updates §5 (bump `spec_version`) if they differ. |
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
