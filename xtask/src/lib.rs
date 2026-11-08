//! Repository automation and quality gates for Liminal, run as `cargo xtask <command>`.
//!
//! Specification: `docs/specs/xtask.md`. All logic lives in this library so that it can be tested
//! directly; `src/main.rs` only forwards the process arguments to [`run`].
//!
//! Commands are registered in [`COMMANDS`]. Library code never prints: it writes to the
//! [`std::io::Write`] values it is given, so tests can capture the output.
#![forbid(unsafe_code)]

mod check;
mod check_arch;
mod check_docs;
mod ci;
mod cli;
mod finding;
mod frontmatter;
mod markdown;
mod process;
mod repo;
#[cfg(test)]
mod test_support;

pub use cli::{COMMANDS, Command, CommandError, CommandFn, Exit, run};
