//! Hand-written test types shared by the integration tests (spec §6 "Assignment": M1-T031 tests
//! against hand-written impls; M1-T032 keeps them as the reference of REFLD-R033).
//!
//! - [`Sample`]: named struct `refl_test::Sample` (REFL-R005 path).
//! - [`Pair`]: tuple struct; [`Marker`]: unit struct; [`Mode`]: enum with unit, tuple and struct
//!   variants.
//! - [`Fake`]: a value whose descriptor and views are set by the test, to build deliberately
//!   wrong impls (REFL-R043, REFL-R080, REFL-R100, REFL-R104).

// Each test binary includes this file and uses a subset of it.
#![allow(dead_code)]

use std::cell::Cell;

use liminal_reflect::info::{
    FieldInfo, PrimitiveKind, Shape, StableTypeId, StructKind, TypeInfo, TypeRef, VariantInfo,
};
use liminal_reflect::value::{
    PrimitiveRef, Reflect, ReflectEnum, ReflectFields, ReflectList, ReflectMut, ReflectRef, Typed,
};

/// A field descriptor.
pub fn field<T: Typed>(name: &'static str) -> FieldInfo {
    FieldInfo {
        name,
        ty: TypeRef::of::<T>(),
    }
}

/// A descriptor whose ID is the hash of its path.
pub fn info(type_path: &str, shape: Shape) -> TypeInfo {
    TypeInfo {
        type_path: type_path.to_string(),
        id: StableTypeId::of_path(type_path),
        shape,
    }
}

/// Implements `Reflect` (struct or enum view) by delegating to `Typed::type_info`.
macro_rules! reflect_via_typed {
    ($type:ty, $view:ident) => {
        impl Reflect for $type {
            fn reflect_type_info(&self) -> TypeInfo {
                <$type as Typed>::type_info()
            }
            fn reflect_ref(&self) -> ReflectRef<'_> {
                ReflectRef::$view(self)
            }
            fn reflect_mut(&mut self) -> ReflectMut<'_> {
                ReflectMut::$view(self)
            }
        }
    };
}

// ------------------------------------------------------------------------------------ Sample

/// Named struct with a primitive, a string, a list and an option.
#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub x: f32,
    pub count: u64,
    pub name: String,
    pub tags: Vec<u8>,
    pub limit: Option<i16>,
}

pub const SAMPLE_FIELDS: [&str; 5] = ["x", "count", "name", "tags", "limit"];

reflect_via_typed!(Sample, Struct);

impl Typed for Sample {
    fn type_path() -> String {
        "refl_test::Sample".to_string()
    }
    fn type_info() -> TypeInfo {
        info(
            "refl_test::Sample",
            Shape::Struct {
                kind: StructKind::Named,
                fields: vec![
                    field::<f32>("x"),
                    field::<u64>("count"),
                    field::<String>("name"),
                    field::<Vec<u8>>("tags"),
                    field::<Option<i16>>("limit"),
                ],
            },
        )
    }
}

impl ReflectFields for Sample {
    fn field_count(&self) -> usize {
        SAMPLE_FIELDS.len()
    }
    fn field_name(&self, index: usize) -> Option<&'static str> {
        SAMPLE_FIELDS.get(index).copied()
    }
    fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
        match index {
            0 => Some(&self.x),
            1 => Some(&self.count),
            2 => Some(&self.name),
            3 => Some(&self.tags),
            4 => Some(&self.limit),
            _ => None,
        }
    }
    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        match index {
            0 => Some(&mut self.x),
            1 => Some(&mut self.count),
            2 => Some(&mut self.name),
            3 => Some(&mut self.tags),
            4 => Some(&mut self.limit),
            _ => None,
        }
    }
}

// -------------------------------------------------------------------------------------- Pair

/// Tuple struct.
#[derive(Clone, Debug, PartialEq)]
pub struct Pair(pub i32, pub Option<u8>);

reflect_via_typed!(Pair, Struct);

impl Typed for Pair {
    fn type_path() -> String {
        "refl_test::Pair".to_string()
    }
    fn type_info() -> TypeInfo {
        info(
            "refl_test::Pair",
            Shape::Struct {
                kind: StructKind::Tuple,
                fields: vec![field::<i32>("0"), field::<Option<u8>>("1")],
            },
        )
    }
}

impl ReflectFields for Pair {
    fn field_count(&self) -> usize {
        2
    }
    fn field_name(&self, index: usize) -> Option<&'static str> {
        ["0", "1"].get(index).copied()
    }
    fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
        match index {
            0 => Some(&self.0),
            1 => Some(&self.1),
            _ => None,
        }
    }
    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        match index {
            0 => Some(&mut self.0),
            1 => Some(&mut self.1),
            _ => None,
        }
    }
}

// ------------------------------------------------------------------------------------ Marker

/// Unit struct.
#[derive(Clone, Debug, PartialEq)]
pub struct Marker;

reflect_via_typed!(Marker, Struct);

impl Typed for Marker {
    fn type_path() -> String {
        "refl_test::Marker".to_string()
    }
    fn type_info() -> TypeInfo {
        info(
            "refl_test::Marker",
            Shape::Struct {
                kind: StructKind::Unit,
                fields: Vec::new(),
            },
        )
    }
}

impl ReflectFields for Marker {
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

// -------------------------------------------------------------------------------------- Mode

/// Enum with a unit, a tuple and a struct variant. `Chasing` and `Fleeing` both have a field
/// `speed`; only `Fleeing` has `from`.
#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    Idle,
    Moving(f32, f32),
    Fleeing { from: u32, speed: f32 },
    Chasing { target: u32, speed: f32 },
}

reflect_via_typed!(Mode, Enum);

impl Typed for Mode {
    fn type_path() -> String {
        "refl_test::Mode".to_string()
    }
    fn type_info() -> TypeInfo {
        let variant = |name, kind, fields| VariantInfo { name, kind, fields };
        info(
            "refl_test::Mode",
            Shape::Enum {
                variants: vec![
                    variant("Idle", StructKind::Unit, Vec::new()),
                    variant(
                        "Moving",
                        StructKind::Tuple,
                        vec![field::<f32>("0"), field::<f32>("1")],
                    ),
                    variant(
                        "Fleeing",
                        StructKind::Named,
                        vec![field::<u32>("from"), field::<f32>("speed")],
                    ),
                    variant(
                        "Chasing",
                        StructKind::Named,
                        vec![field::<u32>("target"), field::<f32>("speed")],
                    ),
                ],
            },
        )
    }
}

impl Mode {
    fn names(&self) -> &'static [&'static str] {
        match self {
            Mode::Idle => &[],
            Mode::Moving(..) => &["0", "1"],
            Mode::Fleeing { .. } => &["from", "speed"],
            Mode::Chasing { .. } => &["target", "speed"],
        }
    }
}

impl ReflectFields for Mode {
    fn field_count(&self) -> usize {
        self.names().len()
    }
    fn field_name(&self, index: usize) -> Option<&'static str> {
        self.names().get(index).copied()
    }
    fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
        match (self, index) {
            (Mode::Moving(a, _), 0) => Some(a),
            (Mode::Moving(_, b), 1) => Some(b),
            (Mode::Fleeing { from, .. }, 0) => Some(from),
            (Mode::Fleeing { speed, .. } | Mode::Chasing { speed, .. }, 1) => Some(speed),
            (Mode::Chasing { target, .. }, 0) => Some(target),
            _ => None,
        }
    }
    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        match (self, index) {
            (Mode::Moving(a, _), 0) => Some(a),
            (Mode::Moving(_, b), 1) => Some(b),
            (Mode::Fleeing { from, .. }, 0) => Some(from),
            (Mode::Fleeing { speed, .. } | Mode::Chasing { speed, .. }, 1) => Some(speed),
            (Mode::Chasing { target, .. }, 0) => Some(target),
            _ => None,
        }
    }
}

impl ReflectEnum for Mode {
    fn variant_index(&self) -> u32 {
        match self {
            Mode::Idle => 0,
            Mode::Moving(..) => 1,
            Mode::Fleeing { .. } => 2,
            Mode::Chasing { .. } => 3,
        }
    }
    fn variant_name(&self) -> &'static str {
        ["Idle", "Moving", "Fleeing", "Chasing"][self.variant_index() as usize]
    }
}

// -------------------------------------------------------------------------------------- Fake

/// Which view a [`Fake`] returns from `reflect_ref` / `reflect_mut`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    Struct,
    Enum,
    List,
    Primitive,
}

/// A value whose descriptor and views are chosen by the test. As a struct or enum its fields,
/// as a list its elements, are `values` (`u32`; `None` is a counted but missing value).
#[derive(Clone, Debug)]
pub struct Fake {
    pub info: TypeInfo,
    pub view: View,
    /// `field_count()` / `len()`.
    pub count: usize,
    /// `field_name(i)` is `names[i]` (`None` past the end).
    pub names: Vec<Option<&'static str>>,
    pub values: Vec<Option<u32>>,
    pub variant_index: u32,
    pub variant_name: &'static str,
    /// The value of the `Primitive` view (`u8`).
    pub primitive: u8,
    /// How often `reflect_type_info` was called.
    pub info_calls: Cell<usize>,
}

impl Fake {
    /// A consistent named struct `refl_test::Fake` with `u32` fields `names` and `values`.
    pub fn named(names: &[&'static str], values: &[u32]) -> Fake {
        Fake {
            info: info(
                "refl_test::Fake",
                Shape::Struct {
                    kind: StructKind::Named,
                    fields: names.iter().map(|name| field::<u32>(name)).collect(),
                },
            ),
            view: View::Struct,
            count: names.len(),
            names: names.iter().map(|name| Some(*name)).collect(),
            values: values.iter().map(|value| Some(*value)).collect(),
            variant_index: 0,
            variant_name: "",
            primitive: 0,
            info_calls: Cell::new(0),
        }
    }

    /// A consistent `Vec<u32>`-shaped list `refl_test::FakeList`.
    pub fn list(values: &[u32]) -> Fake {
        Fake {
            info: info(
                "refl_test::FakeList",
                Shape::List {
                    element: TypeRef::of::<u32>(),
                },
            ),
            view: View::List,
            count: values.len(),
            names: Vec::new(),
            values: values.iter().map(|value| Some(*value)).collect(),
            variant_index: 0,
            variant_name: "",
            primitive: 0,
            info_calls: Cell::new(0),
        }
    }

    /// A consistent enum `refl_test::FakeEnum` with variants `A { x }` and `B { y }`, currently
    /// `A` with `x = value`.
    pub fn enumeration(value: u32) -> Fake {
        Fake {
            info: info(
                "refl_test::FakeEnum",
                Shape::Enum {
                    variants: vec![
                        VariantInfo {
                            name: "A",
                            kind: StructKind::Named,
                            fields: vec![field::<u32>("x")],
                        },
                        VariantInfo {
                            name: "B",
                            kind: StructKind::Named,
                            fields: vec![field::<u32>("y")],
                        },
                    ],
                },
            ),
            view: View::Enum,
            count: 1,
            names: vec![Some("x")],
            values: vec![Some(value)],
            variant_index: 0,
            variant_name: "A",
            primitive: 0,
            info_calls: Cell::new(0),
        }
    }

    /// A consistent primitive `u8`-shaped value `refl_test::FakeU8`.
    pub fn primitive(value: u8) -> Fake {
        Fake {
            info: info("refl_test::FakeU8", Shape::Primitive(PrimitiveKind::U8)),
            view: View::Primitive,
            count: 0,
            names: Vec::new(),
            values: Vec::new(),
            variant_index: 0,
            variant_name: "",
            primitive: value,
            info_calls: Cell::new(0),
        }
    }

    fn value(&self, index: usize) -> Option<&dyn Reflect> {
        if index >= self.count {
            return None;
        }
        self.values
            .as_slice()
            .get(index)
            .and_then(Option::as_ref)
            .map(|value| value as &dyn Reflect)
    }

    fn value_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        if index >= self.count {
            return None;
        }
        self.values
            .as_mut_slice()
            .get_mut(index)
            .and_then(Option::as_mut)
            .map(|value| value as &mut dyn Reflect)
    }
}

impl Reflect for Fake {
    fn reflect_type_info(&self) -> TypeInfo {
        self.info_calls.set(self.info_calls.get() + 1);
        self.info.clone()
    }
    fn reflect_ref(&self) -> ReflectRef<'_> {
        match self.view {
            View::Struct => ReflectRef::Struct(self),
            View::Enum => ReflectRef::Enum(self),
            View::List => ReflectRef::List(self),
            View::Primitive => ReflectRef::Primitive(PrimitiveRef::U8(&self.primitive)),
        }
    }
    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        match self.view {
            View::Struct => ReflectMut::Struct(self),
            View::Enum => ReflectMut::Enum(self),
            View::List => ReflectMut::List(self),
            View::Primitive => ReflectMut::Primitive(PrimitiveKind::U8),
        }
    }
}

/// The static descriptor is that of `Fake::named(&["x", "y"], …)`; a value may report another
/// one through `reflect_type_info` (its `info` field).
impl Typed for Fake {
    fn type_path() -> String {
        "refl_test::Fake".to_string()
    }
    fn type_info() -> TypeInfo {
        Fake::named(&["x", "y"], &[]).info
    }
}

impl ReflectFields for Fake {
    fn field_count(&self) -> usize {
        self.count
    }
    fn field_name(&self, index: usize) -> Option<&'static str> {
        self.names.get(index).copied().flatten()
    }
    fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
        self.value(index)
    }
    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        self.value_mut(index)
    }
}

impl ReflectEnum for Fake {
    fn variant_index(&self) -> u32 {
        self.variant_index
    }
    fn variant_name(&self) -> &'static str {
        self.variant_name
    }
}

impl ReflectList for Fake {
    fn len(&self) -> usize {
        self.count
    }
    fn get(&self, index: usize) -> Option<&dyn Reflect> {
        self.value(index)
    }
    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        self.value_mut(index)
    }
}
