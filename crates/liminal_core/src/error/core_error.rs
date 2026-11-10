//! [`CoreError`]: the errors of `liminal_core` itself (spec §8).

use super::{EngineError, ErrorCode, ErrorContext, write_display};

/// The errors of `liminal_core` (codes `LIM-CORE-NNNN`, spec §8).
///
/// Match on the variant or compare [`EngineError::code`]; message wording is not a stable
/// interface for programs. The variants `InvalidTickRate` and `ZeroRngState` are returned by the
/// `time` and `rng` modules, which are not implemented yet (task M1-T014).
///
/// # Examples
///
/// ```
/// use liminal_core::error::{ContextValue, CoreError, EngineError};
/// use liminal_core::handle::HandleAllocator;
///
/// let mut allocator = HandleAllocator::with_slot_limit(1);
/// allocator.allocate()?;
/// let error = allocator.allocate().unwrap_err();
///
/// assert_eq!(error, CoreError::HandleSlotsExhausted { slot_limit: 1 });
/// assert_eq!(error.code().to_string(), "LIM-CORE-0011");
/// assert_eq!(error.context().get("slot_limit"), Some(&ContextValue::UInt(1)));
/// assert_eq!(
///     error.to_string(),
///     "LIM-CORE-0011: all `1` handle slots are in use or retired. help: free handles that \
///      are no longer needed, or create the allocator with a higher slot limit."
/// );
/// # Ok::<(), CoreError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CoreError {
    /// LIM-CORE-0001: text given to [`ErrorCode::parse`] is not a valid error code.
    InvalidErrorCode {
        /// The rejected input.
        text: String,
    },
    /// LIM-CORE-0002: an error has a context key that does not match `^[a-z][a-z0-9_]*$`.
    InvalidContextKey {
        /// Code of the error that was checked.
        code: ErrorCode,
        /// The first bad key.
        key: String,
    },
    /// LIM-CORE-0003: an error has the same context key more than once.
    DuplicateContextKey {
        /// Code of the error that was checked.
        code: ErrorCode,
        /// The first repeated key.
        key: String,
    },
    /// LIM-CORE-0004: an error's message is empty.
    EmptyMessage {
        /// Code of the error that was checked.
        code: ErrorCode,
    },
    /// LIM-CORE-0005: an error's help is `Some("")`.
    EmptyHelp {
        /// Code of the error that was checked.
        code: ErrorCode,
    },
    /// LIM-CORE-0006: an error's `Display` is not the output of
    /// [`write_display`](super::write_display).
    DisplayMismatch {
        /// Code of the error that was checked.
        code: ErrorCode,
        /// The `write_display` output.
        expected: String,
        /// What the error's `Display` wrote.
        actual: String,
    },
    /// LIM-CORE-0007: an error's code does not use the area of its crate.
    AreaMismatch {
        /// Code of the error that was checked.
        code: ErrorCode,
        /// The area the code should have.
        expected_area: String,
    },
    /// LIM-CORE-0010: a handle is not live (freed already, or not issued by this allocator).
    StaleHandle {
        /// The handle's slot index.
        index: u32,
        /// The handle's generation.
        generation: u32,
    },
    /// LIM-CORE-0011: every handle slot is in use or retired.
    HandleSlotsExhausted {
        /// The allocator's slot limit.
        slot_limit: u32,
    },
    /// LIM-CORE-0020: a tick rate outside 1 to 1000 ticks per second.
    InvalidTickRate {
        /// The rejected rate.
        ticks_per_second: u32,
    },
    /// LIM-CORE-0030: an RNG state whose four words are all zero.
    ZeroRngState,
}

impl EngineError for CoreError {
    fn code(&self) -> ErrorCode {
        const fn core(number: u16) -> ErrorCode {
            ErrorCode::new("CORE", number)
        }
        match self {
            CoreError::InvalidErrorCode { .. } => const { core(1) },
            CoreError::InvalidContextKey { .. } => const { core(2) },
            CoreError::DuplicateContextKey { .. } => const { core(3) },
            CoreError::EmptyMessage { .. } => const { core(4) },
            CoreError::EmptyHelp { .. } => const { core(5) },
            CoreError::DisplayMismatch { .. } => const { core(6) },
            CoreError::AreaMismatch { .. } => const { core(7) },
            CoreError::StaleHandle { .. } => const { core(10) },
            CoreError::HandleSlotsExhausted { .. } => const { core(11) },
            CoreError::InvalidTickRate { .. } => const { core(20) },
            CoreError::ZeroRngState => const { core(30) },
        }
    }

    fn message(&self) -> String {
        match self {
            CoreError::InvalidErrorCode { text } => {
                format!("error code text `{text}` does not have the form LIM-AREA-NNNN.")
            }
            CoreError::InvalidContextKey { code, key } => {
                format!("error `{code}` has context key `{key}`, which is not snake_case.")
            }
            CoreError::DuplicateContextKey { code, key } => {
                format!("error `{code}` has context key `{key}` more than once.")
            }
            CoreError::EmptyMessage { code } => format!("error `{code}` has an empty message."),
            CoreError::EmptyHelp { code } => format!("error `{code}` has an empty help text."),
            CoreError::DisplayMismatch {
                code,
                expected,
                actual,
            } => format!("error `{code}` displays as `{actual}` instead of `{expected}`."),
            CoreError::AreaMismatch {
                code,
                expected_area,
            } => format!("error `{code}` does not use the area `{expected_area}`."),
            CoreError::StaleHandle { index, generation } => {
                format!("handle `{index}v{generation}` is not live.")
            }
            CoreError::HandleSlotsExhausted { slot_limit } => {
                format!("all `{slot_limit}` handle slots are in use or retired.")
            }
            CoreError::InvalidTickRate { ticks_per_second } => {
                format!("tick rate `{ticks_per_second}` is outside 1 to 1000 ticks per second.")
            }
            CoreError::ZeroRngState => {
                "the RNG state is all zeros, which the generator cannot use.".to_string()
            }
        }
    }

    fn help(&self) -> Option<String> {
        let help = match self {
            CoreError::InvalidErrorCode { .. } => {
                "write `LIM-`, the crate's requirement prefix (1 to 8 uppercase ASCII letters), \
                 `-`, and 4 digits from 0001 to 9999, for example `LIM-ECS-0007`."
            }
            CoreError::InvalidContextKey { .. } => "use keys that match `^[a-z][a-z0-9_]*$`.",
            CoreError::DuplicateContextKey { .. } => "give each context field a distinct key.",
            CoreError::EmptyMessage { .. } => "return a message that says what happened.",
            CoreError::EmptyHelp { .. } => {
                "return `None` from `help()` when no fix is known, otherwise a non-empty hint."
            }
            CoreError::DisplayMismatch { .. } => {
                "implement `Display` by calling `liminal_core::error::write_display`."
            }
            CoreError::AreaMismatch { .. } => {
                "use the `req_prefix` of the crate's spec as the code's area."
            }
            CoreError::StaleHandle { .. } => {
                "the handle was freed already or was not issued by this allocator; check \
                 `is_live` before freeing."
            }
            CoreError::HandleSlotsExhausted { .. } => {
                "free handles that are no longer needed, or create the allocator with a higher \
                 slot limit."
            }
            CoreError::InvalidTickRate { .. } => {
                "use a tick rate from 1 to 1000; the default is 60 (`TickRate::DEFAULT`)."
            }
            CoreError::ZeroRngState => {
                "restore a state saved with `Rng::state`, or create the RNG with `Rng::new`."
            }
        };
        Some(help.to_string())
    }

    fn context(&self) -> ErrorContext {
        let context = ErrorContext::new();
        match self {
            CoreError::InvalidErrorCode { text } => context.with("text", text.as_str()),
            CoreError::InvalidContextKey { code, key }
            | CoreError::DuplicateContextKey { code, key } => {
                context.with("code", *code).with("key", key.as_str())
            }
            CoreError::EmptyMessage { code } | CoreError::EmptyHelp { code } => {
                context.with("code", *code)
            }
            CoreError::DisplayMismatch {
                code,
                expected,
                actual,
            } => context
                .with("code", *code)
                .with("expected", expected.as_str())
                .with("actual", actual.as_str()),
            CoreError::AreaMismatch {
                code,
                expected_area,
            } => context
                .with("code", *code)
                .with("expected_area", expected_area.as_str()),
            CoreError::StaleHandle { index, generation } => context
                .with("index", *index)
                .with("generation", *generation),
            CoreError::HandleSlotsExhausted { slot_limit } => {
                context.with("slot_limit", *slot_limit)
            }
            CoreError::InvalidTickRate { ticks_per_second } => {
                context.with("ticks_per_second", *ticks_per_second)
            }
            CoreError::ZeroRngState => context,
        }
    }
}

/// The text form of [`write_display`]: `{code}: {message} help: {help}`.
impl core::fmt::Display for CoreError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write_display(self, f)
    }
}

impl std::error::Error for CoreError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{ContextValue, validate_error};

    /// Number of variants. The `match` is exhaustive, so adding a variant fails to compile here
    /// until it is counted, and then `samples` must grow to match.
    fn variant_count() -> usize {
        fn counted(error: &CoreError) -> usize {
            match error {
                CoreError::InvalidErrorCode { .. }
                | CoreError::InvalidContextKey { .. }
                | CoreError::DuplicateContextKey { .. }
                | CoreError::EmptyMessage { .. }
                | CoreError::EmptyHelp { .. }
                | CoreError::DisplayMismatch { .. }
                | CoreError::AreaMismatch { .. }
                | CoreError::StaleHandle { .. }
                | CoreError::HandleSlotsExhausted { .. }
                | CoreError::InvalidTickRate { .. }
                | CoreError::ZeroRngState => 11,
            }
        }
        counted(&CoreError::ZeroRngState)
    }

    fn text(value: &str) -> ContextValue {
        ContextValue::Text(value.to_string())
    }

    /// An error, its code number and its context fields.
    type Sample = (CoreError, u16, Vec<(&'static str, ContextValue)>);

    /// One value per variant with its code number and context from spec §8.
    fn samples() -> Vec<Sample> {
        let code = ErrorCode::new("GPU", 12);
        let code_text = || text("LIM-GPU-0012");
        vec![
            (
                CoreError::InvalidErrorCode {
                    text: "nope".to_string(),
                },
                1,
                vec![("text", text("nope"))],
            ),
            (
                CoreError::InvalidContextKey {
                    code,
                    key: "Bad".to_string(),
                },
                2,
                vec![("code", code_text()), ("key", text("Bad"))],
            ),
            (
                CoreError::DuplicateContextKey {
                    code,
                    key: "twice".to_string(),
                },
                3,
                vec![("code", code_text()), ("key", text("twice"))],
            ),
            (
                CoreError::EmptyMessage { code },
                4,
                vec![("code", code_text())],
            ),
            (
                CoreError::EmptyHelp { code },
                5,
                vec![("code", code_text())],
            ),
            (
                CoreError::DisplayMismatch {
                    code,
                    expected: "e".to_string(),
                    actual: "a".to_string(),
                },
                6,
                vec![
                    ("code", code_text()),
                    ("expected", text("e")),
                    ("actual", text("a")),
                ],
            ),
            (
                CoreError::AreaMismatch {
                    code,
                    expected_area: "ECS".to_string(),
                },
                7,
                vec![("code", code_text()), ("expected_area", text("ECS"))],
            ),
            (
                CoreError::StaleHandle {
                    index: 5,
                    generation: u32::MAX,
                },
                10,
                vec![
                    ("index", ContextValue::UInt(5)),
                    ("generation", ContextValue::UInt(4_294_967_295)),
                ],
            ),
            (
                CoreError::HandleSlotsExhausted { slot_limit: 0 },
                11,
                vec![("slot_limit", ContextValue::UInt(0))],
            ),
            (
                CoreError::InvalidTickRate {
                    ticks_per_second: 0,
                },
                20,
                vec![("ticks_per_second", ContextValue::UInt(0))],
            ),
            (CoreError::ZeroRngState, 30, vec![]),
        ]
    }

    /// Verifies: CORE-R013
    #[test]
    fn every_variant_has_its_code_and_context_and_passes_the_contract() {
        let samples = samples();
        assert_eq!(samples.len(), variant_count());
        for (error, number, context) in samples {
            assert_eq!(error.code(), ErrorCode::new("CORE", number), "{error:?}");
            let fields: Vec<(&str, ContextValue)> = error
                .context()
                .iter()
                .map(|(key, value)| (key, value.clone()))
                .collect();
            assert_eq!(fields, context, "{error:?}");
            assert_eq!(validate_error(&error, "CORE"), Ok(()), "{error:?}");
            // The message names every context value (spec §8: `{x}` placeholders).
            let message = error.message();
            for (_, value) in &context {
                let shown = match value {
                    ContextValue::Text(text) => text.clone(),
                    ContextValue::UInt(uint) => uint.to_string(),
                    other => panic!("unexpected context value {other:?}"),
                };
                assert!(message.contains(&shown), "{message}");
            }
            assert!(error.help().is_some_and(|help| help.ends_with('.')));
            assert!(std::error::Error::source(&error).is_none());
        }
    }
}
