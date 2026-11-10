//! [`HandleMap<T>`]: a slot map.

use super::allocator::position;
use super::{Handle, HandleAllocator};
use crate::error::CoreError;

/// Slot map: stores one `T` per live [`Handle<T>`].
///
/// Handles are issued by the rules of [`HandleAllocator`]: the most recently removed slot is
/// reused first with the next generation, otherwise the lowest never-used index with generation
/// 1; a slot freed at generation `u32::MAX` is retired. A removed handle is stale: `get`,
/// `get_mut`, `contains` and `remove` treat it as absent, also after its slot is reused.
///
/// `insert`, `get`, `get_mut`, `contains` and `remove` run in amortized constant time. Iteration
/// is in ascending index order and visits every slot ever used. Memory: one `Option<T>` plus 8
/// bytes per slot ever used; slots are never released.
///
/// # Examples
///
/// ```
/// use liminal_core::error::CoreError;
/// use liminal_core::handle::HandleMap;
///
/// let mut scores: HandleMap<u32> = HandleMap::new();
/// let alice = scores.insert(10)?;
/// let bob = scores.insert(20)?;
///
/// if let Some(score) = scores.get_mut(alice) {
///     *score += 5;
/// }
/// assert_eq!(scores.get(alice), Some(&15));
///
/// assert_eq!(scores.remove(alice), Some(15));
/// assert_eq!(scores.remove(alice), None); // stale
/// assert!(!scores.contains(alice));
///
/// // The slot is reused, but the old handle does not reach the new value.
/// let carol = scores.insert(30)?;
/// assert_eq!(carol.raw().index(), alice.raw().index());
/// assert_eq!(scores.get(alice), None);
///
/// let entries: Vec<(u32, u32)> =
///     scores.iter().map(|(handle, score)| (handle.raw().index(), *score)).collect();
/// assert_eq!(entries, [(0, 30), (1, 20)]);
/// assert_eq!(scores.len(), 2);
/// # let _ = bob;
/// # Ok::<(), CoreError>(())
/// ```
#[derive(Clone, Debug)]
pub struct HandleMap<T> {
    /// Decides which handles are live.
    allocator: HandleAllocator,
    /// `values[i]` is `Some` exactly while slot `i` is live (INV-2).
    values: Vec<Option<T>>,
}

impl<T> HandleMap<T> {
    /// An empty map with the slot limit `u32::MAX`.
    #[must_use]
    pub fn new() -> HandleMap<T> {
        HandleMap::with_slot_limit(u32::MAX)
    }

    /// An empty map that uses at most `slot_limit` slots (indices `0..slot_limit`). Nothing is
    /// allocated up front.
    #[must_use]
    pub fn with_slot_limit(slot_limit: u32) -> HandleMap<T> {
        HandleMap {
            allocator: HandleAllocator::with_slot_limit(slot_limit),
            values: Vec::new(),
        }
    }

    /// Stores `value` and returns its handle.
    ///
    /// # Errors
    ///
    /// [`CoreError::HandleSlotsExhausted`] (LIM-CORE-0011) if no slot is free and the slot limit
    /// is reached. The map is unchanged and `value` is dropped.
    pub fn insert(&mut self, value: T) -> Result<Handle<T>, CoreError> {
        let raw = self.allocator.allocate()?;
        if let Some(entry) = position(raw.index()).and_then(|at| self.values.get_mut(at)) {
            debug_assert!(entry.is_none(), "INV-2: a free slot holds a value");
            *entry = Some(value);
        } else {
            debug_assert_eq!(position(raw.index()), Some(self.values.len()));
            self.values.push(Some(value));
        }
        Ok(Handle::from_raw(raw))
    }

    /// The value of a live handle; `None` if the handle is not live.
    #[must_use]
    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        if !self.contains(handle) {
            return None;
        }
        self.values.get(position(handle.raw().index())?)?.as_ref()
    }

    /// The value of a live handle, mutably; `None` if the handle is not live.
    #[must_use]
    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        if !self.contains(handle) {
            return None;
        }
        self.values
            .get_mut(position(handle.raw().index())?)?
            .as_mut()
    }

    /// Removes a live handle's value and returns it; the handle and its copies become stale.
    /// `None` (map unchanged) if the handle is not live.
    pub fn remove(&mut self, handle: Handle<T>) -> Option<T> {
        self.allocator.free(handle.raw()).ok()?;
        let value = self.values.get_mut(position(handle.raw().index())?)?.take();
        debug_assert!(value.is_some(), "INV-2: a live slot holds no value");
        value
    }

    /// `true` if the handle is live: returned by [`insert`](Self::insert) and not removed since.
    #[must_use]
    pub fn contains(&self, handle: Handle<T>) -> bool {
        self.allocator.is_live(handle.raw())
    }

    /// Number of live entries.
    #[must_use]
    pub fn len(&self) -> u32 {
        self.allocator.live_count()
    }

    /// `true` if there is no live entry.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Every live entry once, in ascending index order.
    pub fn iter(&self) -> impl Iterator<Item = (Handle<T>, &T)> + '_ {
        // Both sides are in ascending index order and cover the same slots (INV-2).
        self.allocator
            .iter_live()
            .map(Handle::from_raw)
            .zip(self.values.iter().flatten())
    }

    /// Every live entry once, with a mutable value, in ascending index order.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Handle<T>, &mut T)> + '_ {
        self.allocator
            .iter_live()
            .map(Handle::from_raw)
            .zip(self.values.iter_mut().flatten())
    }
}

/// `new()`: an empty map with the slot limit `u32::MAX`.
impl<T> Default for HandleMap<T> {
    fn default() -> HandleMap<T> {
        HandleMap::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies: CORE-R039
    #[test]
    fn the_map_retires_a_slot_removed_at_generation_max() {
        let mut map: HandleMap<&str> = HandleMap::with_slot_limit(3);
        let first = map.insert("first").unwrap();
        let other = map.insert("other").unwrap();
        assert_eq!(map.remove(first), Some("first"));
        map.allocator.set_generation_for_test(0, u32::MAX);

        let last = map.insert("last").unwrap();
        assert_eq!(
            (last.raw().index(), last.raw().generation().get()),
            (0, u32::MAX)
        );
        assert_eq!(map.get(last), Some(&"last"));
        assert_eq!(map.remove(last), Some("last"));
        assert_eq!(map.remove(last), None);
        assert_eq!(map.get(last), None);
        assert!(!map.contains(last));

        // Slot 0 is retired: the next inserts use index 2, then fail; slot 1 is still reusable.
        let third = map.insert("third").unwrap();
        assert_eq!(
            (third.raw().index(), third.raw().generation().get()),
            (2, 1)
        );
        assert_eq!(
            map.insert("none"),
            Err(CoreError::HandleSlotsExhausted { slot_limit: 3 })
        );
        assert_eq!(map.remove(other), Some("other"));
        let reused = map.insert("reused").unwrap();
        assert_eq!(
            (reused.raw().index(), reused.raw().generation().get()),
            (1, 2)
        );
        let entries: Vec<_> = map.iter().map(|(handle, value)| (handle, *value)).collect();
        assert_eq!(entries, [(reused, "reused"), (third, "third")]);
        assert_eq!(map.len(), 2);
    }
}
