//! [`DetHashMap`]: a hash map that iterates in insertion order.

use core::borrow::Borrow;
use core::hash::{BuildHasher, Hash};

use super::index_table::IndexTable;
use crate::hash::{FixedState, StateHash, StateHasher};

/// One entry of the ordered entry list.
#[derive(Clone)]
struct Entry<K, V> {
    /// `FixedState` hash of `key`, kept so that rebuilding the lookup table needs no `K: Hash`.
    hash: u64,
    key: K,
    value: V,
}

/// Hash map whose iteration order is the order in which its keys were first inserted (spec
/// CORE-R070 to CORE-R076).
///
/// - Inserting a new key appends the entry at the end. Inserting a present key replaces the
///   value and keeps the entry's position.
/// - Removing a key keeps the relative order of the remaining entries. Inserting it again
///   appends it at the end.
/// - The order depends only on the sequence of operations: not on capacity, hash values, the
///   process or the platform (ADR-0009).
///
/// Cost: [`insert`](Self::insert), [`get`](Self::get), [`get_mut`](Self::get_mut) and
/// [`contains_key`](Self::contains_key) take expected constant time (amortized for `insert`).
/// [`remove`](Self::remove) takes O(len). Memory: one entry (key, value, 8-byte hash) per key in
/// one allocation, plus a lookup table of at least `2 × len` machine words.
///
/// `K` must hash and compare consistently (`a == b` implies equal hashes), and a key must not
/// change in a way that affects its hash or equality while it is in the map, as for
/// `std::collections::HashMap`. Lookups by a borrowed form `Q` need `K: Borrow<Q>` with the same
/// `Hash` and `Eq` as `K`.
///
/// Keys are hashed with [`FixedState`] (XXH64, seed 0). It is not resistant to chosen-key
/// collisions; do not key the map by data an adversary controls if that matters.
///
/// # Examples
///
/// ```
/// use liminal_core::collections::DetHashMap;
///
/// let mut health: DetHashMap<&str, u32> = DetHashMap::new();
/// assert_eq!(health.insert("orc", 30), None);
/// assert_eq!(health.insert("bat", 5), None);
/// assert_eq!(health.insert("elf", 20), None);
///
/// // Replacing a value keeps the entry's position.
/// assert_eq!(health.insert("orc", 25), Some(30));
/// assert_eq!(health.keys().copied().collect::<Vec<_>>(), ["orc", "bat", "elf"]);
///
/// if let Some(points) = health.get_mut("bat") {
///     *points += 1;
/// }
/// assert_eq!(health.get("bat"), Some(&6));
///
/// // Removing keeps the order of the rest; inserting the key again appends it.
/// assert_eq!(health.remove("orc"), Some(25));
/// health.insert("orc", 30);
/// let entries: Vec<(&str, u32)> = health.iter().map(|(name, points)| (*name, *points)).collect();
/// assert_eq!(entries, [("bat", 6), ("elf", 20), ("orc", 30)]);
/// assert_eq!(health.len(), 3);
/// ```
#[derive(Clone)]
pub struct DetHashMap<K, V> {
    /// The entries in iteration order.
    entries: Vec<Entry<K, V>>,
    /// Maps a key's hash to the position of its entry in `entries` (INV-3: both hold the same
    /// keys, each once).
    table: IndexTable,
}

impl<K, V> DetHashMap<K, V> {
    /// An empty map. Allocates nothing.
    #[must_use]
    pub fn new() -> DetHashMap<K, V> {
        DetHashMap {
            entries: Vec::new(),
            table: IndexTable::new(),
        }
    }

    /// Number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `true` if the map has no entry.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Adds every key to a `Debug` set, in iteration order (for `DetHashSet`).
    pub(super) fn debug_keys(&self, set: &mut core::fmt::DebugSet<'_, '_>)
    where
        K: core::fmt::Debug,
    {
        set.entries(self.entries.iter().map(|entry| &entry.key));
    }

    /// Writes the state encoding of every key, in iteration order (for `DetHashSet`).
    pub(super) fn state_hash_keys(&self, hasher: &mut StateHasher)
    where
        K: StateHash,
    {
        for entry in &self.entries {
            entry.key.state_hash(hasher);
        }
    }
}

impl<K: Hash + Eq, V> DetHashMap<K, V> {
    /// An empty map that holds `capacity` entries without reallocating. The capacity has no
    /// effect on iteration order or on any result.
    ///
    /// # Panics
    ///
    /// If the allocation size for `capacity` entries exceeds `isize::MAX` bytes, as
    /// `Vec::with_capacity` does.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> DetHashMap<K, V> {
        DetHashMap {
            entries: Vec::with_capacity(capacity),
            table: IndexTable::with_capacity(capacity),
        }
    }

    /// Inserts a key and value. A new key is appended at the end and `None` is returned. For a
    /// key that is present, the value is replaced in place (the entry keeps its position and its
    /// original key) and the old value is returned.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let hash = FixedState.hash_one(&key);
        if let Some(position) = self.position(hash, &key) {
            let entry = self.entries.get_mut(position)?;
            return Some(core::mem::replace(&mut entry.value, value));
        }
        let position = self.entries.len();
        self.entries.push(Entry { hash, key, value });
        if self.table.holds(self.entries.len()) {
            self.table.insert(hash, position);
        } else {
            // Grow geometrically: room for twice the current number of entries.
            self.rebuild_table(self.entries.len().saturating_mul(2));
        }
        debug_assert!(
            self.table.find(hash, |stored| stored == position).is_some(),
            "INV-3: the inserted key is not in the lookup table"
        );
        None
    }

    /// The value of `key`, or `None` if the key is absent.
    #[must_use]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let position = self.position(FixedState.hash_one(key), key)?;
        self.entries.get(position).map(|entry| &entry.value)
    }

    /// The value of `key`, mutably, or `None` if the key is absent.
    #[must_use]
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let position = self.position(FixedState.hash_one(key), key)?;
        self.entries.get_mut(position).map(|entry| &mut entry.value)
    }

    /// `true` if the key is present.
    #[must_use]
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.position(FixedState.hash_one(key), key).is_some()
    }

    /// Removes `key` and returns its value, keeping the relative order of the remaining
    /// entries; `None` (map unchanged) if the key is absent. O(len).
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let position = self.position(FixedState.hash_one(key), key)?;
        let entry = self.entries.remove(position);
        // Every later entry moved one position down: renumber the lookup table.
        self.rebuild_table(0);
        debug_assert_eq!(
            self.table.occupied(),
            self.entries.len(),
            "INV-3: lookup table and entry list differ in length"
        );
        Some(entry.value)
    }

    /// Removes all entries. Keeps the allocated capacity.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.table.clear();
    }

    /// The entries in the order their keys were first inserted.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> + '_ {
        self.entries.iter().map(|entry| (&entry.key, &entry.value))
    }

    /// The entries with mutable values, in the order their keys were first inserted.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> + '_ {
        self.entries
            .iter_mut()
            .map(|entry| (&entry.key, &mut entry.value))
    }

    /// The keys in the order they were first inserted.
    pub fn keys(&self) -> impl Iterator<Item = &K> + '_ {
        self.entries.iter().map(|entry| &entry.key)
    }

    /// The values in the order their keys were first inserted.
    pub fn values(&self) -> impl Iterator<Item = &V> + '_ {
        self.entries.iter().map(|entry| &entry.value)
    }

    /// Position in `entries` of the entry whose key equals `key`, given the key's hash.
    fn position<Q>(&self, hash: u64, key: &Q) -> Option<usize>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.table.find(hash, |position| {
            self.entries
                .get(position)
                .is_some_and(|entry| entry.hash == hash && entry.key.borrow() == key)
        })
    }

    /// Rebuilds the lookup table from the entry list, with room for at least `capacity` entries.
    fn rebuild_table(&mut self, capacity: usize) {
        self.table
            .rebuild(capacity, self.entries.iter().map(|entry| entry.hash));
    }
}

/// `new()`: an empty map.
impl<K, V> Default for DetHashMap<K, V> {
    fn default() -> DetHashMap<K, V> {
        DetHashMap::new()
    }
}

/// Writes the entries as a map, `{key: value, …}`, in iteration order.
impl<K: core::fmt::Debug, V: core::fmt::Debug> core::fmt::Debug for DetHashMap<K, V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_map()
            .entries(self.entries.iter().map(|entry| (&entry.key, &entry.value)))
            .finish()
    }
}

/// The length as `u64`, then each key and value in iteration order. Two maps with the same
/// entries in a different order hash differently.
impl<K: StateHash, V: StateHash> StateHash for DetHashMap<K, V> {
    fn state_hash(&self, hasher: &mut StateHasher) {
        hasher.write_usize(self.entries.len());
        for entry in &self.entries {
            entry.key.state_hash(hasher);
            entry.value.state_hash(hasher);
        }
    }
}
