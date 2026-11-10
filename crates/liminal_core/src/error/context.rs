//! [`ErrorContext`] and [`ContextValue`]: the structured fields of an engine error.

use super::ErrorCode;

/// The structured fields of an error: an ordered list of `key = value` pairs with the data the
/// message talks about.
///
/// Keys should be distinct and match `^[a-z][a-z0-9_]*$`. This type does not check that;
/// [`validate_error`](super::validate_error) does.
///
/// # Examples
///
/// ```
/// use liminal_core::error::{ContextValue, ErrorContext};
///
/// let context = ErrorContext::new().with("index", 3_u32).with("name", "hero");
/// assert_eq!(context.len(), 2);
/// assert_eq!(context.get("index"), Some(&ContextValue::UInt(3)));
/// assert_eq!(context.get("missing"), None);
/// let keys: Vec<&str> = context.iter().map(|(key, _)| key).collect();
/// assert_eq!(keys, ["index", "name"]);
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ErrorContext {
    fields: Vec<(&'static str, ContextValue)>,
}

impl ErrorContext {
    /// An empty context.
    #[must_use]
    pub fn new() -> ErrorContext {
        ErrorContext::default()
    }

    /// Appends a field. Key form and uniqueness are checked by
    /// [`validate_error`](super::validate_error), not here.
    #[must_use]
    pub fn with(mut self, key: &'static str, value: impl Into<ContextValue>) -> ErrorContext {
        self.fields.push((key, value.into()));
        self
    }

    /// The value of the first field with this key, or `None`. O(len).
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&ContextValue> {
        self.fields
            .iter()
            .find(|(field_key, _)| *field_key == key)
            .map(|(_, value)| value)
    }

    /// The fields in the order they were appended, including fields with a repeated key.
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &ContextValue)> {
        self.fields.iter().map(|(key, value)| (*key, value))
    }

    /// Number of fields.
    #[must_use]
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// `true` if there is no field.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

/// The value of one context field.
///
/// `From` conversions: `bool` to `Bool`; `i8`, `i16`, `i32`, `i64`, `isize` to `Int`; `u8`, `u16`,
/// `u32`, `u64`, `usize` to `UInt`; `f32` (widened exactly) and `f64` to `Float`; `&str`, `String`
/// and [`ErrorCode`] (its text form) to `Text`.
#[derive(Clone, Debug, PartialEq)]
pub enum ContextValue {
    /// A boolean.
    Bool(bool),
    /// A signed integer.
    Int(i64),
    /// An unsigned integer.
    UInt(u64),
    /// A floating-point number. May be NaN or infinite; the JSON form renders those as `null`.
    Float(f64),
    /// Text.
    Text(String),
}

/// Implements `From<$source>` through a lossless `From` conversion to the variant's type.
macro_rules! impl_from_lossless {
    ($variant:ident: $($source:ty),+) => {$(
        impl From<$source> for ContextValue {
            fn from(value: $source) -> ContextValue {
                ContextValue::$variant(value.into())
            }
        }
    )+};
}

impl_from_lossless!(Bool: bool);
impl_from_lossless!(Int: i8, i16, i32, i64);
impl_from_lossless!(UInt: u8, u16, u32, u64);
impl_from_lossless!(Float: f32, f64);
impl_from_lossless!(Text: &str, String);

impl From<isize> for ContextValue {
    fn from(value: isize) -> ContextValue {
        // Supported targets are 64-bit (spec §2); `isize` is at most 64 bits on every Rust target.
        ContextValue::Int(i64::try_from(value).expect("invariant: isize is at most 64 bits"))
    }
}

impl From<usize> for ContextValue {
    fn from(value: usize) -> ContextValue {
        // Supported targets are 64-bit (spec §2); `usize` is at most 64 bits on every Rust target.
        ContextValue::UInt(u64::try_from(value).expect("invariant: usize is at most 64 bits"))
    }
}

impl From<ErrorCode> for ContextValue {
    fn from(value: ErrorCode) -> ContextValue {
        ContextValue::Text(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies: CORE-R007
    #[test]
    fn with_appends_and_iter_keeps_order_and_duplicates() {
        let empty = ErrorContext::new();
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
        assert_eq!(empty.iter().count(), 0);
        assert_eq!(empty.get("a"), None);
        assert_eq!(empty, ErrorContext::default());

        let context = ErrorContext::new()
            .with("b", 1_u8)
            .with("a", "x")
            .with("b", true)
            .with("Not a key", -1);
        assert!(!context.is_empty());
        assert_eq!(context.len(), 4);
        let fields: Vec<(&str, ContextValue)> = context
            .iter()
            .map(|(key, value)| (key, value.clone()))
            .collect();
        assert_eq!(
            fields,
            [
                ("b", ContextValue::UInt(1)),
                ("a", ContextValue::Text("x".to_string())),
                ("b", ContextValue::Bool(true)),
                ("Not a key", ContextValue::Int(-1)),
            ]
        );
        // `get` returns the first field with the key.
        assert_eq!(context.get("b"), Some(&ContextValue::UInt(1)));
        assert_eq!(context.get("a"), Some(&ContextValue::Text("x".to_string())));
        assert_eq!(context.get("Not a key"), Some(&ContextValue::Int(-1)));
        assert_eq!(context.get("c"), None);
        assert_eq!(context.get(""), None);
    }

    /// Verifies: CORE-R007
    #[test]
    fn from_conversions_keep_the_value() {
        use ContextValue::{Bool, Float, Int, Text, UInt};
        assert_eq!(ContextValue::from(true), Bool(true));
        assert_eq!(ContextValue::from(false), Bool(false));

        assert_eq!(ContextValue::from(i8::MIN), Int(-128));
        assert_eq!(ContextValue::from(i16::MIN), Int(-32_768));
        assert_eq!(ContextValue::from(i32::MIN), Int(-2_147_483_648));
        assert_eq!(ContextValue::from(i64::MIN), Int(i64::MIN));
        assert_eq!(ContextValue::from(i64::MAX), Int(i64::MAX));
        assert_eq!(ContextValue::from(-7_isize), Int(-7));
        assert_eq!(ContextValue::from(isize::MIN), Int(i64::MIN));

        assert_eq!(ContextValue::from(u8::MAX), UInt(255));
        assert_eq!(ContextValue::from(u16::MAX), UInt(65_535));
        assert_eq!(ContextValue::from(u32::MAX), UInt(4_294_967_295));
        assert_eq!(ContextValue::from(u64::MAX), UInt(u64::MAX));
        assert_eq!(ContextValue::from(7_usize), UInt(7));
        assert_eq!(ContextValue::from(usize::MAX), UInt(u64::MAX));

        assert_eq!(ContextValue::from(1.5_f64), Float(1.5));
        assert_eq!(
            ContextValue::from(f64::MIN_POSITIVE),
            Float(f64::MIN_POSITIVE)
        );
        // f32 0.1 is not f64 0.1: the widening is exact, not a decimal re-parse.
        assert_eq!(ContextValue::from(0.1_f32), Float(0.100_000_001_490_116_12));
        assert_ne!(ContextValue::from(0.1_f32), Float(0.1));
        assert_eq!(ContextValue::from(f32::MAX), Float(f64::from(f32::MAX)));
        match ContextValue::from(-0.0_f32) {
            Float(zero) => assert_eq!(zero.to_bits(), (-0.0_f64).to_bits()),
            other => panic!("{other:?}"),
        }
        match ContextValue::from(f32::NAN) {
            Float(nan) => assert!(nan.is_nan()),
            other => panic!("{other:?}"),
        }

        assert_eq!(ContextValue::from("text"), Text("text".to_string()));
        assert_eq!(
            ContextValue::from(String::from("owned")),
            Text("owned".to_string())
        );
        assert_eq!(
            ContextValue::from(ErrorCode::new("ECS", 3)),
            Text("LIM-ECS-0003".to_string())
        );
    }
}
