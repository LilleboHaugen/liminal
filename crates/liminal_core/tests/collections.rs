//! Integration tests of `liminal_core::collections` through the public API (spec §10, CORE-R070
//! to CORE-R076). Oracle: the `Vec` linear-search reference models in `common/vec_model.rs`
//! (differential property tests) and hand-written expected orders.

#[path = "common/vec_model.rs"]
mod vec_model;

use core::hash::{Hash, Hasher};

use liminal_core::collections::{DetHashMap, DetHashSet};
use proptest::prelude::*;
use vec_model::{VecMap, VecSet};

/// A type that implements no trait (CORE-R076 case (a)).
struct Opaque;

/// A key whose hash ignores its value: every key lands in the same probe sequence, so lookups
/// rely on `Eq` alone (CORE-R073: order must not depend on hash values).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SameHash(u8);

impl Hash for SameHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u8(0);
    }
}

/// One step of a random operation sequence on a map with keys of type `K`.
#[derive(Clone, Debug)]
enum MapOp<K> {
    Insert(K, u32),
    Remove(K),
    Get(K),
    /// Add to the value through `get_mut`, if the key is present.
    AddAssign(K, u32),
    ContainsKey(K),
    /// Add 1 to every value through `iter_mut`.
    BumpAll,
    Clear,
}

fn map_ops<K: Clone + core::fmt::Debug + 'static>(
    key: impl Strategy<Value = K> + Clone + 'static,
) -> impl Strategy<Value = Vec<MapOp<K>>> {
    let op = prop_oneof![
        8 => (key.clone(), any::<u32>()).prop_map(|(key, value)| MapOp::Insert(key, value)),
        5 => key.clone().prop_map(MapOp::Remove),
        3 => key.clone().prop_map(MapOp::Get),
        2 => (key.clone(), 0_u32..1000).prop_map(|(key, add)| MapOp::AddAssign(key, add)),
        2 => key.prop_map(MapOp::ContainsKey),
        1 => Just(MapOp::BumpAll),
        1 => Just(MapOp::Clear),
    ];
    proptest::collection::vec(op, 0..300)
}

/// Applies `op` to the map and to the model and compares every result and, afterwards, every
/// observable property (CORE-R072).
fn apply_and_compare<K: Hash + Eq + Clone + core::fmt::Debug>(
    map: &mut DetHashMap<K, u32>,
    model: &mut VecMap<K, u32>,
    op: &MapOp<K>,
) -> Result<(), TestCaseError> {
    match op {
        MapOp::Insert(key, value) => {
            prop_assert_eq!(
                map.insert(key.clone(), *value),
                model.insert(key.clone(), *value)
            );
        }
        MapOp::Remove(key) => prop_assert_eq!(map.remove(key), model.remove(key)),
        MapOp::Get(key) => prop_assert_eq!(map.get(key), model.get(key)),
        MapOp::AddAssign(key, add) => {
            let (actual, expected) = (map.get_mut(key), model.get_mut(key));
            prop_assert_eq!(actual.is_some(), expected.is_some());
            if let (Some(actual), Some(expected)) = (actual, expected) {
                *actual = actual.wrapping_add(*add);
                *expected = expected.wrapping_add(*add);
            }
        }
        MapOp::ContainsKey(key) => {
            prop_assert_eq!(map.contains_key(key), model.contains_key(key));
        }
        MapOp::BumpAll => {
            let mut visited = Vec::new();
            for (key, value) in map.iter_mut() {
                *value = value.wrapping_add(1);
                visited.push(key.clone());
            }
            let mut expected = Vec::new();
            for (key, value) in &mut model.entries {
                *value = value.wrapping_add(1);
                expected.push(key.clone());
            }
            prop_assert_eq!(visited, expected);
        }
        MapOp::Clear => {
            map.clear();
            model.clear();
        }
    }
    assert_map_matches(map, model)
}

fn assert_map_matches<K: Hash + Eq + Clone + core::fmt::Debug>(
    map: &DetHashMap<K, u32>,
    model: &VecMap<K, u32>,
) -> Result<(), TestCaseError> {
    prop_assert_eq!(map.len(), model.entries.len());
    prop_assert_eq!(map.is_empty(), model.entries.is_empty());
    let entries: Vec<(K, u32)> = map
        .iter()
        .map(|(key, value)| (key.clone(), *value))
        .collect();
    prop_assert_eq!(&entries, &model.entries);
    let keys: Vec<K> = map.keys().cloned().collect();
    let values: Vec<u32> = map.values().copied().collect();
    prop_assert_eq!(
        keys,
        model
            .entries
            .iter()
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>()
    );
    prop_assert_eq!(
        values,
        model
            .entries
            .iter()
            .map(|(_, value)| *value)
            .collect::<Vec<_>>()
    );
    Ok(())
}

fn entries_of<K: Hash + Eq + Clone>(map: &DetHashMap<K, u32>) -> Vec<(K, u32)> {
    map.iter()
        .map(|(key, value)| (key.clone(), *value))
        .collect()
}

/// Verifies: CORE-R070
#[test]
fn iteration_is_in_first_insertion_order_and_insert_replaces_in_place() {
    let mut map: DetHashMap<&str, u32> = DetHashMap::new();
    assert_eq!(map.insert("delta", 4), None);
    assert_eq!(map.insert("alpha", 1), None);
    assert_eq!(map.insert("charlie", 3), None);
    assert_eq!(map.insert("bravo", 2), None);
    let order = ["delta", "alpha", "charlie", "bravo"];
    assert_eq!(map.keys().copied().collect::<Vec<_>>(), order);

    // A present key: the old value is returned, the position is kept.
    assert_eq!(map.insert("alpha", 10), Some(1));
    assert_eq!(map.insert("delta", 40), Some(4));
    assert_eq!(map.len(), 4);
    assert_eq!(
        map.iter()
            .map(|(key, value)| (*key, *value))
            .collect::<Vec<_>>(),
        [("delta", 40), ("alpha", 10), ("charlie", 3), ("bravo", 2)]
    );
    assert_eq!(map.keys().copied().collect::<Vec<_>>(), order);
    assert_eq!(map.values().copied().collect::<Vec<_>>(), [40, 10, 3, 2]);

    // `iter_mut` visits in the same order and its changes are stored.
    let mut visited = Vec::new();
    for (key, value) in map.iter_mut() {
        visited.push(*key);
        *value += 1;
    }
    assert_eq!(visited, order);
    assert_eq!(map.values().copied().collect::<Vec<_>>(), [41, 11, 4, 3]);
    assert_eq!(map.get("charlie"), Some(&4));
}

/// Verifies: CORE-R070
#[test]
fn a_replaced_entry_keeps_its_original_key() {
    // Keys that are equal but distinguishable: equality ignores the second field.
    #[derive(Debug, Clone, Copy)]
    struct Tagged(u8, char);
    impl PartialEq for Tagged {
        fn eq(&self, other: &Tagged) -> bool {
            self.0 == other.0
        }
    }
    impl Eq for Tagged {}
    impl Hash for Tagged {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.0.hash(state);
        }
    }

    let mut map: DetHashMap<Tagged, u32> = DetHashMap::new();
    map.insert(Tagged(1, 'a'), 10);
    assert_eq!(map.insert(Tagged(1, 'b'), 20), Some(10));
    let (key, value) = map.iter().next().unwrap();
    assert_eq!((key.1, *value), ('a', 20));
}

/// Verifies: CORE-R071
#[test]
fn remove_keeps_the_order_of_the_rest_and_reinsert_appends() {
    let mut map: DetHashMap<u32, &str> = DetHashMap::new();
    for (key, value) in [
        (5, "five"),
        (1, "one"),
        (9, "nine"),
        (3, "three"),
        (7, "seven"),
    ] {
        map.insert(key, value);
    }
    assert_eq!(map.remove(&9), Some("nine"));
    assert_eq!(map.keys().copied().collect::<Vec<_>>(), [5, 1, 3, 7]);
    assert_eq!(map.get(&9), None);
    assert!(!map.contains_key(&9));

    // Absent key: `None`, nothing changes.
    assert_eq!(map.remove(&9), None);
    assert_eq!(map.remove(&100), None);
    assert_eq!(
        map.iter()
            .map(|(key, value)| (*key, *value))
            .collect::<Vec<_>>(),
        [(5, "five"), (1, "one"), (3, "three"), (7, "seven")]
    );

    // First and last entries.
    assert_eq!(map.remove(&5), Some("five"));
    assert_eq!(map.remove(&7), Some("seven"));
    assert_eq!(map.keys().copied().collect::<Vec<_>>(), [1, 3]);

    // A removed key that is inserted again goes to the end.
    assert_eq!(map.insert(5, "five again"), None);
    assert_eq!(map.keys().copied().collect::<Vec<_>>(), [1, 3, 5]);
    for (key, value) in [(1, "one"), (3, "three"), (5, "five again")] {
        assert_eq!(map.get(&key), Some(&value));
    }

    map.clear();
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    assert_eq!(map.iter().count(), 0);
    assert_eq!(map.get(&1), None);
    // The map is usable after `clear`, and order starts over.
    map.insert(3, "c");
    map.insert(1, "a");
    assert_eq!(map.keys().copied().collect::<Vec<_>>(), [3, 1]);
}

/// Verifies: CORE-R072
#[test]
fn lookups_accept_borrowed_forms_of_the_key() {
    let mut map: DetHashMap<String, Vec<u8>> = DetHashMap::new();
    map.insert("north".to_string(), vec![1]);
    map.insert("south".to_string(), vec![2]);
    assert_eq!(map.get("north"), Some(&vec![1]));
    assert!(map.contains_key("south"));
    assert!(!map.contains_key("east"));
    map.get_mut("south").unwrap().push(3);
    assert_eq!(map.get(&"south".to_string()), Some(&vec![2, 3]));
    assert_eq!(map.remove("north"), Some(vec![1]));
    assert_eq!(map.remove("north"), None);
    assert_eq!(
        map.keys().map(String::as_str).collect::<Vec<_>>(),
        ["south"]
    );
}

/// Verifies: CORE-R072, CORE-R073
#[test]
fn a_large_map_keeps_every_entry_and_its_order_while_the_table_grows() {
    let mut map: DetHashMap<u64, u64> = DetHashMap::new();
    let mut model: Vec<(u64, u64)> = Vec::new();
    // Keys in a scrambled but fixed order.
    let key = |index: u64| index.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 40;
    for index in 0..5000 {
        assert_eq!(map.insert(key(index), index), None, "key {index}");
        model.push((key(index), index));
    }
    for index in (0..5000).step_by(7) {
        assert_eq!(map.remove(&key(index)), Some(index));
        model.retain(|(present, _)| *present != key(index));
    }
    assert_eq!(map.len(), model.len());
    assert_eq!(
        map.iter()
            .map(|(key, value)| (*key, *value))
            .collect::<Vec<_>>(),
        model
    );
    for index in 0..5000 {
        let expected = (index % 7 != 0).then_some(index);
        assert_eq!(map.get(&key(index)).copied(), expected, "key {index}");
    }
}

proptest! {
    /// Verifies: CORE-R070, CORE-R071, CORE-R072
    #[test]
    fn the_map_matches_the_linear_search_model(ops in map_ops(0_u8..24)) {
        let mut map: DetHashMap<u8, u32> = DetHashMap::new();
        let mut model: VecMap<u8, u32> = VecMap::new();
        for op in &ops {
            apply_and_compare(&mut map, &mut model, op)?;
        }
    }

    /// Verifies: CORE-R072, CORE-R073
    #[test]
    fn the_map_matches_the_model_when_every_key_has_the_same_hash(
        ops in map_ops((0_u8..24).prop_map(SameHash)),
    ) {
        let mut map: DetHashMap<SameHash, u32> = DetHashMap::new();
        let mut model: VecMap<SameHash, u32> = VecMap::new();
        for op in &ops {
            apply_and_compare(&mut map, &mut model, op)?;
        }
    }

    /// Verifies: CORE-R073
    #[test]
    fn iteration_order_does_not_depend_on_capacity(
        ops in map_ops(any::<u16>()),
        capacity in 0_usize..600,
    ) {
        let mut plain: DetHashMap<u16, u32> = DetHashMap::new();
        let mut reserved: DetHashMap<u16, u32> = DetHashMap::with_capacity(capacity);
        prop_assert!(reserved.is_empty());
        prop_assert_eq!(reserved.iter().count(), 0);
        let mut model: VecMap<u16, u32> = VecMap::new();
        for op in &ops {
            apply_and_compare(&mut reserved, &mut model.clone(), op)?;
            apply_and_compare(&mut plain, &mut model, op)?;
        }
        prop_assert_eq!(entries_of(&plain), entries_of(&reserved));
    }

    /// Verifies: CORE-R076
    #[test]
    fn a_default_map_behaves_like_a_new_map(ops in map_ops(0_u8..24)) {
        let mut default: DetHashMap<u8, u32> = DetHashMap::default();
        let mut new: DetHashMap<u8, u32> = DetHashMap::new();
        prop_assert_eq!(default.iter().count(), 0);
        let mut model: VecMap<u8, u32> = VecMap::new();
        for op in &ops {
            apply_and_compare(&mut new, &mut model.clone(), op)?;
            apply_and_compare(&mut default, &mut model, op)?;
            prop_assert_eq!(default.len(), new.len());
            prop_assert_eq!(entries_of(&default), entries_of(&new));
        }
    }
}

/// One step of a random operation sequence on a set.
#[derive(Clone, Debug)]
enum SetOp {
    Insert(u8),
    Remove(u8),
    Contains(u8),
    Clear,
}

fn set_ops() -> impl Strategy<Value = Vec<SetOp>> {
    let op = prop_oneof![
        8 => (0_u8..24).prop_map(SetOp::Insert),
        5 => (0_u8..24).prop_map(SetOp::Remove),
        3 => (0_u8..24).prop_map(SetOp::Contains),
        1 => Just(SetOp::Clear),
    ];
    proptest::collection::vec(op, 0..300)
}

fn apply_set_op(
    set: &mut DetHashSet<u8>,
    model: &mut VecSet<u8>,
    op: &SetOp,
) -> Result<(), TestCaseError> {
    match op {
        SetOp::Insert(value) => prop_assert_eq!(set.insert(*value), model.insert(*value)),
        SetOp::Remove(value) => prop_assert_eq!(set.remove(value), model.remove(value)),
        SetOp::Contains(value) => prop_assert_eq!(set.contains(value), model.contains(value)),
        SetOp::Clear => {
            set.clear();
            model.clear();
        }
    }
    prop_assert_eq!(set.len(), model.values.len());
    prop_assert_eq!(set.is_empty(), model.values.is_empty());
    prop_assert_eq!(
        set.iter().copied().collect::<Vec<_>>(),
        model.values.clone()
    );
    Ok(())
}

/// Verifies: CORE-R074
#[test]
fn the_set_iterates_in_first_insertion_order() {
    let mut set: DetHashSet<&str> = DetHashSet::new();
    assert!(set.insert("delta"));
    assert!(set.insert("alpha"));
    assert!(set.insert("charlie"));
    // Present: `false`, nothing changes.
    assert!(!set.insert("delta"));
    assert!(!set.insert("alpha"));
    assert_eq!(
        set.iter().copied().collect::<Vec<_>>(),
        ["delta", "alpha", "charlie"]
    );
    assert_eq!(set.len(), 3);
    assert!(set.contains("alpha"));
    assert!(!set.contains("bravo"));

    // Removal keeps the order of the rest; an absent value changes nothing.
    assert!(set.remove("alpha"));
    assert!(!set.remove("alpha"));
    assert!(!set.remove("bravo"));
    assert_eq!(
        set.iter().copied().collect::<Vec<_>>(),
        ["delta", "charlie"]
    );

    // A removed value that is inserted again goes to the end.
    assert!(set.insert("alpha"));
    assert_eq!(
        set.iter().copied().collect::<Vec<_>>(),
        ["delta", "charlie", "alpha"]
    );

    set.clear();
    assert!(set.is_empty());
    assert_eq!(set.iter().count(), 0);
    assert!(!set.contains("delta"));
    assert!(set.insert("delta"));
    assert_eq!(set.len(), 1);
}

/// Verifies: CORE-R074
#[test]
fn a_present_value_is_kept_when_an_equal_one_is_inserted() {
    // Values that are equal but distinguishable: equality ignores the second field.
    #[derive(Debug, Clone, Copy)]
    struct Tagged(u8, char);
    impl PartialEq for Tagged {
        fn eq(&self, other: &Tagged) -> bool {
            self.0 == other.0
        }
    }
    impl Eq for Tagged {}
    impl Hash for Tagged {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.0.hash(state);
        }
    }

    let mut set: DetHashSet<Tagged> = DetHashSet::new();
    assert!(set.insert(Tagged(1, 'a')));
    assert!(!set.insert(Tagged(1, 'b')));
    assert_eq!(set.iter().next().unwrap().1, 'a');
    assert_eq!(set.len(), 1);
}

proptest! {
    /// Verifies: CORE-R074
    #[test]
    fn the_set_matches_the_linear_search_model(ops in set_ops(), capacity in 0_usize..100) {
        let mut set: DetHashSet<u8> = DetHashSet::new();
        let mut reserved: DetHashSet<u8> = DetHashSet::with_capacity(capacity);
        let mut model: VecSet<u8> = VecSet::new();
        for op in &ops {
            apply_set_op(&mut reserved, &mut model.clone(), op)?;
            apply_set_op(&mut set, &mut model, op)?;
        }
        // CORE-R073 for the set: the capacity does not change the order.
        prop_assert_eq!(
            set.iter().copied().collect::<Vec<_>>(),
            reserved.iter().copied().collect::<Vec<_>>()
        );
    }

    /// Verifies: CORE-R076
    #[test]
    fn a_default_set_behaves_like_a_new_set(ops in set_ops()) {
        let mut default: DetHashSet<u8> = DetHashSet::default();
        let mut new: DetHashSet<u8> = DetHashSet::new();
        prop_assert_eq!(default.iter().count(), 0);
        let mut model: VecSet<u8> = VecSet::new();
        for op in &ops {
            apply_set_op(&mut new, &mut model.clone(), op)?;
            apply_set_op(&mut default, &mut model, op)?;
            prop_assert_eq!(default.len(), new.len());
            prop_assert_eq!(
                default.iter().copied().collect::<Vec<_>>(),
                new.iter().copied().collect::<Vec<_>>()
            );
        }
    }
}

/// Verifies: CORE-R076
#[test]
fn default_and_new_exist_for_types_without_any_trait() {
    // Case (a): this test fails to compile if `Default`, `new`, `len` or `is_empty` gains a
    // bound on `K`, `V` or `T`.
    let default_map: DetHashMap<Opaque, Opaque> = DetHashMap::default();
    let new_map: DetHashMap<Opaque, Opaque> = DetHashMap::new();
    assert_eq!((default_map.len(), new_map.len()), (0, 0));
    assert!(default_map.is_empty() && new_map.is_empty());

    let default_set: DetHashSet<Opaque> = DetHashSet::default();
    let new_set: DetHashSet<Opaque> = DetHashSet::new();
    assert_eq!((default_set.len(), new_set.len()), (0, 0));
    assert!(default_set.is_empty() && new_set.is_empty());
}

#[test]
fn debug_writes_the_entries_in_iteration_order() {
    let mut map: DetHashMap<&str, u32> = DetHashMap::new();
    map.insert("b", 2);
    map.insert("a", 1);
    assert_eq!(format!("{map:?}"), r#"{"b": 2, "a": 1}"#);
    assert_eq!(format!("{:?}", DetHashMap::<u8, u8>::new()), "{}");

    let mut set: DetHashSet<u32> = DetHashSet::new();
    set.insert(9);
    set.insert(4);
    assert_eq!(format!("{set:?}"), "{9, 4}");
    assert_eq!(format!("{:?}", DetHashSet::<u8>::default()), "{}");
}

#[test]
fn a_clone_has_the_same_entries_and_changes_independently() {
    let mut map: DetHashMap<u32, String> = DetHashMap::new();
    map.insert(2, "two".to_string());
    map.insert(1, "one".to_string());
    let mut clone = map.clone();
    assert_eq!(
        clone
            .iter()
            .map(|(key, value)| (*key, value.clone()))
            .collect::<Vec<_>>(),
        [(2, "two".to_string()), (1, "one".to_string())]
    );
    clone.insert(3, "three".to_string());
    clone.remove(&2);
    assert_eq!(map.keys().copied().collect::<Vec<_>>(), [2, 1]);
    assert_eq!(clone.keys().copied().collect::<Vec<_>>(), [1, 3]);
    assert_eq!(clone.get(&3).map(String::as_str), Some("three"));

    let mut set: DetHashSet<u32> = DetHashSet::new();
    set.insert(5);
    let mut set_clone = set.clone();
    set_clone.insert(6);
    assert_eq!(set.iter().copied().collect::<Vec<_>>(), [5]);
    assert_eq!(set_clone.iter().copied().collect::<Vec<_>>(), [5, 6]);
}
