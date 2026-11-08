//! Workspace members from `[workspace].members` in the root `Cargo.toml`, without running cargo
//! (`docs/specs/xtask.md` §3 and the "`check-docs` details" on member patterns).

use std::path::Path;

use serde::Deserialize;

use super::LoadError;

/// One workspace member crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Member {
    /// Manifest directory relative to the repository root, `/`-separated.
    pub path: String,
    /// `[package].name` from the member's `Cargo.toml`.
    pub name: String,
}

#[derive(Deserialize)]
struct RootManifest {
    workspace: Option<WorkspaceTable>,
}

#[derive(Deserialize)]
struct WorkspaceTable {
    #[serde(default)]
    members: Vec<String>,
}

#[derive(Deserialize)]
struct MemberManifest {
    package: Option<Package>,
}

#[derive(Deserialize)]
struct Package {
    name: String,
}

/// Help text of LIM-XTASK-0004.
const HELP: &str = "Fix the file at the reported location";

fn invalid(path: &str, detail: impl Into<String>) -> LoadError {
    LoadError {
        code: "LIM-XTASK-0004",
        path: path.to_string(),
        detail: detail.into(),
        help: HELP,
    }
}

/// Loads the members of the workspace at `root`, in manifest order (`<dir>/*` expanded sorted).
///
/// # Errors
///
/// LIM-XTASK-0001 if a manifest cannot be read; LIM-XTASK-0004 if a manifest is not valid TOML,
/// a member has no `[package].name`, or a member uses a pattern other than `<dir>/*`.
pub(crate) fn load(root: &Path) -> Result<Vec<Member>, LoadError> {
    let text = super::read(root, "Cargo.toml")?;
    let manifest: RootManifest = toml::from_str(&text).map_err(|error| {
        invalid(
            "Cargo.toml",
            format!("is not valid: {}", error.to_string().trim_end()),
        )
    })?;
    let patterns = manifest.workspace.map(|w| w.members).unwrap_or_default();
    let mut members = Vec::new();
    for pattern in patterns {
        for path in expand(root, &pattern)? {
            let manifest_path = format!("{path}/Cargo.toml");
            let text = super::read(root, &manifest_path)?;
            let manifest: MemberManifest = toml::from_str(&text).map_err(|error| {
                invalid(
                    &manifest_path,
                    format!("is not valid: {}", error.to_string().trim_end()),
                )
            })?;
            let Some(package) = manifest.package else {
                return Err(invalid(&manifest_path, "has no [package] table"));
            };
            members.push(Member {
                path,
                name: package.name,
            });
        }
    }
    Ok(members)
}

/// Expands one `members` entry into member directories.
fn expand(root: &Path, pattern: &str) -> Result<Vec<String>, LoadError> {
    let pattern = pattern.trim_end_matches('/');
    let is_glob = |text: &str| text.contains(['*', '?', '[', ']', '{', '}']);
    let Some(dir) = pattern.strip_suffix("/*").filter(|dir| !is_glob(dir)) else {
        if is_glob(pattern) {
            return Err(invalid(
                "Cargo.toml",
                format!(
                    "workspace member pattern `{pattern}` is not supported; use a path or `<dir>/*`"
                ),
            ));
        }
        return Ok(vec![pattern.to_string()]);
    };
    let entries =
        std::fs::read_dir(root.join(dir)).map_err(|error| LoadError::read(dir, &error))?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| LoadError::read(dir, &error))?;
        if entry.path().join("Cargo.toml").is_file() {
            paths.push(format!("{dir}/{}", entry.file_name().to_string_lossy()));
        }
    }
    paths.sort();
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempRoot;

    fn write(root: &TempRoot, path: &str, text: &str) {
        let full = root.path().join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, text).unwrap();
    }

    #[test]
    fn literal_and_star_members_in_manifest_order() {
        let root = TempRoot::new("members");
        write(
            &root,
            "Cargo.toml",
            "[workspace]\nmembers = [\"tools/one\", \"crates/*\"]\n",
        );
        write(
            &root,
            "tools/one/Cargo.toml",
            "[package]\nname = \"one\"\nversion = \"0.0.0\"\n",
        );
        write(&root, "crates/b/Cargo.toml", "[package]\nname = \"bee\"\n");
        write(&root, "crates/a/Cargo.toml", "[package]\nname = \"ay\"\n");
        write(&root, "crates/not_a_crate/README.md", "");
        let members = load(root.path()).unwrap();
        let pairs: Vec<(&str, &str)> = members
            .iter()
            .map(|m| (m.path.as_str(), m.name.as_str()))
            .collect();
        assert_eq!(
            pairs,
            [
                ("tools/one", "one"),
                ("crates/a", "ay"),
                ("crates/b", "bee")
            ]
        );
    }

    #[test]
    fn no_workspace_table_means_no_members() {
        let root = TempRoot::new("no-workspace");
        write(&root, "Cargo.toml", "[package]\nname = \"solo\"\n");
        assert_eq!(load(root.path()).unwrap(), []);
    }

    #[test]
    fn invalid_manifests_and_patterns_are_lim_xtask_0004() {
        // (case name, files to write, path of the expected error)
        type Case<'a> = (&'a str, &'a [(&'a str, &'a str)], &'a str);
        let cases: [Case; 4] = [
            ("bad-root", &[("Cargo.toml", "[workspace\n")], "Cargo.toml"),
            (
                "bad-glob",
                &[(
                    "Cargo.toml",
                    "[workspace]\nmembers = [\"crates/liminal_*\"]\n",
                )],
                "Cargo.toml",
            ),
            (
                "no-package",
                &[
                    ("Cargo.toml", "[workspace]\nmembers = [\"a\"]\n"),
                    ("a/Cargo.toml", "[lib]\n"),
                ],
                "a/Cargo.toml",
            ),
            (
                "bad-member",
                &[
                    ("Cargo.toml", "[workspace]\nmembers = [\"a\"]\n"),
                    ("a/Cargo.toml", "[package]\n"),
                ],
                "a/Cargo.toml",
            ),
        ];
        for (name, files, path) in cases {
            let root = TempRoot::new(name);
            for (file, text) in files {
                write(&root, file, text);
            }
            let error = load(root.path()).unwrap_err();
            assert_eq!(
                (error.code, error.path.as_str()),
                ("LIM-XTASK-0004", path),
                "{name}: {error}"
            );
        }
    }

    #[test]
    fn missing_member_manifest_is_lim_xtask_0001() {
        let root = TempRoot::new("missing-member");
        write(&root, "Cargo.toml", "[workspace]\nmembers = [\"gone\"]\n");
        let error = load(root.path()).unwrap_err();
        assert_eq!(
            (error.code, error.path.as_str()),
            ("LIM-XTASK-0001", "gone/Cargo.toml")
        );
    }
}
