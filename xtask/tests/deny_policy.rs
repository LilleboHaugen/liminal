//! Tests of the policy in `deny.toml` (XTASK-R082 licences, R083 sources, R084 advisories;
//! `docs/specs/xtask.md` "`doctor` and `deny` details").
//!
//! R082: `cargo deny check licenses` with the repository `deny.toml` on a copy of the fixture
//! `xtask/tests/fixtures/deny/`, a crate that depends on a path crate whose licence each case
//! sets. R083: `check sources` on a crate with a git dependency on a local repository the test
//! creates. R084: `check advisories` with a copy of `deny.toml` that adds only `db-path` and
//! `db-urls`, against a fixture advisory database the test creates.
//!
//! They need `cargo-deny` (R083 and R084 also `git`), so they are ignored in the `test` step of
//! `ci` and run by the `deny` job of the CI workflow (`cargo test --test deny_policy --
//! --ignored`). No network access is needed: path crates, a `file://` git repository, and for
//! R084 crates of the repository `Cargo.lock` from the local registry cache (`--offline`).

use std::fmt::Write as _;
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

/// Runs `cargo deny [--offline] --config <config> --manifest-path <manifest> check <which>`.
/// Returns whether it passed, and stdout followed by stderr.
/// `env` is set for `cargo-deny` and the `cargo` it runs.
fn run_deny(
    config: &Path,
    manifest: &Path,
    offline: bool,
    which: &str,
    env: &[(&str, String)],
) -> (bool, String) {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut command = Command::new(cargo);
    command.envs(env.iter().map(|(key, value)| (key, value)));
    command.arg("deny");
    if offline {
        command.arg("--offline");
    }
    let output = command
        .args(["--color", "never", "--config"])
        .arg(config)
        .arg("--manifest-path")
        .arg(manifest)
        .args(["check", which])
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

/// Runs `git` with `args` in `dir`, with an identity and no signing, so it works on any machine.
fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args([
            "-c",
            "user.name=xtask test",
            "-c",
            "user.email=xtask@invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .args(args)
        .current_dir(dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("test setup: git starts (the R083 and R084 tests need git)");
    assert!(
        status.success(),
        "test setup: git {args:?} in {} failed",
        dir.display()
    );
}

/// Makes `dir` a git repository with one commit of everything in it.
fn commit_all(dir: &Path) {
    git(dir, &["init", "--quiet"]);
    git(dir, &["add", "--all"]);
    git(dir, &["commit", "--quiet", "--message", "fixture"]);
}

/// Writes `text` to `path`, creating parent directories.
fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().expect("test setup: a file path has a parent"))
        .expect("test setup: create a directory");
    std::fs::write(path, text).expect("test setup: write a fixture file");
}

/// A `file://` URL for a local directory (`/a/b` → `file:///a/b`, `C:\a` → `file:///C:/a`).
fn file_url(path: &Path) -> String {
    let path = path.to_string_lossy().replace('\\', "/");
    if path.starts_with('/') {
        format!("file://{path}")
    } else {
        format!("file:///{path}")
    }
}

/// Verifies: XTASK-R083
#[test]
#[ignore = "M0-T009: needs cargo-deny and git; the `deny` job of the CI workflow runs it"]
fn git_dependencies_are_rejected() {
    let root = TempRoot::new("deny-git");
    let dep = root.path().join("dep");
    write(
        &dep.join("Cargo.toml"),
        "[package]\nname = \"fixture_git_dep\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\
         license = \"MIT\"\npublish = false\n\n[workspace]\n",
    );
    write(&dep.join("src/lib.rs"), "");
    commit_all(&dep);

    let app = root.path().join("app");
    write(
        &app.join("Cargo.toml"),
        &format!(
            "[package]\nname = \"fixture_app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\
             license = \"MIT OR Apache-2.0\"\npublish = false\n\n[dependencies]\n\
             fixture_git_dep = {{ git = \"{}\" }}\n\n[workspace]\n",
            file_url(&dep)
        ),
    );
    write(&app.join("src/lib.rs"), "");

    // Not `--offline`: cargo fetches the local git repository (no network involved).
    let (passed, output) = run_deny(
        &repo_root().join("deny.toml"),
        &app.join("Cargo.toml"),
        false,
        "sources",
        &[],
    );
    assert!(!passed, "a git dependency was accepted:\n{output}");
    assert!(output.contains("sources FAILED"), "{output}");
    assert!(output.contains("fixture_git_dep"), "{output}");
}

/// The `checksum` of `name` in the repository `Cargo.lock` (the SHA-256 of its `.crate` file).
fn locked_checksum(name: &str) -> String {
    let lock = std::fs::read_to_string(repo_root().join("Cargo.lock"))
        .expect("test setup: read Cargo.lock");
    let header = format!("name = \"{name}\"\n");
    let at = lock
        .find(&header)
        .unwrap_or_else(|| panic!("test setup: {name} is in Cargo.lock"));
    lock[at..]
        .lines()
        .take_while(|line| !line.is_empty())
        .find_map(|line| line.strip_prefix("checksum = \""))
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or_else(|| panic!("test setup: {name} has a checksum line in Cargo.lock"))
        .to_string()
}

/// The `.crate` file of `name` at its locked version in the local crates.io cache
/// (`$CARGO_HOME/registry/cache/index.crates.io-*/`), which building the repository filled.
fn cached_crate(name: &str) -> PathBuf {
    let cargo_home = std::env::var_os("CARGO_HOME").map_or_else(
        || {
            std::env::home_dir()
                .expect("test setup: a home directory for the default CARGO_HOME")
                .join(".cargo")
        },
        PathBuf::from,
    );
    let file = format!("{name}-{}.crate", locked_version(name));
    let cache = cargo_home.join("registry/cache");
    std::fs::read_dir(&cache)
        .unwrap_or_else(|_| panic!("test setup: read {}", cache.display()))
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("index.crates.io-")
        })
        .map(|entry| entry.path().join(&file))
        .find(|path| path.is_file())
        .unwrap_or_else(|| panic!("test setup: {file} is in {}", cache.display()))
}

/// Verifies: XTASK-R083
#[test]
#[ignore = "M0-T009: needs cargo-deny and git; the `deny` job of the CI workflow runs it"]
fn other_registry_dependencies_are_rejected() {
    // A registry other than crates.io: a git index at a `file://` URL whose `dl` is a `file://`
    // directory holding the crates.io `.crate` file of `itoa` (its checksum is in Cargo.lock), so
    // cargo resolves and downloads it without network access.
    let root = TempRoot::new("deny-registry");
    let dl = root.path().join("dl");
    let file = format!("itoa-{}.crate", locked_version("itoa"));
    std::fs::create_dir_all(&dl).expect("test setup: create the download directory");
    std::fs::copy(cached_crate("itoa"), dl.join(&file)).expect("test setup: copy the .crate");
    let index = root.path().join("index");
    write(
        &index.join("config.json"),
        &format!(
            "{{\"dl\": \"{}/{{crate}}-{{version}}.crate\"}}\n",
            file_url(&dl)
        ),
    );
    write(
        &index.join("it/oa/itoa"),
        &format!(
            "{{\"name\":\"itoa\",\"vers\":\"{}\",\"deps\":[],\"cksum\":\"{}\",\
             \"features\":{{}},\"yanked\":false}}\n",
            locked_version("itoa"),
            locked_checksum("itoa")
        ),
    );
    commit_all(&index);

    let app = root.path().join("app");
    write(
        &app.join("Cargo.toml"),
        &format!(
            "[package]\nname = \"fixture_app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\
             license = \"MIT OR Apache-2.0\"\npublish = false\n\n[dependencies]\n\
             itoa = {{ version = \"={}\", registry = \"fixture\" }}\n\n[workspace]\n",
            locked_version("itoa")
        ),
    );
    write(&app.join("src/lib.rs"), "");

    // Not `--offline`: cargo fetches the local index (no network involved). The registry is
    // named through the environment, as cargo reads config files from the working directory,
    // not the manifest's. A separate CARGO_HOME keeps the fixture registry out of the user's
    // cache; nothing from crates.io is needed.
    let env = [
        ("CARGO_REGISTRIES_FIXTURE_INDEX", file_url(&index)),
        (
            "CARGO_HOME",
            root.path()
                .join("cargo-home")
                .to_string_lossy()
                .into_owned(),
        ),
    ];
    let (passed, output) = run_deny(
        &repo_root().join("deny.toml"),
        &app.join("Cargo.toml"),
        false,
        "sources",
        &env,
    );
    assert!(!passed, "another registry was accepted:\n{output}");
    assert!(output.contains("sources FAILED"), "{output}");
    assert!(output.contains("error[source-not-allowed]"), "{output}");
    assert!(output.contains("itoa"), "{output}");
}

/// One fixture advisory: `(RUSTSEC id, crate, informational kind or None for a vulnerability)`.
type Advisory = (&'static str, &'static str, Option<&'static str>);

/// The `version` of `name` in the repository `Cargo.lock`, so the fixture resolves offline from
/// the local registry cache that building the repository filled.
fn locked_version(name: &str) -> String {
    let lock = std::fs::read_to_string(repo_root().join("Cargo.lock"))
        .expect("test setup: read Cargo.lock");
    let header = format!("name = \"{name}\"\n");
    let at = lock
        .find(&header)
        .unwrap_or_else(|| panic!("test setup: {name} is in Cargo.lock"))
        + header.len();
    lock[at..]
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("version = \""))
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or_else(|| panic!("test setup: {name} has a version line in Cargo.lock"))
        .to_string()
}

/// Crates.io crates of the repository `Cargo.lock` that the advisory fixture depends on.
const ADVISED: &[&str] = &["itoa", "memchr"];

/// The advisory fixture's dependencies on [`ADVISED`] crates: which the app depends on directly,
/// and which it depends on only through the path crate `fixture_middle`.
struct Graph {
    direct: &'static [&'static str],
    transitive: &'static [&'static str],
}

/// Every [`ADVISED`] crate is a transitive dependency only (app → middle → advised).
const TRANSITIVE: Graph = Graph {
    direct: &[],
    transitive: ADVISED,
};

/// `itoa` is a direct dependency of the app; `memchr` stays transitive.
const DIRECT_ITOA: Graph = Graph {
    direct: &["itoa"],
    transitive: &["memchr"],
};

/// `name = "=<locked version>"` lines for `crates`, for a `[dependencies]` table.
fn locked_dependencies(crates: &[&str]) -> String {
    let mut dependencies = String::new();
    for name in crates {
        writeln!(dependencies, "{name} = \"={}\"", locked_version(name))
            .expect("invariant: writing to a String cannot fail");
    }
    dependencies
}

/// The advisory URL in the fixture config. Never fetched (`--offline`); it only names the
/// database directory under `db-path` (cargo-deny 0.20.2 rejects `file://` URLs here).
const FIXTURE_DB_URL: &str = "https://liminal.invalid/advisory-db";

/// Runs `cargo deny --offline check advisories` with the repository `deny.toml`, extended only by
/// `db-path` and `db-urls`, on a crate depending on [`ADVISED`] as `graph` says, with a fixture
/// advisory database holding `advisories`. Returns whether it passed, and the output.
fn check_advisories(name: &str, graph: &Graph, advisories: &[Advisory]) -> (bool, String) {
    let root = TempRoot::new(&format!("deny-advisories-{name}"));
    let db_path = root.path().join("db");
    std::fs::create_dir_all(&db_path).expect("test setup: create db-path");

    let policy =
        std::fs::read_to_string(repo_root().join("deny.toml")).expect("test setup: read deny.toml");
    assert!(
        policy.contains("\n[advisories]\n"),
        "deny.toml has an [advisories] table"
    );
    let config = root.path().join("deny.toml");
    write(
        &config,
        &policy.replacen(
            "\n[advisories]\n",
            &format!(
                "\n[advisories]\ndb-path = '{}'\ndb-urls = ['{FIXTURE_DB_URL}']\n",
                db_path.display()
            ),
            1,
        ),
    );

    // Transitive advised crates (app → middle → advised) and direct ones (app → advised)
    // together distinguish `unmaintained = "all"` from `"workspace"` (direct dependencies of
    // workspace crates only) and `"transitive"`. `middle` is a path crate outside `app/`, so it
    // is a dependency, not a workspace member.
    let middle = root.path().join("middle");
    let dependencies = locked_dependencies(graph.transitive);
    write(
        &middle.join("Cargo.toml"),
        &format!(
            "[package]\nname = \"fixture_middle\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\
             license = \"MIT\"\npublish = false\n\n[dependencies]\n{dependencies}"
        ),
    );
    write(&middle.join("src/lib.rs"), "");
    let app = root.path().join("app");
    write(
        &app.join("Cargo.toml"),
        &format!(
            "[package]\nname = \"fixture_app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\
             license = \"MIT OR Apache-2.0\"\npublish = false\n\n[dependencies]\n\
             fixture_middle = {{ path = \"../middle\" }}\n{}\n[workspace]\n",
            locked_dependencies(graph.direct)
        ),
    );
    write(&app.join("src/lib.rs"), "");
    let manifest = app.join("Cargo.toml");

    // With an empty db-path, cargo-deny names the directory it expects
    // (`<db-path>/advisory-db-<hash of the URL>`); take the name from its output instead of
    // depending on its hash function.
    let (passed, output) = run_deny(&config, &manifest, true, "advisories", &[]);
    assert!(!passed, "advisories passed without a database:\n{output}");
    let start = output
        .find("advisory-db-")
        .unwrap_or_else(|| panic!("cargo-deny names the expected database directory:\n{output}"));
    let dir: String = output[start..]
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();

    let db = db_path.join(&dir);
    write(
        &db.join("README.md"),
        "Fixture advisory database (XTASK-R084).\n",
    );
    // Every database also has an advisory for a crate outside the graph, so the control case has
    // a valid, non-empty database and only an advisory for a dependency can fail the check.
    let unrelated: Advisory = ("RUSTSEC-2026-9000", "liminal_fixture_unused", None);
    for (id, krate, informational) in advisories.iter().chain([&unrelated]) {
        let informational = informational.map_or(String::new(), |kind| {
            format!("informational = \"{kind}\"\n")
        });
        write(
            &db.join(format!("crates/{krate}/{id}.md")),
            &format!(
                "```toml\n[advisory]\nid = \"{id}\"\npackage = \"{krate}\"\ndate = \"2026-01-01\"\n\
                 {informational}\n[versions]\npatched = []\n```\n\n# Fixture advisory for {krate}\n\n\
                 Test fixture (XTASK-R084), not a real advisory.\n"
            ),
        );
    }
    commit_all(&db);

    run_deny(&config, &manifest, true, "advisories", &[])
}

/// Verifies: XTASK-R084
#[test]
#[ignore = "M0-T009: needs cargo-deny and git; the `deny` job of the CI workflow runs it"]
fn unmaintained_and_vulnerable_crates_are_rejected() {
    let (passed, output) = check_advisories("none", &TRANSITIVE, &[]);
    assert!(passed, "control without advisories failed:\n{output}");
    assert!(output.contains("advisories ok"), "{output}");

    // Transitive only: kills `unmaintained = "workspace"`.
    let (passed, output) = check_advisories(
        "unmaintained",
        &TRANSITIVE,
        &[("RUSTSEC-2026-9001", "itoa", Some("unmaintained"))],
    );
    assert!(
        !passed,
        "a transitive unmaintained crate was accepted:\n{output}"
    );
    assert!(output.contains("RUSTSEC-2026-9001"), "{output}");
    assert!(output.contains("error[unmaintained]"), "{output}");

    // Direct dependency of the workspace crate: kills `unmaintained = "transitive"`.
    let (passed, output) = check_advisories("direct-control", &DIRECT_ITOA, &[]);
    assert!(passed, "control with a direct dependency failed:\n{output}");
    let (passed, output) = check_advisories(
        "unmaintained-direct",
        &DIRECT_ITOA,
        &[("RUSTSEC-2026-9003", "itoa", Some("unmaintained"))],
    );
    assert!(
        !passed,
        "a direct unmaintained crate was accepted:\n{output}"
    );
    assert!(output.contains("RUSTSEC-2026-9003"), "{output}");
    assert!(output.contains("error[unmaintained]"), "{output}");

    let (passed, output) = check_advisories(
        "vulnerability",
        &TRANSITIVE,
        &[("RUSTSEC-2026-9002", "memchr", None)],
    );
    assert!(!passed, "a vulnerable crate was accepted:\n{output}");
    assert!(output.contains("RUSTSEC-2026-9002"), "{output}");
    assert!(output.contains("error[vulnerability]"), "{output}");
}
