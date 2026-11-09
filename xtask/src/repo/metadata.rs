//! Workspace packages from `cargo metadata --format-version 1 --no-deps` (`docs/specs/xtask.md`
//! §3, "`check-arch` details").
//!
//! [`run_cargo`] is the only function that spawns a process. [`parse`] turns its JSON into
//! [`Package`]s with repository-relative, `/`-separated paths, so `check-arch` fixtures can use
//! saved JSON instead of running cargo. Only the fields `check-arch` needs are read; serde ignores
//! the rest.

use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Stdio};

use serde::Deserialize;

use super::LoadError;

/// One workspace member package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Package {
    /// `[package].name`.
    pub name: String,
    /// Directory of the package's `Cargo.toml`, relative to the workspace root.
    pub manifest_dir: String,
    /// Build targets, in metadata order.
    pub targets: Vec<Target>,
    /// Declared dependencies of every kind, in metadata order.
    pub dependencies: Vec<Dependency>,
}

/// One build target of a package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Target {
    /// Cargo's target kinds, e.g. `["lib"]`, `["proc-macro"]`, `["bin"]`, `["custom-build"]`.
    pub kinds: Vec<String>,
    /// Root source file, relative to the workspace root.
    pub src_path: String,
}

/// How a dependency is declared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum DependencyKind {
    /// `[dependencies]` (metadata kind `null`).
    Normal,
    /// `[build-dependencies]`.
    Build,
    /// `[dev-dependencies]`.
    Dev,
}

impl DependencyKind {
    /// The name used in messages: `normal`, `build` or `dev`.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            DependencyKind::Normal => "normal",
            DependencyKind::Build => "build",
            DependencyKind::Dev => "dev",
        }
    }
}

/// One declared dependency. Target-specific dependencies (`[target.'cfg(…)'.dependencies]`) are
/// reported like unconditional ones.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Dependency {
    /// The package name (the crates.io name, never the `rename`d one).
    pub name: String,
    /// Normal, build or dev.
    pub kind: DependencyKind,
    /// For a path dependency inside the workspace root: its directory relative to the root.
    /// `None` for registry and git dependencies and for paths outside the workspace root.
    pub path: Option<String>,
}

#[derive(Deserialize)]
struct RawMetadata {
    workspace_root: String,
    workspace_members: Vec<String>,
    packages: Vec<RawPackage>,
}

#[derive(Deserialize)]
struct RawPackage {
    id: String,
    name: String,
    manifest_path: String,
    targets: Vec<RawTarget>,
    dependencies: Vec<RawDependency>,
}

#[derive(Deserialize)]
struct RawTarget {
    kind: Vec<String>,
    src_path: String,
}

#[derive(Deserialize)]
struct RawDependency {
    name: String,
    kind: Option<String>,
    path: Option<String>,
}

/// Help text of LIM-XTASK-0002.
const PARSE_HELP: &str = "Run `cargo xtask doctor`; tool version may be unsupported";

fn unparseable(detail: impl Into<String>) -> LoadError {
    LoadError {
        code: "LIM-XTASK-0002",
        path: "cargo metadata".to_string(),
        detail: detail.into(),
        help: PARSE_HELP,
    }
}

/// `path` with `\` replaced by `/`, so Windows and Unix metadata compare the same way.
fn slashes(path: &str) -> String {
    path.replace('\\', "/")
}

/// `path` relative to `root` (both already `/`-separated): `Some("")` for `root` itself,
/// `Some(rest)` for a path below it, `None` otherwise.
fn relative(root: &str, path: &str) -> Option<String> {
    let rest = path.strip_prefix(root.trim_end_matches('/'))?;
    if rest.is_empty() {
        return Some(String::new());
    }
    let rest = rest.strip_prefix('/')?;
    Some(rest.to_string())
}

/// Parses `cargo metadata --format-version 1` JSON into the workspace member packages, sorted by
/// name.
///
/// # Errors
///
/// LIM-XTASK-0002 if the JSON does not have the expected shape, a dependency kind is unknown, or a
/// member's manifest or target lies outside the workspace root.
pub(crate) fn parse(json: &str) -> Result<Vec<Package>, LoadError> {
    let raw: RawMetadata = serde_json::from_str(json)
        .map_err(|error| unparseable(format!("output is not the expected JSON: {error}")))?;
    let root = slashes(&raw.workspace_root);
    let inside = |path: &str, what: &str, package: &str| {
        relative(&root, &slashes(path)).ok_or_else(|| {
            unparseable(format!(
                "{what} `{path}` of package `{package}` is outside the workspace root `{root}`"
            ))
        })
    };
    let mut packages = Vec::new();
    for package in raw.packages {
        if !raw.workspace_members.contains(&package.id) {
            continue;
        }
        let manifest = inside(&package.manifest_path, "manifest", &package.name)?;
        let manifest_dir = match manifest.rsplit_once('/') {
            Some((dir, "Cargo.toml")) => dir.to_string(),
            None if manifest == "Cargo.toml" => String::new(),
            _ => {
                return Err(unparseable(format!(
                    "manifest `{}` of package `{}` is not a Cargo.toml",
                    package.manifest_path, package.name
                )));
            }
        };
        let mut targets = Vec::new();
        for target in package.targets {
            targets.push(Target {
                kinds: target.kind,
                src_path: inside(&target.src_path, "target", &package.name)?,
            });
        }
        let mut dependencies = Vec::new();
        for dependency in package.dependencies {
            let kind = match dependency.kind.as_deref() {
                None | Some("normal") => DependencyKind::Normal,
                Some("build") => DependencyKind::Build,
                Some("dev") => DependencyKind::Dev,
                Some(other) => {
                    return Err(unparseable(format!(
                        "dependency `{}` of package `{}` has unknown kind `{other}`",
                        dependency.name, package.name
                    )));
                }
            };
            dependencies.push(Dependency {
                path: dependency
                    .path
                    .and_then(|path| relative(&root, &slashes(&path))),
                name: dependency.name,
                kind,
            });
        }
        packages.push(Package {
            name: package.name,
            manifest_dir,
            targets,
            dependencies,
        });
    }
    packages.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(packages)
}

/// Runs `<cargo> metadata --format-version 1 --no-deps --offline` in `root` and returns its JSON.
///
/// `--no-deps` lists the workspace members with their declared dependencies and needs neither
/// the network nor downloaded crates; `--offline` makes sure of it.
///
/// # Errors
///
/// LIM-XTASK-0005 if cargo cannot be started or exits unsuccessfully (the detail holds its
/// stderr); LIM-XTASK-0002 if its output is not UTF-8.
pub(crate) fn run_cargo(root: &Path, cargo: &OsStr) -> Result<String, LoadError> {
    let failed = |detail: String| LoadError {
        code: "LIM-XTASK-0005",
        path: "cargo metadata".to_string(),
        detail,
        help: "Run `cargo metadata --no-deps` in the repository root to see the error, and fix \
               the Cargo.toml it names",
    };
    let output = Command::new(cargo)
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--offline",
        ])
        .current_dir(root)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| failed(crate::process::start_error(cargo, root, &error)))?;
    if !output.status.success() {
        return Err(failed(format!(
            "failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim_end()
        )));
    }
    String::from_utf8(output.stdout).map_err(|_| unparseable("output is not UTF-8"))
}

/// The `cargo` program: `$CARGO` (set by cargo for `cargo run` and `cargo test`), else `cargo`
/// from `PATH`. Same rule as the `ci` runner.
pub(crate) fn cargo_program() -> std::ffi::OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| std::ffi::OsString::from("cargo"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shape of real `cargo metadata` output (cargo 1.99), trimmed to two packages. The second
    /// package is not a workspace member and must be ignored.
    fn sample(root: &str, sep: &str) -> String {
        let p = |path: &str| format!("{root}{sep}{}", path.replace('/', sep)).replace('\\', "\\\\");
        format!(
            r#"{{
  "packages": [
    {{ "name": "app", "version": "0.0.0", "id": "path+file:///w/app#0.0.0",
       "manifest_path": "{manifest}", "source": null, "edition": "2024",
       "targets": [
         {{ "kind": ["lib"], "crate_types": ["lib"], "name": "app", "src_path": "{lib}" }},
         {{ "kind": ["bin"], "crate_types": ["bin"], "name": "tool", "src_path": "{bin}" }}
       ],
       "dependencies": [
         {{ "name": "core", "kind": null, "rename": null, "target": null, "path": "{core}",
            "source": null, "req": "*" }},
         {{ "name": "serde", "kind": null, "rename": "renamed", "target": null,
            "source": "registry+https://github.com/rust-lang/crates.io-index", "req": "^1" }},
         {{ "name": "winit", "kind": null, "rename": null, "target": "cfg(windows)",
            "source": "registry+https://github.com/rust-lang/crates.io-index", "req": "^0.30" }},
         {{ "name": "cc", "kind": "build", "rename": null, "target": null,
            "source": "registry+https://github.com/rust-lang/crates.io-index", "req": "^1" }},
         {{ "name": "proptest", "kind": "dev", "rename": null, "target": null,
            "source": "registry+https://github.com/rust-lang/crates.io-index", "req": "^1" }},
         {{ "name": "outside", "kind": null, "rename": null, "target": null,
            "path": "/elsewhere/outside", "source": null, "req": "*" }}
       ] }},
    {{ "name": "other", "version": "0.0.0", "id": "registry+x#other@1.0.0",
       "manifest_path": "/registry/other/Cargo.toml", "targets": [], "dependencies": [] }}
  ],
  "workspace_members": ["path+file:///w/app#0.0.0"],
  "workspace_root": "{root_json}",
  "version": 1
}}"#,
            manifest = p("crates/app/Cargo.toml"),
            lib = p("crates/app/src/lib.rs"),
            bin = p("crates/app/src/bin/tool.rs"),
            core = p("crates/core"),
            root_json = root.replace('\\', "\\\\"),
        )
    }

    fn expected() -> Vec<Package> {
        let dep = |name: &str, kind, path: Option<&str>| Dependency {
            name: name.to_string(),
            kind,
            path: path.map(ToString::to_string),
        };
        vec![Package {
            name: "app".to_string(),
            manifest_dir: "crates/app".to_string(),
            targets: vec![
                Target {
                    kinds: vec!["lib".to_string()],
                    src_path: "crates/app/src/lib.rs".to_string(),
                },
                Target {
                    kinds: vec!["bin".to_string()],
                    src_path: "crates/app/src/bin/tool.rs".to_string(),
                },
            ],
            dependencies: vec![
                dep("core", DependencyKind::Normal, Some("crates/core")),
                dep("serde", DependencyKind::Normal, None),
                dep("winit", DependencyKind::Normal, None),
                dep("cc", DependencyKind::Build, None),
                dep("proptest", DependencyKind::Dev, None),
                dep("outside", DependencyKind::Normal, None),
            ],
        }]
    }

    #[test]
    fn parses_members_kinds_renames_and_target_specific_dependencies() {
        assert_eq!(parse(&sample("/w", "/")).unwrap(), expected());
    }

    /// Verifies: XTASK-R031
    #[test]
    fn windows_paths_are_compared_with_forward_slashes() {
        assert_eq!(parse(&sample(r"C:\w", r"\")).unwrap(), expected());
    }

    #[test]
    fn a_root_package_has_an_empty_manifest_dir() {
        let json = sample("/w", "/").replace("/w/crates/app/Cargo.toml", "/w/Cargo.toml");
        assert_eq!(parse(&json).unwrap()[0].manifest_dir, "");
    }

    /// Verifies: XTASK-R032
    #[test]
    fn a_path_dependency_on_the_workspace_root_is_inside_with_an_empty_path() {
        for root in ["/w", "/w/"] {
            assert_eq!(relative(root, "/w"), Some(String::new()), "{root}");
            assert_eq!(
                relative(root, "/w/src/lib.rs"),
                Some("src/lib.rs".to_string())
            );
            assert_eq!(relative(root, "/w2"), None, "{root}");
        }
        let json = sample("/w", "/").replace("\"/w/crates/core\"", "\"/w\"");
        assert_eq!(
            parse(&json).unwrap()[0].dependencies[0].path,
            Some(String::new())
        );
        // A root package's root-level target stays `src/lib.rs`.
        let json = sample("/w", "/")
            .replace("/w/crates/app/Cargo.toml", "/w/Cargo.toml")
            .replace("/w/crates/app/src/lib.rs", "/w/src/lib.rs");
        assert_eq!(parse(&json).unwrap()[0].targets[0].src_path, "src/lib.rs");
        // Windows separators: `C:\w` itself is the root too.
        let json = sample(r"C:\w", r"\").replace(r#""C:\\w\\crates\\core""#, r#""C:\\w""#);
        assert_eq!(
            parse(&json).unwrap()[0].dependencies[0].path,
            Some(String::new())
        );
    }

    #[test]
    fn a_root_prefix_that_is_not_a_directory_boundary_is_outside() {
        // `/w2/...` must not count as inside `/w`.
        let json = sample("/w", "/").replace("\"/w/crates/core\"", "\"/w2/crates/core\"");
        assert_eq!(parse(&json).unwrap()[0].dependencies[0].path, None);
    }

    #[test]
    fn unexpected_output_is_lim_xtask_0002() {
        let good = sample("/w", "/");
        for json in [
            "not json".to_string(),
            "{}".to_string(),
            good.replace("\"kind\": \"dev\"", "\"kind\": \"weird\""),
            good.replace("/w/crates/app/src/lib.rs", "/x/lib.rs"),
            good.replace("/w/crates/app/Cargo.toml", "/elsewhere/Cargo.toml"),
            good.replace("/w/crates/app/Cargo.toml", "/w/crates/app/package.json"),
        ] {
            let error = parse(&json).unwrap_err();
            assert_eq!(error.code, "LIM-XTASK-0002", "{json}");
        }
    }

    #[test]
    fn real_cargo_metadata_of_this_repository() {
        let json = run_cargo(&crate::repo::root(), &cargo_program()).unwrap();
        let packages = parse(&json).unwrap();
        let xtask = packages.iter().find(|p| p.name == "xtask").unwrap();
        assert_eq!(xtask.manifest_dir, "xtask");
        let roots: Vec<&str> = xtask.targets.iter().map(|t| t.src_path.as_str()).collect();
        assert!(roots.contains(&"xtask/src/lib.rs") && roots.contains(&"xtask/src/main.rs"));
        assert!(
            xtask
                .dependencies
                .iter()
                .any(|d| d.name == "proptest" && d.kind == DependencyKind::Dev)
        );
    }

    #[test]
    fn cargo_that_cannot_start_or_fails_is_lim_xtask_0005() {
        let missing = run_cargo(
            &crate::repo::root(),
            OsStr::new("/nonexistent/cargo-for-xtask-test"),
        )
        .unwrap_err();
        assert_eq!(missing.code, "LIM-XTASK-0005");
        assert!(
            missing.detail.contains("could not start"),
            "{}",
            missing.detail
        );
        let root = crate::test_support::TempRoot::new("metadata-no-manifest");
        let failed = run_cargo(root.path(), &cargo_program()).unwrap_err();
        assert_eq!(failed.code, "LIM-XTASK-0005");
        assert!(failed.detail.contains("Cargo.toml"), "{}", failed.detail);
    }
}
