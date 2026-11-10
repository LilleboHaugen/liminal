//! The canonical state encoding through the public API (spec §10, REFL-R100 to REFL-R104).
//! Oracles: byte sequences written by hand and hashed with `StateHasher::write_bytes`, and
//! `liminal_core::hash::state_hash_of` (REFL-R102).

#[path = "common/types.rs"]
mod types;

use liminal_core::hash::{StateHasher, state_hash_of};
use liminal_reflect::hash::{reflect_state_hash, write_reflect};
use liminal_reflect::value::Reflect;
use proptest::prelude::*;
use types::{Fake, Marker, Mode, Pair, Sample};

/// XXH64 (seed 0) of the concatenated byte groups.
fn hash_of_bytes(groups: &[&[u8]]) -> u64 {
    let mut hasher = StateHasher::new();
    for group in groups {
        hasher.write_bytes(group);
    }
    hasher.finish()
}

/// Verifies: REFL-R100
#[test]
fn struct_enum_list_and_primitive_bytes() {
    let sample = Sample {
        x: 1.5,
        count: 0x0102,
        name: "hé".to_string(),
        tags: vec![7, 8],
        limit: Some(-2),
    };
    let expected = hash_of_bytes(&[
        &1.5_f32.to_bits().to_le_bytes(),            // x
        &[0x02, 0x01, 0, 0, 0, 0, 0, 0],             // count: u64 little-endian
        &[3, 0, 0, 0, 0, 0, 0, 0, b'h', 0xC3, 0xA9], // name: byte length 3, UTF-8 bytes
        &[2, 0, 0, 0, 0, 0, 0, 0, 7, 8],             // tags: length, elements
        &[1, 0, 0, 0, 0xFE, 0xFF],                   // limit: variant 1 as u32, then -2 as i16
    ]);
    assert_eq!(reflect_state_hash(&sample), expected);

    // Enum: variant index as u32, then the variant's fields; unit variants write the index only.
    let mode = Mode::Fleeing {
        from: 7,
        speed: -0.0,
    };
    let expected = hash_of_bytes(&[
        &[2, 0, 0, 0],
        &[7, 0, 0, 0],
        &(-0.0_f32).to_bits().to_le_bytes(),
    ]);
    assert_eq!(reflect_state_hash(&mode), expected);
    assert_eq!(
        reflect_state_hash(&Mode::Idle),
        hash_of_bytes(&[&[0, 0, 0, 0]])
    );
    // Unit struct: nothing.
    assert_eq!(reflect_state_hash(&Marker), StateHasher::new().finish());
    // Tuple struct, arrays (with length prefix), bool, char, usize, isize, u128.
    assert_eq!(
        reflect_state_hash(&Pair(-1, None)),
        hash_of_bytes(&[&[0xFF; 4], &[0, 0, 0, 0]])
    );
    assert_eq!(
        reflect_state_hash(&[true, false]),
        hash_of_bytes(&[&[2, 0, 0, 0, 0, 0, 0, 0], &[1, 0]])
    );
    assert_eq!(reflect_state_hash(&'é'), hash_of_bytes(&[&[0xE9, 0, 0, 0]]));
    assert_eq!(
        reflect_state_hash(&5_usize),
        hash_of_bytes(&[&[5, 0, 0, 0, 0, 0, 0, 0]])
    );
    assert_eq!(reflect_state_hash(&-1_isize), hash_of_bytes(&[&[0xFF; 8]]));
    assert_eq!(
        reflect_state_hash(&1_u128),
        hash_of_bytes(&[&[1], &[0; 15]])
    );
}

/// The two deliberately wrong impls of the spec: nothing is written for a missing index, the
/// count and length prefixes are written as reported.
///
/// Verifies: REFL-R100
#[test]
fn missing_fields_and_elements_write_nothing() {
    let fake = Fake::named(&["x", "y"], &[0x0A]);
    assert_eq!(
        reflect_state_hash(&fake),
        hash_of_bytes(&[&[0x0A, 0, 0, 0]])
    );

    let mut fake = Fake::list(&[1, 2, 3]);
    fake.values[1] = None;
    assert_eq!(
        reflect_state_hash(&fake),
        hash_of_bytes(&[&[3, 0, 0, 0, 0, 0, 0, 0], &[1, 0, 0, 0], &[3, 0, 0, 0]])
    );

    // An enum with an out-of-range variant index writes it as reported.
    let mut fake = Fake::enumeration(9);
    fake.variant_index = 5;
    assert_eq!(
        reflect_state_hash(&fake),
        hash_of_bytes(&[&[5, 0, 0, 0], &[9, 0, 0, 0]])
    );
}

/// Verifies: REFL-R101
#[test]
fn reflect_state_hash_is_write_reflect_into_a_new_hasher() {
    let value = vec![Some([1_i16, 2, 3]), None];
    let mut hasher = StateHasher::new();
    write_reflect(&value, &mut hasher);
    assert_eq!(reflect_state_hash(&value), hasher.finish());

    // `write_reflect` appends to what the hasher already holds.
    let mut appended = StateHasher::new();
    appended.write_u8(1);
    write_reflect(&2_u8, &mut appended);
    assert_eq!(appended.finish(), hash_of_bytes(&[&[1, 2]]));
}

proptest! {
    /// Verifies: REFL-R102
    #[test]
    fn primitives_and_lists_match_core_state_hash(
        b in any::<bool>(), u in any::<(u8, u16, u32, u64, u128, usize)>(),
        i in any::<(i8, i16, i32, i64, i128, isize)>(), f in any::<(f32, f64)>(),
        c in any::<char>(), s in ".*", list in prop::collection::vec(any::<u32>(), 0..8),
        floats in prop::collection::vec(any::<f64>(), 0..8), array in any::<[i16; 3]>(),
        strings in prop::collection::vec(".*", 0..4),
    ) {
        prop_assert_eq!(reflect_state_hash(&b), state_hash_of(&b));
        prop_assert_eq!(reflect_state_hash(&u.0), state_hash_of(&u.0));
        prop_assert_eq!(reflect_state_hash(&u.1), state_hash_of(&u.1));
        prop_assert_eq!(reflect_state_hash(&u.2), state_hash_of(&u.2));
        prop_assert_eq!(reflect_state_hash(&u.3), state_hash_of(&u.3));
        prop_assert_eq!(reflect_state_hash(&u.4), state_hash_of(&u.4));
        prop_assert_eq!(reflect_state_hash(&u.5), state_hash_of(&u.5));
        prop_assert_eq!(reflect_state_hash(&i.0), state_hash_of(&i.0));
        prop_assert_eq!(reflect_state_hash(&i.1), state_hash_of(&i.1));
        prop_assert_eq!(reflect_state_hash(&i.2), state_hash_of(&i.2));
        prop_assert_eq!(reflect_state_hash(&i.3), state_hash_of(&i.3));
        prop_assert_eq!(reflect_state_hash(&i.4), state_hash_of(&i.4));
        prop_assert_eq!(reflect_state_hash(&i.5), state_hash_of(&i.5));
        prop_assert_eq!(reflect_state_hash(&f.0), state_hash_of(&f.0));
        prop_assert_eq!(reflect_state_hash(&f.1), state_hash_of(&f.1));
        prop_assert_eq!(reflect_state_hash(&c), state_hash_of(&c));
        prop_assert_eq!(reflect_state_hash(&s), state_hash_of(&s));
        prop_assert_eq!(reflect_state_hash(&list), state_hash_of(&list));
        prop_assert_eq!(reflect_state_hash(&floats), state_hash_of(&floats));
        prop_assert_eq!(reflect_state_hash(&array), state_hash_of(&array));
        prop_assert_eq!(reflect_state_hash(&strings), state_hash_of(&strings));
    }
}

/// `Option` is the exception: 4-byte variant index here, 1-byte tag in `liminal_core`.
///
/// Verifies: REFL-R102
#[test]
fn option_differs_from_core_state_hash() {
    assert_eq!(
        reflect_state_hash(&Some(7_u8)),
        hash_of_bytes(&[&[1, 0, 0, 0, 7]])
    );
    assert_eq!(state_hash_of(&Some(7_u8)), hash_of_bytes(&[&[1, 7]]));
}

/// Values compared by their reflected content (float bits, not float `==`).
fn pair_strategy() -> impl Strategy<Value = Pair> {
    (any::<i32>(), any::<Option<u8>>()).prop_map(|(a, b)| Pair(a, b))
}

proptest! {
    /// Verifies: REFL-R103
    #[test]
    fn different_content_gives_different_hashes(
        a in prop::collection::vec(any::<Option<[i16; 3]>>(), 0..4),
        b in prop::collection::vec(any::<Option<[i16; 3]>>(), 0..4),
        c in any::<Option<Vec<String>>>(), d in any::<Option<Vec<String>>>(),
        e in pair_strategy(), f in pair_strategy(),
    ) {
        // Integer and string content: `==` is equality of reflected content.
        prop_assert_eq!(a == b, reflect_state_hash(&a) == reflect_state_hash(&b));
        prop_assert_eq!(c == d, reflect_state_hash(&c) == reflect_state_hash(&d));
        prop_assert_eq!(e == f, reflect_state_hash(&e) == reflect_state_hash(&f));
    }

    /// Verifies: REFL-R103
    #[test]
    fn neighbouring_values_differ(value in prop::collection::vec(any::<Option<[i16; 3]>>(), 1..4), at in 0_usize..4) {
        let at = at % value.len();
        let mut changed = value.clone();
        changed[at] = match changed[at] {
            None => Some([0; 3]),
            Some([x, y, z]) => Some([x ^ 1, y, z]),
        };
        prop_assert_ne!(reflect_state_hash(&value), reflect_state_hash(&changed));
        let mut longer = value.clone();
        longer.push(None);
        prop_assert_ne!(reflect_state_hash(&value), reflect_state_hash(&longer));
    }
}

/// Verifies: REFL-R103
#[test]
fn equal_content_gives_equal_hashes_regardless_of_capacity() {
    let mut roomy: Vec<Option<Vec<String>>> = Vec::with_capacity(64);
    roomy.push(Some(vec!["a".to_string()]));
    let tight = vec![Some(vec!["a".to_string()])];
    assert_ne!(roomy.capacity(), tight.capacity());
    assert_eq!(reflect_state_hash(&roomy), reflect_state_hash(&tight));
    // NaN payloads and the sign of zero are content.
    assert_ne!(reflect_state_hash(&0.0_f32), reflect_state_hash(&-0.0_f32));
    assert_ne!(
        reflect_state_hash(&f64::from_bits(0x7FF8_0000_0000_0001)),
        reflect_state_hash(&f64::from_bits(0x7FF8_0000_0000_0002))
    );
}

/// Verifies: REFL-R104
#[test]
fn the_hash_does_not_depend_on_the_descriptor() {
    let original = Fake::named(&["x", "y"], &[1, 2]);
    let mut renamed = Fake::named(&["x", "y"], &[1, 2]);
    renamed.info.type_path = "refl_test::Elsewhere".to_string();
    renamed.info.id = liminal_reflect::info::StableTypeId::of_path("refl_test::Elsewhere");
    assert_eq!(reflect_state_hash(&original), reflect_state_hash(&renamed));
    assert_eq!(original.info_calls.get(), 0);
    assert_eq!(renamed.info_calls.get(), 0);

    // Reading only: the value is unchanged afterwards.
    let value: Vec<Option<String>> = vec![Some("s".to_string()), None];
    let before = value.clone();
    let r: &dyn Reflect = &value;
    let _ = reflect_state_hash(r);
    assert_eq!(value, before);
}
