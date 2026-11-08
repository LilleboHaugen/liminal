//! Demo crate. Every line here is allowed.
#![forbid(unsafe_code)]

// TODO(M0-T005): a tracked marker naming an open task.
// FIXME(M0-T003): a task whose frontmatter is broken still exists.

/// Mentions in string literals are not code or comments.
pub const TEXT: &str = "todo!() dbg!(x) TODO: #[ignore]";

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "M0-T005: reason"]
    fn slow() {}

    #[test]
    #[ignore = "M0-T001: long-lived ignore of a finished task"]
    fn needs_gpu() {}
}
