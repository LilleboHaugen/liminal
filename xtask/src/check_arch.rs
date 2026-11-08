//! The `check-arch` command and `ci` step (`docs/specs/xtask.md` XTASK-R030 … R037 and the
//! "`check-arch` details").
//!
//! [`check`] is a pure function of the [`ArchModel`]. Only [`run_check`] touches the file system and
//! spawns `cargo metadata`, through the shared loader in [`crate::repo`].

mod attributes;
mod rules;

use std::io::Write;
use std::path::Path;

use crate::check;
use crate::cli::{CommandError, Exit};
use crate::finding::Finding;
use crate::repo::{self, ArchModel, LoadError, metadata};

/// Command and step name.
pub(crate) const NAME: &str = "check-arch";

/// The `check-arch` command as registered in the command table.
pub(crate) fn command(args: &[String], out: &mut dyn Write) -> Result<Exit, CommandError> {
    check::command(NAME, args, out, run_check)
}

/// Runs `cargo metadata` in `root`, loads the repository and checks it (the [`check::CheckFn`] of
/// this check).
///
/// # Errors
///
/// [`LoadError`] if `cargo metadata` fails or the repository cannot be read into a model.
pub(crate) fn run_check(root: &Path) -> Result<Vec<Finding>, LoadError> {
    let json = metadata::run_cargo(root, &metadata::cargo_program())?;
    Ok(check(&repo::load_arch(root, &json)?))
}

/// Every `check-arch` finding for `model`, sorted (spec "Check command output"), without
/// duplicates.
pub(crate) fn check(model: &ArchModel) -> Vec<Finding> {
    let mut findings = Vec::new();
    rules::registry(&model.architecture, &mut findings);
    rules::members(model, &mut findings);
    rules::dependencies(model, &mut findings);
    rules::forbid_unsafe(model, &mut findings);
    check::sort(&mut findings);
    findings.dedup();
    findings
}

#[cfg(test)]
mod fixture_tests;
#[cfg(test)]
mod tests;
