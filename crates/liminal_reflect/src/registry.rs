//! [`TypeRegistry`]: type descriptors by stable type ID (spec §5.3, REFL-R060 to REFL-R065).

use std::any::TypeId;
use std::collections::BTreeMap;

use crate::error::ReflectError;
use crate::info::{FieldInfo, Shape, StableTypeId, TypeInfo};
use crate::value::Typed;

/// Type descriptors by stable type ID.
///
/// Lookups use ordered maps (logarithmic in [`len`](Self::len)); iteration is in ascending
/// `StableTypeId` order, independent of registration order. Each entry also keeps the
/// `core::any::TypeId` of the registered Rust type, used only to tell two Rust types with the
/// same stable ID apart in this process (REFL-R062); it is never printed, hashed or ordered on.
///
/// # Examples
///
/// ```
/// use liminal_reflect::error::ReflectError;
/// use liminal_reflect::info::StableTypeId;
/// use liminal_reflect::registry::TypeRegistry;
///
/// let mut registry = TypeRegistry::new();
/// let id = registry.register::<Vec<u8>>()?;
/// assert_eq!(id, StableTypeId::of_path("Vec<u8>"));
/// assert_eq!(registry.get_by_path("Vec<u8>").map(|info| info.id), Some(id));
/// // Registering the same type again changes nothing.
/// assert_eq!(registry.register::<Vec<u8>>()?, id);
/// assert_eq!(registry.len(), 1);
/// # Ok::<(), ReflectError>(())
/// ```
#[derive(Clone, Default)]
pub struct TypeRegistry {
    types: BTreeMap<StableTypeId, (TypeInfo, TypeId)>,
    by_path: BTreeMap<String, StableTypeId>,
}

/// Prints the descriptors only, in [`TypeRegistry::iter`] order, never a `TypeId` (REFL-R065):
/// `TypeRegistry { types: [..] }`.
impl core::fmt::Debug for TypeRegistry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let types: Vec<&TypeInfo> = self.iter().collect();
        f.debug_struct("TypeRegistry")
            .field("types", &types)
            .finish()
    }
}

impl TypeRegistry {
    /// An empty registry.
    #[must_use]
    pub fn new() -> TypeRegistry {
        TypeRegistry::default()
    }

    /// Registers `T` only (not its field types) and returns its stable type ID, which equals
    /// `T::stable_type_id()` for every impl that meets REFL-R002. Registering the same Rust type
    /// again returns the same ID and changes nothing.
    ///
    /// # Errors
    ///
    /// Checked in this order; on error the registry is unchanged:
    /// - [`ReflectError::EmptyTypePath`] (LIM-REFL-0004), [`ReflectError::TypeIdMismatch`]
    ///   (LIM-REFL-0002), [`ReflectError::DuplicateName`] (LIM-REFL-0003): `T::type_info()` is
    ///   not a consistent descriptor (REFL-R061).
    /// - [`ReflectError::TypeIdCollision`] (LIM-REFL-0001): another Rust type is stored under
    ///   the same stable type ID.
    pub fn register<T: Typed>(&mut self) -> Result<StableTypeId, ReflectError> {
        let info = T::type_info();
        check_descriptor(&info)?;
        let type_id = TypeId::of::<T>();
        let id = info.id;
        if let Some((existing, existing_type_id)) = self.types.get(&id) {
            if *existing_type_id == type_id {
                return Ok(id);
            }
            return Err(ReflectError::TypeIdCollision {
                id,
                existing_path: existing.type_path.clone(),
                new_path: info.type_path,
            });
        }
        self.by_path.insert(info.type_path.clone(), id);
        self.types.insert(id, (info, type_id));
        debug_assert_eq!(self.types.len(), self.by_path.len(), "invariant: INV-3");
        Ok(id)
    }

    /// The descriptor stored under `id`.
    #[must_use]
    pub fn get(&self, id: StableTypeId) -> Option<&TypeInfo> {
        self.types.get(&id).map(|(info, _)| info)
    }

    /// The descriptor of the registered type with this path.
    #[must_use]
    pub fn get_by_path(&self, type_path: &str) -> Option<&TypeInfo> {
        self.by_path.get(type_path).and_then(|id| self.get(*id))
    }

    /// The number of registered types.
    #[must_use]
    pub fn len(&self) -> usize {
        self.types.len()
    }

    /// `len() == 0`.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    /// Every registered descriptor once, in ascending `StableTypeId` order.
    pub fn iter(&self) -> impl Iterator<Item = &TypeInfo> + '_ {
        self.types.values().map(|(info, _)| info)
    }
}

/// The descriptor checks of REFL-R061, in order: non-empty path, ID is the path's hash, no
/// repeated name in any field or variant list. Shared with `validate_reflect` (REFL-R080).
pub(crate) fn check_descriptor(info: &TypeInfo) -> Result<(), ReflectError> {
    if info.type_path.is_empty() {
        return Err(ReflectError::EmptyTypePath);
    }
    let expected_id = StableTypeId::of_path(&info.type_path);
    if info.id != expected_id {
        return Err(ReflectError::TypeIdMismatch {
            type_path: info.type_path.clone(),
            id: info.id,
            expected_id,
        });
    }
    let repeated = match &info.shape {
        Shape::Struct { fields, .. } => first_repeated_field(fields),
        Shape::Enum { variants } => variants.iter().enumerate().find_map(|(index, variant)| {
            let name_repeated = variants[..index].iter().any(|v| v.name == variant.name);
            if name_repeated {
                Some(variant.name)
            } else {
                first_repeated_field(&variant.fields)
            }
        }),
        Shape::Primitive(_) | Shape::List { .. } | Shape::Array { .. } => None,
    };
    match repeated {
        Some(name) => Err(ReflectError::DuplicateName {
            type_path: info.type_path.clone(),
            name: name.to_string(),
        }),
        None => Ok(()),
    }
}

/// The first field name equal to the name of an earlier field of the same list.
fn first_repeated_field(fields: &[FieldInfo]) -> Option<&'static str> {
    fields.iter().enumerate().find_map(|(index, field)| {
        fields[..index]
            .iter()
            .any(|earlier| earlier.name == field.name)
            .then_some(field.name)
    })
}
