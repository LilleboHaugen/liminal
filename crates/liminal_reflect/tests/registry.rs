//! `TypeRegistry` through the public API (spec §10, REFL-R060 to REFL-R063, REFL-R065).
//! Oracles: hand-written expected errors, a sorted `Vec` of IDs, and the `Debug` string built
//! from `iter()`. REFL-R064 (logarithmic lookups) is checked by review, as the spec says.

#[path = "common/types.rs"]
mod types;

use liminal_reflect::error::ReflectError;
use liminal_reflect::info::{Shape, StableTypeId, StructKind, TypeInfo, VariantInfo};
use liminal_reflect::registry::TypeRegistry;
use liminal_reflect::value::{Reflect, ReflectFields, ReflectMut, ReflectRef, Typed};
use proptest::prelude::*;
use types::{Marker, Mode, Pair, Sample, field, info};

/// Defines a unit struct whose hand-written `Typed::type_info` returns `$info` (which may be a
/// deliberately invalid descriptor). Its value view is a struct with no fields.
macro_rules! descriptor_type {
    ($name:ident, $info:expr) => {
        struct $name;

        impl Reflect for $name {
            fn reflect_type_info(&self) -> TypeInfo {
                <$name as Typed>::type_info()
            }
            fn reflect_ref(&self) -> ReflectRef<'_> {
                ReflectRef::Struct(self)
            }
            fn reflect_mut(&mut self) -> ReflectMut<'_> {
                ReflectMut::Struct(self)
            }
        }

        impl Typed for $name {
            fn type_path() -> String {
                <$name as Typed>::type_info().type_path
            }
            fn type_info() -> TypeInfo {
                $info
            }
        }

        impl ReflectFields for $name {
            fn field_count(&self) -> usize {
                0
            }
            fn field_name(&self, _: usize) -> Option<&'static str> {
                None
            }
            fn field_at(&self, _: usize) -> Option<&dyn Reflect> {
                None
            }
            fn field_at_mut(&mut self, _: usize) -> Option<&mut dyn Reflect> {
                None
            }
        }
    };
}

fn named(fields: &[&'static str]) -> Shape {
    Shape::Struct {
        kind: StructKind::Named,
        fields: fields.iter().map(|name| field::<u8>(name)).collect(),
    }
}

fn variant(name: &'static str, fields: &[&'static str]) -> VariantInfo {
    VariantInfo {
        name,
        kind: if fields.is_empty() {
            StructKind::Unit
        } else {
            StructKind::Named
        },
        fields: fields.iter().map(|name| field::<u8>(name)).collect(),
    }
}

descriptor_type!(
    EmptyPath,
    TypeInfo {
        type_path: String::new(),
        id: StableTypeId::new(1),
        shape: named(&["x", "x"]),
    }
);
descriptor_type!(
    WrongId,
    TypeInfo {
        type_path: "refl_test::WrongId".to_string(),
        id: StableTypeId::new(1),
        shape: named(&["x", "x"]),
    }
);
descriptor_type!(
    DupStruct,
    info("refl_test::DupStruct", named(&["a", "b", "a", "b"]))
);
descriptor_type!(
    EnumE,
    info(
        "refl_test::E",
        Shape::Enum {
            variants: vec![
                variant("A", &["x", "x"]),
                variant("B", &[]),
                variant("A", &[])
            ],
        },
    )
);
descriptor_type!(
    EnumF,
    info(
        "refl_test::F",
        Shape::Enum {
            variants: vec![variant("A", &[]), variant("A", &["x", "x"])],
        },
    )
);
descriptor_type!(
    EnumOk,
    info(
        "refl_test::EnumOk",
        Shape::Enum {
            variants: vec![variant("A", &["x"]), variant("B", &["x"])],
        },
    )
);
descriptor_type!(TwinA, info("refl_test::Twin", named(&["a"])));
descriptor_type!(TwinB, info("refl_test::Twin", named(&["b"])));
descriptor_type!(TwinBad, info("refl_test::Twin", named(&["c", "c"])));

/// Registered types, sorted by ID.
fn ids(registry: &TypeRegistry) -> Vec<u64> {
    registry.iter().map(|info| info.id.value()).collect()
}

fn snapshot(registry: &TypeRegistry) -> (usize, String) {
    (registry.len(), format!("{registry:?}"))
}

/// Verifies: REFL-R060
#[test]
fn register_stores_and_is_idempotent() {
    let empty = TypeRegistry::new();
    assert!(empty.is_empty() && empty.len() == 0 && empty.iter().next().is_none());
    assert_eq!(snapshot(&TypeRegistry::default()), snapshot(&empty));

    let mut registry = TypeRegistry::new();
    assert_eq!(registry.register::<Sample>(), Ok(Sample::stable_type_id()));
    assert_eq!(
        registry.get(Sample::stable_type_id()),
        Some(&Sample::type_info())
    );
    let before = snapshot(&registry);
    assert_eq!(registry.register::<Sample>(), Ok(Sample::stable_type_id()));
    assert_eq!(snapshot(&registry), before);
    assert_eq!(registry.register::<EnumOk>(), Ok(EnumOk::stable_type_id()));
    assert_eq!(registry.len(), 2);
}

/// Verifies: REFL-R061
#[test]
fn register_checks_descriptors_in_order() {
    let mut registry = TypeRegistry::new();
    registry.register::<Sample>().unwrap();
    let before = snapshot(&registry);

    // Empty path is reported before the wrong ID and the repeated name.
    assert_eq!(
        registry.register::<EmptyPath>(),
        Err(ReflectError::EmptyTypePath)
    );
    // Wrong ID before the repeated name.
    assert_eq!(
        registry.register::<WrongId>(),
        Err(ReflectError::TypeIdMismatch {
            type_path: "refl_test::WrongId".to_string(),
            id: StableTypeId::new(1),
            expected_id: StableTypeId::of_path("refl_test::WrongId"),
        })
    );
    let duplicate = |type_path: &str, name: &str| {
        Err(ReflectError::DuplicateName {
            type_path: type_path.to_string(),
            name: name.to_string(),
        })
    };
    assert_eq!(
        registry.register::<DupStruct>(),
        duplicate("refl_test::DupStruct", "a")
    );
    // The two examples of the spec.
    assert_eq!(registry.register::<EnumE>(), duplicate("refl_test::E", "x"));
    assert_eq!(registry.register::<EnumF>(), duplicate("refl_test::F", "A"));
    assert_eq!(snapshot(&registry), before);
}

/// Verifies: REFL-R062
#[test]
fn two_rust_types_with_one_path_collide_in_both_orders() {
    let id = StableTypeId::of_path("refl_test::Twin");
    let mut registry = TypeRegistry::new();
    assert_eq!(registry.register::<TwinA>(), Ok(id));
    let before = snapshot(&registry);
    let collision = |existing: &str, new: &str| {
        Err(ReflectError::TypeIdCollision {
            id,
            existing_path: existing.to_string(),
            new_path: new.to_string(),
        })
    };
    assert_eq!(
        registry.register::<TwinB>(),
        collision("refl_test::Twin", "refl_test::Twin")
    );
    assert_eq!(snapshot(&registry), before);
    assert_eq!(registry.get(id), Some(&TwinA::type_info()));
    // A descriptor that fails REFL-R061 reports that error, even though its ID is stored.
    assert!(matches!(
        registry.register::<TwinBad>(),
        Err(ReflectError::DuplicateName { .. })
    ));

    let mut registry = TypeRegistry::new();
    assert_eq!(registry.register::<TwinB>(), Ok(id));
    assert_eq!(
        registry.register::<TwinA>(),
        collision("refl_test::Twin", "refl_test::Twin")
    );
    assert_eq!(registry.get(id), Some(&TwinB::type_info()));
}

/// Verifies: REFL-R063
#[test]
fn lookups_by_id_and_path() {
    let mut registry = TypeRegistry::new();
    for register in registrations() {
        register(&mut registry).unwrap();
    }
    assert_eq!(registry.len(), 6);
    assert_eq!(
        registry.get(Mode::stable_type_id()),
        Some(&Mode::type_info())
    );
    assert_eq!(
        registry.get_by_path("refl_test::Pair"),
        Some(&Pair::type_info())
    );
    assert_eq!(
        registry.get_by_path("Vec<u8>"),
        Some(&<Vec<u8>>::type_info())
    );
    assert_eq!(registry.get(u8::stable_type_id()), None);
    assert_eq!(registry.get_by_path("u8"), None);
    assert_eq!(registry.get_by_path(""), None);
    let mut sorted = ids(&registry);
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(ids(&registry), sorted);
}

type Registration = fn(&mut TypeRegistry) -> Result<StableTypeId, ReflectError>;

fn registrations() -> Vec<Registration> {
    vec![
        TypeRegistry::register::<Sample>,
        TypeRegistry::register::<Pair>,
        TypeRegistry::register::<Marker>,
        TypeRegistry::register::<Mode>,
        TypeRegistry::register::<Vec<u8>>,
        TypeRegistry::register::<Option<String>>,
    ]
}

proptest! {
    /// Verifies: REFL-R063, REFL-R065
    #[test]
    fn iteration_and_debug_do_not_depend_on_registration_order(
        order in Just(registrations()).prop_shuffle(),
    ) {
        let mut reference = TypeRegistry::new();
        for register in registrations() {
            register(&mut reference).unwrap();
        }
        let mut registry = TypeRegistry::new();
        for register in order {
            register(&mut registry).unwrap();
        }
        prop_assert_eq!(registry.iter().collect::<Vec<_>>(), reference.iter().collect::<Vec<_>>());
        prop_assert_eq!(format!("{registry:?}"), format!("{reference:?}"));
        prop_assert_eq!(format!("{registry:#?}"), format!("{reference:#?}"));
    }
}

fn expected_debug(registry: &TypeRegistry) -> String {
    let types: Vec<String> = registry.iter().map(|info| format!("{info:?}")).collect();
    format!("TypeRegistry {{ types: [{}] }}", types.join(", "))
}

/// Verifies: REFL-R065
#[test]
fn debug_prints_the_descriptors_only() {
    let empty = TypeRegistry::new();
    assert_eq!(format!("{empty:?}"), "TypeRegistry { types: [] }");
    assert_eq!(format!("{empty:?}"), expected_debug(&empty));

    let mut forward = TypeRegistry::new();
    forward.register::<Sample>().unwrap();
    forward.register::<Mode>().unwrap();
    forward.register::<Vec<u8>>().unwrap();
    let mut backward = TypeRegistry::new();
    backward.register::<Vec<u8>>().unwrap();
    backward.register::<Mode>().unwrap();
    backward.register::<Sample>().unwrap();
    let text = format!("{forward:?}");
    assert_eq!(text, expected_debug(&forward));
    assert_eq!(text, format!("{backward:?}"));
    // `core::any::TypeId` prints as `TypeId(…)`; `StableTypeId(…)` is the only allowed match.
    assert!(!text.replace("StableTypeId(", "").contains("TypeId"));
}
