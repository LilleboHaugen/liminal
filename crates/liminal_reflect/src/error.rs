//! [`ReflectError`]: the errors of `liminal_reflect` (codes `LIM-REFL-NNNN`, spec §8).

use liminal_core::error::{EngineError, ErrorCode, ErrorContext, write_display};

use crate::info::StableTypeId;

/// A recoverable reflection failure: an invalid descriptor from a hand-written impl, a stable
/// type ID collision, or a field or type request that does not match a value.
///
/// Codes 0001–0009 are descriptor and registry errors, 0010–0019 access errors.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ReflectError {
    /// LIM-REFL-0001: two different Rust types have the same stable type ID.
    TypeIdCollision {
        /// The shared ID.
        id: StableTypeId,
        /// Path of the type already registered under `id`.
        existing_path: String,
        /// Path of the type being registered.
        new_path: String,
    },
    /// LIM-REFL-0002: a descriptor's ID is not the hash of its path.
    TypeIdMismatch {
        /// The descriptor's path.
        type_path: String,
        /// The ID the descriptor reports.
        id: StableTypeId,
        /// `StableTypeId::of_path(type_path)`.
        expected_id: StableTypeId,
    },
    /// LIM-REFL-0003: a field or variant name occurs twice in one list.
    DuplicateName {
        /// The descriptor's path.
        type_path: String,
        /// The first repeated name.
        name: String,
    },
    /// LIM-REFL-0004: a descriptor has an empty type path.
    EmptyTypePath,
    /// LIM-REFL-0005: a value's `reflect_ref` disagrees with its descriptor.
    ShapeMismatch {
        /// Path of the value in violation.
        type_path: String,
        /// Location of that value from the root: `(root)`, or `.name` / `[i]` steps.
        at: String,
        /// What the descriptor says.
        expected: String,
        /// What the value reports.
        actual: String,
    },
    /// LIM-REFL-0010: the value has no field of that name.
    NoSuchField {
        /// The value's type path.
        type_path: String,
        /// The requested name.
        field: String,
    },
    /// LIM-REFL-0011: a downcast to a type that is not the value's concrete type.
    TypeMismatch {
        /// The requested type's path.
        expected: String,
        /// The value's type path.
        actual: String,
    },
    /// LIM-REFL-0012: field access on a list or primitive value.
    NoFields {
        /// The value's type path.
        type_path: String,
        /// `"list"` or `"primitive"`.
        shape: String,
    },
    /// LIM-REFL-0013: the field exists in another variant than the current one.
    FieldNotInVariant {
        /// The value's type path.
        type_path: String,
        /// The current variant's name.
        variant: String,
        /// The requested name.
        field: String,
    },
}

impl EngineError for ReflectError {
    fn code(&self) -> ErrorCode {
        const fn refl(number: u16) -> ErrorCode {
            ErrorCode::new("REFL", number)
        }
        match self {
            ReflectError::TypeIdCollision { .. } => const { refl(1) },
            ReflectError::TypeIdMismatch { .. } => const { refl(2) },
            ReflectError::DuplicateName { .. } => const { refl(3) },
            ReflectError::EmptyTypePath => const { refl(4) },
            ReflectError::ShapeMismatch { .. } => const { refl(5) },
            ReflectError::NoSuchField { .. } => const { refl(10) },
            ReflectError::TypeMismatch { .. } => const { refl(11) },
            ReflectError::NoFields { .. } => const { refl(12) },
            ReflectError::FieldNotInVariant { .. } => const { refl(13) },
        }
    }

    fn message(&self) -> String {
        match self {
            ReflectError::TypeIdCollision {
                id,
                existing_path,
                new_path,
            } => format!(
                "stable type ID `{id}` of type `{new_path}` is already used by type \
                 `{existing_path}`."
            ),
            ReflectError::TypeIdMismatch {
                type_path,
                id,
                expected_id,
            } => format!(
                "type `{type_path}` reports stable type ID `{id}`, but its path hashes to \
                 `{expected_id}`."
            ),
            ReflectError::DuplicateName { type_path, name } => {
                format!("type `{type_path}` uses the field or variant name `{name}` more than once.")
            }
            ReflectError::EmptyTypePath => "a type descriptor has an empty type path.".to_string(),
            ReflectError::ShapeMismatch {
                type_path,
                at,
                expected,
                actual,
            } => format!(
                "value of type `{type_path}` at `{at}` does not match its descriptor: expected \
                 {expected}, found {actual}."
            ),
            ReflectError::NoSuchField { type_path, field } => {
                format!("type `{type_path}` has no field `{field}`.")
            }
            ReflectError::TypeMismatch { expected, actual } => {
                format!("value has type `{actual}`, not `{expected}`.")
            }
            ReflectError::NoFields { type_path, shape } => {
                format!("type `{type_path}` is a {shape} and has no fields.")
            }
            ReflectError::FieldNotInVariant {
                type_path,
                variant,
                field,
            } => format!(
                "field `{field}` of type `{type_path}` does not exist in the current variant \
                 `{variant}`."
            ),
        }
    }

    fn help(&self) -> Option<String> {
        let help = match self {
            ReflectError::TypeIdCollision { .. } => {
                "give one of the types a distinct path with `#[reflect(type_path = \"…\")]`."
            }
            ReflectError::TypeIdMismatch { .. } => {
                "compute the ID with `StableTypeId::of_path` in hand-written `Typed` impls."
            }
            ReflectError::DuplicateName { .. } => {
                "give each field of a struct or variant, and each variant of an enum, a distinct \
                 name."
            }
            ReflectError::EmptyTypePath => {
                "return a non-empty path from `Typed::type_path`, for example \
                 `my_crate::module::Type`."
            }
            ReflectError::ShapeMismatch { .. } => {
                "make `reflect_ref` agree with `type_info` in the hand-written impl, or use \
                 `#[derive(Reflect)]`."
            }
            ReflectError::NoSuchField { .. } => {
                "use a field name from the type's `TypeInfo`; tuple fields are named `0`, `1`, …"
            }
            ReflectError::TypeMismatch { .. } => {
                "request the type listed in the field's `FieldInfo`."
            }
            ReflectError::NoFields { .. } => {
                "read list elements through `ReflectRef::List`, and primitive values with \
                 `downcast_ref`."
            }
            ReflectError::FieldNotInVariant { .. } => {
                "check `ReflectEnum::variant_name` before accessing a variant's fields."
            }
        };
        Some(help.to_string())
    }

    fn context(&self) -> ErrorContext {
        let context = ErrorContext::new();
        match self {
            ReflectError::TypeIdCollision {
                id,
                existing_path,
                new_path,
            } => context
                .with("id", id.to_string())
                .with("existing_path", existing_path.as_str())
                .with("new_path", new_path.as_str()),
            ReflectError::TypeIdMismatch {
                type_path,
                id,
                expected_id,
            } => context
                .with("type_path", type_path.as_str())
                .with("id", id.to_string())
                .with("expected_id", expected_id.to_string()),
            ReflectError::DuplicateName { type_path, name } => context
                .with("type_path", type_path.as_str())
                .with("name", name.as_str()),
            ReflectError::EmptyTypePath => context,
            ReflectError::ShapeMismatch {
                type_path,
                at,
                expected,
                actual,
            } => context
                .with("type_path", type_path.as_str())
                .with("at", at.as_str())
                .with("expected", expected.as_str())
                .with("actual", actual.as_str()),
            ReflectError::NoSuchField { type_path, field } => context
                .with("type_path", type_path.as_str())
                .with("field", field.as_str()),
            ReflectError::TypeMismatch { expected, actual } => context
                .with("expected", expected.as_str())
                .with("actual", actual.as_str()),
            ReflectError::NoFields { type_path, shape } => context
                .with("type_path", type_path.as_str())
                .with("shape", shape.as_str()),
            ReflectError::FieldNotInVariant {
                type_path,
                variant,
                field,
            } => context
                .with("type_path", type_path.as_str())
                .with("variant", variant.as_str())
                .with("field", field.as_str()),
        }
    }
}

/// [`write_display`]: `{code}: {message} help: {help}`.
impl core::fmt::Display for ReflectError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write_display(self, f)
    }
}

impl std::error::Error for ReflectError {}
