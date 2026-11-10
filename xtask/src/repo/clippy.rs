//! Clippy configuration files, as `check-arch` needs them for XTASK-R039 (`docs/specs/xtask.md`
//! "`check-arch` details").
//!
//! Clippy reads exactly one configuration file per crate: the first `.clippy.toml` or
//! `clippy.toml` it finds in the crate's manifest directory or an ancestor (`.clippy.toml` first
//! within a directory). Files are not merged (verified with clippy 0.1.99, M1-T011 journal). The
//! lookup here stops at the repository root. `CLIPPY_CONF_DIR` is not modelled: the repository
//! never sets it.

use std::collections::BTreeSet;
use std::path::Path;

use super::LoadError;

/// File names clippy looks for in each directory, in its order of preference.
const FILE_NAMES: &[&str] = &[".clippy.toml", "clippy.toml"];

/// The clippy configuration that applies to one crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClippyConfig {
    /// Repository-relative `/`-separated path of the file.
    pub path: String,
    /// The `path` of every `disallowed-methods` entry.
    pub disallowed_methods: BTreeSet<String>,
}

/// The configuration file clippy applies to the crate in `manifest_dir` (repository-relative,
/// `/`-separated, empty for the root), or `None` if there is none up to the repository root.
///
/// # Errors
///
/// LIM-XTASK-0001 if the file cannot be read; LIM-XTASK-0004 if it is invalid ([`parse`]).
pub(crate) fn effective(
    root: &Path,
    manifest_dir: &str,
) -> Result<Option<ClippyConfig>, LoadError> {
    let mut dir = manifest_dir.trim_end_matches('/');
    loop {
        for name in FILE_NAMES {
            let path = if dir.is_empty() {
                (*name).to_string()
            } else {
                format!("{dir}/{name}")
            };
            if root.join(&path).is_file() {
                let disallowed_methods = parse(&path, &super::read(root, &path)?)?;
                return Ok(Some(ClippyConfig {
                    path,
                    disallowed_methods,
                }));
            }
        }
        if dir.is_empty() {
            return Ok(None);
        }
        dir = dir.rfind('/').map_or("", |slash| &dir[..slash]);
    }
}

/// The `path` of every `disallowed-methods` entry of the clippy configuration `text` (from the
/// file `path`). An entry is a string or a table with a string `path`; a missing key is an empty
/// list. Other keys are not read.
///
/// # Errors
///
/// LIM-XTASK-0004 if `text` is not valid TOML, `disallowed-methods` is not an array, or an entry
/// has another shape.
pub(crate) fn parse(path: &str, text: &str) -> Result<BTreeSet<String>, LoadError> {
    let invalid = |detail: String| LoadError {
        code: "LIM-XTASK-0004",
        path: path.to_string(),
        detail,
        help: "Fix the file at the reported location",
    };
    let table: toml::Table = toml::from_str(text)
        .map_err(|error| invalid(format!("is not valid: {}", error.to_string().trim_end())))?;
    let Some(value) = table.get("disallowed-methods") else {
        return Ok(BTreeSet::new());
    };
    let entries = value
        .as_array()
        .ok_or_else(|| invalid("`disallowed-methods` is not an array".to_string()))?;
    entries
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .or_else(|| entry.get("path").and_then(toml::Value::as_str))
                .map(str::to_string)
                .ok_or_else(|| {
                    invalid(format!(
                        "`disallowed-methods` entry `{entry}` is neither a string nor a table \
                         with a string `path`"
                    ))
                })
        })
        .collect()
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

    const ONE_BAN: &str = "disallowed-methods = [\"f32::sin\"]\n";

    #[test]
    fn parse_reads_strings_and_tables() {
        let text = "allow-unwrap-in-tests = true\ndisallowed-methods = [\"f32::sin\", \
                    { path = \"f64::cos\", reason = \"ADR-0009\" }]\n";
        assert_eq!(
            parse("clippy.toml", text).unwrap(),
            BTreeSet::from(["f32::sin".to_string(), "f64::cos".to_string()])
        );
        assert_eq!(
            parse("clippy.toml", "allow-unwrap-in-tests = true\n").unwrap(),
            BTreeSet::new()
        );
    }

    #[test]
    fn parse_rejects_invalid_files_with_lim_xtask_0004() {
        for text in [
            "disallowed-methods = [",
            "disallowed-methods = \"f32::sin\"",
            "disallowed-methods = [1]",
            "disallowed-methods = [{ reason = \"no path\" }]",
        ] {
            let error = parse("a/clippy.toml", text).unwrap_err();
            assert_eq!(
                (error.code, error.path.as_str()),
                ("LIM-XTASK-0004", "a/clippy.toml"),
                "{text}"
            );
        }
    }

    #[test]
    fn effective_takes_the_nearest_file_without_merging() {
        let root = TempRoot::new("clippy-effective-nearest");
        write(
            &root,
            "clippy.toml",
            "disallowed-methods = [\"f32::cos\"]\n",
        );
        write(&root, "crates/det/clippy.toml", ONE_BAN);
        std::fs::create_dir_all(root.path().join("crates/plain")).unwrap();
        let det = effective(root.path(), "crates/det").unwrap().unwrap();
        assert_eq!(det.path, "crates/det/clippy.toml");
        assert_eq!(
            det.disallowed_methods,
            BTreeSet::from(["f32::sin".to_string()])
        );
        let plain = effective(root.path(), "crates/plain").unwrap().unwrap();
        assert_eq!(plain.path, "clippy.toml");
        assert_eq!(
            effective(root.path(), "").unwrap().unwrap().path,
            "clippy.toml"
        );
    }

    #[test]
    fn effective_prefers_the_dot_file_and_finds_intermediate_directories() {
        let root = TempRoot::new("clippy-effective-dot");
        write(&root, "crates/clippy.toml", ONE_BAN);
        write(&root, "crates/a/.clippy.toml", "");
        write(&root, "crates/a/clippy.toml", ONE_BAN);
        std::fs::create_dir_all(root.path().join("crates/b/inner")).unwrap();
        let a = effective(root.path(), "crates/a").unwrap().unwrap();
        assert_eq!(
            (a.path.as_str(), a.disallowed_methods.len()),
            ("crates/a/.clippy.toml", 0)
        );
        let b = effective(root.path(), "crates/b/inner").unwrap().unwrap();
        assert_eq!(b.path, "crates/clippy.toml");
    }

    #[test]
    fn effective_is_none_without_any_file() {
        let root = TempRoot::new("clippy-effective-none");
        std::fs::create_dir_all(root.path().join("crates/a")).unwrap();
        assert_eq!(effective(root.path(), "crates/a").unwrap(), None);
    }
}
