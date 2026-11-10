//! The `.rs` files of the workspace crates and the scan exclusions in `ci/hygiene.toml`
//! (`docs/specs/xtask.md` XTASK-R041, R050, R052 and the "`trace` and `hygiene` details").
//!
//! This module checks the TOML shape only. A missing or unknown field, or a value of the wrong TOML
//! type, is internal error LIM-XTASK-0004. Value rules (path form, path exists, non-empty reason)
//! are XTASK-R052 findings of `hygiene`.

use std::path::Path;

use serde::Deserialize;
use toml::Spanned;

use super::{LoadError, RepoModel};

/// Repository-relative path of the exclusion file.
pub(crate) const CONFIG_PATH: &str = "ci/hygiene.toml";

/// One excluded path and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Exclusion {
    /// A repository-relative, `/`-separated file or directory path.
    pub path: String,
    /// Why the path is excluded.
    pub reason: String,
    /// 1-based line of the entry in `ci/hygiene.toml`.
    pub line: usize,
}

impl Exclusion {
    /// Whether this exclusion covers the repository-relative file `file`: it is the file itself
    /// or a directory that contains it.
    pub(crate) fn covers(&self, file: &str) -> bool {
        let path = self.path.trim_end_matches('/');
        file == path
            || file
                .strip_prefix(path)
                .is_some_and(|rest| rest.starts_with('/'))
    }
}

/// `ci/hygiene.toml`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ScanConfig {
    /// `[[hygiene.exclude]]`: paths `hygiene` does not scan (XTASK-R052).
    pub hygiene: Vec<Exclusion>,
    /// `[[trace.exclude]]`: paths whose annotations `trace` does not read.
    pub trace: Vec<Exclusion>,
}

/// One `.rs` file of a workspace crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceFile {
    /// Repository-relative, `/`-separated path.
    pub path: String,
    /// The workspace crate whose manifest directory contains the file.
    pub crate_name: String,
    /// The file's contents.
    pub text: String,
}

/// What `trace` and `hygiene` read: the repo model, the scan exclusions and the source files.
#[derive(Debug, Clone)]
pub(crate) struct SourceModel {
    /// The shared repo model (specs, tasks, workspace members, paths).
    pub repo: RepoModel,
    /// `ci/hygiene.toml`.
    pub config: ScanConfig,
    /// Every `.rs` file below a workspace member's manifest directory, sorted by path, excluded
    /// or not. Each check applies its own exclusions.
    pub sources: Vec<SourceFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    #[serde(default)]
    hygiene: RawSection,
    #[serde(default)]
    trace: RawSection,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct RawSection {
    #[serde(default)]
    exclude: Vec<Spanned<RawExclusion>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawExclusion {
    path: String,
    reason: String,
}

/// Deserializes the contents of `ci/hygiene.toml`.
///
/// # Errors
///
/// LIM-XTASK-0004 if the text is not valid TOML or does not have the expected shape.
pub(crate) fn parse_config(text: &str) -> Result<ScanConfig, LoadError> {
    let raw: RawConfig = toml::from_str(text).map_err(|error| LoadError {
        code: "LIM-XTASK-0004",
        path: CONFIG_PATH.to_string(),
        detail: format!("is not valid: {}", error.to_string().trim_end()),
        help: "Fix the file at the reported location",
    })?;
    let line = |offset: usize| text[..offset].matches('\n').count() + 1;
    let section = |section: RawSection| -> Vec<Exclusion> {
        section
            .exclude
            .into_iter()
            .map(|entry| {
                let line = line(entry.span().start);
                let entry = entry.into_inner();
                Exclusion {
                    path: entry.path,
                    reason: entry.reason,
                    line,
                }
            })
            .collect()
    };
    Ok(ScanConfig {
        hygiene: section(raw.hygiene),
        trace: section(raw.trace),
    })
}

/// Loads the repo model of `root`, `ci/hygiene.toml`, and every `.rs` file of the workspace
/// members.
///
/// # Errors
///
/// [`LoadError`]: everything [`super::load`] reports, LIM-XTASK-0001 if `ci/hygiene.toml` or a
/// source file cannot be read, LIM-XTASK-0004 if `ci/hygiene.toml` is invalid.
pub(crate) fn load(root: &Path) -> Result<SourceModel, LoadError> {
    let repo = super::load(root)?;
    let config = parse_config(&super::read(root, CONFIG_PATH)?)?;
    let mut sources = Vec::new();
    // `repo.paths` is sorted, so `sources` is sorted by path.
    for path in &repo.paths {
        let is_rust = Path::new(path).extension().is_some_and(|ext| ext == "rs");
        if !is_rust {
            continue;
        }
        // A file inside two members' directories (nested crates) belongs to the inner one.
        if let Some(member) = super::workspace::owner(&repo.members, path)
            && root.join(path).is_file()
        {
            sources.push(SourceFile {
                path: path.clone(),
                crate_name: member.name.clone(),
                text: super::read(root, path)?,
            });
        }
    }
    Ok(SourceModel {
        repo,
        config,
        sources,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exclusions_cover_the_path_itself_and_everything_below_it() {
        let exclusion = Exclusion {
            path: "xtask/tests/fixtures".to_string(),
            reason: "r".to_string(),
            line: 1,
        };
        assert!(exclusion.covers("xtask/tests/fixtures"));
        assert!(exclusion.covers("xtask/tests/fixtures/a/b.rs"));
        assert!(!exclusion.covers("xtask/tests/fixtures_other/b.rs"));
        assert!(!exclusion.covers("xtask/tests/b.rs"));
        let slash = Exclusion {
            path: "xtask/src/".to_string(),
            ..exclusion
        };
        assert!(slash.covers("xtask/src/lib.rs"));
    }

    #[test]
    fn config_sections_with_lines_and_shape_errors() {
        let text = "# comment\n[[hygiene.exclude]]\npath = \"a\"\nreason = \"why\"\n\n\
                    [[trace.exclude]]\npath = \"b\"\nreason = \"because\"\n";
        let config = parse_config(text).unwrap();
        assert_eq!(
            config,
            ScanConfig {
                hygiene: vec![Exclusion {
                    path: "a".to_string(),
                    reason: "why".to_string(),
                    line: 2,
                }],
                trace: vec![Exclusion {
                    path: "b".to_string(),
                    reason: "because".to_string(),
                    line: 6,
                }],
            }
        );
        assert_eq!(parse_config("").unwrap(), ScanConfig::default());
        for bad in [
            "[[hygiene.exclude]]\npath = \"a\"\n",
            "[[hygiene.exclude]]\npath = \"a\"\nreason = \"r\"\nnote = \"x\"\n",
            "[[hygiene.exclude]]\npath = 1\nreason = \"r\"\n",
            "[[other.exclude]]\npath = \"a\"\nreason = \"r\"\n",
            "[[hygiene.skip]]\npath = \"a\"\nreason = \"r\"\n",
            "not toml",
        ] {
            let error = parse_config(bad).unwrap_err();
            assert_eq!(
                (error.code, error.path.as_str()),
                ("LIM-XTASK-0004", CONFIG_PATH),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn a_file_below_nested_members_belongs_to_the_inner_one() {
        // `outer/inner` is a member inside member `outer`, and is listed first in `members`, so
        // neither "first member wins" nor "last member wins" gives the right owner.
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sources/nested");
        let model = load(&root).unwrap();
        let owners: Vec<(&str, &str)> = model
            .sources
            .iter()
            .map(|s| (s.path.as_str(), s.crate_name.as_str()))
            .collect();
        assert_eq!(
            owners,
            [
                ("outer/inner/src/lib.rs", "inner"),
                ("outer/src/lib.rs", "outer"),
            ]
        );
    }

    /// Verifies: XTASK-R060
    #[test]
    fn a_member_spelled_dot_slash_owns_its_files() {
        let root = crate::test_support::TempRoot::new("sources-dot-member");
        let files = [
            ("Cargo.toml", "[workspace]\nmembers = [\"./xtask\"]\n"),
            (
                "architecture.toml",
                "schema_version = 1\n\n[layers]\n\"5\" = \"tools\"\n",
            ),
            ("ci/hygiene.toml", ""),
            ("xtask/Cargo.toml", "[package]\nname = \"xtask\"\n"),
            ("xtask/src/lib.rs", "//! Lib.\n"),
        ];
        for (path, text) in files {
            let full = root.path().join(path);
            std::fs::create_dir_all(full.parent().unwrap()).unwrap();
            std::fs::write(full, text).unwrap();
        }
        let model = load(root.path()).unwrap();
        let owners: Vec<(&str, &str)> = model
            .sources
            .iter()
            .map(|s| (s.path.as_str(), s.crate_name.as_str()))
            .collect();
        assert_eq!(owners, [("xtask/src/lib.rs", "xtask")]);
    }

    #[test]
    fn loads_the_rust_files_of_the_real_workspace() {
        let model = load(&super::super::root()).unwrap();
        let paths: Vec<&str> = model.sources.iter().map(|s| s.path.as_str()).collect();
        assert!(paths.contains(&"xtask/src/lib.rs"));
        assert!(paths.contains(&"xtask/tests/cli.rs"));
        assert!(paths.iter().all(|p| {
            p.starts_with("xtask/") && Path::new(p).extension().is_some_and(|ext| ext == "rs")
        }));
        assert!(model.sources.iter().all(|s| s.crate_name == "xtask"));
        assert_eq!(model.config.hygiene[0].path, "xtask/tests/fixtures");
    }
}
