//! Case definitions (spec §4 "Cases", BENCH-R001 … R006).

use std::num::NonZeroU64;

use crate::BenchError;

/// Longest segment of a case ID or bench target name, in bytes.
const SEGMENT_MAX: usize = 64;

/// Whether `text` is 1 to 64 bytes of `[a-z0-9_]`.
pub(crate) fn is_segment(text: &str) -> bool {
    (1..=SEGMENT_MAX).contains(&text.len())
        && text
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

/// Whether a case may run on more than one thread (BENCH-R005).
///
/// Only [`Threading::Single`] cases are measured by the CI instruction-count gate (ADR-0023
/// item 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Threading {
    /// The routine runs on the calling thread only. The default.
    Single,
    /// The routine uses more than one thread; excluded from the instruction-count gate.
    Multi,
}

/// One benchmark case: a stable ID, an optional element count for throughput, and its
/// threading marker.
///
/// # Examples
///
/// ```
/// use std::num::NonZeroU64;
/// use liminal_benches::{Case, Threading};
///
/// let case = Case::new("liminal_core/hash/state_hasher_bytes_1k")?
///     .with_elements(NonZeroU64::new(1024).expect("non-zero"));
/// assert_eq!(case.id(), "liminal_core/hash/state_hasher_bytes_1k");
/// assert_eq!(case.crate_name(), "liminal_core");
/// assert_eq!(case.elements(), Some(1024));
/// assert_eq!(case.threading(), Threading::Single);
/// assert_eq!(case.multi_threaded().threading(), Threading::Multi);
/// # Ok::<(), liminal_benches::BenchError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Case {
    id: String,
    elements: Option<NonZeroU64>,
    threading: Threading,
}

impl Case {
    /// A single-threaded case without an element count.
    ///
    /// # Errors
    ///
    /// [`BenchError::InvalidCaseId`] unless `id` is three segments separated by `/`, each 1 to
    /// 64 characters of `[a-z0-9_]` (BENCH-R001).
    pub fn new(id: &str) -> Result<Case, BenchError> {
        let segments: Vec<&str> = id.split('/').collect();
        if segments.len() != 3 || !segments.iter().all(|s| is_segment(s)) {
            return Err(BenchError::InvalidCaseId { id: id.to_string() });
        }
        Ok(Case {
            id: id.to_string(),
            elements: None,
            threading: Threading::Single,
        })
    }

    /// Sets the number of elements one iteration processes; the result then reports throughput
    /// (BENCH-R023).
    #[must_use]
    pub fn with_elements(mut self, elements: NonZeroU64) -> Case {
        self.elements = Some(elements);
        self
    }

    /// Marks the case as using more than one thread (BENCH-R005).
    #[must_use]
    pub fn multi_threaded(mut self) -> Case {
        self.threading = Threading::Multi;
        self
    }

    /// The case ID, `<crate>/<group>/<case>`.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The first segment of the ID: the crate the case measures.
    #[must_use]
    pub fn crate_name(&self) -> &str {
        self.id.split('/').next().unwrap_or_default()
    }

    /// Elements per iteration, if declared.
    #[must_use]
    pub fn elements(&self) -> Option<u64> {
        self.elements.map(NonZeroU64::get)
    }

    /// The threading marker.
    #[must_use]
    pub fn threading(&self) -> Threading {
        self.threading
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies: BENCH-R001
    #[test]
    fn case_ids_are_three_lowercase_segments() {
        for id in [
            "liminal_core/hash/u64",
            "a/b/c",
            "x9/group_2/case_3",
            &format!("{0}/{0}/{0}", "a".repeat(64)),
        ] {
            assert_eq!(
                Case::new(id).map(|c| c.id().to_string()),
                Ok(id.to_string())
            );
        }
        for id in [
            "",
            "a/b",
            "a/b/c/d",
            "a//c",
            "A/b/c",
            "a/b-c/d",
            "a/b/c ",
            "a/b/ü",
            &format!("a/b/{}", "c".repeat(65)),
        ] {
            assert_eq!(
                Case::new(id),
                Err(BenchError::InvalidCaseId { id: id.to_string() }),
                "{id:?}"
            );
        }
    }

    /// Verifies: BENCH-R005
    #[test]
    fn cases_are_single_threaded_unless_marked() {
        let case = Case::new("a/b/c").expect("valid");
        assert_eq!(case.threading(), Threading::Single);
        assert_eq!(case.elements(), None);
        assert_eq!(case.multi_threaded().threading(), Threading::Multi);
    }
}
