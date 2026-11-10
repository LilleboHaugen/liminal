//! Runtime views of values (spec §5.2): the object-safe [`Reflect`] trait, the static [`Typed`]
//! trait, the access traits [`ReflectFields`], [`ReflectEnum`] and [`ReflectList`], the views
//! [`ReflectRef`], [`ReflectMut`] and [`PrimitiveRef`], field access by name and checked
//! downcasts on `dyn Reflect`, and the consistency check [`validate_reflect`].
//!
//! Hand-written impls exist for `bool`, the integer types, `f32`, `f64`, `char`, `String`,
//! `Vec<T>`, `[T; N]` (every `N`) and `Option<T>`, for `T: Typed`.

mod access;
mod impls;
mod validate;

use crate::info::{PrimitiveKind, StableTypeId, TypeInfo};

pub use validate::validate_reflect;

/// Object-safe runtime view of a value. `'static` through [`core::any::Any`].
///
/// A type that implements `Reflect` also implements [`Typed`]; the derive macro (M1-T032)
/// writes both. Hand-written impls must keep `reflect_ref` consistent with the descriptor;
/// [`validate_reflect`] checks it.
///
/// # Examples
///
/// A hand-written impl of a named struct, then field access by name:
///
/// ```
/// use liminal_reflect::error::ReflectError;
/// use liminal_reflect::info::{FieldInfo, Shape, StructKind, TypeInfo, TypeRef};
/// use liminal_reflect::value::{Reflect, ReflectFields, ReflectMut, ReflectRef, Typed};
///
/// struct Velocity { x: f32, y: f32 }
///
/// impl Reflect for Velocity {
///     fn reflect_type_info(&self) -> TypeInfo { Velocity::type_info() }
///     fn reflect_ref(&self) -> ReflectRef<'_> { ReflectRef::Struct(self) }
///     fn reflect_mut(&mut self) -> ReflectMut<'_> { ReflectMut::Struct(self) }
/// }
///
/// impl Typed for Velocity {
///     fn type_path() -> String { "my_game::Velocity".to_string() }
///     fn type_info() -> TypeInfo {
///         let field = |name| FieldInfo { name, ty: TypeRef::of::<f32>() };
///         TypeInfo {
///             type_path: Velocity::type_path(),
///             id: Velocity::stable_type_id(),
///             shape: Shape::Struct { kind: StructKind::Named, fields: vec![field("x"), field("y")] },
///         }
///     }
/// }
///
/// impl ReflectFields for Velocity {
///     fn field_count(&self) -> usize { 2 }
///     fn field_name(&self, index: usize) -> Option<&'static str> {
///         ["x", "y"].get(index).copied()
///     }
///     fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
///         match index { 0 => Some(&self.x), 1 => Some(&self.y), _ => None }
///     }
///     fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
///         match index { 0 => Some(&mut self.x), 1 => Some(&mut self.y), _ => None }
///     }
/// }
///
/// fn main() -> Result<(), ReflectError> {
///     let mut v = Velocity { x: 1.0, y: 2.0 };
///     let r: &mut dyn Reflect = &mut v;
///     *r.field_mut("y")?.downcast_mut::<f32>()? = 5.0;
///     assert_eq!(*r.field("y")?.downcast_ref::<f32>()?, 5.0);
///     // LIM-REFL-0010.
///     assert!(matches!(r.field("z"), Err(ReflectError::NoSuchField { .. })));
///     assert_eq!(v.y, 5.0);
///     Ok(())
/// }
/// ```
pub trait Reflect: core::any::Any {
    /// Equals `<Self as Typed>::type_info()` for the concrete type.
    fn reflect_type_info(&self) -> TypeInfo;
    /// Read access to the value's fields, variant, elements or primitive value.
    fn reflect_ref(&self) -> ReflectRef<'_>;
    /// Write access; the same variant as [`Reflect::reflect_ref`].
    fn reflect_mut(&mut self) -> ReflectMut<'_>;
}

/// Static type information. Implemented together with [`Reflect`] (derive or by hand).
///
/// # Examples
///
/// ```
/// use liminal_reflect::info::StableTypeId;
/// use liminal_reflect::value::Typed;
///
/// assert_eq!(<Option<[u8; 4]>>::type_path(), "Option<[u8; 4]>");
/// assert_eq!(u8::stable_type_id(), StableTypeId::of_path("u8"));
/// assert_eq!(u8::type_info().id, u8::stable_type_id());
/// ```
pub trait Typed: Reflect + Sized {
    /// The type path (spec §4).
    fn type_path() -> String;
    /// The descriptor; its `type_path` is [`Typed::type_path`] and its `id`
    /// [`Typed::stable_type_id`].
    fn type_info() -> TypeInfo;
    /// `StableTypeId::of_path(&Self::type_path())`.
    #[must_use]
    fn stable_type_id() -> StableTypeId {
        StableTypeId::of_path(&Self::type_path())
    }
}

/// Structs (named, tuple, unit) and the fields of an enum's current variant.
pub trait ReflectFields {
    /// The number of reflected fields.
    fn field_count(&self) -> usize;
    /// `Some` for `index < field_count()`.
    fn field_name(&self, index: usize) -> Option<&'static str>;
    /// The field at `index`; `Some` for `index < field_count()`.
    fn field_at(&self, index: usize) -> Option<&dyn Reflect>;
    /// The field at `index`, mutable; `Some` for `index < field_count()`.
    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;
}

/// Enums: the current variant, and through [`ReflectFields`] its fields.
pub trait ReflectEnum: ReflectFields {
    /// 0-based declaration position of the current variant.
    fn variant_index(&self) -> u32;
    /// The current variant's name.
    fn variant_name(&self) -> &'static str;
}

/// `Vec<T>` and `[T; N]`. No insertion or removal (spec §2).
pub trait ReflectList {
    /// The number of elements.
    fn len(&self) -> usize;
    /// `len() == 0`.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Element `index`; `Some` for `index < len()`.
    fn get(&self, index: usize) -> Option<&dyn Reflect>;
    /// Element `index`, mutable; `Some` for `index < len()`.
    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;
}

/// Read view of a value, returned by [`Reflect::reflect_ref`].
pub enum ReflectRef<'a> {
    /// A struct.
    Struct(&'a dyn ReflectFields),
    /// An enum.
    Enum(&'a dyn ReflectEnum),
    /// `Vec<T>` and `[T; N]`.
    List(&'a dyn ReflectList),
    /// A primitive value.
    Primitive(PrimitiveRef<'a>),
}

/// Write view of a value, returned by [`Reflect::reflect_mut`].
///
/// Primitive values are changed through `dyn Reflect::downcast_mut`, so `Primitive` carries only
/// the kind (one way to write a leaf).
pub enum ReflectMut<'a> {
    /// A struct.
    Struct(&'a mut dyn ReflectFields),
    /// An enum.
    Enum(&'a mut dyn ReflectEnum),
    /// `Vec<T>` and `[T; N]`.
    List(&'a mut dyn ReflectList),
    /// A primitive value of this kind.
    Primitive(PrimitiveKind),
}

/// A reference to a primitive value, by kind.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PrimitiveRef<'a> {
    /// A `bool`.
    Bool(&'a bool),
    /// A `u8`.
    U8(&'a u8),
    /// A `u16`.
    U16(&'a u16),
    /// A `u32`.
    U32(&'a u32),
    /// A `u64`.
    U64(&'a u64),
    /// A `u128`.
    U128(&'a u128),
    /// A `usize`.
    Usize(&'a usize),
    /// A `i8`.
    I8(&'a i8),
    /// A `i16`.
    I16(&'a i16),
    /// A `i32`.
    I32(&'a i32),
    /// A `i64`.
    I64(&'a i64),
    /// A `i128`.
    I128(&'a i128),
    /// A `isize`.
    Isize(&'a isize),
    /// A `f32`.
    F32(&'a f32),
    /// A `f64`.
    F64(&'a f64),
    /// A `char`.
    Char(&'a char),
    /// A `String`.
    String(&'a String),
}

impl PrimitiveRef<'_> {
    /// The kind of the referenced value.
    #[must_use]
    pub fn kind(&self) -> PrimitiveKind {
        match self {
            PrimitiveRef::Bool(_) => PrimitiveKind::Bool,
            PrimitiveRef::U8(_) => PrimitiveKind::U8,
            PrimitiveRef::U16(_) => PrimitiveKind::U16,
            PrimitiveRef::U32(_) => PrimitiveKind::U32,
            PrimitiveRef::U64(_) => PrimitiveKind::U64,
            PrimitiveRef::U128(_) => PrimitiveKind::U128,
            PrimitiveRef::Usize(_) => PrimitiveKind::Usize,
            PrimitiveRef::I8(_) => PrimitiveKind::I8,
            PrimitiveRef::I16(_) => PrimitiveKind::I16,
            PrimitiveRef::I32(_) => PrimitiveKind::I32,
            PrimitiveRef::I64(_) => PrimitiveKind::I64,
            PrimitiveRef::I128(_) => PrimitiveKind::I128,
            PrimitiveRef::Isize(_) => PrimitiveKind::Isize,
            PrimitiveRef::F32(_) => PrimitiveKind::F32,
            PrimitiveRef::F64(_) => PrimitiveKind::F64,
            PrimitiveRef::Char(_) => PrimitiveKind::Char,
            PrimitiveRef::String(_) => PrimitiveKind::String,
        }
    }
}
