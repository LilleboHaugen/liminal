//! `validate_reflect` through the public API (spec §10, REFL-R021, REFL-R080). Oracles:
//! hand-written expected `at`, `expected` and `actual` for one deliberately wrong impl per
//! check (`types::Fake`).

#[path = "common/types.rs"]
mod types;

use liminal_reflect::error::ReflectError;
use liminal_reflect::info::{PrimitiveKind, Shape, StableTypeId, TypeRef};
use liminal_reflect::value::{Typed, validate_reflect};
use proptest::prelude::*;
use types::{Fake, Marker, Mode, Pair, Sample, View, field, info};

fn mismatch(type_path: &str, at: &str, expected: &str, actual: &str) -> Result<(), ReflectError> {
    Err(ReflectError::ShapeMismatch {
        type_path: type_path.to_string(),
        at: at.to_string(),
        expected: expected.to_string(),
        actual: actual.to_string(),
    })
}

fn check_all<T: Typed>(values: &[T]) -> Result<(), TestCaseError> {
    for value in values {
        prop_assert_eq!(validate_reflect(value), Ok(()), "{}", T::type_path());
    }
    Ok(())
}

proptest! {
    /// Verifies: REFL-R021
    #[test]
    fn every_impl_of_this_crate_validates(
        nested in prop::collection::vec(any::<Option<[i16; 3]>>(), 0..6),
        strings in any::<Option<Vec<String>>>(),
        b in any::<bool>(), u in any::<(u8, u16, u32, u64, u128, usize)>(),
        i in any::<(i8, i16, i32, i64, i128, isize)>(), f in any::<(f32, f64)>(),
        c in any::<char>(), s in ".*",
    ) {
        prop_assert_eq!(validate_reflect(&nested), Ok(()));
        prop_assert_eq!(validate_reflect(&strings), Ok(()));
        check_all(&[b])?;
        check_all(&[u.0])?; check_all(&[u.1])?; check_all(&[u.2])?;
        check_all(&[u.3])?; check_all(&[u.4])?; check_all(&[u.5])?;
        check_all(&[i.0])?; check_all(&[i.1])?; check_all(&[i.2])?;
        check_all(&[i.3])?; check_all(&[i.4])?; check_all(&[i.5])?;
        check_all(&[f.0])?; check_all(&[f.1])?; check_all(&[c])?; check_all(&[s])?;
    }
}

/// Verifies: REFL-R021
#[test]
fn empty_lists_and_test_types_validate() {
    assert_eq!(validate_reflect(&Vec::<Option<[i16; 3]>>::new()), Ok(()));
    assert_eq!(validate_reflect(&Some(Vec::<String>::new())), Ok(()));
    assert_eq!(validate_reflect(&[0_u8; 0]), Ok(()));
    let sample = Sample {
        x: 1.0,
        count: 2,
        name: "n".to_string(),
        tags: vec![1],
        limit: None,
    };
    assert_eq!(validate_reflect(&sample), Ok(()));
    assert_eq!(validate_reflect(&Pair(1, Some(2))), Ok(()));
    assert_eq!(validate_reflect(&Marker), Ok(()));
    for mode in [
        Mode::Idle,
        Mode::Moving(1.0, 2.0),
        Mode::Fleeing {
            from: 1,
            speed: 0.5,
        },
    ] {
        assert_eq!(validate_reflect(&mode), Ok(()));
    }
    assert_eq!(validate_reflect(&Fake::named(&["x", "y"], &[1, 2])), Ok(()));
    assert_eq!(validate_reflect(&Fake::enumeration(1)), Ok(()));
    assert_eq!(validate_reflect(&Fake::list(&[1, 2])), Ok(()));
    assert_eq!(validate_reflect(&Fake::primitive(1)), Ok(()));
}

/// Verifies: REFL-R080
#[test]
fn descriptor_checks_come_first() {
    let mut fake = Fake::named(&["x", "x"], &[1, 2]);
    assert!(
        matches!(validate_reflect(&fake), Err(ReflectError::DuplicateName { name, .. }) if name == "x")
    );
    fake.info.id = StableTypeId::new(7);
    assert!(matches!(
        validate_reflect(&fake),
        Err(ReflectError::TypeIdMismatch { .. })
    ));
    fake.info.type_path = String::new();
    assert_eq!(validate_reflect(&fake), Err(ReflectError::EmptyTypePath));
}

/// Verifies: REFL-R080
#[test]
fn struct_view_violations_at_the_root() {
    let path = "refl_test::Fake";
    let mut fake = Fake::named(&["x", "y"], &[1, 2, 3]);
    fake.count = 3;
    assert_eq!(
        validate_reflect(&fake),
        mismatch(
            path,
            "(root)",
            "struct with 2 fields",
            "struct with 3 fields"
        )
    );

    let mut fake = Fake::named(&["x", "y"], &[1, 2]);
    fake.names[1] = Some("z");
    assert_eq!(
        validate_reflect(&fake),
        mismatch(path, "(root)", "field 1 named y", "field 1 named z")
    );
    fake.names[1] = None;
    assert_eq!(
        validate_reflect(&fake),
        mismatch(path, "(root)", "field 1 named y", "field 1 without a name")
    );

    // The deliberately wrong impl of REFL-R043: counted, named, but no value.
    let fake = Fake::named(&["x", "y"], &[1]);
    assert_eq!(
        validate_reflect(&fake),
        mismatch(path, "(root)", "field y", "no value")
    );

    // The descriptor says field `x` is a `u64`; the value is a `u32`.
    let mut fake = Fake::named(&["x"], &[1]);
    fake.info = info(
        path,
        Shape::Struct {
            kind: liminal_reflect::info::StructKind::Named,
            fields: vec![field::<u64>("x")],
        },
    );
    assert_eq!(
        validate_reflect(&fake),
        mismatch(
            path,
            "(root)",
            &format!("field x of type `u64` (ID {})", u64::stable_type_id()),
            &format!("type `u32` (ID {})", u32::stable_type_id()),
        )
    );
}

/// Verifies: REFL-R080
#[test]
fn shape_and_kind_violations() {
    let mut fake = Fake::primitive(1);
    fake.info.shape = Shape::Primitive(PrimitiveKind::U16);
    assert_eq!(
        validate_reflect(&fake),
        mismatch("refl_test::FakeU8", "(root)", "u16", "u8")
    );

    let mut fake = Fake::named(&["x", "y"], &[1, 2]);
    fake.view = View::List;
    assert_eq!(
        validate_reflect(&fake),
        mismatch("refl_test::Fake", "(root)", "struct with 2 fields", "list")
    );
    let mut fake = Fake::list(&[1]);
    fake.view = View::Primitive;
    assert_eq!(
        validate_reflect(&fake),
        mismatch("refl_test::FakeList", "(root)", "list", "u8")
    );
    fake.view = View::Enum;
    assert_eq!(
        validate_reflect(&fake),
        mismatch("refl_test::FakeList", "(root)", "list", "enum")
    );
    fake.view = View::Struct;
    fake.info.shape = Shape::Enum {
        variants: Vec::new(),
    };
    assert_eq!(
        validate_reflect(&fake),
        mismatch(
            "refl_test::FakeList",
            "(root)",
            "enum with 0 variants",
            "struct with 1 fields"
        )
    );
    fake.info.shape = Shape::Array {
        element: TypeRef::of::<u32>(),
        len: 1,
    };
    fake.view = View::Struct;
    assert_eq!(
        validate_reflect(&fake),
        mismatch(
            "refl_test::FakeList",
            "(root)",
            "array of length 1",
            "struct with 1 fields"
        )
    );
}

/// Verifies: REFL-R080
#[test]
fn enum_view_violations() {
    let path = "refl_test::FakeEnum";
    let mut fake = Fake::enumeration(1);
    fake.variant_index = 2;
    assert_eq!(
        validate_reflect(&fake),
        mismatch(path, "(root)", "variant index below 2", "variant index 2")
    );
    fake.variant_index = 1;
    assert_eq!(
        validate_reflect(&fake),
        mismatch(path, "(root)", "variant B", "variant A")
    );
    fake.variant_name = "B";
    assert_eq!(
        validate_reflect(&fake),
        mismatch(path, "(root)", "field 0 named y", "field 0 named x")
    );
    fake.names = vec![Some("y")];
    assert_eq!(validate_reflect(&fake), Ok(()));
    fake.count = 0;
    assert_eq!(
        validate_reflect(&fake),
        mismatch(
            path,
            "(root)",
            "variant with 1 fields",
            "variant with 0 fields"
        )
    );
    fake.count = 1;
    fake.values = vec![None];
    assert_eq!(
        validate_reflect(&fake),
        mismatch(path, "(root)", "field y", "no value")
    );
}

/// Verifies: REFL-R080
#[test]
fn list_and_array_violations() {
    let path = "refl_test::FakeList";
    // The deliberately wrong list impl of REFL-R100: `len() == 3`, `get(1) == None`.
    let mut fake = Fake::list(&[1, 2, 3]);
    fake.values[1] = None;
    assert_eq!(
        validate_reflect(&fake),
        mismatch(path, "(root)", "element 1", "no value")
    );

    let mut fake = Fake::list(&[1, 2, 3]);
    fake.info.shape = Shape::Array {
        element: TypeRef::of::<u32>(),
        len: 4,
    };
    assert_eq!(
        validate_reflect(&fake),
        mismatch(path, "(root)", "array of length 4", "list of length 3")
    );
    fake.info.shape = Shape::Array {
        element: TypeRef::of::<u32>(),
        len: 3,
    };
    assert_eq!(validate_reflect(&fake), Ok(()));
    fake.info.shape = Shape::List {
        element: TypeRef::of::<i32>(),
    };
    assert_eq!(
        validate_reflect(&fake),
        mismatch(
            path,
            "(root)",
            &format!("element 0 of type `i32` (ID {})", i32::stable_type_id()),
            &format!("type `u32` (ID {})", u32::stable_type_id()),
        )
    );
}

/// Verifies: REFL-R080
#[test]
fn nested_violations_report_their_location_depth_first() {
    let broken = || Fake::named(&["x", "y"], &[1]);
    let good = || Fake::named(&["x", "y"], &[1, 2]);
    let value = vec![Some(good()), None, Some(broken()), Some(broken())];
    assert_eq!(
        validate_reflect(&value),
        mismatch("refl_test::Fake", "[2].0", "field y", "no value")
    );
    let value = [vec![good()], vec![good(), broken()]];
    assert_eq!(
        validate_reflect(&value),
        mismatch("refl_test::Fake", "[1][1]", "field y", "no value")
    );
    // A violation at the parent (an element's ID) is reported before visiting the children.
    let mut wrong_type = good();
    wrong_type.info = u8::type_info();
    wrong_type.view = View::Primitive;
    let value = vec![broken(), wrong_type];
    assert!(matches!(
        validate_reflect(&value),
        Err(ReflectError::ShapeMismatch { at, expected, .. }) if at == "(root)" && expected.starts_with("element 1 of type `refl_test::Fake`")
    ));
}

/// Verifies: REFL-R080
#[test]
fn type_info_is_built_once_per_visited_value() {
    let value = vec![
        Fake::named(&["x", "y"], &[1, 2]),
        Fake::named(&["x", "y"], &[3, 4]),
    ];
    assert_eq!(validate_reflect(&value), Ok(()));
    for fake in &value {
        assert_eq!(fake.info_calls.get(), 1);
    }
    let root = Fake::list(&[1]);
    assert_eq!(validate_reflect(&root), Ok(()));
    assert_eq!(root.info_calls.get(), 1);
}
