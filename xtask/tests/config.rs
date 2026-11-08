//! Repository configuration facts that xtask relies on (XTASK-R001, XTASK-R011).
//!
//! These read the files as text: xtask's TOML dependency is not used before M0-T002, and the
//! checks only need line-level facts. `cargo xtask check-arch` (M0-T004) enforces R011 fully from
//! `cargo metadata`.

use std::fs;
use std::path::PathBuf;

fn repo_file(relative: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("test setup: xtask/ has a parent directory")
        .to_path_buf();
    let path = root.join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()))
}

/// Returns `(section header, line)` for every non-comment, non-blank line of a TOML file.
fn lines_with_sections(text: &str) -> Vec<(String, String)> {
    let mut section = String::new();
    let mut result = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            section = line.to_string();
        }
        result.push((section.clone(), line.to_string()));
    }
    result
}

/// Verifies: XTASK-R001
#[test]
fn cargo_config_defines_the_xtask_alias() {
    let config = repo_file(".cargo/config.toml");
    let found = lines_with_sections(&config).iter().any(|(section, line)| {
        section == "[alias]" && line == r#"xtask = "run --package xtask --""#
    });
    assert!(found, ".cargo/config.toml:\n{config}");
}

/// Lines inside any dependency table (`[dependencies]`, `[dev-dependencies]`,
/// `[target.'cfg(..)'.build-dependencies]`, `[dependencies.<name>]`, ...) that mention `liminal`.
fn liminal_dependency_lines(manifest: &str) -> Vec<String> {
    lines_with_sections(manifest)
        .into_iter()
        .filter(|(section, line)| section.contains("dependencies") && line.contains("liminal"))
        .map(|(_, line)| line)
        .collect()
}

/// Verifies: XTASK-R011
#[test]
fn xtask_manifest_has_no_liminal_dependency() {
    let manifest = repo_file("xtask/Cargo.toml");
    assert_eq!(liminal_dependency_lines(&manifest), Vec::<String>::new());
}

/// Verifies: XTASK-R011
#[test]
fn liminal_dependency_scan_finds_every_dependency_form() {
    let cases = [
        "[dependencies]\nliminal_core = { path = \"../crates/liminal_core\" }\n",
        "[dev-dependencies]\nliminal_math.workspace = true\n",
        "[build-dependencies]\ncore = { package = \"liminal_core\", path = \"p\" }\n",
        "[target.'cfg(unix)'.dependencies]\nliminal_core = \"0.1\"\n",
        "[dependencies.liminal_core]\npath = \"../crates/liminal_core\"\n",
    ];
    for manifest in cases {
        assert!(
            !liminal_dependency_lines(manifest).is_empty(),
            "not detected: {manifest}"
        );
    }
    let clean = "[package]\nname = \"xtask\"\ndescription = \"for liminal\"\n[dependencies]\nserde = \"1\"\n";
    assert_eq!(liminal_dependency_lines(clean), Vec::<String>::new());
}
