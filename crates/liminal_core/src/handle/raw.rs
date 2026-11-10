//! [`RawHandle`]: the untyped generational handle.

use core::num::NonZeroU32;

/// Untyped generational handle: a slot `index` and the slot's `generation` when the handle was
/// issued.
///
/// 8 bytes; `Option<RawHandle>` is also 8 bytes (the generation is never zero). Ordered by index,
/// then by generation. Handles are issued by [`HandleAllocator`](super::HandleAllocator) and
/// [`HandleMap`](super::HandleMap); [`RawHandle::from_bits`] restores one from its `u64` form.
///
/// A handle carries no reference to the allocator that issued it: using it with another
/// allocator is not detected and names whatever that allocator has in the same slot and
/// generation.
///
/// # Examples
///
/// ```
/// use liminal_core::error::CoreError;
/// use liminal_core::handle::{HandleAllocator, RawHandle};
///
/// let mut allocator = HandleAllocator::new();
/// let handle = allocator.allocate()?;
/// assert_eq!((handle.index(), handle.generation().get()), (0, 1));
/// assert_eq!(handle.to_string(), "0v1");
/// assert_eq!(handle.to_bits(), 1 << 32);
/// assert_eq!(RawHandle::from_bits(handle.to_bits()), Some(handle));
/// assert_eq!(RawHandle::from_bits(7), None);
/// # Ok::<(), CoreError>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct RawHandle {
    index: u32,
    generation: NonZeroU32,
}

impl RawHandle {
    /// The handle of slot `index` at `generation`. Only allocators issue handles.
    pub(super) fn new(index: u32, generation: NonZeroU32) -> RawHandle {
        RawHandle { index, generation }
    }

    /// The slot index.
    #[must_use]
    pub fn index(self) -> u32 {
        self.index
    }

    /// The generation of the slot when this handle was issued. Starts at 1.
    #[must_use]
    pub fn generation(self) -> NonZeroU32 {
        self.generation
    }

    /// The handle as one integer: `generation × 2^32 + index`. Never below `2^32`.
    #[must_use]
    pub fn to_bits(self) -> u64 {
        (u64::from(self.generation.get()) << 32) | u64::from(self.index)
    }

    /// The handle with `index = bits mod 2^32` and `generation = bits / 2^32`; `None` if the
    /// upper 32 bits (the generation) are zero. Inverse of [`RawHandle::to_bits`].
    ///
    /// The result is a handle value, not a live handle: whether it is live is decided by the
    /// allocator it is used with.
    #[must_use]
    pub fn from_bits(bits: u64) -> Option<RawHandle> {
        let generation = NonZeroU32::new(u32::try_from(bits >> 32).ok()?)?;
        let index = u32::try_from(bits & u64::from(u32::MAX)).ok()?;
        Some(RawHandle { index, generation })
    }
}

/// Writes `{index}v{generation}` in decimal, for example `3v2`.
impl core::fmt::Display for RawHandle {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}v{}", self.index, self.generation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn handle(index: u32, generation: u32) -> RawHandle {
        RawHandle::new(index, NonZeroU32::new(generation).unwrap())
    }

    /// Verifies: CORE-R030
    #[test]
    fn raw_handle_and_its_option_are_8_bytes() {
        assert_eq!(size_of::<RawHandle>(), 8);
        assert_eq!(size_of::<Option<RawHandle>>(), 8);
    }

    /// Verifies: CORE-R031
    #[test]
    fn bits_are_generation_times_2_pow_32_plus_index() {
        for (index, generation, bits) in [
            (0, 1, 0x0000_0001_0000_0000_u64),
            (3, 2, 0x0000_0002_0000_0003),
            (u32::MAX, 1, 0x0000_0001_FFFF_FFFF),
            (0, u32::MAX, 0xFFFF_FFFF_0000_0000),
            (u32::MAX, u32::MAX, u64::MAX),
            (0x1234_5678, 0x9ABC_DEF0, 0x9ABC_DEF0_1234_5678),
        ] {
            let handle = handle(index, generation);
            assert_eq!(handle.to_bits(), bits);
            assert_eq!(
                handle.to_bits(),
                u64::from(generation) * (1 << 32) + u64::from(index)
            );
            assert_eq!(RawHandle::from_bits(bits), Some(handle));
            assert_eq!(
                (handle.index(), handle.generation().get()),
                (index, generation)
            );
        }
    }

    /// Verifies: CORE-R031
    #[test]
    fn from_bits_rejects_a_zero_generation() {
        for bits in [0, 1, 7, u64::from(u32::MAX)] {
            assert_eq!(RawHandle::from_bits(bits), None, "{bits}");
        }
        assert_eq!(RawHandle::from_bits(1 << 32), Some(handle(0, 1)));
    }

    /// Verifies: CORE-R031
    #[test]
    fn display_is_index_v_generation_in_decimal() {
        assert_eq!(handle(3, 2).to_string(), "3v2");
        assert_eq!(handle(0, 1).to_string(), "0v1");
        assert_eq!(
            handle(u32::MAX, u32::MAX).to_string(),
            "4294967295v4294967295"
        );
    }

    #[test]
    fn order_is_by_index_then_generation() {
        let ascending = [
            handle(0, 1),
            handle(0, 2),
            handle(1, 1),
            handle(2, 9),
            handle(3, 1),
        ];
        for (i, a) in ascending.iter().enumerate() {
            for (j, b) in ascending.iter().enumerate() {
                assert_eq!(a.cmp(b), i.cmp(&j), "{a} vs {b}");
            }
        }
    }
}
