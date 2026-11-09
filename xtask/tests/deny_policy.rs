//! Tests of the licence policy in `deny.toml` (XTASK-R082, `docs/specs/xtask.md` "`doctor` and
//! `deny` details").
//!
//! They run `cargo deny check licenses` with the repository `deny.toml` on a copy of the fixture
//! `xtask/tests/fixtures/deny/`, a crate that depends on a path crate whose licence each case
//! sets. They need `cargo-deny`, so they are ignored in the `test` step of `ci` and run by the
//! `deny` job of the CI workflow (`cargo test --test deny_policy -- --ignored`). No network access
//! is needed (`--offline`, path crates only).

use std::path::{Path, PathBuf};
use std::process::Command;

/// The repository root (the parent of `xtask/`).
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask is one level below the repository root")
        .to_path_buf()
}

/// A fresh temporary directory, removed on drop.
struct TempRoot(PathBuf);

impl TempRoot {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("xtask-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("test setup: create a temporary directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Copies the contents of `from` into `to` recursively.
fn copy_dir(from: &Path, to: &Path) {
    for entry in std::fs::read_dir(from).expect("test setup: read the fixture directory") {
        let entry = entry.expect("test setup: read a fixture entry");
        let target = to.join(entry.file_name());
        if entry
            .file_type()
            .expect("test setup: fixture entry type")
            .is_dir()
        {
            std::fs::create_dir_all(&target).expect("test setup: create a directory");
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("test setup: copy a fixture file");
        }
    }
}

/// Licences that are not allowed in any scope, so `deny.toml` must reject them.
const COPYLEFT: &[&str] = &[
    "GPL-2.0-only",
    "GPL-3.0-only",
    "GPL-3.0-or-later",
    "LGPL-2.1-or-later",
    "LGPL-3.0-only",
    "AGPL-3.0-only",
];

/// The licence list of `docs/process/DEPENDENCIES.md` "Licence policy": every licence after
/// "Allowed dependency licences:", with whether it carries a scope restriction.
fn documented_licences() -> Vec<(String, bool)> {
    let text = std::fs::read_to_string(repo_root().join("docs/process/DEPENDENCIES.md"))
        .expect("test setup: read docs/process/DEPENDENCIES.md");
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let start = text
        .find("Allowed dependency licences:")
        .expect("DEPENDENCIES.md has the sentence `Allowed dependency licences: …`")
        + "Allowed dependency licences:".len();
    let list = &text[start
        ..start
            + text[start..]
                .find(". ")
                .expect("the licence sentence in DEPENDENCIES.md ends with `. `")];
    let licences: Vec<(String, bool)> = list
        .split(", ")
        .map(|entry| {
            let entry = entry.trim();
            match entry.split_once(" (") {
                Some((licence, _scope)) => (licence.to_string(), true),
                None => (entry.to_string(), false),
            }
        })
        .collect();
    assert!(
        licences.len() >= 8 && licences.iter().any(|(l, _)| l == "MIT"),
        "the licence sentence in DEPENDENCIES.md changed shape: {licences:?}"
    );
    licences
}

/// Runs the licence check on a fixture copy whose dependency has `licence`. Returns whether it
/// passed, and the output.
fn check_licence(root: &Path, licence: &str) -> (bool, String) {
    let manifest = root.join("copyleft/Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("test setup: read the fixture manifest");
    let line = text
        .lines()
        .find(|line| line.starts_with("license = "))
        .expect("the fixture manifest has a `license = ` line")
        .to_string();
    std::fs::write(
        &manifest,
        text.replace(&line, &format!("license = \"{licence}\"")),
    )
    .expect("test setup: write the fixture manifest");

    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(cargo)
        .args(["deny", "--offline", "--color", "never", "--config"])
        .arg(repo_root().join("deny.toml"))
        .arg("--manifest-path")
        .arg(root.join("app/Cargo.toml"))
        .args(["check", "licenses"])
        .output()
        .expect("cargo starts (these tests run under cargo)");
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    assert!(
        !text.contains("no such command"),
        "cargo-deny is not installed; `cargo xtask doctor` shows the install hint"
    );
    (output.status.success(), text)
}

fn fixture(name: &str) -> TempRoot {
    let root = TempRoot::new(&format!("deny-{name}"));
    copy_dir(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/deny"),
        root.path(),
    );
    root
}

/// Verifies: XTASK-R082
#[test]
#[ignore = "M0-T009: needs cargo-deny; the `deny` job of the CI workflow runs it"]
fn copyleft_dependencies_are_rejected() {
    let root = fixture("copyleft");
    for licence in COPYLEFT {
        let (passed, output) = check_licence(root.path(), licence);
        assert!(!passed, "{licence} was accepted:\n{output}");
        assert!(output.contains("rejected"), "{licence}:\n{output}");
        assert!(output.contains("fixture_copyleft"), "{licence}:\n{output}");
    }
}

/// Verifies: XTASK-R082
#[test]
#[ignore = "M0-T009: needs cargo-deny; the `deny` job of the CI workflow runs it"]
fn the_documented_allow_list_is_exactly_what_deny_toml_accepts() {
    let root = fixture("documented");
    for (licence, scoped) in documented_licences() {
        let (passed, output) = check_licence(root.path(), &licence);
        if scoped {
            // MPL-2.0 (tool and dev scopes only) needs a per-crate exception.
            assert!(
                !passed,
                "scoped {licence} accepted without an exception:\n{output}"
            );
        } else {
            assert!(
                passed,
                "documented licence {licence} was rejected:\n{output}"
            );
        }
    }
}

#[test]
fn the_documented_licence_list_is_read() {
    let licences = documented_licences();
    assert!(
        licences.contains(&("MPL-2.0".to_string(), true)),
        "{licences:?}"
    );
    assert!(
        licences.contains(&("CC0-1.0".to_string(), false)),
        "{licences:?}"
    );
}
