//! [`validate_reflect`]: checks that a value's views agree with its descriptor (REFL-R080).

use crate::error::ReflectError;
use crate::info::{FieldInfo, Shape, TypeInfo, TypeRef};
use crate::registry::check_descriptor;
use crate::value::{Reflect, ReflectFields, ReflectList, ReflectRef};

/// Checks that `reflect_ref` agrees with `reflect_type_info` for `value` and every value
/// reachable from it through fields and list elements (REFL-R080).
///
/// Values are visited depth-first in field and element order. For each value it checks, in
/// order: the descriptor passes the checks of `TypeRegistry::register`; the view matches the
/// shape (kind, field count, field names, variant index and name, array length, every counted
/// field and element present); each field's and element's stable ID equals the `TypeRef` in the
/// descriptor. It calls `reflect_type_info()` once per visited value, so time and allocations are
/// linear in the number of reachable values. Meant for tests and debug checks.
///
/// # Errors
///
/// The first violation: [`ReflectError::EmptyTypePath`], [`ReflectError::TypeIdMismatch`] or
/// [`ReflectError::DuplicateName`] (LIM-REFL-0004, 0002, 0003) for a bad descriptor, else
/// [`ReflectError::ShapeMismatch`] (LIM-REFL-0005) with the location `at` of the value in
/// violation: `(root)`, or `.name` per field and `[i]` per element from the root
/// (`.items[2].0`).
///
/// # Examples
///
/// ```
/// use liminal_reflect::value::validate_reflect;
///
/// let value: Vec<Option<[i16; 3]>> = vec![None, Some([1, 2, 3])];
/// assert_eq!(validate_reflect(&value), Ok(()));
/// ```
pub fn validate_reflect(value: &dyn Reflect) -> Result<(), ReflectError> {
    let info = value.reflect_type_info();
    let mut at = String::new();
    visit(value, &info, &mut at)
}

/// Checks `value` (whose descriptor is `info`), then its children in order.
fn visit(value: &dyn Reflect, info: &TypeInfo, at: &mut String) -> Result<(), ReflectError> {
    check_descriptor(info)?;
    let fail = |at: &str, expected: String, actual: String| ReflectError::ShapeMismatch {
        type_path: info.type_path.clone(),
        at: if at.is_empty() {
            "(root)".to_string()
        } else {
            at.to_string()
        },
        expected,
        actual,
    };
    let view = value.reflect_ref();
    let children: Vec<Child<'_, '_>> = match (&info.shape, view) {
        (Shape::Primitive(kind), ReflectRef::Primitive(primitive)) => {
            if primitive.kind() != *kind {
                return Err(fail(
                    at,
                    kind.type_path().to_string(),
                    primitive.kind().type_path().to_string(),
                ));
            }
            Vec::new()
        }
        (Shape::Struct { fields, .. }, ReflectRef::Struct(view)) => {
            struct_children(fields, view, "struct").map_err(|(e, a)| fail(at, e, a))?
        }
        (Shape::Enum { variants }, ReflectRef::Enum(view)) => {
            let index = view.variant_index();
            let Some(variant) = usize::try_from(index).ok().and_then(|i| variants.get(i)) else {
                return Err(fail(
                    at,
                    format!("variant index below {}", variants.len()),
                    format!("variant index {index}"),
                ));
            };
            if view.variant_name() != variant.name {
                return Err(fail(
                    at,
                    format!("variant {}", variant.name),
                    format!("variant {}", view.variant_name()),
                ));
            }
            struct_children(&variant.fields, view, "variant").map_err(|(e, a)| fail(at, e, a))?
        }
        (Shape::List { element }, ReflectRef::List(view)) => {
            list_children(element, view).map_err(|(e, a)| fail(at, e, a))?
        }
        (Shape::Array { element, len }, ReflectRef::List(view)) => {
            if view.len() != *len {
                return Err(fail(
                    at,
                    format!("array of length {len}"),
                    format!("list of length {}", view.len()),
                ));
            }
            list_children(element, view).map_err(|(e, a)| fail(at, e, a))?
        }
        (shape, view) => return Err(fail(at, describe_shape(shape), describe_view(&view))),
    };
    let child_infos: Vec<TypeInfo> = children
        .iter()
        .map(|(_, child, _)| child.reflect_type_info())
        .collect();
    for ((step, _, expected), child_info) in children.iter().zip(&child_infos) {
        if child_info.id != expected.id {
            return Err(fail(
                at,
                format!(
                    "{} of type `{}` (ID {})",
                    step_name(step),
                    expected.type_path,
                    expected.id
                ),
                format!("type `{}` (ID {})", child_info.type_path, child_info.id),
            ));
        }
    }
    for ((step, child, _), child_info) in children.iter().zip(&child_infos) {
        let len = at.len();
        at.push_str(step);
        let result = visit(*child, child_info, at);
        at.truncate(len);
        result?;
    }
    Ok(())
}

/// A child value: its location step (`.name` or `[i]`), the value, its expected type.
type Child<'a, 'i> = (String, &'a dyn Reflect, &'i TypeRef);
/// The children of a value, or `(expected, actual)` of the first view violation.
type Children<'a, 'i> = Result<Vec<Child<'a, 'i>>, (String, String)>;

/// `".name"` → `"field name"`, `"[3]"` → `"element 3"`.
fn step_name(step: &str) -> String {
    match step.strip_prefix('.') {
        Some(name) => format!("field {name}"),
        None => format!("element {}", step.trim_matches(['[', ']'])),
    }
}

/// The fields of a struct or variant view, checked against `fields`; `Err((expected, actual))`.
fn struct_children<'a, 'i, F: ReflectFields + ?Sized>(
    fields: &'i [FieldInfo],
    view: &'a F,
    what: &str,
) -> Children<'a, 'i> {
    if view.field_count() != fields.len() {
        return Err((
            format!("{what} with {} fields", fields.len()),
            format!("{what} with {} fields", view.field_count()),
        ));
    }
    let mut children = Vec::with_capacity(fields.len());
    for (index, field) in fields.iter().enumerate() {
        let name = view.field_name(index);
        if name != Some(field.name) {
            return Err((
                format!("field {index} named {}", field.name),
                match name {
                    Some(name) => format!("field {index} named {name}"),
                    None => format!("field {index} without a name"),
                },
            ));
        }
        let Some(child) = view.field_at(index) else {
            return Err((format!("field {}", field.name), "no value".to_string()));
        };
        children.push((format!(".{}", field.name), child, &field.ty));
    }
    Ok(children)
}

/// The elements of a list view, all of type `element`; `Err((expected, actual))`.
fn list_children<'a, 'i>(element: &'i TypeRef, view: &'a dyn ReflectList) -> Children<'a, 'i> {
    (0..view.len())
        .map(|index| match view.get(index) {
            Some(child) => Ok((format!("[{index}]"), child, element)),
            None => Err((format!("element {index}"), "no value".to_string())),
        })
        .collect()
}

fn describe_shape(shape: &Shape) -> String {
    match shape {
        Shape::Primitive(kind) => kind.type_path().to_string(),
        Shape::Struct { fields, .. } => format!("struct with {} fields", fields.len()),
        Shape::Enum { variants } => format!("enum with {} variants", variants.len()),
        Shape::List { .. } => "list".to_string(),
        Shape::Array { len, .. } => format!("array of length {len}"),
    }
}

fn describe_view(view: &ReflectRef<'_>) -> String {
    match view {
        ReflectRef::Primitive(primitive) => primitive.kind().type_path().to_string(),
        ReflectRef::Struct(fields) => format!("struct with {} fields", fields.field_count()),
        ReflectRef::Enum(_) => "enum".to_string(),
        ReflectRef::List(_) => "list".to_string(),
    }
}
