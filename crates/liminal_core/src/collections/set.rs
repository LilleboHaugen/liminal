//! [`DetHashSet`]: a hash set that iterates in insertion order.

use core::borrow::Borrow;
use core::hash::Hash;

use super::DetHashMap;
use crate::hash::{StateHash, StateHasher};

/// Hash set whose iteration order is the order in which its values were first inserted (spec
/// CORE-R074).
///
/// - Inserting a new value appends it at the end. Inserting a present value changes nothing.
/// - Removing a value keeps the relative order of the remaining values. Inserting it again
///   appends it at the end.
/// - The order depends only on the sequence of operations: not on capacity, hash values, the
///   process or the platform (ADR-0009).
///
/// Cost: [`insert`](Self::insert) and [`contains`](Self::contains) take expected constant time
/// (amortized for `insert`); [`remove`](Self::remove) takes O(len). The requirements on `T` are
/// those of [`DetHashMap`] keys.
///
/// # Examples
///
/// ```
/// use liminal_core::collections::DetHashSet;
///
/// let mut visited: DetHashSet<(i32, i32)> = DetHashSet::new();
/// assert!(visited.insert((2, 0)));
/// assert!(visited.insert((0, 1)));
/// assert!(!visited.insert((2, 0))); // present: no change
/// assert!(visited.contains(&(0, 1)));
///
/// assert!(visited.remove(&(2, 0)));
/// visited.insert((2, 0)); // appended at the end
/// assert_eq!(visited.iter().copied().collect::<Vec<_>>(), [(0, 1), (2, 0)]);
/// assert_eq!(visited.len(), 2);
/// ```
#[derive(Clone)]
pub struct DetHashSet<T> {
    /// The values are the keys of a map without values.
    map: DetHashMap<T, ()>,
}

impl<T> DetHashSet<T> {
    /// An empty set. Allocates nothing.
    #[must_use]
    pub fn new() -> DetHashSet<T> {
        DetHashSet {
            map: DetHashMap::new(),
        }
    }

    /// Number of values.
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// `true` if the set has no value.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl<T: Hash + Eq> DetHashSet<T> {
    /// An empty set that holds `capacity` values without reallocating. The capacity has no
    /// effect on iteration order or on any result.
    ///
    /// # Panics
    ///
    /// If the allocation size for `capacity` values exceeds `isize::MAX` bytes, as
    /// `Vec::with_capacity` does.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> DetHashSet<T> {
        DetHashSet {
            map: DetHashMap::with_capacity(capacity),
        }
    }

    /// Inserts a value. Returns `true` if it was not present (it is appended at the end). If an
    /// equal value is present, the set is unchanged (the stored value is kept) and the result is
    /// `false`.
    pub fn insert(&mut self, value: T) -> bool {
        if self.map.contains_key(&value) {
            return false;
        }
        self.map.insert(value, ());
        true
    }

    /// `true` if the value is present.
    #[must_use]
    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.contains_key(value)
    }

    /// Removes a value, keeping the relative order of the remaining values. Returns `true` if
    /// it was present; `false` (set unchanged) otherwise. O(len).
    pub fn remove<Q>(&mut self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.remove(value).is_some()
    }

    /// Removes all values. Keeps the allocated capacity.
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// The values in the order they were first inserted.
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.map.keys()
    }
}

/// `new()`: an empty set.
impl<T> Default for DetHashSet<T> {
    fn default() -> DetHashSet<T> {
        DetHashSet::new()
    }
}

/// Writes the values as a set, `{value, …}`, in iteration order.
impl<T: core::fmt::Debug> core::fmt::Debug for DetHashSet<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut set = f.debug_set();
        // The map's `Debug` form of its keys, without requiring `T: Hash + Eq` for `keys()`.
        self.map.debug_keys(&mut set);
        set.finish()
    }
}

/// The length as `u64`, then each value in iteration order.
impl<T: StateHash> StateHash for DetHashSet<T> {
    fn state_hash(&self, hasher: &mut StateHasher) {
        hasher.write_usize(self.map.len());
        self.map.state_hash_keys(hasher);
    }
}
