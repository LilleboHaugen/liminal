//! Integration tests of `liminal_core::handle` through the public API (spec §10, CORE-R030 to
//! CORE-R040). Oracle: the linear-search model in `common/handle_model.rs`. The generation wrap
//! (CORE-R035) needs private access and is tested inside the crate.

#[path = "common/handle_model.rs"]
mod handle_model;

use std::collections::BTreeSet;
use std::hash::{BuildHasher, BuildHasherDefault, DefaultHasher};

use handle_model::{Expected, HandleModel, handle};
use liminal_core::error::CoreError;
use liminal_core::handle::{Handle, HandleAllocator, HandleMap, RawHandle};
use proptest::prelude::*;

/// A type that implements no trait (CORE-R032).
struct Opaque;

/// A 1 KiB type (CORE-R030).
struct Large {
    _bytes: [u8; 1024],
}

/// One step of a random operation sequence.
#[derive(Clone, Debug)]
enum Op {
    Allocate,
    /// Free the live handle selected by the index (no-op if none is live).
    FreeLive(prop::sample::Index),
    /// Free a handle that was issued at some point: usually stale.
    FreeIssued(prop::sample::Index),
    /// Free an arbitrary handle value: usually never issued.
    FreeArbitrary {
        index: u32,
        generation: u32,
    },
}

fn ops() -> impl Strategy<Value = Vec<Op>> {
    let op = prop_oneof![
        5 => Just(Op::Allocate),
        3 => any::<prop::sample::Index>().prop_map(Op::FreeLive),
        1 => any::<prop::sample::Index>().prop_map(Op::FreeIssued),
        1 => (0_u32..8, 1_u32..5).prop_map(|(index, generation)| Op::FreeArbitrary { index, generation }),
    ];
    proptest::collection::vec(op, 0..200)
}

/// Small limits (exhaustion is common) and the default limit.
fn slot_limit() -> impl Strategy<Value = u32> {
    prop_oneof![0_u32..7, Just(u32::MAX)]
}

/// The handle an `Op` frees, if any.
fn target(op: &Op, model: &HandleModel) -> Option<RawHandle> {
    match op {
        Op::Allocate => None,
        Op::FreeLive(selector) => {
            (!model.live.is_empty()).then(|| model.live[selector.index(model.live.len())])
        }
        Op::FreeIssued(selector) => {
            (!model.issued.is_empty()).then(|| model.issued[selector.index(model.issued.len())])
        }
        Op::FreeArbitrary { index, generation } => Some(handle(*index, *generation)),
    }
}

fn stale(handle: RawHandle) -> CoreError {
    CoreError::StaleHandle {
        index: handle.index(),
        generation: handle.generation().get(),
    }
}

/// Every observable property of `allocator` equals the model's (CORE-R038).
fn assert_matches_model(
    allocator: &HandleAllocator,
    model: &HandleModel,
) -> Result<(), TestCaseError> {
    prop_assert_eq!(
        allocator.iter_live().collect::<Vec<_>>(),
        model.live_sorted()
    );
    prop_assert_eq!(allocator.live_count() as usize, model.live.len());
    for issued in &model.issued {
        prop_assert_eq!(
            allocator.is_live(*issued),
            model.is_live(*issued),
            "{}",
            issued
        );
    }
    // Handle values that were never issued are not live.
    for index in 0..8 {
        for generation in 1..5 {
            let probe = handle(index, generation);
            prop_assert_eq!(allocator.is_live(probe), model.is_live(probe), "{}", probe);
        }
    }
    Ok(())
}

proptest! {
    /// Verifies: CORE-R031
    #[test]
    fn bits_round_trip(bits in any::<u64>()) {
        match RawHandle::from_bits(bits) {
            None => prop_assert!(bits < (1 << 32)),
            Some(handle) => {
                prop_assert!(bits >= (1 << 32));
                prop_assert_eq!(u64::from(handle.index()), bits % (1 << 32));
                prop_assert_eq!(u64::from(handle.generation().get()), bits / (1 << 32));
                prop_assert_eq!(handle.to_bits(), bits);
                prop_assert_eq!(RawHandle::from_bits(handle.to_bits()), Some(handle));
                prop_assert_eq!(
                    handle.to_string(),
                    format!("{}v{}", bits % (1 << 32), bits / (1 << 32))
                );
            }
        }
    }

    /// Verifies: CORE-R032
    #[test]
    fn typed_handles_compare_like_their_raw_handles(a in (1_u64 << 32).., b in (1_u64 << 32)..) {
        let (raw_a, raw_b) = (RawHandle::from_bits(a).unwrap(), RawHandle::from_bits(b).unwrap());
        let (typed_a, typed_b) = (Handle::<Opaque>::from_raw(raw_a), Handle::<Opaque>::from_raw(raw_b));
        prop_assert_eq!(typed_a.raw(), raw_a);
        prop_assert_eq!(typed_a == typed_b, raw_a == raw_b);
        prop_assert_eq!(typed_a.cmp(&typed_b), raw_a.cmp(&raw_b));
        prop_assert_eq!(typed_a.partial_cmp(&typed_b), raw_a.partial_cmp(&raw_b));
        // Raw order: index first, then generation.
        prop_assert_eq!(
            raw_a.cmp(&raw_b),
            (raw_a.index(), raw_a.generation()).cmp(&(raw_b.index(), raw_b.generation()))
        );
        let hasher = BuildHasherDefault::<DefaultHasher>::default();
        prop_assert_eq!(hasher.hash_one(typed_a), hasher.hash_one(raw_a));
    }

    /// Verifies: CORE-R033, CORE-R034, CORE-R036, CORE-R037, CORE-R038
    #[test]
    fn allocator_follows_the_model(slot_limit in slot_limit(), ops in ops()) {
        let mut allocator = HandleAllocator::with_slot_limit(slot_limit);
        let mut model = HandleModel::new(slot_limit);
        for op in &ops {
            let before = format!("{allocator:?}");
            match target(op, &model) {
                None => match model.allocate() {
                    Expected::Handle(expected) => {
                        // The model asserts that `expected` was never issued before (CORE-R034).
                        prop_assert_eq!(allocator.allocate(), Ok(expected));
                        prop_assert!(expected.index() < slot_limit);
                    }
                    Expected::Exhausted => {
                        prop_assert_eq!(
                            allocator.allocate(),
                            Err(CoreError::HandleSlotsExhausted { slot_limit })
                        );
                        prop_assert_eq!(format!("{allocator:?}"), before, "changed by a failed allocate");
                    }
                },
                Some(handle) => {
                    if model.free(handle) {
                        prop_assert_eq!(allocator.free(handle), Ok(()));
                        prop_assert!(!allocator.is_live(handle));
                    } else {
                        prop_assert_eq!(allocator.free(handle), Err(stale(handle)));
                        prop_assert_eq!(format!("{allocator:?}"), before, "changed by a failed free");
                    }
                }
            }
            assert_matches_model(&allocator, &model)?;
        }
        // No handle value was issued twice.
        let distinct: BTreeSet<RawHandle> = model.issued.iter().copied().collect();
        prop_assert_eq!(distinct.len(), model.issued.len());
    }

    /// Verifies: CORE-R039
    #[test]
    fn map_follows_the_allocator_and_the_model(slot_limit in slot_limit(), ops in ops()) {
        let mut map: HandleMap<u64> = HandleMap::with_slot_limit(slot_limit);
        let mut allocator = HandleAllocator::with_slot_limit(slot_limit);
        let mut model = HandleModel::new(slot_limit);
        // The value stored for each handle ever issued.
        let mut values: Vec<(RawHandle, u64)> = Vec::new();
        let value_of = |values: &[(RawHandle, u64)], handle: RawHandle| {
            values.iter().find(|(issued, _)| *issued == handle).map(|(_, value)| *value)
        };

        for (step, op) in (0_u64..).zip(&ops) {
            let before = format!("{map:?}");
            match target(op, &model) {
                None => {
                    let value = step * 1000 + 7;
                    let inserted = map.insert(value).map(Handle::raw);
                    // Same handle sequence as the allocator, which follows the model.
                    prop_assert_eq!(&inserted, &allocator.allocate());
                    match model.allocate() {
                        Expected::Handle(expected) => {
                            prop_assert_eq!(inserted, Ok(expected));
                            values.push((expected, value));
                        }
                        Expected::Exhausted => {
                            prop_assert_eq!(inserted, Err(CoreError::HandleSlotsExhausted { slot_limit }));
                            prop_assert_eq!(format!("{map:?}"), before, "changed by a failed insert");
                        }
                    }
                }
                Some(raw) => {
                    let removed = map.remove(Handle::from_raw(raw));
                    let freed = allocator.free(raw);
                    if model.free(raw) {
                        prop_assert_eq!(removed, value_of(&values, raw));
                        prop_assert!(removed.is_some());
                        prop_assert_eq!(freed, Ok(()));
                    } else {
                        prop_assert_eq!(removed, None);
                        prop_assert_eq!(freed, Err(stale(raw)));
                        prop_assert_eq!(format!("{map:?}"), before, "changed by a failed remove");
                    }
                }
            }

            // Mutate one live value through `get_mut`, so stale reads would show old data.
            if let Some(raw) = model.live.first().copied() {
                let slot = map.get_mut(Handle::from_raw(raw)).unwrap();
                *slot += 1;
                values.iter_mut().find(|(issued, _)| *issued == raw).unwrap().1 += 1;
            }

            prop_assert_eq!(map.len() as usize, model.live.len());
            prop_assert_eq!(map.is_empty(), model.live.is_empty());
            let expected_entries: Vec<(RawHandle, u64)> = model
                .live_sorted()
                .into_iter()
                .map(|raw| (raw, value_of(&values, raw).unwrap()))
                .collect();
            let entries: Vec<(RawHandle, u64)> = map.iter().map(|(h, v)| (h.raw(), *v)).collect();
            prop_assert_eq!(&entries, &expected_entries);
            let entries_mut: Vec<(RawHandle, u64)> = map.iter_mut().map(|(h, v)| (h.raw(), *v)).collect();
            prop_assert_eq!(&entries_mut, &expected_entries);
            for (raw, value) in &values {
                let typed = Handle::<u64>::from_raw(*raw);
                let expected = model.is_live(*raw).then_some(*value);
                prop_assert_eq!(map.get(typed).copied(), expected, "{}", raw);
                prop_assert_eq!(map.get_mut(typed).copied(), expected, "{}", raw);
                prop_assert_eq!(map.contains(typed), expected.is_some(), "{}", raw);
            }
        }
    }
}

/// Verifies: CORE-R030
#[test]
fn handles_are_8_bytes_for_any_value_type() {
    assert_eq!(size_of::<RawHandle>(), 8);
    assert_eq!(size_of::<Option<RawHandle>>(), 8);
    assert_eq!(size_of::<Large>(), 1024);
    assert_eq!(size_of::<Opaque>(), 0);
    assert_eq!(size_of::<Handle<Opaque>>(), 8);
    assert_eq!(size_of::<Option<Handle<Opaque>>>(), 8);
    assert_eq!(size_of::<Handle<Large>>(), 8);
    assert_eq!(size_of::<Option<Handle<Large>>>(), 8);
}

/// Verifies: CORE-R032
#[test]
fn typed_handles_have_every_trait_without_bounds_on_the_value_type() {
    fn assert_traits<
        H: Copy + Clone + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash + std::fmt::Debug,
    >() {
    }
    assert_traits::<Handle<Opaque>>();
    assert_traits::<Handle<Large>>();
    assert_traits::<Handle<f32>>(); // `f32` is neither `Eq` nor `Ord` nor `Hash`
    assert_traits::<Handle<std::cell::Cell<u8>>>();

    let raw = handle(3, 2);
    let typed: Handle<Opaque> = Handle::from_raw(raw);
    let copy = typed;
    #[expect(
        clippy::clone_on_copy,
        reason = "the test calls `Clone::clone` on purpose"
    )]
    let clone = typed.clone();
    assert_eq!(typed.raw(), raw);
    assert_eq!(copy, typed);
    assert_eq!(clone, typed);
    assert_ne!(typed, Handle::from_raw(handle(3, 3)));
    assert!(typed < Handle::from_raw(handle(3, 3)));
    assert!(typed < Handle::from_raw(handle(4, 1)));
    assert!(typed > Handle::from_raw(handle(2, 9)));
    assert_eq!(format!("{typed:?}"), format!("Handle({raw:?})"));
}

/// Verifies: CORE-R033
#[test]
fn allocate_takes_ascending_indices_then_reuses_the_most_recently_freed_slot() {
    let mut allocator = HandleAllocator::new();
    let issued: Vec<RawHandle> = (0..5).map(|_| allocator.allocate().unwrap()).collect();
    assert_eq!(
        issued,
        (0..5).map(|index| handle(index, 1)).collect::<Vec<_>>()
    );

    // Freed in the order 1, 3, 0: reused in the order 0, 3, 1 (LIFO), each with generation 2.
    for index in [1, 3, 0] {
        allocator.free(issued[index]).unwrap();
    }
    assert_eq!(allocator.allocate(), Ok(handle(0, 2)));
    assert_eq!(allocator.allocate(), Ok(handle(3, 2)));
    // A free in between goes to the top of the stack.
    allocator.free(issued[4]).unwrap();
    assert_eq!(allocator.allocate(), Ok(handle(4, 2)));
    assert_eq!(allocator.allocate(), Ok(handle(1, 2)));
    // No free slot left: the lowest never-used index, generation 1.
    assert_eq!(allocator.allocate(), Ok(handle(5, 1)));
    assert_eq!(allocator.allocate(), Ok(handle(6, 1)));
}

/// Verifies: CORE-R034, CORE-R036
#[test]
fn free_makes_the_handle_stale_and_a_second_free_fails() {
    let mut allocator = HandleAllocator::new();
    let first = allocator.allocate().unwrap();
    let second = allocator.allocate().unwrap();
    assert_eq!(allocator.free(first), Ok(()));
    assert!(!allocator.is_live(first) && allocator.is_live(second));

    let before = format!("{allocator:?}");
    // Freed already; generation ahead of the slot's; generation behind; index never used.
    for not_live in [
        first,
        handle(0, 2),
        handle(1, 2),
        handle(1, 7),
        handle(2, 1),
        handle(u32::MAX, 1),
    ] {
        assert_eq!(allocator.free(not_live), Err(stale(not_live)), "{not_live}");
        assert_eq!(format!("{allocator:?}"), before, "{not_live}");
    }
    // The reused slot has the next generation; the old handle stays stale.
    let reused = allocator.allocate().unwrap();
    assert_eq!(reused, handle(0, 2));
    assert_eq!(allocator.free(first), Err(stale(first)));
    assert!(allocator.is_live(reused));
    assert_eq!(allocator.free(reused), Ok(()));
    assert_eq!(allocator.allocate(), Ok(handle(0, 3)));
}

/// Verifies: CORE-R037
#[test]
fn slot_limit_bounds_the_indices_and_exhaustion_changes_nothing() {
    let mut none = HandleAllocator::with_slot_limit(0);
    assert_eq!(
        none.allocate(),
        Err(CoreError::HandleSlotsExhausted { slot_limit: 0 })
    );
    assert_eq!(none.live_count(), 0);

    let mut allocator = HandleAllocator::with_slot_limit(3);
    let issued: Vec<RawHandle> = (0..3).map(|_| allocator.allocate().unwrap()).collect();
    assert_eq!(issued, [handle(0, 1), handle(1, 1), handle(2, 1)]);
    let before = format!("{allocator:?}");
    for _ in 0..3 {
        assert_eq!(
            allocator.allocate(),
            Err(CoreError::HandleSlotsExhausted { slot_limit: 3 })
        );
        assert_eq!(format!("{allocator:?}"), before);
    }
    assert_eq!(allocator.live_count(), 3);
    // A freed slot is available again; a fourth index never is.
    allocator.free(issued[1]).unwrap();
    assert_eq!(allocator.allocate(), Ok(handle(1, 2)));
    assert_eq!(
        allocator.allocate(),
        Err(CoreError::HandleSlotsExhausted { slot_limit: 3 })
    );

    // `new()` and `default()` have the limit `u32::MAX`: reported when a stale free … cannot
    // show it, so compare the `Debug` form with an explicit `with_slot_limit(u32::MAX)`.
    let explicit = format!("{:?}", HandleAllocator::with_slot_limit(u32::MAX));
    assert_eq!(format!("{:?}", HandleAllocator::new()), explicit);
    assert_eq!(format!("{:?}", HandleAllocator::default()), explicit);
    assert_ne!(
        format!("{:?}", HandleAllocator::with_slot_limit(u32::MAX - 1)),
        explicit
    );
    assert!(explicit.contains("4294967295"), "{explicit}");
}

/// Verifies: CORE-R038
#[test]
fn live_count_and_iter_live_track_allocations_and_frees() {
    let mut allocator = HandleAllocator::new();
    assert_eq!(allocator.live_count(), 0);
    assert_eq!(allocator.iter_live().count(), 0);
    assert!(!allocator.is_live(handle(0, 1)));

    let issued: Vec<RawHandle> = (0..4).map(|_| allocator.allocate().unwrap()).collect();
    assert_eq!(allocator.live_count(), 4);
    assert_eq!(allocator.iter_live().collect::<Vec<_>>(), issued);

    allocator.free(issued[2]).unwrap();
    allocator.free(issued[0]).unwrap();
    assert_eq!(allocator.live_count(), 2);
    assert_eq!(
        allocator.iter_live().collect::<Vec<_>>(),
        [issued[1], issued[3]]
    );

    // Reused slots appear at their index position, not in allocation order.
    let reused = allocator.allocate().unwrap();
    assert_eq!(reused, handle(0, 2));
    assert_eq!(
        allocator.iter_live().collect::<Vec<_>>(),
        [reused, issued[1], issued[3]]
    );
    assert_eq!(allocator.live_count(), 3);

    // A clone is independent.
    let mut clone = allocator.clone();
    clone.free(reused).unwrap();
    assert!(allocator.is_live(reused) && !clone.is_live(reused));
    assert_eq!((allocator.live_count(), clone.live_count()), (3, 2));
}

/// Verifies: CORE-R039
#[test]
fn map_stores_one_value_per_live_handle() {
    let mut map: HandleMap<String> = HandleMap::default();
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    assert_eq!(map.iter().count(), 0);

    let a = map.insert("a".to_string()).unwrap();
    let b = map.insert("b".to_string()).unwrap();
    let c = map.insert("c".to_string()).unwrap();
    assert_eq!(
        [a.raw(), b.raw(), c.raw()],
        [handle(0, 1), handle(1, 1), handle(2, 1)]
    );
    assert_eq!(map.len(), 3);
    assert!(!map.is_empty());
    assert_eq!(map.get(b).map(String::as_str), Some("b"));
    assert!(map.contains(b));

    map.get_mut(b).unwrap().push('!');
    for (_, value) in map.iter_mut() {
        value.push('?');
    }
    assert_eq!(map.remove(b).as_deref(), Some("b!?"));

    // Stale: every access reports absence and changes nothing.
    let before = format!("{map:?}");
    assert_eq!(map.remove(b), None);
    assert_eq!(map.get(b), None);
    assert_eq!(map.get_mut(b), None);
    assert!(!map.contains(b));
    let never_issued: Handle<String> = Handle::from_raw(handle(9, 1));
    assert_eq!(map.remove(never_issued), None);
    assert_eq!(map.get(never_issued), None);
    assert_eq!(format!("{map:?}"), before);
    assert_eq!(map.len(), 2);

    // The slot is reused with generation 2; the stale handle does not reach the new value.
    let d = map.insert("d".to_string()).unwrap();
    assert_eq!(d.raw(), handle(1, 2));
    assert_eq!(map.get(b), None);
    assert_eq!(map.get(d).map(String::as_str), Some("d"));
    let entries: Vec<(Handle<String>, &str)> = map.iter().map(|(h, v)| (h, v.as_str())).collect();
    assert_eq!(entries, [(a, "a?"), (d, "d"), (c, "c?")]);

    // A clone is independent.
    let mut clone = map.clone();
    assert_eq!(clone.remove(a).as_deref(), Some("a?"));
    assert!(map.contains(a) && !clone.contains(a));
}

/// Verifies: CORE-R039
#[test]
fn map_insert_on_exhaustion_drops_the_value_and_changes_nothing() {
    let marker = std::rc::Rc::new(());
    let mut map: HandleMap<std::rc::Rc<()>> = HandleMap::with_slot_limit(1);
    let kept = map.insert(marker.clone()).unwrap();
    assert_eq!(std::rc::Rc::strong_count(&marker), 2);

    let before = format!("{map:?}");
    assert_eq!(
        map.insert(marker.clone()),
        Err(CoreError::HandleSlotsExhausted { slot_limit: 1 })
    );
    assert_eq!(
        std::rc::Rc::strong_count(&marker),
        2,
        "the rejected value was not dropped"
    );
    assert_eq!(format!("{map:?}"), before);
    assert_eq!(map.len(), 1);

    assert!(map.remove(kept).is_some());
    assert_eq!(std::rc::Rc::strong_count(&marker), 1);
    let mut zero: HandleMap<u8> = HandleMap::with_slot_limit(0);
    assert_eq!(
        zero.insert(1),
        Err(CoreError::HandleSlotsExhausted { slot_limit: 0 })
    );
}

/// CORE-R040 is a SHOULD that the spec checks by review. This test has no timing assertion
/// (TESTING.md forbids them); it only makes a per-operation scan over slots or live handles
/// unusable: with 200,000 slots, 800,000 operations would then visit about 10^11 slots and the
/// test would run for minutes instead of well under a second.
///
/// Verifies: CORE-R040
#[test]
fn operations_on_many_slots_do_not_scan() {
    const SLOTS: u32 = 200_000;
    let mut allocator = HandleAllocator::new();
    let mut map: HandleMap<u32> = HandleMap::new();
    let raws: Vec<RawHandle> = (0..SLOTS).map(|_| allocator.allocate().unwrap()).collect();
    let typed: Vec<Handle<u32>> = (0..SLOTS).map(|value| map.insert(value).unwrap()).collect();

    let mut checksum = 0_u64;
    for round in 0..2_u32 {
        for (raw, handle) in raws.iter().zip(&typed) {
            // After the first round every original handle is stale; the calls stay O(1).
            checksum += u64::from(allocator.is_live(*raw));
            checksum += u64::from(map.contains(*handle));
            checksum += u64::from(map.get(*handle).is_some());
            checksum += u64::from(map.get_mut(*handle).is_some());
            if round == 0 {
                allocator.free(*raw).unwrap();
                allocator.allocate().unwrap();
                let value = map.remove(*handle).unwrap();
                map.insert(value).unwrap();
            }
        }
    }
    assert_eq!(checksum, 4 * u64::from(SLOTS));
    assert_eq!(allocator.live_count(), SLOTS);
    assert_eq!(map.len(), SLOTS);
}
