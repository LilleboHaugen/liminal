//! [`validate_error`]: the error contract check (CORE-R012).

use super::{CoreError, EngineError, write_display};

/// Checks the error contract for one error value and returns the first violation.
///
/// Each crate calls this in its tests for one value of every variant of its error enums, with
/// the `req_prefix` of its spec as `expected_area`.
///
/// # Errors
///
/// The checks run in this order; the first one that fails is returned:
///
/// | Check | Error |
/// |---|---|
/// | the code's area equals `expected_area` | [`CoreError::AreaMismatch`] (LIM-CORE-0007) |
/// | `message()` is not empty | [`CoreError::EmptyMessage`] (LIM-CORE-0004) |
/// | `help()` is not `Some("")` | [`CoreError::EmptyHelp`] (LIM-CORE-0005) |
/// | every context key matches `^[a-z][a-z0-9_]*$` | [`CoreError::InvalidContextKey`] (LIM-CORE-0002), naming the first bad key |
/// | no context key occurs twice | [`CoreError::DuplicateContextKey`] (LIM-CORE-0003), naming the first repeated key |
/// | `to_string()` equals the [`write_display`] output | [`CoreError::DisplayMismatch`] (LIM-CORE-0006), with both strings |
///
/// # Examples
///
/// ```
/// use liminal_core::error::{CoreError, EngineError, validate_error};
///
/// let error = CoreError::ZeroRngState;
/// assert_eq!(validate_error(&error, "CORE"), Ok(()));
/// assert_eq!(
///     validate_error(&error, "ECS"),
///     Err(CoreError::AreaMismatch { code: error.code(), expected_area: "ECS".to_string() })
/// );
/// ```
pub fn validate_error(error: &dyn EngineError, expected_area: &str) -> Result<(), CoreError> {
    let code = error.code();
    if code.area() != expected_area {
        return Err(CoreError::AreaMismatch {
            code,
            expected_area: expected_area.to_string(),
        });
    }
    if error.message().is_empty() {
        return Err(CoreError::EmptyMessage { code });
    }
    if error.help().is_some_and(|help| help.is_empty()) {
        return Err(CoreError::EmptyHelp { code });
    }
    let context = error.context();
    if let Some((key, _)) = context.iter().find(|(key, _)| !is_snake_case(key)) {
        return Err(CoreError::InvalidContextKey {
            code,
            key: key.to_string(),
        });
    }
    // Quadratic in the number of fields; errors have a handful, and this runs in tests.
    let repeated = context.iter().enumerate().find(|(position, (key, _))| {
        context
            .iter()
            .take(*position)
            .any(|(earlier, _)| earlier == *key)
    });
    if let Some((_, (key, _))) = repeated {
        return Err(CoreError::DuplicateContextKey {
            code,
            key: key.to_string(),
        });
    }
    let expected = TextForm(error).to_string();
    let actual = error.to_string();
    if actual != expected {
        return Err(CoreError::DisplayMismatch {
            code,
            expected,
            actual,
        });
    }
    Ok(())
}

/// `true` if `key` matches `^[a-z][a-z0-9_]*$`.
fn is_snake_case(key: &str) -> bool {
    let mut bytes = key.bytes();
    bytes.next().is_some_and(|first| first.is_ascii_lowercase())
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

/// Displays an error with [`write_display`], whatever its own `Display` does.
struct TextForm<'a>(&'a dyn EngineError);

impl core::fmt::Display for TextForm<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write_display(self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies: CORE-R012
    #[test]
    fn snake_case_keys_are_lowercase_letter_then_lowercase_digits_underscores() {
        for key in ["a", "z", "index", "slot_limit", "a1", "a_", "a__b", "x9_y"] {
            assert!(is_snake_case(key), "{key}");
        }
        for key in [
            "", "A", "aB", "1a", "_a", "a-b", "a b", " a", "a ", "é", "aé", "a.b", "a\n", "Index",
        ] {
            assert!(!is_snake_case(key), "{key:?}");
        }
    }
}
