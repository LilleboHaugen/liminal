//! [`FixedState`] and [`DetHasher`]: the fixed-seed `BuildHasher` for hash collections.

use super::xxh64::{Xxh64, len_u64};

/// Fixed-seed `core::hash::BuildHasher`: every hasher it builds starts in the same state (XXH64
/// with seed 0, nothing written), in every process and on every supported platform (spec
/// CORE-R060).
///
/// Use it for `std` hash collections in deterministic crates, where a randomly seeded hasher is
/// not allowed (ADR-0009): `HashMap::with_hasher(FixedState)`.
///
/// **Iteration order is still not defined.** The iteration order of a `std::collections::HashMap`
/// or `HashSet` that uses `FixedState` depends on the `std` implementation and on the collection's
/// capacity history. Use such a collection only where iteration order cannot affect results;
/// otherwise use [`DetHashMap`](crate::collections::DetHashMap) or
/// [`DetHashSet`](crate::collections::DetHashSet).
///
/// The value of `hash_one(&v)` is the same for two `FixedState` instances in one process. It is
/// not fixed across Rust releases: which `Hasher` calls a `Hash` impl makes is decided by that
/// impl. Do not record such values; record state hashes ([`StateHasher`](super::StateHasher))
/// instead.
///
/// # Examples
///
/// ```
/// use core::hash::BuildHasher;
/// use std::collections::HashSet;
///
/// use liminal_core::hash::FixedState;
///
/// let mut seen: HashSet<u32, FixedState> = HashSet::with_hasher(FixedState);
/// seen.insert(7);
/// assert!(seen.contains(&7)); // membership does not depend on iteration order
///
/// assert_eq!(FixedState.hash_one("key"), FixedState::default().hash_one("key"));
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct FixedState;

impl core::hash::BuildHasher for FixedState {
    type Hasher = DetHasher;

    /// A hasher with nothing written.
    fn build_hasher(&self) -> DetHasher {
        DetHasher {
            state: Xxh64::new(),
        }
    }
}

/// `core::hash::Hasher` over XXH64 with seed 0, built by [`FixedState`] (spec CORE-R061).
///
/// `finish` is XXH64 of all bytes appended. The integer methods append little-endian bytes of
/// the integer's width, and `write_usize` / `write_isize` append 8 bytes, so one sequence of
/// `Hasher` calls gives the same result on every supported platform (the default methods of
/// `core::hash::Hasher` would append native-endian, pointer-width bytes).
///
/// # Examples
///
/// ```
/// use core::hash::{BuildHasher, Hasher};
///
/// use liminal_core::hash::{FixedState, StateHasher};
///
/// let mut hasher = FixedState.build_hasher();
/// hasher.write_u32(7);
/// hasher.write_usize(2);
///
/// // The same bytes through the state hasher: 4 bytes, then 8 bytes, little-endian.
/// let mut bytes = StateHasher::new();
/// bytes.write_bytes(&[7, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0]);
/// assert_eq!(hasher.finish(), bytes.finish());
/// ```
#[derive(Clone, Debug)]
pub struct DetHasher {
    state: Xxh64,
}

/// Defines `Hasher` integer methods that append the value's little-endian bytes.
macro_rules! hasher_le_methods {
    ($($name:ident: $type:ty;)*) => {
        $(
            fn $name(&mut self, value: $type) {
                self.state.write(&value.to_le_bytes());
            }
        )*
    };
}

impl core::hash::Hasher for DetHasher {
    /// XXH64 (seed 0) of all bytes appended so far. Does not change the hasher.
    fn finish(&self) -> u64 {
        self.state.finish()
    }

    /// Appends the bytes as they are.
    fn write(&mut self, bytes: &[u8]) {
        self.state.write(bytes);
    }

    hasher_le_methods! {
        write_u8: u8;
        write_u16: u16;
        write_u32: u32;
        write_u64: u64;
        write_u128: u128;
        write_i8: i8;
        write_i16: i16;
        write_i32: i32;
        write_i64: i64;
        write_i128: i128;
    }

    /// Appends the value as a `u64`: 8 little-endian bytes.
    fn write_usize(&mut self, value: usize) {
        self.write_u64(len_u64(value));
    }

    /// Appends the value as an `i64`: 8 little-endian two's-complement bytes.
    fn write_isize(&mut self, value: isize) {
        self.write_i64(i64::try_from(value).expect("invariant: isize is at most 64 bits"));
    }
}
