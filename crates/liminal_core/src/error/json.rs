//! [`render_json`]: the JSON form of an engine error (CORE-R008 to CORE-R011).
//!
//! Hand-written because no serializer exists at layer 0 (spec §12, open question 2).

use core::fmt::Write as _;

use super::{ContextValue, EngineError};

/// Renders the JSON form of `error`: one JSON object (RFC 8259) on one line, without whitespace
/// outside strings and without a trailing newline.
///
/// Members, in this order:
/// - `code`: the code's text form;
/// - `message`: [`EngineError::message`];
/// - `help`: [`EngineError::help`], or `null` when it is `None`;
/// - `context`: an object with one member per context field, in
///   [`ErrorContext`](super::ErrorContext) order (`{}` when empty).
///
/// Context values: `Bool` as `true` or `false`; `Int` and `UInt` as decimal integers; a finite
/// `Float` as Rust's `Display` for `f64` (shortest decimal that parses back to the same bits, no
/// exponent; `-0.0` is `-0`); NaN and infinities as `null`; `Text` as a string.
///
/// Strings escape `"`, `\` and U+0000 to U+001F (`\b`, `\f`, `\n`, `\r`, `\t`, otherwise
/// `\u00xx` with lowercase hex digits); every other code point is written unchanged. The output
/// depends only on the error value (same bytes in every process and on every platform).
/// `std::error::Error::source` is not rendered.
///
/// # Examples
///
/// ```
/// use liminal_core::error::{CoreError, render_json};
///
/// let error = CoreError::StaleHandle { index: 3, generation: 2 };
/// assert_eq!(
///     render_json(&error),
///     concat!(
///         r#"{"code":"LIM-CORE-0010","#,
///         r#""message":"handle `3v2` is not live.","#,
///         r#""help":"the handle was freed already or was not issued by this allocator; "#,
///         r#"check `is_live` before freeing.","#,
///         r#""context":{"index":3,"generation":2}}"#,
///     )
/// );
/// ```
#[must_use]
pub fn render_json(error: &dyn EngineError) -> String {
    let mut out = String::new();
    out.push_str("{\"code\":");
    write_string(&mut out, &error.code().to_string());
    out.push_str(",\"message\":");
    write_string(&mut out, &error.message());
    out.push_str(",\"help\":");
    match error.help() {
        Some(help) => write_string(&mut out, &help),
        None => out.push_str("null"),
    }
    out.push_str(",\"context\":{");
    for (position, (key, value)) in error.context().iter().enumerate() {
        if position > 0 {
            out.push(',');
        }
        write_string(&mut out, key);
        out.push(':');
        write_value(&mut out, value);
    }
    out.push_str("}}");
    out
}

/// Appends `value` as a JSON value (CORE-R010).
fn write_value(out: &mut String, value: &ContextValue) {
    match value {
        ContextValue::Bool(true) => out.push_str("true"),
        ContextValue::Bool(false) => out.push_str("false"),
        ContextValue::Int(int) => push_display(out, int),
        ContextValue::UInt(uint) => push_display(out, uint),
        ContextValue::Float(float) if float.is_finite() => push_display(out, float),
        ContextValue::Float(_) => out.push_str("null"),
        ContextValue::Text(text) => write_string(out, text),
    }
}

/// Appends `text` as a JSON string (CORE-R009).
fn write_string(out: &mut String, text: &str) {
    out.push('"');
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{00}'..='\u{1F}' => {
                out.push_str("\\u00");
                push_display(out, format_args!("{:02x}", u32::from(character)));
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

/// Appends the `Display` form of `value`.
fn push_display(out: &mut String, value: impl core::fmt::Display) {
    // Writing to a `String` cannot fail: its `fmt::Write` impl always returns `Ok`.
    let _ = write!(out, "{value}");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn string(text: &str) -> String {
        let mut out = String::new();
        write_string(&mut out, text);
        out
    }

    fn value(value: impl Into<ContextValue>) -> String {
        let mut out = String::new();
        write_value(&mut out, &value.into());
        out
    }

    /// Verifies: CORE-R009
    #[test]
    fn strings_escape_quote_backslash_and_every_control_character() {
        assert_eq!(string(""), r#""""#);
        assert_eq!(string("plain"), r#""plain""#);
        assert_eq!(string("\"\\"), r#""\"\\""#);
        assert_eq!(string("\u{08}\u{0C}\n\r\t"), r#""\b\f\n\r\t""#);
        // Every code point U+0000..=U+001F: the five short forms, otherwise \u00xx lowercase.
        for code in 0_u32..=0x1F {
            let character = char::from_u32(code).unwrap();
            let expected = match code {
                0x08 => "\\b".to_string(),
                0x09 => "\\t".to_string(),
                0x0A => "\\n".to_string(),
                0x0C => "\\f".to_string(),
                0x0D => "\\r".to_string(),
                _ => format!("\\u00{code:02x}"),
            };
            assert_eq!(string(&character.to_string()), format!("\"{expected}\""));
        }
        assert_eq!(string("\u{00}\u{1b}\u{1F}"), r#""\u0000\u001b\u001f""#);
    }

    /// Verifies: CORE-R009
    #[test]
    fn strings_keep_every_other_code_point_unchanged() {
        // `/`, U+0020, U+007F, U+0080, U+2028, non-BMP: UTF-8 bytes as they are.
        let text = "a/b \u{7F}\u{80}é\u{2028}\u{2029}\u{FEFF}😀'<>&";
        assert_eq!(string(text), format!("\"{text}\""));
    }

    /// Verifies: CORE-R010
    #[test]
    fn values_render_as_json_values() {
        assert_eq!(value(true), "true");
        assert_eq!(value(false), "false");

        assert_eq!(value(0_i64), "0");
        assert_eq!(value(-1_i64), "-1");
        assert_eq!(value(i64::MIN), "-9223372036854775808");
        assert_eq!(value(i64::MAX), "9223372036854775807");
        assert_eq!(value(0_u64), "0");
        assert_eq!(value(u64::MAX), "18446744073709551615");

        assert_eq!(value(0.0_f64), "0");
        assert_eq!(value(-0.0_f64), "-0");
        assert_eq!(value(1.0_f64), "1");
        assert_eq!(value(-1.5_f64), "-1.5");
        assert_eq!(value(0.1_f64), "0.1");
        assert_eq!(value(0.1_f32), "0.10000000149011612");
        assert_eq!(value(1e21_f64), "1000000000000000000000");
        assert_eq!(value(1e-7_f64), "0.0000001");
        assert_eq!(value(5e-324_f64).len(), 2 + 324);
        assert!(value(f64::MAX).bytes().all(|byte| byte.is_ascii_digit()));
        assert_eq!(value(f64::MAX).len(), 309);

        assert_eq!(value(f64::NAN), "null");
        assert_eq!(value(-f64::NAN), "null");
        assert_eq!(value(f64::INFINITY), "null");
        assert_eq!(value(f64::NEG_INFINITY), "null");

        assert_eq!(value("a\"b\n"), r#""a\"b\n""#);
    }

    /// Verifies: CORE-R010
    #[test]
    fn finite_floats_parse_back_to_the_same_bits() {
        for float in [
            0.0,
            -0.0,
            1.0,
            0.1,
            1.0 / 3.0,
            f64::MIN_POSITIVE,
            5e-324,
            f64::MAX,
            f64::MIN,
            f64::EPSILON,
            123_456_789.123_456_79,
            9_007_199_254_740_993.0,
        ] {
            let text = value(float);
            assert!(!text.contains(['e', 'E', '+']), "{text}");
            assert_eq!(
                text.parse::<f64>().unwrap().to_bits(),
                float.to_bits(),
                "{text}"
            );
        }
    }
}
