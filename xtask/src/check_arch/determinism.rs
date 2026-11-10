//! XTASK-R039: every deterministic crate's clippy configuration bans the methods that
//! `docs/process/CODE_STYLE.md` §Lints lists for deterministic crates (ADR-0009 item 7).

use crate::finding::Finding;
use crate::repo::ArchModel;

/// The `disallowed-methods` paths every deterministic crate's clippy configuration must contain:
/// the list in `docs/process/CODE_STYLE.md` §Lints, in its order.
pub(crate) const BANNED_METHODS: &[&str] = &[
    "f32::sin",
    "f32::cos",
    "f32::tan",
    "f32::powf",
    "f32::exp",
    "f32::ln",
    "f64::sin",
    "f64::cos",
    "f64::tan",
    "f64::powf",
    "f64::exp",
    "f64::ln",
    "std::collections::HashMap::new",
    "std::time::Instant::now",
];

/// XTASK-R039, for every package in [`ArchModel::clippy`] (the workspace crates registered with
/// `deterministic = true`).
pub(super) fn clippy_bans(model: &ArchModel, findings: &mut Vec<Finding>) {
    for package in &model.packages {
        let Some(config) = model.clippy.get(&package.name) else {
            continue;
        };
        let name = &package.name;
        let dir = &package.manifest_dir;
        let own = if dir.is_empty() {
            "clippy.toml".to_string()
        } else {
            format!("{dir}/clippy.toml")
        };
        let help = format!(
            "Ban every missing path in `disallowed-methods` of `{own}` (create it, repeating the \
             settings of the root clippy.toml, because clippy reads only the nearest file); \
             CODE_STYLE.md §Lints, ADR-0009"
        );
        let Some(config) = config else {
            findings.push(Finding::error(
                "XTASK-R039",
                manifest(dir),
                format!(
                    "deterministic crate `{name}` has no clippy configuration (no `.clippy.toml` \
                     or `clippy.toml` in `{dir}` or a parent directory), so clippy bans none of \
                     the methods CODE_STYLE.md lists"
                ),
                help,
            ));
            continue;
        };
        let missing: Vec<String> = BANNED_METHODS
            .iter()
            .filter(|path| !config.disallowed_methods.contains(**path))
            .map(|path| format!("`{path}`"))
            .collect();
        if missing.is_empty() {
            continue;
        }
        findings.push(Finding::error(
            "XTASK-R039",
            config.path.clone(),
            format!(
                "deterministic crate `{name}`: its clippy configuration `{}` does not ban {} in \
                 `disallowed-methods`",
                config.path,
                missing.join(", ")
            ),
            help,
        ));
    }
}

/// `<manifest directory>/Cargo.toml`.
fn manifest(dir: &str) -> String {
    if dir.is_empty() {
        "Cargo.toml".to_string()
    } else {
        format!("{dir}/Cargo.toml")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::*;
    use crate::repo::architecture;
    use crate::repo::clippy::ClippyConfig;
    use crate::repo::metadata::Package;

    /// A registry with crates `det` (deterministic) and `free` (not), both layer 0.
    const REGISTRY: &str = r#"schema_version = 1
[layers]
"0" = "foundation"
"1" = "runtime"
"2" = "platform-gpu"
"3" = "subsystems"
"4" = "agent-interface-integration"
"5" = "tools"

[[crate]]
name = "det"
path = "crates/det"
layer = 0
kind = "lib"
unsafe = "forbid"
deterministic = true
same_layer_deps = []
external = []
dev_external = []

[[crate]]
name = "free"
path = "crates/free"
layer = 0
kind = "lib"
unsafe = "forbid"
deterministic = false
same_layer_deps = []
external = []
dev_external = []
"#;

    fn package(name: &str) -> Package {
        Package {
            name: name.to_string(),
            manifest_dir: format!("crates/{name}"),
            targets: Vec::new(),
            dependencies: Vec::new(),
            features: Vec::new(),
        }
    }

    /// A configuration at `path` banning `bans`.
    fn config(path: &str, bans: &[&str]) -> ClippyConfig {
        ClippyConfig {
            path: path.to_string(),
            disallowed_methods: bans.iter().map(|b| (*b).to_string()).collect(),
        }
    }

    /// Runs R039 with `det` as the configuration of crate `det`. `free` has none, as the loader leaves it.
    fn run(det: Option<ClippyConfig>) -> Vec<(String, String, String)> {
        let model = ArchModel {
            architecture: architecture::parse(REGISTRY).unwrap(),
            packages: vec![package("det"), package("free")],
            sources: BTreeMap::new(),
            clippy: BTreeMap::from([("det".to_string(), det)]),
        };
        let mut findings = Vec::new();
        clippy_bans(&model, &mut findings);
        assert!(
            findings
                .iter()
                .all(|f| f.help.contains("crates/det/clippy.toml"))
        );
        findings
            .into_iter()
            .map(|f| (f.rule, f.location, f.message))
            .collect()
    }

    /// Verifies: XTASK-R039
    #[test]
    fn a_complete_configuration_passes() {
        assert_eq!(
            run(Some(config("crates/det/clippy.toml", BANNED_METHODS))),
            []
        );
        // Extra bans and an inherited file that bans everything are fine too.
        let mut more = BANNED_METHODS.to_vec();
        more.push("f32::sqrt");
        assert_eq!(run(Some(config("clippy.toml", &more))), []);
    }

    /// Verifies: XTASK-R039
    #[test]
    fn no_configuration_at_all_is_reported_at_the_manifest() {
        assert_eq!(
            run(None),
            [(
                "XTASK-R039".to_string(),
                "crates/det/Cargo.toml".to_string(),
                "deterministic crate `det` has no clippy configuration (no `.clippy.toml` or \
                 `clippy.toml` in `crates/det` or a parent directory), so clippy bans none of the \
                 methods CODE_STYLE.md lists"
                    .to_string()
            )]
        );
    }

    /// Verifies: XTASK-R039
    #[test]
    fn a_configuration_missing_paths_names_each_missing_path() {
        let without_ln: Vec<&str> = BANNED_METHODS
            .iter()
            .copied()
            .filter(|path| *path != "f64::ln")
            .collect();
        assert_eq!(
            run(Some(config("crates/det/clippy.toml", &without_ln))),
            [(
                "XTASK-R039".to_string(),
                "crates/det/clippy.toml".to_string(),
                "deterministic crate `det`: its clippy configuration `crates/det/clippy.toml` \
                 does not ban `f64::ln` in `disallowed-methods`"
                    .to_string()
            )]
        );
        let inherited = run(Some(config("clippy.toml", &[])));
        assert_eq!(inherited.len(), 1);
        assert_eq!(inherited[0].1, "clippy.toml");
        assert!(inherited[0].2.contains("`f32::sin`, `f32::cos`"));
        assert!(inherited[0].2.contains("`std::time::Instant::now` in"));
    }

    /// Verifies: XTASK-R039
    #[test]
    fn a_non_deterministic_crate_needs_no_configuration() {
        let model = ArchModel {
            architecture: architecture::parse(REGISTRY).unwrap(),
            packages: vec![package("free")],
            sources: BTreeMap::new(),
            clippy: BTreeMap::new(),
        };
        let mut findings = Vec::new();
        clippy_bans(&model, &mut findings);
        assert_eq!(findings, Vec::new());
    }

    /// The constant is the `CODE_STYLE.md` list: same paths, same count (`f32` methods are listed
    /// there once, "and `f64` equivalents").
    #[test]
    fn banned_methods_match_code_style() {
        let style = std::fs::read_to_string(crate::repo::root().join("docs/process/CODE_STYLE.md"))
            .unwrap();
        let line = style
            .split("\n- ")
            .find(|item| item.starts_with("Deterministic crates"))
            .and_then(|item| item.split("\n\n").next())
            .unwrap();
        let listed: BTreeSet<String> = line
            .split('`')
            .skip(1)
            .step_by(2)
            .filter(|code| {
                code.contains("::") && !code.contains(' ') && !code.starts_with("liminal_")
            })
            .flat_map(|code| {
                let f64 = code.strip_prefix("f32::").map(|m| format!("f64::{m}"));
                std::iter::once(code.to_string()).chain(f64)
            })
            .collect();
        let constant: BTreeSet<String> = BANNED_METHODS.iter().map(|p| (*p).to_string()).collect();
        assert_eq!(constant.len(), BANNED_METHODS.len());
        assert_eq!(listed, constant);
    }
}
