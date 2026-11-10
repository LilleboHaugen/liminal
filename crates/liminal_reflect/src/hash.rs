//! The canonical state encoding of a reflected value (spec §4, REFL-R100 to REFL-R104),
//! written into `liminal_core::hash::StateHasher` (ADR-0009 item 6, ADR-0024).
//!
//! The encoding holds values only (no type paths, IDs or field names), by `reflect_ref`:
//!
//! | View | Bytes |
//! |---|---|
//! | `Primitive` | the matching `StateHasher` typed write (CORE-R053) |
//! | `Struct` | each field in order, no prefix |
//! | `Enum` | `variant_index()` as `u32` little-endian, then the variant's fields |
//! | `List` | `len()` as `u64` little-endian, then each element in order |

use liminal_core::hash::StateHasher;

use crate::value::{PrimitiveRef, Reflect, ReflectFields, ReflectRef};

/// Appends the canonical state encoding of `value` (REFL-R100) to `hasher`.
///
/// Reads the value only and does not call `reflect_type_info` (REFL-R104). For an impl whose
/// `field_at(i)` or `get(i)` returns `None` below the count, nothing is written for that index
/// (such an encoding is not canonical; `validate_reflect` reports the impl).
///
/// # Examples
///
/// ```
/// use liminal_core::hash::StateHasher;
/// use liminal_reflect::hash::write_reflect;
///
/// let mut hasher = StateHasher::new();
/// write_reflect(&Some(7_u8), &mut hasher);
///
/// let mut expected = StateHasher::new();
/// expected.write_bytes(&[1, 0, 0, 0, 7]); // variant index 1 as u32, then the u8
/// assert_eq!(hasher.finish(), expected.finish());
/// ```
pub fn write_reflect(value: &dyn Reflect, hasher: &mut StateHasher) {
    match value.reflect_ref() {
        ReflectRef::Primitive(primitive) => write_primitive(primitive, hasher),
        ReflectRef::Struct(fields) => write_fields(fields, hasher),
        ReflectRef::Enum(value) => {
            hasher.write_u32(value.variant_index());
            write_fields(value, hasher);
        }
        ReflectRef::List(list) => {
            hasher.write_usize(list.len());
            for index in 0..list.len() {
                if let Some(element) = list.get(index) {
                    write_reflect(element, hasher);
                }
            }
        }
    }
}

/// `let mut h = StateHasher::new(); write_reflect(value, &mut h); h.finish()` (REFL-R101).
///
/// # Examples
///
/// ```
/// use liminal_core::hash::state_hash_of;
/// use liminal_reflect::hash::reflect_state_hash;
///
/// // For primitives, `Vec` and arrays of primitives it equals `liminal_core`'s `StateHash`.
/// let value = vec![1.5_f32, -0.0];
/// assert_eq!(reflect_state_hash(&value), state_hash_of(&value));
/// ```
#[must_use]
pub fn reflect_state_hash(value: &dyn Reflect) -> u64 {
    let mut hasher = StateHasher::new();
    write_reflect(value, &mut hasher);
    hasher.finish()
}

fn write_fields<F: ReflectFields + ?Sized>(fields: &F, hasher: &mut StateHasher) {
    for index in 0..fields.field_count() {
        if let Some(field) = fields.field_at(index) {
            write_reflect(field, hasher);
        }
    }
}

fn write_primitive(primitive: PrimitiveRef<'_>, hasher: &mut StateHasher) {
    match primitive {
        PrimitiveRef::Bool(v) => hasher.write_bool(*v),
        PrimitiveRef::U8(v) => hasher.write_u8(*v),
        PrimitiveRef::U16(v) => hasher.write_u16(*v),
        PrimitiveRef::U32(v) => hasher.write_u32(*v),
        PrimitiveRef::U64(v) => hasher.write_u64(*v),
        PrimitiveRef::U128(v) => hasher.write_u128(*v),
        PrimitiveRef::Usize(v) => hasher.write_usize(*v),
        PrimitiveRef::I8(v) => hasher.write_i8(*v),
        PrimitiveRef::I16(v) => hasher.write_i16(*v),
        PrimitiveRef::I32(v) => hasher.write_i32(*v),
        PrimitiveRef::I64(v) => hasher.write_i64(*v),
        PrimitiveRef::I128(v) => hasher.write_i128(*v),
        PrimitiveRef::Isize(v) => hasher.write_isize(*v),
        PrimitiveRef::F32(v) => hasher.write_f32(*v),
        PrimitiveRef::F64(v) => hasher.write_f64(*v),
        PrimitiveRef::Char(v) => hasher.write_char(*v),
        PrimitiveRef::String(v) => hasher.write_str(v),
    }
}
