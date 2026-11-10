//! The engine error model (ADR-0017, spec §5.1): stable codes, the trait every engine error
//! implements, structured context, and the text and JSON forms.
//!
//! - [`ErrorCode`]: `LIM-<AREA>-<NNNN>`, never reused or renumbered.
//! - [`EngineError`]: code, message, optional help and context of one error value.
//! - [`write_display`]: the one-line text form, used by every error's `Display`.
//! - [`render_json`]: the JSON form returned by tools (`code`, `message`, `help`, `context`).
//! - [`validate_error`]: the error contract check each crate runs on its own errors in tests.
//! - [`CoreError`]: the errors of `liminal_core` itself (`LIM-CORE-NNNN`).

mod code;
mod context;
mod core_error;
mod json;
mod validate;

pub use code::ErrorCode;
pub use context::{ContextValue, ErrorContext};
pub use core_error::CoreError;
pub use json::render_json;
pub use validate::validate_error;

/// Implemented by every engine error type (ADR-0017). Dyn-compatible.
///
/// Rules for implementations (checked by [`validate_error`], which each crate calls in its tests
/// for one value of every variant):
/// - the code's area is the `req_prefix` of the crate's spec;
/// - `Display` is exactly [`write_display`];
/// - the message is not empty, and help is `None` or a non-empty text;
/// - context keys are distinct and match `^[a-z][a-z0-9_]*$`.
///
/// # Examples
///
/// A complete small error enum:
///
/// ```
/// use liminal_core::error::{
///     EngineError, ErrorCode, ErrorContext, render_json, validate_error, write_display,
/// };
///
/// #[derive(Debug)]
/// enum SaveError {
///     SlotOutOfRange { slot: u32, slot_count: u32 },
///     Corrupt,
/// }
///
/// impl EngineError for SaveError {
///     fn code(&self) -> ErrorCode {
///         match self {
///             SaveError::SlotOutOfRange { .. } => const { ErrorCode::new("SAVE", 1) },
///             SaveError::Corrupt => const { ErrorCode::new("SAVE", 2) },
///         }
///     }
///
///     fn message(&self) -> String {
///         match self {
///             SaveError::SlotOutOfRange { slot, slot_count } => {
///                 format!("save slot `{slot}` does not exist; there are `{slot_count}` slots.")
///             }
///             SaveError::Corrupt => "the save file is corrupt.".to_string(),
///         }
///     }
///
///     fn help(&self) -> Option<String> {
///         match self {
///             SaveError::SlotOutOfRange { .. } => {
///                 Some("use a slot index below the slot count.".to_string())
///             }
///             SaveError::Corrupt => None,
///         }
///     }
///
///     fn context(&self) -> ErrorContext {
///         match self {
///             SaveError::SlotOutOfRange { slot, slot_count } => ErrorContext::new()
///                 .with("slot", *slot)
///                 .with("slot_count", *slot_count),
///             SaveError::Corrupt => ErrorContext::new(),
///         }
///     }
/// }
///
/// impl std::fmt::Display for SaveError {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         write_display(self, f)
///     }
/// }
///
/// impl std::error::Error for SaveError {}
///
/// let error = SaveError::SlotOutOfRange { slot: 9, slot_count: 4 };
/// assert_eq!(
///     error.to_string(),
///     "LIM-SAVE-0001: save slot `9` does not exist; there are `4` slots. \
///      help: use a slot index below the slot count."
/// );
/// assert_eq!(
///     render_json(&SaveError::Corrupt),
///     r#"{"code":"LIM-SAVE-0002","message":"the save file is corrupt.","help":null,"context":{}}"#
/// );
/// // In the crate's tests: every variant meets the error contract.
/// assert_eq!(validate_error(&error, "SAVE"), Ok(()));
/// assert_eq!(validate_error(&SaveError::Corrupt, "SAVE"), Ok(()));
/// ```
pub trait EngineError: std::error::Error {
    /// The stable code of this error. Its area is the `req_prefix` of the defining crate's spec.
    fn code(&self) -> ErrorCode;

    /// What happened. Non-empty. Does not contain the code or the help text.
    fn message(&self) -> String;

    /// How to fix it, if knowable. `Some` is never empty.
    fn help(&self) -> Option<String>;

    /// Structured fields with the data the message talks about, in a fixed order per variant.
    fn context(&self) -> ErrorContext;
}

/// Writes the text form of `error`: `{code}: {message}`, then ` help: {help}` if
/// [`EngineError::help`] is `Some`. Nothing else is written (no trailing newline).
///
/// Every [`EngineError`] implementation's `Display` is exactly this call. The output depends
/// only on the error value (same bytes in every process and on every platform).
///
/// # Errors
///
/// Returns the error of the formatter `f`, if writing to it fails.
///
/// # Examples
///
/// ```
/// use liminal_core::error::CoreError;
///
/// // `CoreError`'s `Display` calls `write_display`.
/// let error = CoreError::InvalidTickRate { ticks_per_second: 0 };
/// assert_eq!(
///     error.to_string(),
///     "LIM-CORE-0020: tick rate `0` is outside 1 to 1000 ticks per second. \
///      help: use a tick rate from 1 to 1000; the default is 60 (`TickRate::DEFAULT`)."
/// );
/// ```
pub fn write_display(
    error: &dyn EngineError,
    f: &mut core::fmt::Formatter<'_>,
) -> core::fmt::Result {
    write!(f, "{}: {}", error.code(), error.message())?;
    if let Some(help) = error.help() {
        write!(f, " help: {help}")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Plain {
        help: Option<&'static str>,
    }

    impl EngineError for Plain {
        fn code(&self) -> ErrorCode {
            ErrorCode::new("TEST", 42)
        }
        fn message(&self) -> String {
            "something happened.".to_string()
        }
        fn help(&self) -> Option<String> {
            self.help.map(str::to_string)
        }
        fn context(&self) -> ErrorContext {
            ErrorContext::new()
        }
    }

    impl core::fmt::Display for Plain {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write_display(self, f)
        }
    }

    impl std::error::Error for Plain {}

    /// Verifies: CORE-R006
    #[test]
    fn write_display_writes_code_message_and_optional_help() {
        assert_eq!(
            Plain { help: None }.to_string(),
            "LIM-TEST-0042: something happened."
        );
        assert_eq!(
            Plain {
                help: Some("do this.")
            }
            .to_string(),
            "LIM-TEST-0042: something happened. help: do this."
        );
        // `Some("")` violates the contract, but the text form is still defined.
        assert_eq!(
            Plain { help: Some("") }.to_string(),
            "LIM-TEST-0042: something happened. help: "
        );
    }

    /// A formatter target that accepts `budget` bytes and then fails.
    struct Limited {
        budget: usize,
    }

    impl core::fmt::Write for Limited {
        fn write_str(&mut self, text: &str) -> core::fmt::Result {
            self.budget = self
                .budget
                .checked_sub(text.len())
                .ok_or(core::fmt::Error)?;
            Ok(())
        }
    }

    /// Verifies: CORE-R006
    #[test]
    fn write_display_returns_the_formatter_error() {
        use core::fmt::Write as _;
        let error = Plain {
            help: Some("do this."),
        };
        let full = error.to_string().len();
        let before_help = "LIM-TEST-0042: something happened.".len();
        for (budget, ok) in [
            (full, true),
            (full - 1, false),
            (before_help, false),
            (0, false),
        ] {
            let mut target = Limited { budget };
            assert_eq!(write!(target, "{error}").is_ok(), ok, "budget {budget}");
        }
    }
}
