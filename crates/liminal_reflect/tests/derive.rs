//! Behaviour of `#[derive(Reflect)]` through the public API (derive spec §10, "Generated
//! behaviour"). Oracles: hand-written expected descriptors, paths and field lists; direct access
//! to the concrete value after writing through reflection.
//!
//! The derived types here compile under the workspace lints with `-D warnings` (REFLD-R001).

use liminal_reflect::error::ReflectError;
use liminal_reflect::hash::reflect_state_hash;
use liminal_reflect::info::{
    FieldInfo, Shape, StableTypeId, StructKind, TypeInfo, TypeRef, VariantInfo,
};
use liminal_reflect::registry::TypeRegistry;
use liminal_reflect::value::{Reflect, ReflectFields, ReflectRef, Typed, validate_reflect};

fn field<T: Typed>(name: &'static str) -> FieldInfo {
    FieldInfo {
        name,
        ty: TypeRef::of::<T>(),
    }
}

fn path(name: &str) -> String {
    format!("{}::{name}", module_path!())
}

/// Named struct.
#[derive(Reflect)]
struct Health {
    current: u32,
    max: u32,
}

/// Tuple struct with an ignored field in the middle.
#[derive(Reflect)]
struct Spaced(u8, #[reflect(ignore)] std::cell::Cell<u8>, u16);

/// Unit struct.
#[derive(Reflect)]
struct Tag;

/// Enum with all three variant kinds and explicit discriminants (ignored by REFLD-R031).
#[derive(Clone, Reflect)]
#[repr(u8)]
enum State {
    Idle = 5,
    Moving(f32, f32) = 1,
    Fleeing { from: u32 } = 9,
}

/// Enum of unit variants only.
#[derive(Reflect)]
enum Toggle {
    Off,
    On,
}

/// Generic type with two parameters, one with its own bound, and a `where` clause.
#[derive(Clone, Reflect)]
struct Pair<T: Copy, U>
where
    U: Clone,
{
    first: T,
    second: U,
}

/// Recursive type (REFLD-R005).
#[derive(Reflect)]
struct Node {
    value: u32,
    children: Vec<Node>,
}

/// Raw identifiers (REFLD-R004, REFLD-R020, REFLD-R030).
#[derive(Reflect)]
#[allow(non_camel_case_types)]
enum r#match {
    r#loop { r#type: u8 },
}

/// A type with an ignored field whose type does not implement `Typed` (REFLD-R034).
#[derive(Reflect)]
struct Cached {
    value: u32,
    #[reflect(ignore)]
    cache: NotReflected,
}

struct NotReflected(u8);

/// Type path override (REFLD-R021).
#[derive(Reflect)]
#[reflect(type_path = "my_game::Renamed")]
struct Renamed<T> {
    inner: T,
}

mod inner {
    use liminal_reflect::value::Reflect;

    /// Private fields, constructed only through `new` (REFLD-R004).
    #[derive(Reflect)]
    pub struct Secret {
        hidden: u32,
        pub(crate) shown: i8,
    }

    impl Secret {
        pub fn new() -> Secret {
            Secret {
                hidden: 7,
                shown: -1,
            }
        }
        pub fn hidden(&self) -> u32 {
            self.hidden
        }
    }
}

fn fields_of(value: &dyn Reflect) -> Vec<&'static str> {
    let view: &dyn ReflectFields = match value.reflect_ref() {
        ReflectRef::Struct(fields) => fields,
        ReflectRef::Enum(e) => {
            return (0..e.field_count())
                .filter_map(|i| e.field_name(i))
                .collect();
        }
        _ => return Vec::new(),
    };
    (0..view.field_count())
        .filter_map(|i| view.field_name(i))
        .collect()
}

/// Verifies: REFLD-R001
#[test]
fn every_accepted_kind_derives_and_validates() {
    let mut registry = TypeRegistry::new();
    for id in [
        registry.register::<Health>(),
        registry.register::<Spaced>(),
        registry.register::<Tag>(),
        registry.register::<State>(),
        registry.register::<Toggle>(),
        registry.register::<Pair<u8, String>>(),
        registry.register::<Node>(),
        registry.register::<r#match>(),
        registry.register::<Cached>(),
        registry.register::<Renamed<u8>>(),
    ] {
        assert!(id.is_ok());
    }
    let values: [&dyn Reflect; 5] = [
        &Health { current: 1, max: 2 },
        &Spaced(1, std::cell::Cell::new(0), 2),
        &Tag,
        &State::Moving(1.0, 2.0),
        &Toggle::On,
    ];
    for value in values {
        assert_eq!(validate_reflect(value), Ok(()));
    }
}

/// Verifies: REFLD-R002
#[test]
fn generic_parameters_get_the_typed_bound() {
    let value = Pair::<u8, String> {
        first: 3,
        second: "two".to_string(),
    };
    assert_eq!(
        <Pair<u8, String>>::type_path(),
        format!("{}<u8, String>", path("Pair"))
    );
    assert_eq!(validate_reflect(&value), Ok(()));
    let r: &dyn Reflect = &value;
    assert!(matches!(
        r.field("second")
            .map(|v| v.downcast_ref::<String>().map(String::as_str)),
        Ok(Ok("two"))
    ));
    assert_eq!(
        <Pair<u8, String>>::type_info().shape,
        Shape::Struct {
            kind: StructKind::Named,
            fields: vec![field::<u8>("first"), field::<String>("second")],
        }
    );
}

/// Verifies: REFLD-R004
#[test]
fn field_names_follow_identifiers_and_positions() {
    assert_eq!(
        fields_of(&Health { current: 0, max: 0 }),
        ["current", "max"]
    );
    assert_eq!(
        fields_of(&Spaced(0, std::cell::Cell::new(0), 0)),
        ["0", "2"]
    );
    assert_eq!(fields_of(&r#match::r#loop { r#type: 1 }), ["type"]);
    assert_eq!(fields_of(&Tag), Vec::<&str>::new());

    // Private fields of a type in another module are read and written by name.
    let mut secret = inner::Secret::new();
    assert_eq!(fields_of(&secret), ["hidden", "shown"]);
    let r: &mut dyn Reflect = &mut secret;
    match r
        .field_mut("hidden")
        .map(|v| v.downcast_mut::<u32>().map(|x| *x = 99))
    {
        Ok(Ok(())) => {}
        _ => panic!("hidden is reflected"),
    }
    assert_eq!(secret.hidden(), 99);
}

/// Verifies: REFLD-R005
#[test]
fn recursive_types_have_finite_descriptors() {
    let info = Node::type_info();
    let Shape::Struct { fields, .. } = &info.shape else {
        panic!("struct shape");
    };
    assert_eq!(fields[1].name, "children");
    assert_eq!(
        fields[1].ty.type_path,
        format!("Vec<{}>", Node::type_path())
    );
    let tree = Node {
        value: 1,
        children: vec![Node {
            value: 2,
            children: Vec::new(),
        }],
    };
    assert_eq!(validate_reflect(&tree), Ok(()));
}

/// Verifies: REFLD-R020
#[test]
fn default_type_paths_use_the_module_path() {
    assert_eq!(Health::type_path(), "derive::Health");
    assert_eq!(Health::type_path(), path("Health"));
    assert_eq!(r#match::type_path(), "derive::match");
    assert_eq!(inner::Secret::type_path(), "derive::inner::Secret");
    assert_eq!(
        <Pair<u8, Vec<u16>>>::type_path(),
        "derive::Pair<u8, Vec<u16>>"
    );
    assert_eq!(
        <Pair<bool, Pair<u8, u8>>>::type_path(),
        "derive::Pair<bool, derive::Pair<u8, u8>>"
    );
}

/// Verifies: REFLD-R021
#[test]
fn type_path_override_replaces_the_base_path() {
    assert_eq!(<Renamed<u8>>::type_path(), "my_game::Renamed<u8>");
    assert_eq!(
        <Renamed<u8>>::stable_type_id(),
        StableTypeId::of_path("my_game::Renamed<u8>")
    );
}

/// Verifies: REFLD-R030
#[test]
fn descriptors_list_kinds_variants_and_fields() {
    assert_eq!(
        Health::type_info(),
        TypeInfo {
            type_path: path("Health"),
            id: StableTypeId::of_path(&path("Health")),
            shape: Shape::Struct {
                kind: StructKind::Named,
                fields: vec![field::<u32>("current"), field::<u32>("max")],
            },
        }
    );
    assert_eq!(
        Spaced::type_info().shape,
        Shape::Struct {
            kind: StructKind::Tuple,
            fields: vec![field::<u8>("0"), field::<u16>("2")],
        }
    );
    assert_eq!(
        Tag::type_info().shape,
        Shape::Struct {
            kind: StructKind::Unit,
            fields: Vec::new(),
        }
    );
    let variant = |name, kind, fields| VariantInfo { name, kind, fields };
    assert_eq!(
        State::type_info().shape,
        Shape::Enum {
            variants: vec![
                variant("Idle", StructKind::Unit, Vec::new()),
                variant(
                    "Moving",
                    StructKind::Tuple,
                    vec![field::<f32>("0"), field::<f32>("1")]
                ),
                variant("Fleeing", StructKind::Named, vec![field::<u32>("from")]),
            ],
        }
    );
    assert_eq!(
        r#match::type_info().shape,
        Shape::Enum {
            variants: vec![variant(
                "loop",
                StructKind::Named,
                vec![field::<u8>("type")]
            )],
        }
    );
    let value = State::Idle;
    assert_eq!(value.reflect_type_info(), State::type_info());
}

/// Verifies: REFLD-R031
#[test]
fn enum_views_report_declaration_positions() {
    let cases = [
        (State::Idle, 0, "Idle"),
        (State::Moving(1.0, 2.0), 1, "Moving"),
        (State::Fleeing { from: 3 }, 2, "Fleeing"),
    ];
    for (mut value, index, name) in cases {
        match value.reflect_ref() {
            ReflectRef::Enum(e) => assert_eq!((e.variant_index(), e.variant_name()), (index, name)),
            _ => panic!("enum view"),
        }
        match value.reflect_mut() {
            liminal_reflect::value::ReflectMut::Enum(e) => assert_eq!(e.variant_index(), index),
            _ => panic!("enum view"),
        }
    }
    let health = Health { current: 0, max: 0 };
    assert!(matches!(health.reflect_ref(), ReflectRef::Struct(_)));
    assert!(matches!(Toggle::Off.reflect_ref(), ReflectRef::Enum(e) if e.variant_name() == "Off"));
}

/// Verifies: REFLD-R032
#[test]
fn field_access_by_index() {
    let mut state = State::Moving(1.0, 2.0);
    let ReflectRef::Enum(e) = state.reflect_ref() else {
        panic!("enum view");
    };
    assert_eq!(e.field_count(), 2);
    assert_eq!(e.field_name(1), Some("1"));
    assert_eq!(e.field_name(2), None);
    assert!(e.field_at(2).is_none());
    if let liminal_reflect::value::ReflectMut::Enum(e) = state.reflect_mut() {
        if let Some(value) = e.field_at_mut(1).and_then(|v| v.downcast_mut::<f32>().ok()) {
            *value = 8.0;
        }
        assert!(e.field_at_mut(2).is_none());
    }
    assert!(
        matches!(state, State::Moving(a, b) if a.to_bits() == 1.0_f32.to_bits() && b.to_bits() == 8.0_f32.to_bits())
    );

    let ReflectRef::Enum(idle) = Toggle::Off.reflect_ref() else {
        panic!("enum view");
    };
    assert_eq!((idle.field_count(), idle.field_name(0)), (0, None));
    assert!(idle.field_at(0).is_none());
    let mut tag = Tag;
    let ReflectRef::Struct(unit) = tag.reflect_ref() else {
        panic!("struct view");
    };
    assert_eq!((unit.field_count(), unit.field_name(0)), (0, None));
    assert!(unit.field_at(0).is_none());
    if let liminal_reflect::value::ReflectMut::Struct(unit) = tag.reflect_mut() {
        assert!(unit.field_at_mut(0).is_none());
    }
    let mut health = Health { current: 1, max: 2 };
    if let liminal_reflect::value::ReflectMut::Struct(s) = health.reflect_mut() {
        assert!(s.field_at_mut(2).is_none());
        assert!(s.field_at_mut(1).is_some());
    }
}

/// Verifies: REFLD-R034
#[test]
fn ignored_fields_are_invisible() {
    let mut cached = Cached {
        value: 4,
        cache: NotReflected(1),
    };
    assert_eq!(fields_of(&cached), ["value"]);
    let hash = reflect_state_hash(&cached);
    cached.cache.0 = 200;
    assert_eq!(reflect_state_hash(&cached), hash);
    let r: &mut dyn Reflect = &mut cached;
    assert!(matches!(
        r.field("cache"),
        Err(ReflectError::NoSuchField { .. })
    ));
    assert!(matches!(
        r.field_mut("cache"),
        Err(ReflectError::NoSuchField { .. })
    ));
    let spaced = Spaced(1, std::cell::Cell::new(0), 2);
    let hash = reflect_state_hash(&spaced);
    spaced.1.set(77);
    assert_eq!(reflect_state_hash(&spaced), hash);
    assert!(matches!(
        (&spaced as &dyn Reflect).field("1"),
        Err(ReflectError::NoSuchField { .. })
    ));
}
