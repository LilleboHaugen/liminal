//! Differential test of the derive against hand-written reference impls (REFLD-R033) and the
//! `liminal_reflect` impl requirements on derived types (REFLD-R035).
//!
//! Twins: `types::{Sample, Pair, Marker, Mode}` (hand-written, M1-T031) against
//! `types::derived::*`; `PairTwin`, `IgnoredTwin` and `NodeTwin` below against the derived
//! `Gen`, `Ignored` and `Node`. Each twin's `type_path()` returns the derived type's path, so the
//! descriptors are equal in every part. A derived type and its twin share a stable type ID, so
//! each is registered in its own `TypeRegistry`.

#[path = "common/types.rs"]
mod types;

use std::fmt::Debug;

use liminal_reflect::error::ReflectError;
use liminal_reflect::hash::reflect_state_hash;
use liminal_reflect::info::{FieldInfo, Shape, StructKind, TypeInfo, TypeRef};
use liminal_reflect::registry::TypeRegistry;
use liminal_reflect::value::{
    Reflect, ReflectFields, ReflectMut, ReflectRef, Typed, validate_reflect,
};
use proptest::prelude::*;
use types::derived;

// ------------------------------------------------------------------ derived types and twins

/// Generic type with two parameters.
#[derive(Clone, Debug, PartialEq, Reflect)]
struct Gen<T, U> {
    a: T,
    b: U,
}

#[derive(Clone, Debug, PartialEq)]
struct PairTwin<T, U> {
    a: T,
    b: U,
}

/// Type with ignored fields (named and tuple positions are covered by `tests/derive.rs`).
#[derive(Clone, Debug, PartialEq, Reflect)]
struct Ignored {
    #[reflect(ignore)]
    scratch: u64,
    kept: i16,
    #[reflect(ignore)]
    other: bool,
    last: char,
}

#[derive(Clone, Debug, PartialEq)]
struct IgnoredTwin {
    scratch: u64,
    kept: i16,
    other: bool,
    last: char,
}

/// Recursive type.
#[derive(Clone, Debug, PartialEq, Reflect)]
struct Node {
    value: u32,
    children: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq)]
struct NodeTwin {
    value: u32,
    children: Vec<NodeTwin>,
}

fn struct_info<T: Typed>(kind: StructKind, fields: Vec<FieldInfo>) -> TypeInfo {
    TypeInfo {
        type_path: T::type_path(),
        id: T::stable_type_id(),
        shape: Shape::Struct { kind, fields },
    }
}

fn field<T: Typed>(name: &'static str) -> FieldInfo {
    FieldInfo {
        name,
        ty: TypeRef::of::<T>(),
    }
}

/// Implements `Reflect` and `ReflectFields` of a named twin struct from a field list.
macro_rules! twin_struct {
    ([$($generics:tt)*] $type:ty, $($index:literal => $name:literal $member:ident),*) => {
        impl<$($generics)*> Reflect for $type {
            fn reflect_type_info(&self) -> TypeInfo {
                <Self as Typed>::type_info()
            }
            fn reflect_ref(&self) -> ReflectRef<'_> {
                ReflectRef::Struct(self)
            }
            fn reflect_mut(&mut self) -> ReflectMut<'_> {
                ReflectMut::Struct(self)
            }
        }
        impl<$($generics)*> ReflectFields for $type {
            fn field_count(&self) -> usize {
                [$($name),*].len()
            }
            fn field_name(&self, index: usize) -> Option<&'static str> {
                [$($name),*].as_slice().get(index).copied()
            }
            fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
                match index {
                    $($index => Some(&self.$member),)*
                    _ => None,
                }
            }
            fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
                match index {
                    $($index => Some(&mut self.$member),)*
                    _ => None,
                }
            }
        }
    };
}

twin_struct!([T: Typed, U: Typed] PairTwin<T, U>, 0 => "a" a, 1 => "b" b);
twin_struct!([] IgnoredTwin, 0 => "kept" kept, 1 => "last" last);
twin_struct!([] NodeTwin, 0 => "value" value, 1 => "children" children);

impl<T: Typed, U: Typed> Typed for PairTwin<T, U> {
    fn type_path() -> String {
        <Gen<T, U> as Typed>::type_path()
    }
    fn type_info() -> TypeInfo {
        struct_info::<Self>(StructKind::Named, vec![field::<T>("a"), field::<U>("b")])
    }
}

impl Typed for IgnoredTwin {
    fn type_path() -> String {
        <Ignored as Typed>::type_path()
    }
    fn type_info() -> TypeInfo {
        struct_info::<Self>(
            StructKind::Named,
            vec![field::<i16>("kept"), field::<char>("last")],
        )
    }
}

impl Typed for NodeTwin {
    fn type_path() -> String {
        <Node as Typed>::type_path()
    }
    fn type_info() -> TypeInfo {
        struct_info::<Self>(
            StructKind::Named,
            vec![field::<u32>("value"), field::<Vec<NodeTwin>>("children")],
        )
    }
}

// --------------------------------------------------------------------------- comparison

/// Names requested from every value: all field names of all test types plus one unknown name.
const NAMES: [&str; 18] = [
    "x", "count", "name", "tags", "limit", "0", "1", "2", "from", "speed", "target", "a", "b",
    "kept", "last", "scratch", "value", "children",
];

fn view(value: &dyn Reflect) -> Option<&dyn ReflectFields> {
    match value.reflect_ref() {
        ReflectRef::Struct(fields) => Some(fields),
        ReflectRef::Enum(e) => Some(e as &dyn ReflectFields),
        _ => None,
    }
}

/// The observable behaviour of a value by name: a hash of the value read back, or the error.
fn read(value: &dyn Reflect, name: &str) -> Result<(TypeInfo, u64), ReflectError> {
    value
        .field(name)
        .map(|f| (f.reflect_type_info(), reflect_state_hash(f)))
}

fn read_mut(value: &mut dyn Reflect, name: &str) -> Result<(TypeInfo, u64), ReflectError> {
    value
        .field_mut(name)
        .map(|f| (f.reflect_type_info(), reflect_state_hash(f)))
}

/// Asserts that a derived value and its twin behave identically (REFLD-R033).
fn assert_same<D: Typed, H: Typed>(mut derived: D, mut twin: H) {
    assert_eq!(D::type_info(), H::type_info());
    assert_eq!(derived.reflect_type_info(), twin.reflect_type_info());
    assert_eq!(validate_reflect(&derived), Ok(()));
    assert_eq!(validate_reflect(&twin), Ok(()));
    assert_eq!(reflect_state_hash(&derived), reflect_state_hash(&twin));
    let (d, h) = (
        view(&derived).map(|v| {
            (
                v.field_count(),
                (0..=v.field_count())
                    .map(|i| v.field_name(i))
                    .collect::<Vec<_>>(),
            )
        }),
        view(&twin).map(|v| {
            (
                v.field_count(),
                (0..=v.field_count())
                    .map(|i| v.field_name(i))
                    .collect::<Vec<_>>(),
            )
        }),
    );
    assert_eq!(d, h);
    for name in NAMES {
        assert_eq!(read(&derived, name), read(&twin, name), "field {name}");
        assert_eq!(
            read_mut(&mut derived, name),
            read_mut(&mut twin, name),
            "field_mut {name}"
        );
    }
    let mut derived_registry = TypeRegistry::new();
    let mut twin_registry = TypeRegistry::new();
    assert_eq!(
        derived_registry.register::<D>(),
        twin_registry.register::<H>()
    );
}

/// Writes `x` into field `name` and checks the round trip of REFL-R045: `x` reads back, every
/// other field is unchanged.
fn round_trip<F: Typed + Clone + PartialEq + Debug>(value: &mut dyn Reflect, name: &str, x: &F) {
    let names: Vec<&'static str> = view(value)
        .map(|v| {
            (0..v.field_count())
                .filter_map(|i| v.field_name(i))
                .collect()
        })
        .unwrap_or_default();
    let before: Vec<_> = names.iter().map(|n| read(value, n)).collect();
    match value
        .field_mut(name)
        .map(|f| f.downcast_mut::<F>().map(|f| *f = x.clone()))
    {
        Ok(Ok(())) => {}
        _ => panic!("field {name} is writable as {}", F::type_path()),
    }
    match value.field(name).map(|f| f.downcast_ref::<F>().cloned()) {
        Ok(Ok(read_back)) => assert_eq!(&read_back, x),
        _ => panic!("field {name} reads back"),
    }
    for (other, old) in names.iter().zip(before) {
        if *other != name {
            assert_eq!(read(value, other), old, "field {other} unchanged");
        }
    }
}

// --------------------------------------------------------------------------- strategies

fn sample_parts() -> impl Strategy<Value = (f32, u64, String, Vec<u8>, Option<i16>)> {
    (
        -1.0e6_f32..1.0e6,
        any::<u64>(),
        ".{0,8}",
        proptest::collection::vec(any::<u8>(), 0..4),
        any::<Option<i16>>(),
    )
}

fn mode_parts() -> impl Strategy<Value = (u8, u32, f32, f32)> {
    (0_u8..4, any::<u32>(), -1.0e6_f32..1.0e6, -1.0e6_f32..1.0e6)
}

fn modes((variant, n, f, g): (u8, u32, f32, f32)) -> (derived::Mode, types::Mode) {
    match variant {
        0 => (derived::Mode::Idle, types::Mode::Idle),
        1 => (derived::Mode::Moving(f, g), types::Mode::Moving(f, g)),
        2 => (
            derived::Mode::Fleeing { from: n, speed: f },
            types::Mode::Fleeing { from: n, speed: f },
        ),
        _ => (
            derived::Mode::Chasing {
                target: n,
                speed: f,
            },
            types::Mode::Chasing {
                target: n,
                speed: f,
            },
        ),
    }
}

fn nodes(values: &[u32]) -> (Node, NodeTwin) {
    let root = values
        .iter()
        .fold(0_u32, |sum, value| sum.wrapping_add(*value));
    let children = values
        .iter()
        .map(|value| Node {
            value: *value,
            children: Vec::new(),
        })
        .collect();
    let twin_children = values
        .iter()
        .map(|value| NodeTwin {
            value: *value,
            children: Vec::new(),
        })
        .collect();
    (
        Node {
            value: root,
            children,
        },
        NodeTwin {
            value: root,
            children: twin_children,
        },
    )
}

// -------------------------------------------------------------------------------- tests

/// Verifies: REFLD-R033
#[test]
fn recursive_twin_refers_to_the_derived_path() {
    let Shape::Struct { fields, .. } = NodeTwin::type_info().shape else {
        panic!("struct");
    };
    assert_eq!(
        fields[1].ty.type_path,
        format!("Vec<{}>", Node::type_path())
    );
    assert_eq!(fields[1].ty.id, TypeRef::of::<Vec<Node>>().id);
    assert_eq!(derived::Marker::type_info(), types::Marker::type_info());
    assert_same(derived::Marker, types::Marker);
}

proptest! {
    /// Verifies: REFLD-R033
    #[test]
    fn derived_and_hand_written_impls_agree(
        (x, count, name, tags, limit) in sample_parts(),
        pair in (any::<i32>(), any::<Option<u8>>()),
        mode in mode_parts(),
        generic in (any::<u8>(), ".{0,6}"),
        ignored in (any::<u64>(), any::<i16>(), any::<bool>(), any::<char>()),
        children in proptest::collection::vec(any::<u32>(), 0..4),
    ) {
        assert_same(
            derived::Sample { x, count, name: name.clone(), tags: tags.clone(), limit },
            types::Sample { x, count, name, tags, limit },
        );
        assert_same(derived::Pair(pair.0, pair.1), types::Pair(pair.0, pair.1));
        let (d, h) = modes(mode);
        assert_same(d, h);
        assert_same(
            Gen { a: generic.0, b: generic.1.clone() },
            PairTwin { a: generic.0, b: generic.1 },
        );
        let (scratch, kept, other, last) = ignored;
        assert_same(
            Ignored { scratch, kept, other, last },
            IgnoredTwin { scratch: 0, kept, other: !other, last },
        );
        let (d, h) = nodes(&children);
        assert_same(d, h);
    }

    /// Verifies: REFLD-R035
    #[test]
    fn derived_impls_meet_the_impl_requirements(
        (x, count, name, tags, limit) in sample_parts(),
        writes in sample_parts(),
        mode in mode_parts(),
        written in (any::<u32>(), -1.0e6_f32..1.0e6),
        values in (any::<u8>(), any::<i16>(), any::<char>(), any::<u32>()),
        children in proptest::collection::vec(any::<u32>(), 0..4),
    ) {
        let mut sample = derived::Sample { x, count, name, tags, limit };
        prop_assert_eq!(validate_reflect(&sample), Ok(()));
        round_trip(&mut sample, "x", &writes.0);
        round_trip(&mut sample, "count", &writes.1);
        round_trip(&mut sample, "name", &writes.2);
        round_trip(&mut sample, "tags", &writes.3);
        round_trip(&mut sample, "limit", &writes.4);
        prop_assert_eq!(sample.reflect_type_info(), derived::Sample::type_info());

        let mut pair = derived::Pair(1, None);
        round_trip(&mut pair, "0", &values.3.cast_signed());
        round_trip(&mut pair, "1", &Some(values.0));

        let (mut mode, _) = modes(mode);
        prop_assert_eq!(validate_reflect(&mode), Ok(()));
        match mode {
            derived::Mode::Idle => {}
            derived::Mode::Moving(..) => round_trip(&mut mode, "1", &written.1),
            derived::Mode::Fleeing { .. } => {
                round_trip(&mut mode, "from", &written.0);
                round_trip(&mut mode, "speed", &written.1);
            }
            derived::Mode::Chasing { .. } => round_trip(&mut mode, "target", &written.0),
        }

        let mut generic = Gen { a: values.0, b: values.2 };
        round_trip(&mut generic, "b", &'q');
        let mut ignored = Ignored { scratch: 1, kept: 2, other: true, last: 'a' };
        round_trip(&mut ignored, "kept", &values.1);
        round_trip(&mut ignored, "last", &values.2);
        let (mut node, _) = nodes(&children);
        prop_assert_eq!(validate_reflect(&node), Ok(()));
        round_trip(&mut node, "children", &Vec::<Node>::new());

        let mut registry = TypeRegistry::new();
        prop_assert!(registry.register::<derived::Sample>().is_ok());
        prop_assert!(registry.register::<derived::Pair>().is_ok());
        prop_assert!(registry.register::<derived::Marker>().is_ok());
        prop_assert!(registry.register::<derived::Mode>().is_ok());
        prop_assert!(registry.register::<Gen<u8, char>>().is_ok());
        prop_assert!(registry.register::<Ignored>().is_ok());
        prop_assert!(registry.register::<Node>().is_ok());
        prop_assert_eq!(registry.len(), 7);
    }
}
