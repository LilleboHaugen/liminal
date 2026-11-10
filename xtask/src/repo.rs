//! The shared repo-model loader (`docs/specs/xtask.md` §3, §4 "Repo model").
//!
//! Loaders are the only code that touches the file system. They read the repository into a
//! [`RepoModel`]. Checks are pure functions of that model, so they run on fixtures without cargo.
//! Every check extends this module instead of reading files itself.

pub(crate) mod architecture;
pub(crate) mod clippy;
pub(crate) mod coverage;
pub(crate) mod metadata;
pub(crate) mod sources;
pub(crate) mod workspace;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

pub(crate) use architecture::Architecture;
pub(crate) use metadata::Package;
pub(crate) use workspace::Member;

/// The repository root: the parent of the `xtask/` manifest directory.
///
/// Fixed at compile time, so `cargo xtask` works the same from any directory (XTASK-R001).
pub(crate) fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("invariant: xtask/ is a direct child of the repository root")
        .to_path_buf()
}

/// Directories at the repository root that are never part of the model: build output and git data.
const EXCLUDED_ROOT_DIRS: &[&str] = &[".git", "target"];

/// An internal failure (spec §8): the repository could not be read into a model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LoadError {
    /// `LIM-XTASK-nnnn`.
    pub code: &'static str,
    /// The file concerned, relative to the repository root where possible.
    pub path: String,
    /// What went wrong.
    pub detail: String,
    /// How to fix it.
    pub help: &'static str,
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {}: {} (help: {})",
            self.code, self.path, self.detail, self.help
        )
    }
}

impl LoadError {
    /// LIM-XTASK-0001: a file or directory could not be read.
    pub(crate) fn read(path: impl Into<String>, error: &std::io::Error) -> Self {
        Self {
            code: "LIM-XTASK-0001",
            path: path.into(),
            detail: format!("could not be read: {error}"),
            help: "Check the path and permissions",
        }
    }
}

/// Everything the checks know about the repository.
#[derive(Debug, Clone)]
pub(crate) struct RepoModel {
    /// Markdown files at the root and under `docs/`: repository-relative `/`-separated path →
    /// contents.
    pub markdown: BTreeMap<String, String>,
    /// Every file and directory in the repository except [`EXCLUDED_ROOT_DIRS`], as
    /// repository-relative `/`-separated paths. Used to resolve links.
    pub paths: BTreeSet<String>,
    /// `architecture.toml`, deserialized (semantic validation is `check-arch`).
    pub architecture: Architecture,
    /// Workspace members from the root `Cargo.toml`, in manifest order.
    pub members: Vec<Member>,
}

impl RepoModel {
    /// ADR files: `docs/adr/NNNN-*.md` except `0000-template.md` (XTASK-R021).
    pub(crate) fn adr_files(&self) -> impl Iterator<Item = (&str, &str)> {
        self.markdown_in("docs/adr/").filter(|(path, _)| {
            let name = file_name(path);
            let digits = name.get(..4).unwrap_or("");
            digits.bytes().all(|byte| byte.is_ascii_digit())
                && name.as_bytes().get(4) == Some(&b'-')
                && name != "0000-template.md"
        })
    }

    /// Spec files: `docs/specs/*.md` except `_TEMPLATE.md` and `README.md` (XTASK-R022).
    pub(crate) fn spec_files(&self) -> impl Iterator<Item = (&str, &str)> {
        self.markdown_in("docs/specs/")
            .filter(|(path, _)| !matches!(file_name(path), "_TEMPLATE.md" | "README.md"))
    }

    /// Task files: `docs/tasks/M*-T*.md` (XTASK-R023).
    pub(crate) fn task_files(&self) -> impl Iterator<Item = (&str, &str)> {
        self.markdown_in("docs/tasks/").filter(|(path, _)| {
            let name = file_name(path);
            name.strip_prefix('M')
                .is_some_and(|rest| rest.contains("-T"))
        })
    }

    /// Markdown files directly inside `dir` (which ends with `/`).
    fn markdown_in<'a>(&'a self, dir: &'a str) -> impl Iterator<Item = (&'a str, &'a str)> {
        self.markdown
            .iter()
            .filter(move |(path, _)| {
                path.strip_prefix(dir)
                    .is_some_and(|rest| !rest.contains('/'))
            })
            .map(|(path, text)| (path.as_str(), text.as_str()))
    }
}

/// The last `/`-separated component of `path`.
pub(crate) fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// Reads the repository at `root` into a model.
///
/// # Errors
///
/// [`LoadError`] if a file cannot be read (LIM-XTASK-0001), `architecture.toml` is invalid
/// (LIM-XTASK-0003), or a `Cargo.toml` is invalid (LIM-XTASK-0004).
pub(crate) fn load(root: &Path) -> Result<RepoModel, LoadError> {
    let mut paths = BTreeSet::new();
    walk(root, root, &mut paths)?;
    let mut markdown = BTreeMap::new();
    for path in &paths {
        let at_root = !path.contains('/');
        // `*.md` exactly, as the spec's file patterns say (case-sensitive on purpose).
        let is_markdown = Path::new(path).extension().is_some_and(|ext| ext == "md");
        if is_markdown && (at_root || path.starts_with("docs/")) {
            let full = root.join(path);
            if full.is_file() {
                markdown.insert(path.clone(), read(root, path)?);
            }
        }
    }
    let architecture = architecture::parse(&read(root, "architecture.toml")?)?;
    let members = workspace::load(root)?;
    Ok(RepoModel {
        markdown,
        paths,
        architecture,
        members,
    })
}

/// What `check-arch` needs (spec "`check-arch` details"): `architecture.toml`, the workspace
/// packages from `cargo metadata`, and the crate root files of those packages.
#[derive(Debug, Clone)]
pub(crate) struct ArchModel {
    /// `architecture.toml`, deserialized.
    pub architecture: Architecture,
    /// Workspace member packages, sorted by name.
    pub packages: Vec<Package>,
    /// Contents of every crate root file ([`crate_roots`]): repository-relative path → text.
    pub sources: BTreeMap<String, String>,
    /// For each workspace package registered with `deterministic = true` (first registry entry of
    /// its name), the clippy configuration that applies to it ([`clippy::effective`]), by package
    /// name. Other packages have no entry.
    pub clippy: BTreeMap<String, Option<clippy::ClippyConfig>>,
}

/// Target kinds whose root file is a crate root checked by XTASK-R035: libraries of every crate
/// type, proc-macros and binaries. Tests, examples, benches and build scripts are not.
const CRATE_ROOT_KINDS: &[&str] = &[
    "lib",
    "rlib",
    "dylib",
    "cdylib",
    "staticlib",
    "proc-macro",
    "bin",
];

/// The crate root files of `package` ([`CRATE_ROOT_KINDS`]), in target order.
pub(crate) fn crate_roots(package: &Package) -> impl Iterator<Item = &str> {
    package
        .targets
        .iter()
        .filter(|target| {
            target
                .kinds
                .iter()
                .any(|kind| CRATE_ROOT_KINDS.contains(&kind.as_str()))
        })
        .map(|target| target.src_path.as_str())
}

/// Builds the `check-arch` model of the repository at `root` from `cargo metadata` JSON (from
/// [`metadata::run_cargo`], or a saved fixture).
///
/// # Errors
///
/// [`LoadError`]: LIM-XTASK-0001 if `architecture.toml` or a crate root file cannot be read,
/// LIM-XTASK-0002 if the JSON is not `cargo metadata` output, LIM-XTASK-0003 if
/// `architecture.toml` is invalid, LIM-XTASK-0001 or LIM-XTASK-0004 if the clippy configuration
/// of a deterministic crate cannot be read or is invalid.
pub(crate) fn load_arch(root: &Path, metadata_json: &str) -> Result<ArchModel, LoadError> {
    let architecture = architecture::parse(&read(root, "architecture.toml")?)?;
    let packages = metadata::parse(metadata_json)?;
    let mut sources = BTreeMap::new();
    for package in &packages {
        for path in crate_roots(package) {
            if !sources.contains_key(path) {
                sources.insert(path.to_string(), read(root, path)?);
            }
        }
    }
    let mut clippy = BTreeMap::new();
    for package in &packages {
        let deterministic = architecture
            .crates
            .iter()
            .find(|entry| entry.name == package.name)
            .is_some_and(|entry| entry.deterministic);
        if deterministic {
            let config = clippy::effective(root, &package.manifest_dir)?;
            clippy.insert(package.name.clone(), config);
        }
    }
    Ok(ArchModel {
        architecture,
        packages,
        sources,
        clippy,
    })
}

/// Reads a repository-relative file as UTF-8.
pub(crate) fn read(root: &Path, path: &str) -> Result<String, LoadError> {
    std::fs::read_to_string(root.join(path)).map_err(|error| LoadError::read(path, &error))
}

/// Collects every file and directory below `dir` into `paths`, sorted by the set.
fn walk(root: &Path, dir: &Path, paths: &mut BTreeSet<String>) -> Result<(), LoadError> {
    let relative = |path: &Path| -> String {
        path.strip_prefix(root)
            .unwrap_or(path)
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/")
    };
    let entries = std::fs::read_dir(dir).map_err(|error| LoadError::read(relative(dir), &error))?;
    for entry in entries {
        let entry = entry.map_err(|error| LoadError::read(relative(dir), &error))?;
        let path = entry.path();
        let name = relative(&path);
        if dir == root && EXCLUDED_ROOT_DIRS.contains(&name.as_str()) {
            continue;
        }
        let is_dir = entry
            .file_type()
            .map_err(|error| LoadError::read(name.clone(), &error))?
            .is_dir();
        paths.insert(name);
        if is_dir {
            walk(root, &path, paths)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_contains_the_workspace_manifest_and_xtask() {
        let root = root();
        assert!(root.join("Cargo.toml").is_file());
        assert!(root.join("xtask/Cargo.toml").is_file());
        assert!(root.join("architecture.toml").is_file());
    }

    #[test]
    fn loads_the_real_repository() {
        let model = load(&root()).unwrap();
        assert!(model.markdown.contains_key("AGENTS.md"));
        assert!(model.markdown.contains_key("docs/process/WORKFLOW.md"));
        assert!(!model.markdown.keys().any(|path| path.starts_with("xtask/")));
        assert!(model.paths.contains("xtask/src") && model.paths.contains("xtask/src/repo.rs"));
        let excluded: Vec<&String> = model
            .paths
            .iter()
            .filter(|path| {
                let first = path.split('/').next().unwrap_or("");
                first == "target" || first == ".git"
            })
            .collect();
        assert_eq!(excluded, Vec::<&String>::new());
        // The real workspace gains a member per engine crate; check membership, not the exact
        // list (M1-T013 added the first engine crate).
        let members: Vec<&str> = model.members.iter().map(|m| m.name.as_str()).collect();
        assert!(members.contains(&"xtask") && members.contains(&"liminal_core"));
        assert!(
            members
                .iter()
                .all(|name| *name == "xtask" || name.starts_with("liminal_"))
        );
        assert!(model.architecture.crates.iter().any(|c| c.name == "xtask"));

        let adrs: Vec<&str> = model.adr_files().map(|(path, _)| file_name(path)).collect();
        assert!(adrs.contains(&"0001-record-architecture-decisions.md"));
        assert!(!adrs.contains(&"0000-template.md") && !adrs.contains(&"README.md"));
        let specs: Vec<&str> = model.spec_files().map(|(path, _)| path).collect();
        // The real repository gains a spec per crate; check membership and exclusions, not the
        // exact list (M1-T012 added the first engine spec).
        assert!(specs.contains(&"docs/specs/xtask.md"));
        assert!(specs.iter().all(|path| path.starts_with("docs/specs/")));
        assert!(
            !specs
                .iter()
                .any(|path| path.ends_with("/_TEMPLATE.md") || path.ends_with("/README.md"))
        );
        let tasks: Vec<&str> = model
            .task_files()
            .map(|(path, _)| file_name(path))
            .collect();
        assert!(tasks.contains(&"M0-T003-check-docs.md"));
        assert!(!tasks.iter().any(|name| name.starts_with(['_', 'R'])));
    }
}
