//! Test-local reference models of `DetHashMap` and `DetHashSet` (spec §10, CORE-R072,
//! CORE-R074): entries in a `Vec`, found by linear search with `==`. Written from CORE-R070,
//! CORE-R071 and CORE-R074; they use no hashing.

/// Reference model of `DetHashMap<K, V>`: a `Vec<(K, V)>` in insertion order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VecMap<K, V> {
    pub entries: Vec<(K, V)>,
}

impl<K: PartialEq, V> VecMap<K, V> {
    pub fn new() -> VecMap<K, V> {
        VecMap {
            entries: Vec::new(),
        }
    }

    fn position(&self, key: &K) -> Option<usize> {
        self.entries.iter().position(|(present, _)| present == key)
    }

    /// CORE-R070: a present key keeps its position and gets the new value; a new key is
    /// appended.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if let Some(position) = self.position(&key) {
            Some(core::mem::replace(&mut self.entries[position].1, value))
        } else {
            self.entries.push((key, value));
            None
        }
    }

    /// CORE-R071: the remaining entries keep their relative order.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let position = self.position(key)?;
        Some(self.entries.remove(position).1)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.position(key).map(|position| &self.entries[position].1)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.position(key)
            .map(|position| &mut self.entries[position].1)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.position(key).is_some()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// Reference model of `DetHashSet<T>`: a `Vec<T>` in insertion order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VecSet<T> {
    pub values: Vec<T>,
}

impl<T: PartialEq> VecSet<T> {
    pub fn new() -> VecSet<T> {
        VecSet { values: Vec::new() }
    }

    /// CORE-R074: a present value changes nothing.
    pub fn insert(&mut self, value: T) -> bool {
        if self.values.contains(&value) {
            return false;
        }
        self.values.push(value);
        true
    }

    pub fn remove(&mut self, value: &T) -> bool {
        match self.values.iter().position(|present| present == value) {
            Some(position) => {
                self.values.remove(position);
                true
            }
            None => false,
        }
    }

    pub fn contains(&self, value: &T) -> bool {
        self.values.contains(value)
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }
}
