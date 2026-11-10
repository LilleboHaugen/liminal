//! [`BenchError`]: the recoverable errors of the harness (spec §8, ADR-0017).

use liminal_core::error::{EngineError, ErrorCode, ErrorContext, write_display};

/// The errors of `liminal_benches` (codes `LIM-BENCH-NNNN`, spec §8).
///
/// Match on the variant or compare [`EngineError::code`]; message wording is not a stable
/// interface for programs.
///
/// # Examples
///
/// ```
/// use liminal_benches::{BenchError, Case};
/// use liminal_core::error::EngineError;
///
/// let error = Case::new("not a case id").unwrap_err();
/// assert_eq!(error.code().to_string(), "LIM-BENCH-0001");
/// assert!(matches!(error, BenchError::InvalidCaseId { .. }));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum BenchError {
    /// LIM-BENCH-0001: a case ID is not `<crate>/<group>/<case>` (BENCH-R001).
    InvalidCaseId {
        /// The rejected ID.
        id: String,
    },
    /// LIM-BENCH-0002: a bench target name is not 1 to 64 characters of `[a-z0-9_]`
    /// (BENCH-R003).
    InvalidTarget {
        /// The rejected name.
        target: String,
    },
    /// LIM-BENCH-0003: a case's crate segment differs from the bench target name (BENCH-R002).
    CaseOutsideTarget {
        /// The case ID.
        id: String,
        /// The bench target name.
        target: String,
    },
    /// LIM-BENCH-0004: a case ID was run twice in one bench target (BENCH-R004).
    DuplicateCase {
        /// The repeated case ID.
        id: String,
    },
    /// LIM-BENCH-0005: a command-line argument of a bench binary is not valid (BENCH-R041).
    InvalidArgument {
        /// The argument as given.
        argument: String,
        /// What is wrong with it.
        reason: String,
    },
    /// LIM-BENCH-0006: the result file could not be written (BENCH-R033).
    WriteFailed {
        /// The file path.
        path: String,
        /// The operating system's error text.
        reason: String,
    },
}

impl BenchError {
    /// The number part of the code.
    fn number(&self) -> u16 {
        match self {
            BenchError::InvalidCaseId { .. } => 1,
            BenchError::InvalidTarget { .. } => 2,
            BenchError::CaseOutsideTarget { .. } => 3,
            BenchError::DuplicateCase { .. } => 4,
            BenchError::InvalidArgument { .. } => 5,
            BenchError::WriteFailed { .. } => 6,
        }
    }
}

impl EngineError for BenchError {
    fn code(&self) -> ErrorCode {
        ErrorCode::new("BENCH", self.number())
    }

    fn message(&self) -> String {
        match self {
            BenchError::InvalidCaseId { id } => {
                format!("case ID `{id}` is not `<crate>/<group>/<case>`.")
            }
            BenchError::InvalidTarget { target } => {
                format!("bench target name `{target}` is not 1 to 64 characters of `[a-z0-9_]`.")
            }
            BenchError::CaseOutsideTarget { id, target } => {
                format!("case `{id}` does not belong to bench target `{target}`.")
            }
            BenchError::DuplicateCase { id } => {
                format!("case `{id}` was run twice in one bench target.")
            }
            BenchError::InvalidArgument { argument, reason } => {
                format!("bench argument `{argument}` is not valid: {reason}.")
            }
            BenchError::WriteFailed { path, reason } => {
                format!("could not write result file `{path}`: {reason}.")
            }
        }
    }

    fn help(&self) -> Option<String> {
        let help = match self {
            BenchError::InvalidCaseId { .. } => {
                "use three segments of 1 to 64 characters of `[a-z0-9_]`, separated by `/`, for \
                 example `liminal_core/hash/state_hasher_u64`."
            }
            BenchError::InvalidTarget { .. } => {
                "name the bench target after the crate it measures, for example `liminal_core`."
            }
            BenchError::CaseOutsideTarget { .. } => {
                "start the case ID with the bench target name, or move the case to the bench \
                 target of its crate."
            }
            BenchError::DuplicateCase { .. } => "give every case of a bench target its own ID.",
            BenchError::InvalidArgument { .. } => {
                "use `--filter <text>`, `--iterations <n>`, `--samples <n>` or `--out-dir <path>`; \
                 numbers are positive integers."
            }
            BenchError::WriteFailed { .. } => {
                "check that the output directory can be created and written, or pass another one \
                 with `--out-dir`."
            }
        };
        Some(help.to_string())
    }

    fn context(&self) -> ErrorContext {
        match self {
            BenchError::InvalidCaseId { id } | BenchError::DuplicateCase { id } => {
                ErrorContext::new().with("id", id.as_str())
            }
            BenchError::InvalidTarget { target } => {
                ErrorContext::new().with("target", target.as_str())
            }
            BenchError::CaseOutsideTarget { id, target } => ErrorContext::new()
                .with("id", id.as_str())
                .with("target", target.as_str()),
            BenchError::InvalidArgument { argument, reason } => ErrorContext::new()
                .with("argument", argument.as_str())
                .with("reason", reason.as_str()),
            BenchError::WriteFailed { path, reason } => ErrorContext::new()
                .with("path", path.as_str())
                .with("reason", reason.as_str()),
        }
    }
}

impl core::fmt::Display for BenchError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write_display(self, f)
    }
}

impl std::error::Error for BenchError {}

#[cfg(test)]
mod tests {
    use super::*;
    use liminal_core::error::validate_error;

    fn all() -> Vec<BenchError> {
        vec![
            BenchError::InvalidCaseId { id: "x".into() },
            BenchError::InvalidTarget { target: "X".into() },
            BenchError::CaseOutsideTarget {
                id: "a/b/c".into(),
                target: "d".into(),
            },
            BenchError::DuplicateCase { id: "a/b/c".into() },
            BenchError::InvalidArgument {
                argument: "--x".into(),
                reason: "unknown option".into(),
            },
            BenchError::WriteFailed {
                path: "/x.json".into(),
                reason: "denied".into(),
            },
        ]
    }

    /// Verifies: BENCH-R050
    #[test]
    fn every_variant_meets_the_error_contract_with_its_code() {
        let codes: Vec<String> = all().iter().map(|e| e.code().to_string()).collect();
        assert_eq!(
            codes,
            [
                "LIM-BENCH-0001",
                "LIM-BENCH-0002",
                "LIM-BENCH-0003",
                "LIM-BENCH-0004",
                "LIM-BENCH-0005",
                "LIM-BENCH-0006"
            ]
        );
        for error in all() {
            assert_eq!(validate_error(&error, "BENCH"), Ok(()), "{error}");
        }
    }
}
