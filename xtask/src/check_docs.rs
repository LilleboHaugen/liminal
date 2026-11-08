//! The `check-docs` command and `ci` step (`docs/specs/xtask.md` XTASK-R020 … R027 and the
//! "`check-docs` details").
//!
//! [`check`] is a pure function of the [`RepoModel`]. Only [`run_check`] touches the file system,
//! through the shared loader in [`crate::repo`].

mod fields;
mod indexes;
mod rules;

use std::io::Write;
use std::path::Path;

use crate::check;
use crate::cli::{CommandError, Exit};
use crate::finding::Finding;
use crate::frontmatter::{self, Frontmatter};
use crate::markdown;
use crate::repo::{self, LoadError, RepoModel};

/// Command and step name.
pub(crate) const NAME: &str = "check-docs";

/// The `check-docs` command as registered in the command table.
pub(crate) fn command(args: &[String], out: &mut dyn Write) -> Result<Exit, CommandError> {
    check::command(NAME, args, out, run_check)
}

/// Loads the repository at `root` and checks it (the [`check::CheckFn`] of this check).
///
/// # Errors
///
/// [`LoadError`] if the repository cannot be read into a model.
pub(crate) fn run_check(root: &Path) -> Result<Vec<Finding>, LoadError> {
    Ok(check(&repo::load(root)?))
}

/// Every `check-docs` finding for `model`, sorted (spec "Check command output").
pub(crate) fn check(model: &RepoModel) -> Vec<Finding> {
    let mut findings = Vec::new();
    let adrs = rules::adrs(model, &mut findings);
    let specs = rules::specs(model, &adrs, &mut findings);
    rules::tasks(model, &mut findings);
    links(model, &mut findings);
    indexes::adr_index(model, &adrs, &mut findings);
    member_specs(model, &mut findings);
    indexes::spec_index(model, &specs, &mut findings);
    check::sort(&mut findings);
    findings
}

/// Parses the frontmatter of `path`, reporting a syntax error as XTASK-R020.
fn parse_frontmatter(path: &str, text: &str, findings: &mut Vec<Finding>) -> Option<Frontmatter> {
    match frontmatter::parse(text) {
        Ok(frontmatter) => Some(frontmatter),
        Err(error) => {
            findings.push(Finding::error(
                "XTASK-R020",
                format!("{path}:{}", error.line),
                format!("frontmatter: {}", error.message),
                "Follow the restricted YAML subset in docs/process/DOCUMENTATION.md §Frontmatter",
            ));
            None
        }
    }
}

/// Resolves a link `target` written in `from` (a repository-relative file) to a
/// repository-relative path. `#…` and `?…` are removed first. Returns `None` if the target leaves
/// the repository. The empty string is the repository root.
pub(crate) fn resolve(from: &str, target: &str) -> Option<String> {
    let end = target.find(['#', '?']).unwrap_or(target.len());
    let target = &target[..end];
    let (mut parts, target): (Vec<&str>, &str) = if let Some(rest) = target.strip_prefix('/') {
        (Vec::new(), rest)
    } else {
        let dir: Vec<&str> = from.split('/').collect();
        (dir[..dir.len() - 1].to_vec(), target)
    };
    for part in target.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            part => parts.push(part),
        }
    }
    Some(parts.join("/"))
}

/// Whether `target` starts with a URI scheme (`^[A-Za-z][A-Za-z0-9+.-]*:`).
fn has_scheme(target: &str) -> bool {
    let Some((scheme, _)) = target.split_once(':') else {
        return false;
    };
    let mut chars = scheme.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-'))
}

/// XTASK-R024: every relative link in root and `docs/` Markdown resolves.
fn links(model: &RepoModel, findings: &mut Vec<Finding>) {
    for (path, text) in &model.markdown {
        for link in markdown::links(text) {
            let target = link.target.as_str();
            if target.starts_with('#') || has_scheme(target) {
                continue;
            }
            let resolved = resolve(path, target);
            let exists = resolved
                .as_deref()
                .is_some_and(|resolved| resolved.is_empty() || model.paths.contains(resolved));
            if !exists {
                let message = match &resolved {
                    Some(resolved) => {
                        format!("link target `{target}` does not exist (resolved to `{resolved}`)")
                    }
                    None => format!("link target `{target}` leaves the repository"),
                };
                findings.push(Finding::error(
                    "XTASK-R024",
                    format!("{path}:{}", link.line),
                    message,
                    "Fix the path; it is relative to the linking file's directory, or to the \
                     repository root if it starts with `/`",
                ));
            }
        }
    }
}

/// XTASK-R026: every workspace member has a spec at `docs/specs/<crate>.md`.
fn member_specs(model: &RepoModel, findings: &mut Vec<Finding>) {
    for member in &model.members {
        let spec = format!("docs/specs/{}.md", member.name);
        if !model.markdown.contains_key(&spec) {
            findings.push(Finding::error(
                "XTASK-R026",
                format!("{}/Cargo.toml", member.path),
                format!("workspace member `{}` has no spec at {spec}", member.name),
                "Write the spec from docs/specs/_TEMPLATE.md before writing code for the crate",
            ));
        }
    }
}

#[cfg(test)]
mod fixture_tests;
#[cfg(test)]
mod tests;
