//! Value access through the public API (spec §10, REFL-R040 to REFL-R045). Oracles: direct
//! access to the concrete value after writing through reflection, hand-written expected errors.

#[path = "common/types.rs"]
mod types;

use liminal_reflect::error::ReflectError;
use liminal_reflect::info::PrimitiveKind;
use liminal_reflect::value::{PrimitiveRef, Reflect, ReflectMut, ReflectRef, Typed};
use proptest::prelude::*;
use types::{Fake, Marker, Mode, Pair, SAMPLE_FIELDS, Sample};

fn sample() -> Sample {
    Sample {
        x: 1.5,
        count: 9,
        name: "boid".to_string(),
        tags: vec![1, 2],
        limit: Some(-3),
    }
}

/// Checks the primitive view of one value (REFL-R040).
fn check_primitive<T: Typed + Clone>(
    mut value: T,
    kind: PrimitiveKind,
    matches: fn(PrimitiveRef<'_>, &T) -> bool,
) {
    let copy = value.clone();
    match value.reflect_ref() {
        ReflectRef::Primitive(primitive) => {
            assert_eq!(primitive.kind(), kind);
            assert!(matches(primitive, &copy), "{kind:?}");
        }
        _ => panic!("{kind:?}: not a primitive view"),
    }
    assert!(matches!(value.reflect_mut(), ReflectMut::Primitive(k) if k == kind));
}

/// Verifies: REFL-R040
#[test]
fn primitive_views_refer_to_the_value() {
    check_primitive(true, PrimitiveKind::Bool, |p, v| p == PrimitiveRef::Bool(v));
    check_primitive(7_u8, PrimitiveKind::U8, |p, v| p == PrimitiveRef::U8(v));
    check_primitive(7_u16, PrimitiveKind::U16, |p, v| p == PrimitiveRef::U16(v));
    check_primitive(7_u32, PrimitiveKind::U32, |p, v| p == PrimitiveRef::U32(v));
    check_primitive(7_u64, PrimitiveKind::U64, |p, v| p == PrimitiveRef::U64(v));
    check_primitive(7_u128, PrimitiveKind::U128, |p, v| {
        p == PrimitiveRef::U128(v)
    });
    check_primitive(7_usize, PrimitiveKind::Usize, |p, v| {
        p == PrimitiveRef::Usize(v)
    });
    check_primitive(-7_i8, PrimitiveKind::I8, |p, v| p == PrimitiveRef::I8(v));
    check_primitive(-7_i16, PrimitiveKind::I16, |p, v| p == PrimitiveRef::I16(v));
    check_primitive(-7_i32, PrimitiveKind::I32, |p, v| p == PrimitiveRef::I32(v));
    check_primitive(-7_i64, PrimitiveKind::I64, |p, v| p == PrimitiveRef::I64(v));
    check_primitive(-7_i128, PrimitiveKind::I128, |p, v| {
        p == PrimitiveRef::I128(v)
    });
    check_primitive(-7_isize, PrimitiveKind::Isize, |p, v| {
        p == PrimitiveRef::Isize(v)
    });
    check_primitive(0.5_f32, PrimitiveKind::F32, |p, v| {
        p == PrimitiveRef::F32(v)
    });
    check_primitive(0.5_f64, PrimitiveKind::F64, |p, v| {
        p == PrimitiveRef::F64(v)
    });
    check_primitive('é', PrimitiveKind::Char, |p, v| p == PrimitiveRef::Char(v));
    check_primitive("ü".to_string(), PrimitiveKind::String, |p, v| {
        p == PrimitiveRef::String(v)
    });

    // The reference is to the value itself, not a copy.
    let value = 3_u32;
    let ReflectRef::Primitive(PrimitiveRef::U32(inner)) = value.reflect_ref() else {
        panic!("not a u32 view");
    };
    assert!(std::ptr::eq(inner, &raw const value));
}

/// Verifies: REFL-R040
#[test]
fn container_views() {
    let mut list = vec![1_u8, 2];
    assert!(matches!(list.reflect_ref(), ReflectRef::List(l) if l.len() == 2));
    assert!(matches!(list.reflect_mut(), ReflectMut::List(l) if l.len() == 2));
    let mut array = [1_u8; 3];
    assert!(matches!(array.reflect_ref(), ReflectRef::List(l) if l.len() == 3));
    assert!(matches!(array.reflect_mut(), ReflectMut::List(l) if l.len() == 3));

    let mut none: Option<u8> = None;
    let ReflectRef::Enum(view) = none.reflect_ref() else {
        panic!("None: not an enum view");
    };
    assert_eq!((view.variant_index(), view.variant_name()), (0, "None"));
    assert_eq!(view.field_count(), 0);
    assert!(view.field_name(0).is_none() && view.field_at(0).is_none());
    let ReflectMut::Enum(view) = none.reflect_mut() else {
        panic!("None: not an enum view");
    };
    assert!(view.field_at_mut(0).is_none());

    let mut some = Some(5_u8);
    let ReflectRef::Enum(view) = some.reflect_ref() else {
        panic!("Some: not an enum view");
    };
    assert_eq!((view.variant_index(), view.variant_name()), (1, "Some"));
    assert_eq!((view.field_count(), view.field_name(0)), (1, Some("0")));
    assert_eq!(
        view.field_at(0).map(|f| f.downcast_ref::<u8>()),
        Some(Ok(&5))
    );
    let ReflectMut::Enum(view) = some.reflect_mut() else {
        panic!("Some: not an enum view");
    };
    *view.field_at_mut(0).unwrap().downcast_mut::<u8>().unwrap() = 6;
    assert_eq!(some, Some(6));
}

/// Verifies: REFL-R041
#[test]
fn reflect_fields_bounds_and_writes() {
    let mut value = sample();
    let ReflectMut::Struct(view) = value.reflect_mut() else {
        panic!("not a struct view");
    };
    let count = view.field_count();
    assert_eq!(count, SAMPLE_FIELDS.len());
    for index in 0..count + 3 {
        let inside = index < count;
        assert_eq!(view.field_name(index).is_some(), inside, "{index}");
        assert_eq!(view.field_at(index).is_some(), inside, "{index}");
        assert_eq!(view.field_at_mut(index).is_some(), inside, "{index}");
    }
    *view.field_at_mut(1).unwrap().downcast_mut::<u64>().unwrap() = 77;
    assert_eq!(view.field_at(1).unwrap().downcast_ref::<u64>(), Ok(&77));
    assert_eq!(
        value,
        Sample {
            count: 77,
            ..sample()
        }
    );

    // Tuple and unit structs, and the fields of an enum's current variant.
    let pair = Pair(1, None);
    let ReflectRef::Struct(view) = pair.reflect_ref() else {
        panic!("not a struct view");
    };
    assert_eq!(
        (view.field_count(), view.field_name(1), view.field_name(2)),
        (2, Some("1"), None)
    );
    let ReflectRef::Struct(view) = Marker.reflect_ref() else {
        panic!("not a struct view");
    };
    assert_eq!((view.field_count(), view.field_name(0)), (0, None));
    assert!(view.field_at(0).is_none());
}

/// Verifies: REFL-R042
#[test]
fn reflect_list_bounds_and_writes() {
    let mut list = vec![10_u16, 20, 30];
    let ReflectMut::List(view) = list.reflect_mut() else {
        panic!("not a list view");
    };
    assert_eq!((view.len(), view.is_empty()), (3, false));
    for index in 0..6 {
        assert_eq!(view.get(index).is_some(), index < 3);
        assert_eq!(view.get_mut(index).is_some(), index < 3);
    }
    *view.get_mut(1).unwrap().downcast_mut::<u16>().unwrap() = 99;
    assert_eq!(view.get(1).unwrap().downcast_ref::<u16>(), Ok(&99));
    assert_eq!(list, [10, 99, 30]);

    let mut array = [1_i8, 2];
    let ReflectMut::List(view) = array.reflect_mut() else {
        panic!("not a list view");
    };
    assert_eq!((view.len(), view.is_empty()), (2, false));
    assert!(view.get(2).is_none() && view.get_mut(2).is_none());
    *view.get_mut(0).unwrap().downcast_mut::<i8>().unwrap() = -1;
    assert_eq!(array, [-1, 2]);

    let empty: Vec<u8> = Vec::new();
    let ReflectRef::List(view) = empty.reflect_ref() else {
        panic!("not a list view");
    };
    assert!(view.is_empty() && view.get(0).is_none());
    let ReflectRef::List(view) = [0_u8; 0].reflect_ref() else {
        panic!("not a list view");
    };
    assert_eq!((view.len(), view.is_empty()), (0, true));
}

/// Verifies: REFL-R043
#[test]
fn field_by_name_on_structs() {
    let mut value = sample();
    let r: &mut dyn Reflect = &mut value;
    assert_eq!(
        r.field("name").unwrap().downcast_ref::<String>().unwrap(),
        "boid"
    );
    assert!(matches!(
        r.field("Name").err(),
        Some(ReflectError::NoSuchField { type_path, field }) if type_path == "refl_test::Sample" && field == "Name"
    ));
    assert!(matches!(
        r.field_mut("").err(),
        Some(ReflectError::NoSuchField { .. })
    ));
    *r.field_mut("count").unwrap().downcast_mut::<u64>().unwrap() = 4;
    assert_eq!(value.count, 4);

    let mut pair = Pair(5, Some(1));
    let r: &mut dyn Reflect = &mut pair;
    assert_eq!(r.field("0").unwrap().downcast_ref::<i32>(), Ok(&5));
    *r.field_mut("1")
        .unwrap()
        .downcast_mut::<Option<u8>>()
        .unwrap() = None;
    assert!(matches!(
        r.field("2").err(),
        Some(ReflectError::NoSuchField { .. })
    ));
    assert_eq!(pair, Pair(5, None));
}

/// Verifies: REFL-R043
#[test]
fn field_by_name_on_enums() {
    let mut mode = Mode::Fleeing {
        from: 3,
        speed: 1.0,
    };
    let r: &mut dyn Reflect = &mut mode;
    assert_eq!(r.field("from").unwrap().downcast_ref::<u32>(), Ok(&3));
    *r.field_mut("speed").unwrap().downcast_mut::<f32>().unwrap() = 2.0;
    // `target` exists in `Chasing` only.
    let expected = ReflectError::FieldNotInVariant {
        type_path: "refl_test::Mode".to_string(),
        variant: "Fleeing".to_string(),
        field: "target".to_string(),
    };
    assert_eq!(r.field("target").err(), Some(expected.clone()));
    assert_eq!(r.field_mut("target").err(), Some(expected));
    // No variant has `height`.
    let no_such = ReflectError::NoSuchField {
        type_path: "refl_test::Mode".to_string(),
        field: "height".to_string(),
    };
    assert_eq!(r.field("height").err(), Some(no_such.clone()));
    assert_eq!(r.field_mut("height").err(), Some(no_such));
    assert_eq!(
        mode,
        Mode::Fleeing {
            from: 3,
            speed: 2.0
        }
    );

    let mut idle = Mode::Idle;
    let r: &mut dyn Reflect = &mut idle;
    assert!(matches!(
        r.field("speed").err(),
        Some(ReflectError::FieldNotInVariant { variant, .. }) if variant == "Idle"
    ));
}

/// Verifies: REFL-R043
#[test]
fn field_by_name_on_lists_and_primitives() {
    let mut list = vec![1_u8];
    let r: &mut dyn Reflect = &mut list;
    let expected = ReflectError::NoFields {
        type_path: "Vec<u8>".to_string(),
        shape: "list".to_string(),
    };
    assert_eq!(r.field("0").err(), Some(expected.clone()));
    assert_eq!(r.field_mut("0").err(), Some(expected));
    let mut array = [1_u8; 2];
    let r: &mut dyn Reflect = &mut array;
    assert!(
        matches!(r.field_mut("len").err(), Some(ReflectError::NoFields { shape, .. }) if shape == "list")
    );

    let mut number = 1_u32;
    let r: &mut dyn Reflect = &mut number;
    let expected = ReflectError::NoFields {
        type_path: "u32".to_string(),
        shape: "primitive".to_string(),
    };
    assert_eq!(r.field("x").err(), Some(expected.clone()));
    assert_eq!(r.field_mut("x").err(), Some(expected));
}

/// A deliberately wrong impl: `field_count() == 2`, `field_name(1) == Some("y")`, `field_at(1)`
/// and `field_at_mut(1)` return `None`; and a nameless index that matches nothing.
///
/// Verifies: REFL-R043
#[test]
fn field_by_name_on_a_wrong_impl_returns_no_such_field() {
    let mut fake = Fake::named(&["x", "y"], &[1]);
    let expected = ReflectError::NoSuchField {
        type_path: "refl_test::Fake".to_string(),
        field: "y".to_string(),
    };
    let r: &mut dyn Reflect = &mut fake;
    assert_eq!(r.field("x").unwrap().downcast_ref::<u32>(), Ok(&1));
    assert_eq!(r.field("y").err(), Some(expected.clone()));
    assert_eq!(r.field_mut("y").err(), Some(expected));

    // The same on an enum value whose current variant lists the name.
    let mut fake = Fake::enumeration(4);
    fake.values = vec![None];
    let r: &mut dyn Reflect = &mut fake;
    assert!(matches!(
        r.field("x").err(),
        Some(ReflectError::NoSuchField { .. })
    ));
    assert!(matches!(
        r.field_mut("x").err(),
        Some(ReflectError::NoSuchField { .. })
    ));

    // Index 0 has no name: it matches no name; the lowest matching index wins.
    let mut fake = Fake::named(&["x", "x"], &[1, 2]);
    fake.names[0] = None;
    let r: &mut dyn Reflect = &mut fake;
    assert_eq!(r.field("x").unwrap().downcast_ref::<u32>(), Ok(&2));
    fake.names[0] = Some("x");
    let r: &mut dyn Reflect = &mut fake;
    assert_eq!(r.field_mut("x").unwrap().downcast_ref::<u32>(), Ok(&1));
}

/// Verifies: REFL-R044
#[test]
fn downcasts_check_the_concrete_type() {
    let mut value = 5_u32;
    let r: &mut dyn Reflect = &mut value;
    assert_eq!(r.downcast_ref::<u32>(), Ok(&5));
    let expected = ReflectError::TypeMismatch {
        expected: "i32".to_string(),
        actual: "u32".to_string(),
    };
    assert_eq!(r.downcast_ref::<i32>().err(), Some(expected.clone()));
    assert_eq!(r.downcast_mut::<i32>().err(), Some(expected));
    *r.downcast_mut::<u32>().unwrap() = 6;
    assert_eq!(value, 6);

    // The check is the Rust type, not the stable ID: `Fake` reports the path and ID of `u8`,
    // but it is not a `u8`.
    let mut fake = Fake::primitive(1);
    fake.info = u8::type_info();
    let r: &mut dyn Reflect = &mut fake;
    assert_eq!(
        r.downcast_ref::<u8>().err(),
        Some(ReflectError::TypeMismatch {
            expected: "u8".to_string(),
            actual: "u8".to_string()
        })
    );
    assert!(r.downcast_mut::<u8>().is_err());
    let list: &dyn Reflect = &vec![Some(1_u8)];
    assert!(list.downcast_ref::<Vec<Option<u8>>>().is_ok());
    assert!(matches!(
        list.downcast_ref::<Vec<u8>>().err(),
        Some(ReflectError::TypeMismatch { expected, actual }) if expected == "Vec<u8>" && actual == "Vec<Option<u8>>"
    ));
}

/// Writes `x` into field `name` of `value` by reflection and returns the value read back by
/// reflection. The callers compare the whole value by direct access afterwards.
fn round_trip<V: Typed + Clone, X: Typed + Clone + PartialEq + core::fmt::Debug>(
    value: &mut V,
    name: &str,
    x: &X,
) -> Result<X, ReflectError> {
    let r: &mut dyn Reflect = value;
    *r.field_mut(name)?.downcast_mut::<X>()? = x.clone();
    Ok(r.field(name)?.downcast_ref::<X>()?.clone())
}

proptest! {
    /// Verifies: REFL-R045
    #[test]
    fn named_struct_field_round_trip(
        x in any::<f32>(), count in any::<u64>(), name in ".*", tags in prop::collection::vec(any::<u8>(), 0..8),
        limit in any::<Option<i16>>(),
    ) {
        let mut value = sample();
        // `f32` compared by bits (NaN).
        prop_assert_eq!(round_trip(&mut value, "x", &x).unwrap().to_bits(), x.to_bits());
        prop_assert_eq!(value.x.to_bits(), x.to_bits());
        prop_assert_eq!(&value, &Sample { x: value.x, ..sample() });

        let mut value = sample();
        prop_assert_eq!(round_trip(&mut value, "count", &count).unwrap(), count);
        prop_assert_eq!(&value, &Sample { count, ..sample() });
        let mut value = sample();
        prop_assert_eq!(round_trip(&mut value, "name", &name).unwrap(), name.clone());
        prop_assert_eq!(&value, &Sample { name: name.clone(), ..sample() });
        let mut value = sample();
        prop_assert_eq!(round_trip(&mut value, "tags", &tags).unwrap(), tags.clone());
        prop_assert_eq!(&value, &Sample { tags: tags.clone(), ..sample() });
        let mut value = sample();
        prop_assert_eq!(round_trip(&mut value, "limit", &limit).unwrap(), limit);
        prop_assert_eq!(&value, &Sample { limit, ..sample() });
    }

    /// Verifies: REFL-R045
    #[test]
    fn tuple_struct_and_variant_field_round_trip(
        a in any::<i32>(), b in any::<Option<u8>>(), from in any::<u32>(), speed in any::<u32>(),
    ) {
        let mut pair = Pair(1, Some(2));
        prop_assert_eq!(round_trip(&mut pair, "0", &a).unwrap(), a);
        prop_assert_eq!(&pair, &Pair(a, Some(2)));
        prop_assert_eq!(round_trip(&mut pair, "1", &b).unwrap(), b);
        prop_assert_eq!(&pair, &Pair(a, b));

        let speed = f32::from_bits(speed);
        let mut mode = Mode::Fleeing { from: 0, speed: 0.5 };
        prop_assert_eq!(round_trip(&mut mode, "from", &from).unwrap(), from);
        prop_assert_eq!(&mode, &Mode::Fleeing { from, speed: 0.5 });
        prop_assert_eq!(round_trip(&mut mode, "speed", &speed).unwrap().to_bits(), speed.to_bits());
        let Mode::Fleeing { from: f, speed: s } = mode else { panic!("variant changed") };
        prop_assert_eq!((f, s.to_bits()), (from, speed.to_bits()));

        let mut moving = Mode::Moving(1.0, 2.0);
        prop_assert_eq!(round_trip(&mut moving, "1", &speed).unwrap().to_bits(), speed.to_bits());
        let Mode::Moving(first, second) = moving else { panic!("variant changed") };
        prop_assert_eq!((first, second.to_bits()), (1.0, speed.to_bits()));
    }
}
