//! Field access by name and checked downcasts on `dyn Reflect` (REFL-R043, REFL-R044).

use crate::error::ReflectError;
use crate::info::Shape;
use crate::value::{Reflect, ReflectFields, ReflectMut, ReflectRef, Typed};

/// Why a name lookup found no field index. Owns nothing, so the error can be built after the
/// borrow of the value ends.
#[derive(Clone, Copy)]
enum Miss {
    /// Struct without a field of that name.
    Struct,
    /// Enum whose current variant (this name) lacks the field.
    Enum(&'static str),
    /// `"list"` or `"primitive"`.
    NoFields(&'static str),
}

/// The lowest index below `field_count()` whose `field_name` is `name` (byte-wise).
fn find_index<F: ReflectFields + ?Sized>(fields: &F, name: &str) -> Option<usize> {
    (0..fields.field_count()).find(|&index| fields.field_name(index) == Some(name))
}

impl dyn Reflect {
    /// The field named `name` (REFL-R043): on a struct its field, on an enum the field of the
    /// current variant. Names compare byte-wise; tuple fields are `"0"`, `"1"`, ….
    ///
    /// # Errors
    ///
    /// - [`ReflectError::NoSuchField`] (LIM-REFL-0010): no field of that name, or the matching
    ///   index's `field_at` returns `None`.
    /// - [`ReflectError::NoFields`] (LIM-REFL-0012): the value is a list or a primitive.
    /// - [`ReflectError::FieldNotInVariant`] (LIM-REFL-0013): the enum's current variant lacks
    ///   the field, but another variant has it.
    ///
    /// # Examples
    ///
    /// ```
    /// use liminal_reflect::error::ReflectError;
    /// use liminal_reflect::value::Reflect;
    ///
    /// let some: &dyn Reflect = &Some(7_u8);
    /// assert_eq!(some.field("0")?.downcast_ref::<u8>()?, &7);
    /// let none: &dyn Reflect = &None::<u8>;
    /// assert!(matches!(none.field("0"), Err(ReflectError::FieldNotInVariant { .. })));
    /// # Ok::<(), ReflectError>(())
    /// ```
    pub fn field(&self, name: &str) -> Result<&dyn Reflect, ReflectError> {
        let found = match self.reflect_ref() {
            ReflectRef::Struct(fields) => find_index(fields, name)
                .map(|index| fields.field_at(index))
                .ok_or(Miss::Struct),
            ReflectRef::Enum(value) => find_index(value, name)
                .map(|index| value.field_at(index))
                .ok_or(Miss::Enum(value.variant_name())),
            ReflectRef::List(_) => Err(Miss::NoFields("list")),
            ReflectRef::Primitive(_) => Err(Miss::NoFields("primitive")),
        };
        match found {
            Ok(Some(field)) => Ok(field),
            Ok(None) => Err(self.no_such_field(name)),
            Err(miss) => Err(self.miss_error(miss, name)),
        }
    }

    /// The field named `name`, mutable. Same lookup and errors as `field`, with
    /// `field_at_mut` in place of `field_at`.
    ///
    /// On success it builds the value's [`TypeInfo`](crate::info::TypeInfo) once (for the
    /// error of an impl whose `field_at_mut` returns `None` at a named index).
    ///
    /// # Errors
    ///
    /// LIM-REFL-0010, 0012, 0013, as for `field`.
    pub fn field_mut(&mut self, name: &str) -> Result<&mut dyn Reflect, ReflectError> {
        let index = match self.reflect_mut() {
            ReflectMut::Struct(fields) => find_index(&*fields, name).ok_or(Miss::Struct),
            ReflectMut::Enum(value) => {
                find_index(&*value, name).ok_or(Miss::Enum(value.variant_name()))
            }
            ReflectMut::List(_) => Err(Miss::NoFields("list")),
            ReflectMut::Primitive(_) => Err(Miss::NoFields("primitive")),
        };
        let index = match index {
            Ok(index) => index,
            Err(miss) => return Err(self.miss_error(miss, name)),
        };
        // Built before the mutable borrow: the borrow checker does not let the `None` branch
        // use `self` while the `Some` branch returns a borrow of it.
        let missing = self.no_such_field(name);
        let field = match self.reflect_mut() {
            ReflectMut::Struct(fields) => fields.field_at_mut(index),
            ReflectMut::Enum(value) => value.field_at_mut(index),
            ReflectMut::List(_) | ReflectMut::Primitive(_) => None,
        };
        field.ok_or(missing)
    }

    /// The value as `&T`, if its concrete type is `T` (a `core::any::Any` check, not the
    /// stable ID; REFL-R044).
    ///
    /// # Errors
    ///
    /// [`ReflectError::TypeMismatch`] (LIM-REFL-0011) with `expected = T::type_path()` and
    /// `actual` the value's type path.
    ///
    /// # Examples
    ///
    /// ```
    /// use liminal_reflect::error::ReflectError;
    /// use liminal_reflect::value::Reflect;
    ///
    /// let value: &dyn Reflect = &5_u32;
    /// assert_eq!(value.downcast_ref::<u32>(), Ok(&5));
    /// assert!(matches!(value.downcast_ref::<i32>(), Err(ReflectError::TypeMismatch { .. })));
    /// ```
    pub fn downcast_ref<T: Typed>(&self) -> Result<&T, ReflectError> {
        let any: &dyn core::any::Any = self;
        match any.downcast_ref::<T>() {
            Some(value) => Ok(value),
            None => Err(self.type_mismatch::<T>()),
        }
    }

    /// The value as `&mut T`, if its concrete type is `T`.
    ///
    /// # Errors
    ///
    /// [`ReflectError::TypeMismatch`] (LIM-REFL-0011), as for
    /// `downcast_ref`.
    ///
    /// # Panics
    ///
    /// Never: the downcast runs only after `core::any::Any::is::<T>()` returned `true`.
    pub fn downcast_mut<T: Typed>(&mut self) -> Result<&mut T, ReflectError> {
        if !(&*self as &dyn core::any::Any).is::<T>() {
            return Err(self.type_mismatch::<T>());
        }
        let any: &mut dyn core::any::Any = self;
        // Checked in two steps: returning the borrow from one branch of a `match` and using
        // `self` in the other does not pass the borrow checker.
        Ok(any
            .downcast_mut::<T>()
            .expect("invariant: `is::<T>()` held above, so the downcast succeeds"))
    }

    fn type_mismatch<T: Typed>(&self) -> ReflectError {
        ReflectError::TypeMismatch {
            expected: T::type_path(),
            actual: self.reflect_type_info().type_path,
        }
    }

    fn no_such_field(&self, name: &str) -> ReflectError {
        ReflectError::NoSuchField {
            type_path: self.reflect_type_info().type_path,
            field: name.to_string(),
        }
    }

    fn miss_error(&self, miss: Miss, name: &str) -> ReflectError {
        let info = self.reflect_type_info();
        match miss {
            Miss::Struct => ReflectError::NoSuchField {
                type_path: info.type_path,
                field: name.to_string(),
            },
            Miss::Enum(variant) => {
                let elsewhere = match &info.shape {
                    Shape::Enum { variants } => variants
                        .iter()
                        .any(|v| v.fields.iter().any(|f| f.name == name)),
                    _ => false,
                };
                if elsewhere {
                    ReflectError::FieldNotInVariant {
                        type_path: info.type_path,
                        variant: variant.to_string(),
                        field: name.to_string(),
                    }
                } else {
                    ReflectError::NoSuchField {
                        type_path: info.type_path,
                        field: name.to_string(),
                    }
                }
            }
            Miss::NoFields(shape) => ReflectError::NoFields {
                type_path: info.type_path,
                shape: shape.to_string(),
            },
        }
    }
}
