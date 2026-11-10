//! Type paths, stable type IDs and descriptors through the public API (spec §10, REFL-R001 to
//! REFL-R005, REFL-R020). Oracles: `liminal_core::hash::StateHasher`, hand-written expected
//! paths and descriptors, and XXH64 literals from the independent Python implementation
//! `crates/liminal_core/tests/reference/kat.py` (`kat.py xxh64 <path>…`; `xxhsum -H64` was not
//! installed on the machine that recorded them).

#[path = "common/types.rs"]
mod types;

use liminal_core::hash::StateHasher;
use liminal_reflect::info::{
    FieldInfo, PrimitiveKind, Shape, StableTypeId, StructKind, TypeInfo, TypeRef, VariantInfo,
};
use liminal_reflect::value::Typed;
use proptest::prelude::*;
use types::{Marker, Mode, Pair, Sample};

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = StateHasher::new();
    hasher.write_bytes(bytes);
    hasher.finish()
}

proptest! {
    /// Verifies: REFL-R001
    #[test]
    fn of_path_is_xxh64_of_the_utf8_bytes(path in ".*") {
        prop_assert_eq!(StableTypeId::of_path(&path), StableTypeId::new(hash_bytes(path.as_bytes())));
    }

    /// Verifies: REFL-R001
    #[test]
    fn new_value_display_and_order(a in any::<u64>(), b in any::<u64>()) {
        prop_assert_eq!(StableTypeId::new(a).value(), a);
        let text = StableTypeId::new(a).to_string();
        prop_assert_eq!(text.len(), 16);
        prop_assert!(text.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)));
        prop_assert_eq!(u64::from_str_radix(&text, 16).unwrap(), a);
        prop_assert_eq!(StableTypeId::new(a).cmp(&StableTypeId::new(b)), a.cmp(&b));
    }
}

/// Checks REFL-R002 for one type and one value of it.
fn check_consistent<T: Typed>(value: &T) {
    let path = T::type_path();
    assert_eq!(T::stable_type_id(), StableTypeId::of_path(&path), "{path}");
    let info = T::type_info();
    assert_eq!(info.type_path, path);
    assert_eq!(info.id, T::stable_type_id());
    assert_eq!(value.reflect_type_info(), info);
    let reference = TypeRef::of::<T>();
    assert_eq!(reference.type_path, path);
    assert_eq!(reference.id, T::stable_type_id());
}

/// Verifies: REFL-R002
#[test]
fn every_impl_is_consistent_with_its_type_path() {
    check_consistent(&true);
    check_consistent(&1_u8);
    check_consistent(&1_u16);
    check_consistent(&1_u32);
    check_consistent(&1_u64);
    check_consistent(&1_u128);
    check_consistent(&1_usize);
    check_consistent(&1_i8);
    check_consistent(&1_i16);
    check_consistent(&1_i32);
    check_consistent(&1_i64);
    check_consistent(&1_i128);
    check_consistent(&1_isize);
    check_consistent(&1.0_f32);
    check_consistent(&1.0_f64);
    check_consistent(&'x');
    check_consistent(&"text".to_string());
    check_consistent(&vec![1_u8]);
    check_consistent(&[1_u8; 3]);
    check_consistent(&[0_u8; 0]);
    check_consistent(&Some(1_u8));
    check_consistent(&None::<String>);
    check_consistent(&vec![Some([1_i16; 3])]);
    // The hand-written test types.
    check_consistent(&Pair(1, None));
    check_consistent(&Marker);
    check_consistent(&Mode::Idle);
}

/// Checks the path and primitive shape of one primitive type (REFL-R003).
fn check_primitive<T: Typed>(path: &str, kind: PrimitiveKind) {
    assert_eq!(T::type_path(), path);
    assert_eq!(T::type_info().shape, Shape::Primitive(kind), "{path}");
}

/// Verifies: REFL-R003
#[test]
fn primitive_paths_and_shapes() {
    check_primitive::<bool>("bool", PrimitiveKind::Bool);
    check_primitive::<u8>("u8", PrimitiveKind::U8);
    check_primitive::<u16>("u16", PrimitiveKind::U16);
    check_primitive::<u32>("u32", PrimitiveKind::U32);
    check_primitive::<u64>("u64", PrimitiveKind::U64);
    check_primitive::<u128>("u128", PrimitiveKind::U128);
    check_primitive::<usize>("usize", PrimitiveKind::Usize);
    check_primitive::<i8>("i8", PrimitiveKind::I8);
    check_primitive::<i16>("i16", PrimitiveKind::I16);
    check_primitive::<i32>("i32", PrimitiveKind::I32);
    check_primitive::<i64>("i64", PrimitiveKind::I64);
    check_primitive::<i128>("i128", PrimitiveKind::I128);
    check_primitive::<isize>("isize", PrimitiveKind::Isize);
    check_primitive::<f32>("f32", PrimitiveKind::F32);
    check_primitive::<f64>("f64", PrimitiveKind::F64);
    check_primitive::<char>("char", PrimitiveKind::Char);
    check_primitive::<String>("String", PrimitiveKind::String);
}

/// Verifies: REFL-R004
#[test]
fn container_paths() {
    assert_eq!(<Vec<u8>>::type_path(), "Vec<u8>");
    assert_eq!(<Option<String>>::type_path(), "Option<String>");
    assert_eq!(<[f32; 4]>::type_path(), "[f32; 4]");
    assert_eq!(<[u8; 0]>::type_path(), "[u8; 0]");
    assert_eq!(<[u8; 100]>::type_path(), "[u8; 100]");
    assert_eq!(<Vec<Option<[u8; 4]>>>::type_path(), "Vec<Option<[u8; 4]>>");
    assert_eq!(
        <Option<Vec<Sample>>>::type_path(),
        "Option<Vec<refl_test::Sample>>"
    );
}

/// Known-answer literals, recorded with `kat.py xxh64` (see the module doc). The same literals
/// must hold on every CI platform.
///
/// Verifies: REFL-R005
#[test]
fn stable_type_id_known_answers() {
    let expected: [(&str, StableTypeId, u64); 8] = [
        ("bool", bool::stable_type_id(), 0x31A0_9F12_F681_5CBD),
        ("u8", u8::stable_type_id(), 0x9DDC_BAC7_4B6C_CEC3),
        ("f32", f32::stable_type_id(), 0xB964_B9AD_A17B_DE2C),
        ("String", String::stable_type_id(), 0x9B3A_C42F_3D57_4C27),
        (
            "Vec<u8>",
            <Vec<u8>>::stable_type_id(),
            0xE442_ABA4_B705_8D0C,
        ),
        (
            "Option<String>",
            <Option<String>>::stable_type_id(),
            0x04C4_3B8B_23B8_9008,
        ),
        (
            "[f32; 4]",
            <[f32; 4]>::stable_type_id(),
            0x4372_A5E7_FF62_5411,
        ),
        (
            "refl_test::Sample",
            Sample::stable_type_id(),
            0x6EFD_058C_59B8_8835,
        ),
    ];
    for (path, id, literal) in expected {
        assert_eq!(id.value(), literal, "{path}");
        assert_eq!(StableTypeId::of_path(path).value(), literal, "{path}");
    }
}

/// Verifies: REFL-R020
#[test]
fn container_shapes() {
    assert_eq!(
        <Vec<u16>>::type_info().shape,
        Shape::List {
            element: TypeRef::of::<u16>()
        }
    );
    assert_eq!(
        <[i8; 7]>::type_info().shape,
        Shape::Array {
            element: TypeRef::of::<i8>(),
            len: 7
        }
    );
    assert_eq!(
        <Option<String>>::type_info(),
        TypeInfo {
            type_path: "Option<String>".to_string(),
            id: StableTypeId::of_path("Option<String>"),
            shape: Shape::Enum {
                variants: vec![
                    VariantInfo {
                        name: "None",
                        kind: StructKind::Unit,
                        fields: vec![]
                    },
                    VariantInfo {
                        name: "Some",
                        kind: StructKind::Tuple,
                        fields: vec![FieldInfo {
                            name: "0",
                            ty: TypeRef {
                                type_path: "String".to_string(),
                                id: StableTypeId::of_path("String"),
                            },
                        }],
                    },
                ],
            },
        }
    );
}
