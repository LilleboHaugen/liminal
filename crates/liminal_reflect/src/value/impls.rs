//! Hand-written `Reflect` and `Typed` impls (REFL-R003, REFL-R004, REFL-R020, REFL-R040):
//! primitives, `String`, `Vec<T>`, `[T; N]` and `Option<T>`.

use crate::info::{FieldInfo, PrimitiveKind, Shape, StructKind, TypeInfo, TypeRef, VariantInfo};
use crate::value::{
    PrimitiveRef, Reflect, ReflectEnum, ReflectFields, ReflectList, ReflectMut, ReflectRef, Typed,
};

/// Implements `Reflect` and `Typed` for primitive types; the type path is the kind's path.
macro_rules! primitive_impls {
    ($($type:ty => $kind:ident;)*) => {
        $(
            impl Reflect for $type {
                fn reflect_type_info(&self) -> TypeInfo {
                    <$type as Typed>::type_info()
                }
                fn reflect_ref(&self) -> ReflectRef<'_> {
                    ReflectRef::Primitive(PrimitiveRef::$kind(self))
                }
                fn reflect_mut(&mut self) -> ReflectMut<'_> {
                    ReflectMut::Primitive(PrimitiveKind::$kind)
                }
            }

            impl Typed for $type {
                fn type_path() -> String {
                    PrimitiveKind::$kind.type_path().to_string()
                }
                fn type_info() -> TypeInfo {
                    TypeInfo {
                        type_path: Self::type_path(),
                        id: Self::stable_type_id(),
                        shape: Shape::Primitive(PrimitiveKind::$kind),
                    }
                }
            }
        )*
    };
}

primitive_impls! {
    bool => Bool;
    u8 => U8;
    u16 => U16;
    u32 => U32;
    u64 => U64;
    u128 => U128;
    usize => Usize;
    i8 => I8;
    i16 => I16;
    i32 => I32;
    i64 => I64;
    i128 => I128;
    isize => Isize;
    f32 => F32;
    f64 => F64;
    char => Char;
    String => String;
}

impl<T: Typed> Reflect for Vec<T> {
    fn reflect_type_info(&self) -> TypeInfo {
        <Vec<T> as Typed>::type_info()
    }
    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::List(self)
    }
    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::List(self)
    }
}

impl<T: Typed> Typed for Vec<T> {
    fn type_path() -> String {
        format!("Vec<{}>", T::type_path())
    }
    fn type_info() -> TypeInfo {
        TypeInfo {
            type_path: Self::type_path(),
            id: Self::stable_type_id(),
            shape: Shape::List {
                element: TypeRef::of::<T>(),
            },
        }
    }
}

impl<T: Typed> ReflectList for Vec<T> {
    fn len(&self) -> usize {
        self.as_slice().len()
    }
    fn get(&self, index: usize) -> Option<&dyn Reflect> {
        self.as_slice()
            .get(index)
            .map(|element| element as &dyn Reflect)
    }
    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        self.as_mut_slice()
            .get_mut(index)
            .map(|element| element as &mut dyn Reflect)
    }
}

impl<T: Typed, const N: usize> Reflect for [T; N] {
    fn reflect_type_info(&self) -> TypeInfo {
        <[T; N] as Typed>::type_info()
    }
    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::List(self)
    }
    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::List(self)
    }
}

impl<T: Typed, const N: usize> Typed for [T; N] {
    fn type_path() -> String {
        format!("[{}; {N}]", T::type_path())
    }
    fn type_info() -> TypeInfo {
        TypeInfo {
            type_path: Self::type_path(),
            id: Self::stable_type_id(),
            shape: Shape::Array {
                element: TypeRef::of::<T>(),
                len: N,
            },
        }
    }
}

impl<T: Typed, const N: usize> ReflectList for [T; N] {
    fn len(&self) -> usize {
        N
    }
    fn get(&self, index: usize) -> Option<&dyn Reflect> {
        self.as_slice()
            .get(index)
            .map(|element| element as &dyn Reflect)
    }
    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        self.as_mut_slice()
            .get_mut(index)
            .map(|element| element as &mut dyn Reflect)
    }
}

impl<T: Typed> Reflect for Option<T> {
    fn reflect_type_info(&self) -> TypeInfo {
        <Option<T> as Typed>::type_info()
    }
    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::Enum(self)
    }
    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::Enum(self)
    }
}

impl<T: Typed> Typed for Option<T> {
    fn type_path() -> String {
        format!("Option<{}>", T::type_path())
    }
    fn type_info() -> TypeInfo {
        TypeInfo {
            type_path: Self::type_path(),
            id: Self::stable_type_id(),
            shape: Shape::Enum {
                variants: vec![
                    VariantInfo {
                        name: "None",
                        kind: StructKind::Unit,
                        fields: Vec::new(),
                    },
                    VariantInfo {
                        name: "Some",
                        kind: StructKind::Tuple,
                        fields: vec![FieldInfo {
                            name: "0",
                            ty: TypeRef::of::<T>(),
                        }],
                    },
                ],
            },
        }
    }
}

impl<T: Typed> ReflectFields for Option<T> {
    fn field_count(&self) -> usize {
        usize::from(self.is_some())
    }
    fn field_name(&self, index: usize) -> Option<&'static str> {
        (index < self.field_count()).then_some("0")
    }
    fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
        match (self, index) {
            (Some(value), 0) => Some(value),
            _ => None,
        }
    }
    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        match (self, index) {
            (Some(value), 0) => Some(value),
            _ => None,
        }
    }
}

impl<T: Typed> ReflectEnum for Option<T> {
    fn variant_index(&self) -> u32 {
        u32::from(self.is_some())
    }
    fn variant_name(&self) -> &'static str {
        if self.is_some() { "Some" } else { "None" }
    }
}
