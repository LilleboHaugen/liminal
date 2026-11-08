//! Repository automation and quality gates for Liminal, run as `cargo xtask <command>`.
//!
//! Specification: `docs/specs/xtask.md`. All logic lives in this library so that it can be tested
//! directly; `src/main.rs` only forwards the process arguments to [`run`].
//!
//! Commands are registered in [`COMMANDS`]. Library code never prints: it writes to the
//! [`std::io::Write`] values it is given, so tests can capture the output.
#![forbid(unsafe_code)]

mod ci;
mod cli;
mod finding;
mod process;
mod repo;

pub use cli::{COMMANDS, Command, CommandError, CommandFn, Exit, run};
