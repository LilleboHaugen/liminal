//! Generational handles (spec §5.2): stable 8-byte names for values in slots, which become
//! stale instead of dangling when the value is removed.
//!
//! A [`HandleAllocator`] owns slots numbered by a `u32` index. Each slot has a generation (`u32`,
//! starting at 1). A handle names a slot and the generation it had when the handle was issued; it
//! is *live* while the slot is occupied with that generation. Freeing a slot increments its
//! generation, so every earlier handle to it is stale. A slot whose generation would pass
//! `u32::MAX` is *retired* and never used again, so no handle value is issued twice.
//!
//! ```text
//! slot 3:  gen 1 (live: 3v1) --free--> gen 2 (free) --allocate--> gen 2 (live: 3v2) --free--> gen 3 ...
//!          ... gen 4294967295 (live) --free--> retired (never reallocated)
//! ```
//!
//! - [`RawHandle`]: the untyped handle (index and generation).
//! - [`Handle<T>`]: a `RawHandle` that names a value of type `T`.
//! - [`HandleAllocator`]: issues and frees raw handles; stores no values.
//! - [`HandleMap<T>`]: a slot map that stores one `T` per live handle.
//!
//! Slot reuse is deterministic (ADR-0009): the same sequence of operations issues the same
//! handles in every process.

mod allocator;
mod map;
mod raw;
mod typed;

pub use allocator::HandleAllocator;
pub use map::HandleMap;
pub use raw::RawHandle;
pub use typed::Handle;
