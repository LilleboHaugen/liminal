//! [`HandleAllocator`]: issues and frees raw handles.

use core::num::NonZeroU32;

use super::RawHandle;
use crate::error::CoreError;

/// What a slot is doing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SlotState {
    /// Occupied: the handle with the slot's generation is live.
    Live,
    /// In the free list, waiting to be issued with the slot's generation.
    Free,
    /// Freed at generation `u32::MAX`; never issued again.
    Retired,
}

/// One slot. Its position in `HandleAllocator::slots` is its index.
#[derive(Clone, Copy, Debug)]
struct Slot {
    /// Generation of the live handle (`Live`), of the next handle to issue (`Free`), or of the
    /// last handle issued (`Retired`).
    generation: NonZeroU32,
    state: SlotState,
}

/// Issues and frees [`RawHandle`]s. Stores no values; [`HandleMap`](super::HandleMap) adds them.
///
/// Rules (spec CORE-R033 to CORE-R038):
/// - [`allocate`](Self::allocate) reuses the most recently freed slot (LIFO) if one exists,
///   otherwise takes the lowest never-used index. A never-used slot is issued with generation
///   1; a reused slot with its current generation.
/// - [`free`](Self::free) increments the slot's generation, so the freed handle and all its
///   copies are stale. A slot freed at generation `u32::MAX` is retired and never issued again.
/// - No handle value is issued twice by one allocator.
///
/// `allocate`, `free` and `is_live` run in amortized constant time. Memory: 8 bytes per slot ever
/// used plus 4 bytes per free slot; slots are never released.
///
/// The same sequence of calls issues the same handles in every process (ADR-0009).
///
/// # Examples
///
/// ```
/// use liminal_core::error::CoreError;
/// use liminal_core::handle::HandleAllocator;
///
/// let mut allocator = HandleAllocator::new();
/// let a = allocator.allocate()?;
/// let b = allocator.allocate()?;
/// assert_eq!((a.to_string(), b.to_string()), ("0v1".to_string(), "1v1".to_string()));
///
/// allocator.free(a)?;
/// assert!(!allocator.is_live(a));
/// // Freeing again is an error, not a silent no-op.
/// assert_eq!(allocator.free(a), Err(CoreError::StaleHandle { index: 0, generation: 1 }));
///
/// // The freed slot is reused with the next generation; `a` stays stale.
/// let c = allocator.allocate()?;
/// assert_eq!(c.to_string(), "0v2");
/// assert!(allocator.is_live(c) && !allocator.is_live(a));
/// assert_eq!(allocator.live_count(), 2);
/// assert_eq!(allocator.iter_live().collect::<Vec<_>>(), [c, b]);
/// # Ok::<(), CoreError>(())
/// ```
#[derive(Clone, Debug)]
pub struct HandleAllocator {
    /// Every slot ever used; `slots.len()` is the lowest never-used index.
    slots: Vec<Slot>,
    /// Indices of the `Free` slots, most recently freed last.
    free: Vec<u32>,
    /// Number of `Live` slots.
    live_count: u32,
    /// Indices `>= slot_limit` are never used.
    slot_limit: u32,
}

impl HandleAllocator {
    /// An empty allocator with the slot limit `u32::MAX` (indices 0 to `u32::MAX - 1`).
    #[must_use]
    pub fn new() -> HandleAllocator {
        HandleAllocator::with_slot_limit(u32::MAX)
    }

    /// An empty allocator that uses at most `slot_limit` slots (indices `0..slot_limit`). Nothing
    /// is allocated up front. With `slot_limit` 0 every [`allocate`](Self::allocate) fails.
    #[must_use]
    pub fn with_slot_limit(slot_limit: u32) -> HandleAllocator {
        HandleAllocator {
            slots: Vec::new(),
            free: Vec::new(),
            live_count: 0,
            slot_limit,
        }
    }

    /// Issues a handle: the most recently freed non-retired slot with its current generation, or
    /// else the lowest never-used index with generation 1.
    ///
    /// # Errors
    ///
    /// [`CoreError::HandleSlotsExhausted`] (LIM-CORE-0011) if no slot is free and `slot_limit`
    /// indices have been used. The allocator is unchanged.
    ///
    /// # Panics
    ///
    /// Only if an internal invariant is broken (a bug in this crate): the free list names a slot
    /// that does not exist.
    pub fn allocate(&mut self) -> Result<RawHandle, CoreError> {
        let handle = if let Some(index) = self.free.pop() {
            let slot = position(index).and_then(|position| self.slots.get_mut(position));
            let slot = slot.expect("invariant: the free list holds indices of existing slots");
            debug_assert_eq!(
                slot.state,
                SlotState::Free,
                "INV-1: free list entry is not free"
            );
            slot.state = SlotState::Live;
            RawHandle::new(index, slot.generation)
        } else {
            let index = u32::try_from(self.slots.len())
                .ok()
                .filter(|index| *index < self.slot_limit)
                .ok_or(CoreError::HandleSlotsExhausted {
                    slot_limit: self.slot_limit,
                })?;
            let generation = NonZeroU32::MIN;
            self.slots.push(Slot {
                generation,
                state: SlotState::Live,
            });
            RawHandle::new(index, generation)
        };
        self.live_count += 1;
        self.debug_check_counts();
        Ok(handle)
    }

    /// Frees a live handle: it and all its copies become stale, and the slot's generation is
    /// incremented. A slot freed at generation `u32::MAX` is retired instead.
    ///
    /// # Errors
    ///
    /// [`CoreError::StaleHandle`] (LIM-CORE-0010) with the handle's index and generation if the
    /// handle is not live: freed already, slot retired, index never used, or a generation
    /// different from the slot's. The allocator is unchanged.
    pub fn free(&mut self, handle: RawHandle) -> Result<(), CoreError> {
        let slot = position(handle.index())
            .and_then(|position| self.slots.get_mut(position))
            .filter(|slot| slot.state == SlotState::Live && slot.generation == handle.generation())
            .ok_or(CoreError::StaleHandle {
                index: handle.index(),
                generation: handle.generation().get(),
            })?;
        if let Some(next) = slot.generation.checked_add(1) {
            slot.generation = next;
            slot.state = SlotState::Free;
            self.free.push(handle.index());
        } else {
            slot.state = SlotState::Retired;
        }
        self.live_count -= 1;
        self.debug_check_counts();
        Ok(())
    }

    /// `true` exactly for handles returned by [`allocate`](Self::allocate) and not freed since.
    #[must_use]
    pub fn is_live(&self, handle: RawHandle) -> bool {
        position(handle.index())
            .and_then(|position| self.slots.get(position))
            .is_some_and(|slot| {
                slot.state == SlotState::Live && slot.generation == handle.generation()
            })
    }

    /// Number of live handles.
    #[must_use]
    pub fn live_count(&self) -> u32 {
        self.live_count
    }

    /// The live handles, each once, in ascending index order. Visits every slot ever used:
    /// O(highest index used).
    pub fn iter_live(&self) -> impl Iterator<Item = RawHandle> + '_ {
        (0_u32..)
            .zip(&self.slots)
            .filter(|(_, slot)| slot.state == SlotState::Live)
            .map(|(index, slot)| RawHandle::new(index, slot.generation))
    }

    /// INV-2 (the part that is O(1) to check): live and free slots are disjoint subsets of the
    /// slots ever used.
    fn debug_check_counts(&self) {
        debug_assert!(
            position(self.live_count)
                .is_some_and(|live| live + self.free.len() <= self.slots.len()),
            "INV-2: {} live + {} free slots exceed {} slots",
            self.live_count,
            self.free.len(),
            self.slots.len()
        );
    }

    /// Test access for the generation wrap (spec §10): sets the generation of slot `index`, which
    /// 2^32 frees would otherwise be needed to reach.
    #[cfg(test)]
    pub(super) fn set_generation_for_test(&mut self, index: u32, generation: u32) {
        self.slots[position(index).unwrap()].generation = NonZeroU32::new(generation).unwrap();
    }
}

/// `new()`: slot limit `u32::MAX`.
impl Default for HandleAllocator {
    fn default() -> HandleAllocator {
        HandleAllocator::new()
    }
}

/// The position of slot `index` in a `Vec`. `None` only where `usize` is narrower than 32 bits,
/// which no supported target is (spec §2).
pub(super) fn position(index: u32) -> Option<usize> {
    usize::try_from(index).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn handle(index: u32, generation: u32) -> RawHandle {
        RawHandle::new(index, NonZeroU32::new(generation).unwrap())
    }

    fn stale(index: u32, generation: u32) -> Result<(), CoreError> {
        Err(CoreError::StaleHandle { index, generation })
    }

    /// Verifies: CORE-R035
    #[test]
    fn freeing_at_generation_max_retires_the_slot() {
        let mut allocator = HandleAllocator::new();
        let first = allocator.allocate().unwrap();
        let other = allocator.allocate().unwrap();
        assert_eq!((first, other), (handle(0, 1), handle(1, 1)));
        allocator.free(first).unwrap();
        allocator.set_generation_for_test(0, u32::MAX - 1);

        // Second-to-last generation: freed and reused as usual.
        let penultimate = allocator.allocate().unwrap();
        assert_eq!(penultimate, handle(0, u32::MAX - 1));
        allocator.free(penultimate).unwrap();
        let last = allocator.allocate().unwrap();
        assert_eq!(last, handle(0, u32::MAX));
        assert!(allocator.is_live(last));

        // Freeing the last generation retires slot 0.
        allocator.free(last).unwrap();
        assert!(!allocator.is_live(last));
        assert_eq!(allocator.live_count(), 1);
        assert_eq!(allocator.iter_live().collect::<Vec<_>>(), [other]);
        assert_eq!(allocator.free(last), stale(0, u32::MAX));
        // No wrapped-around generation is live either.
        assert!(!allocator.is_live(handle(0, 1)));
        assert_eq!(allocator.free(handle(0, 1)), stale(0, 1));

        // `allocate` takes a never-used index instead …
        assert_eq!(allocator.allocate().unwrap(), handle(2, 1));
        // … or another free slot, never slot 0.
        allocator.free(other).unwrap();
        assert_eq!(allocator.allocate().unwrap(), handle(1, 2));
        for expected_index in 3..20 {
            assert_eq!(allocator.allocate().unwrap(), handle(expected_index, 1));
        }
        assert!(allocator.iter_live().all(|live| live.index() != 0));
    }

    /// Verifies: CORE-R035, CORE-R037
    #[test]
    fn a_retired_slot_stays_used_up_under_a_slot_limit() {
        let mut allocator = HandleAllocator::with_slot_limit(2);
        let first = allocator.allocate().unwrap();
        allocator.set_generation_for_test(0, u32::MAX);
        let first = handle(first.index(), u32::MAX);
        let second = allocator.allocate().unwrap();
        allocator.free(first).unwrap();

        let exhausted = Err(CoreError::HandleSlotsExhausted { slot_limit: 2 });
        assert_eq!(allocator.allocate(), exhausted);
        allocator.free(second).unwrap();
        assert_eq!(allocator.allocate().unwrap(), handle(1, 2));
        assert_eq!(allocator.allocate(), exhausted);
        assert_eq!(allocator.live_count(), 1);
    }

    /// Verifies: CORE-R034
    #[test]
    fn free_increments_the_generation_by_one() {
        let mut allocator = HandleAllocator::new();
        let mut current = allocator.allocate().unwrap();
        for generation in 1..=50 {
            assert_eq!(current, handle(0, generation));
            allocator.free(current).unwrap();
            assert!(!allocator.is_live(current));
            current = allocator.allocate().unwrap();
        }
    }
}
