//! Type descriptors and stable type IDs (spec §5.1).
//!
//! A [`TypeInfo`] names a type by its type path and [`StableTypeId`] and describes its
//! [`Shape`]. Fields and elements refer to their types by [`TypeRef`], not by nested
//! descriptors, so descriptors of recursive types are finite.

use liminal_core::hash::StateHasher;

use crate::value::Typed;

/// XXH64 (seed 0) of a type path's UTF-8 bytes: an ID that is the same in every build, with
/// every compiler version, on every platform and in every process (REFL-R001, REFL-R005).
///
/// Unlike `core::any::TypeId` it depends only on the type path string. Moving or renaming a type
/// whose path is derived from its module changes the ID.
///
/// # Examples
///
/// ```
/// use liminal_core::hash::StateHasher;
/// use liminal_reflect::info::StableTypeId;
///
/// let id = StableTypeId::of_path("u8");
/// let mut hasher = StateHasher::new();
/// hasher.write_bytes(b"u8");
/// assert_eq!(id, StableTypeId::new(hasher.finish()));
/// assert_eq!(StableTypeId::new(0xab).to_string(), "00000000000000ab");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct StableTypeId(u64);

impl StableTypeId {
    /// The ID with the given value.
    #[must_use]
    pub const fn new(value: u64) -> StableTypeId {
        StableTypeId(value)
    }

    /// The ID of a type path: `StateHasher::new()`, `write_bytes(type_path.as_bytes())`,
    /// `finish()`.
    #[must_use]
    pub fn of_path(type_path: &str) -> StableTypeId {
        let mut hasher = StateHasher::new();
        hasher.write_bytes(type_path.as_bytes());
        StableTypeId(hasher.finish())
    }

    /// The ID as a `u64`.
    #[must_use]
    pub fn value(self) -> u64 {
        self.0
    }
}

/// Exactly 16 lowercase hexadecimal digits, zero-padded.
impl core::fmt::Display for StableTypeId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

/// A reference to another type by path and ID (fields, list elements).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeRef {
    /// The referenced type's path.
    pub type_path: String,
    /// The referenced type's stable ID.
    pub id: StableTypeId,
}

impl TypeRef {
    /// `TypeRef { type_path: T::type_path(), id: T::stable_type_id() }`.
    #[must_use]
    pub fn of<T: Typed>() -> TypeRef {
        TypeRef {
            type_path: T::type_path(),
            id: T::stable_type_id(),
        }
    }
}

/// The runtime description of one type: path, stable ID and shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeInfo {
    /// The type path (spec §4), for example `Vec<u8>` or `my_game::boids::Velocity`.
    pub type_path: String,
    /// `StableTypeId::of_path(&type_path)` for every consistent descriptor (REFL-R061).
    pub id: StableTypeId,
    /// What kind of value the type holds.
    pub shape: Shape,
}

/// The shape of a type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Shape {
    /// One of the primitive kinds, `String` included.
    Primitive(PrimitiveKind),
    /// A named, tuple or unit struct with its reflected fields in declaration order.
    Struct {
        /// Named, tuple or unit.
        kind: StructKind,
        /// The reflected fields in declaration order.
        fields: Vec<FieldInfo>,
    },
    /// An enum with its variants in declaration order.
    Enum {
        /// The variants in declaration order; the position is the variant index.
        variants: Vec<VariantInfo>,
    },
    /// Variable length (`Vec<T>`).
    List {
        /// The element type.
        element: TypeRef,
    },
    /// Fixed length (`[T; N]`).
    Array {
        /// The element type.
        element: TypeRef,
        /// The number of elements, `N`.
        len: usize,
    },
}

/// The form of a struct or enum variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructKind {
    /// Fields with identifiers (`{ x: f32 }`).
    Named,
    /// Positional fields (`(f32, f32)`), named `"0"`, `"1"`, ….
    Tuple,
    /// No fields.
    Unit,
}

/// One reflected field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldInfo {
    /// Identifier without `r#`, or the tuple position in decimal.
    pub name: &'static str,
    /// The field's type.
    pub ty: TypeRef,
}

/// One enum variant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VariantInfo {
    /// The variant's identifier.
    pub name: &'static str,
    /// Named, tuple or unit.
    pub kind: StructKind,
    /// The variant's reflected fields in declaration order.
    pub fields: Vec<FieldInfo>,
}

/// The 17 primitive kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveKind {
    /// `bool`.
    Bool,
    /// `u8`.
    U8,
    /// `u16`.
    U16,
    /// `u32`.
    U32,
    /// `u64`.
    U64,
    /// `u128`.
    U128,
    /// `usize`.
    Usize,
    /// `i8`.
    I8,
    /// `i16`.
    I16,
    /// `i32`.
    I32,
    /// `i64`.
    I64,
    /// `i128`.
    I128,
    /// `isize`.
    Isize,
    /// `f32`.
    F32,
    /// `f64`.
    F64,
    /// `char`.
    Char,
    /// `String`.
    String,
}

impl PrimitiveKind {
    /// The type path of the primitive type of this kind (REFL-R003).
    pub(crate) fn type_path(self) -> &'static str {
        match self {
            PrimitiveKind::Bool => "bool",
            PrimitiveKind::U8 => "u8",
            PrimitiveKind::U16 => "u16",
            PrimitiveKind::U32 => "u32",
            PrimitiveKind::U64 => "u64",
            PrimitiveKind::U128 => "u128",
            PrimitiveKind::Usize => "usize",
            PrimitiveKind::I8 => "i8",
            PrimitiveKind::I16 => "i16",
            PrimitiveKind::I32 => "i32",
            PrimitiveKind::I64 => "i64",
            PrimitiveKind::I128 => "i128",
            PrimitiveKind::Isize => "isize",
            PrimitiveKind::F32 => "f32",
            PrimitiveKind::F64 => "f64",
            PrimitiveKind::Char => "char",
            PrimitiveKind::String => "String",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies: REFL-R001
    #[test]
    fn display_is_sixteen_lowercase_hex_digits() {
        assert_eq!(StableTypeId::new(0).to_string(), "0000000000000000");
        assert_eq!(
            StableTypeId::new(u64::MAX).to_string(),
            "ffffffffffffffff"
        );
        assert_eq!(
            StableTypeId::new(0x0123_4567_89ab_cdef).to_string(),
            "0123456789abcdef"
        );
    }
}
