//! The `check-arch` rules, XTASK-R030 … R036. Every finding names the crate and, where there is
//! one, the dependency, and carries a `help` text (XTASK-R037).

use std::collections::{BTreeMap, BTreeSet};

use super::attributes::forbids_unsafe_code;
use crate::finding::Finding;
use crate::repo::architecture::{Architecture, CrateEntry};
use crate::repo::metadata::{Dependency, DependencyKind, Package};
use crate::repo::{ArchModel, crate_roots};

/// The registry file, as a finding location.
const FILE: &str = "architecture.toml";

/// Valid `kind` values.
const KINDS: &[&str] = &["lib", "proc-macro", "bin"];
/// Valid `unsafe` values.
const UNSAFE: &[&str] = &["forbid", "allow"];
/// Valid `[external.*] scope` values.
const SCOPES: &[&str] = &["runtime", "build", "dev", "tool"];
/// The tools layer (ADR-0004).
const TOOLS_LAYER: i64 = 5;

/// `architecture.toml:<line>`.
fn at(line: usize) -> String {
    format!("{FILE}:{line}")
}

/// Whether `layer` is a valid layer number, 0–5.
fn valid_layer(layer: i64) -> bool {
    (0..=TOOLS_LAYER).contains(&layer)
}

/// Whether `text` is an ADR ID, `^ADR-\d{4}$`.
fn is_adr_id(text: &str) -> bool {
    text.strip_prefix("ADR-")
        .is_some_and(|digits| digits.len() == 4 && digits.bytes().all(|b| b.is_ascii_digit()))
}

/// XTASK-R030: `architecture.toml` values and references (its shape is checked when loading).
pub(super) fn registry(architecture: &Architecture, findings: &mut Vec<Finding>) {
    registry_header(architecture, findings);
    crate_entries(architecture, findings);
    external_entries(architecture, findings);
}

/// XTASK-R030: `schema_version` and `[layers]`.
fn registry_header(architecture: &Architecture, findings: &mut Vec<Finding>) {
    let mut error = |location: String, message: String, help: &str| {
        findings.push(Finding::error("XTASK-R030", location, message, help));
    };
    if architecture.schema_version != 1 {
        error(
            FILE.to_string(),
            format!(
                "schema_version is {}; check-arch supports 1",
                architecture.schema_version
            ),
            "Set `schema_version = 1`, or update xtask for the new schema in the same change",
        );
    }
    let expected_layers: BTreeSet<String> = (0..=TOOLS_LAYER).map(|l| l.to_string()).collect();
    let layers: BTreeSet<String> = architecture.layers.keys().cloned().collect();
    if layers != expected_layers {
        error(
            FILE.to_string(),
            format!(
                "[layers] has keys {}; it must have exactly \"0\" … \"5\"",
                quoted(layers.iter().map(String::as_str))
            ),
            "List the six layers of ADR-0004 in [layers], each with its name",
        );
    }
}

/// XTASK-R030: `[[crate]]` values, unique names, `same_layer_deps` and approval references.
fn crate_entries(architecture: &Architecture, findings: &mut Vec<Finding>) {
    let mut error = |location: String, message: String, help: &str| {
        findings.push(Finding::error("XTASK-R030", location, message, help));
    };
    let mut by_name: BTreeMap<&str, &CrateEntry> = BTreeMap::new();
    for entry in &architecture.crates {
        if let Some(first) = by_name.get(entry.name.as_str()) {
            error(
                at(entry.line),
                format!(
                    "crate `{}` is registered twice (first at line {})",
                    entry.name, first.line
                ),
                "Keep one [[crate]] entry per crate",
            );
        } else {
            by_name.insert(&entry.name, entry);
        }
    }

    for entry in &architecture.crates {
        let name = &entry.name;
        if !valid_layer(entry.layer) {
            error(
                at(entry.line),
                format!("crate `{name}` has layer {}; layers are 0–5", entry.layer),
                "Use the layer of ADR-0004 that the crate belongs to",
            );
        }
        if !KINDS.contains(&entry.kind.as_str()) {
            error(
                at(entry.line),
                format!(
                    "crate `{name}` has kind `{}`; kind is one of {}",
                    entry.kind,
                    quoted(KINDS.iter().copied())
                ),
                "Use `lib`, `proc-macro`, or `bin` (has a binary target, may also have a library)",
            );
        }
        if !UNSAFE.contains(&entry.unsafe_policy.as_str()) {
            error(
                at(entry.line),
                format!(
                    "crate `{name}` has unsafe = `{}`; it is `forbid` or `allow`",
                    entry.unsafe_policy
                ),
                "Use `forbid` unless the crate needs unsafe code (ADR-0018)",
            );
        }
        for dependency in &entry.same_layer_deps {
            match by_name.get(dependency.as_str()) {
                None => error(
                    at(entry.line),
                    format!(
                        "crate `{name}` lists same_layer_deps entry `{dependency}`, which is not a \
                         registered crate"
                    ),
                    "Register the crate first, or remove the entry",
                ),
                Some(other) if other.layer != entry.layer => error(
                    at(entry.line),
                    format!(
                        "crate `{name}` (layer {}) lists same_layer_deps entry `{dependency}`, \
                         which is in layer {}",
                        entry.layer, other.layer
                    ),
                    "same_layer_deps holds crates of the same layer only; a lower layer needs no \
                     entry, a higher layer is not allowed",
                ),
                Some(_) => {}
            }
        }
        for (list, names) in [
            ("external", &entry.external),
            ("dev_external", &entry.dev_external),
        ] {
            for external in names {
                if !architecture.external.contains_key(external) {
                    error(
                        at(entry.line),
                        format!(
                            "crate `{name}` lists {list} entry `{external}`, which has no \
                             [external.{external}] entry"
                        ),
                        "Approve the crate in [external.<name>] with scope, adr and reason \
                         (docs/process/DEPENDENCIES.md)",
                    );
                }
            }
        }
    }
}

/// XTASK-R030: `[external.*]` values.
fn external_entries(architecture: &Architecture, findings: &mut Vec<Finding>) {
    let mut error = |location: String, message: String, help: &str| {
        findings.push(Finding::error("XTASK-R030", location, message, help));
    };
    for (name, external) in &architecture.external {
        if !SCOPES.contains(&external.scope.as_str()) {
            error(
                at(external.line),
                format!(
                    "external crate `{name}` has scope `{}`; scope is one of {}",
                    external.scope,
                    quoted(SCOPES.iter().copied())
                ),
                "Pick the scope from docs/process/DEPENDENCIES.md §Scopes",
            );
        }
        if !is_adr_id(&external.adr) {
            error(
                at(external.line),
                format!(
                    "external crate `{name}` has adr `{}`; it must be an ADR ID like `ADR-0003`",
                    external.adr
                ),
                "Name the ADR that approves the dependency",
            );
        }
        if external.reason.trim().is_empty() {
            error(
                at(external.line),
                format!("external crate `{name}` has an empty reason"),
                "Say why the crate is needed",
            );
        }
    }
}

/// `"a", "b"` (or `none`).
fn quoted<'a>(items: impl Iterator<Item = &'a str>) -> String {
    let items: Vec<String> = items.map(|item| format!("\"{item}\"")).collect();
    if items.is_empty() {
        "none".to_string()
    } else {
        items.join(", ")
    }
}

/// `<manifest_dir>/Cargo.toml`, the location of findings about a package.
pub(super) fn manifest(package: &Package) -> String {
    if package.manifest_dir.is_empty() {
        "Cargo.toml".to_string()
    } else {
        format!("{}/Cargo.toml", package.manifest_dir)
    }
}

/// The `kind` the registry must state for `package` (spec "`check-arch` details"):
/// `proc-macro` if it has a proc-macro target, else `bin` if it has a binary target, else `lib`.
pub(super) fn actual_kind(package: &Package) -> &'static str {
    let has = |kind: &str| {
        package
            .targets
            .iter()
            .any(|target| target.kinds.iter().any(|k| k == kind))
    };
    if has("proc-macro") {
        "proc-macro"
    } else if has("bin") {
        "bin"
    } else {
        "lib"
    }
}

/// The first registry entry named `name` (duplicates are an R030 finding).
fn entry<'a>(architecture: &'a Architecture, name: &str) -> Option<&'a CrateEntry> {
    architecture.crates.iter().find(|entry| entry.name == name)
}

/// The registry entry of a crate that R032–R036 judge: registered (else R031 reports it) and
/// with a layer in 0–5 (else R030 reports it). Spec "`check-arch` details": these rules skip
/// every other crate, so one registry mistake gives one finding.
pub(super) fn judged<'a>(architecture: &'a Architecture, name: &str) -> Option<&'a CrateEntry> {
    entry(architecture, name).filter(|entry| valid_layer(entry.layer))
}

/// XTASK-R031: every workspace member is registered with its path and kind.
pub(super) fn members(model: &ArchModel, findings: &mut Vec<Finding>) {
    for package in &model.packages {
        let name = &package.name;
        let Some(entry) = entry(&model.architecture, name) else {
            findings.push(Finding::error(
                "XTASK-R031",
                manifest(package),
                format!("workspace crate `{name}` is not registered in architecture.toml"),
                "Add a [[crate]] entry for it to architecture.toml (AGENTS.md rule 6) before \
                 adding the crate to the workspace",
            ));
            continue;
        };
        if entry.path != package.manifest_dir {
            findings.push(Finding::error(
                "XTASK-R031",
                manifest(package),
                format!(
                    "crate `{name}` is at `{}`, but architecture.toml line {} says path = `{}`",
                    package.manifest_dir, entry.line, entry.path
                ),
                "Make the [[crate]] path the manifest directory, relative to the repository \
                 root, with `/` separators",
            ));
        }
        let kind = actual_kind(package);
        if KINDS.contains(&entry.kind.as_str()) && entry.kind != kind {
            findings.push(Finding::error(
                "XTASK-R031",
                manifest(package),
                format!(
                    "crate `{name}` is a `{kind}` crate by its targets, but architecture.toml \
                     line {} says kind = `{}`",
                    entry.line, entry.kind
                ),
                "Set kind: `proc-macro` for a proc-macro library, `bin` if the package has a \
                 binary target, `lib` otherwise",
            ));
        }
    }
}

/// The workspace package a dependency points at, if it is a path dependency on one.
fn workspace_target<'a>(model: &'a ArchModel, dependency: &Dependency) -> Option<&'a Package> {
    let path = dependency.path.as_deref()?;
    model
        .packages
        .iter()
        .find(|package| package.manifest_dir == path)
}

/// XTASK-R032 … R034 and R036: every dependency of every workspace crate that is [`judged`].
pub(super) fn dependencies(model: &ArchModel, findings: &mut Vec<Finding>) {
    for package in &model.packages {
        // An unregistered crate (R031) or an invalid layer (R030): its dependencies cannot be
        // judged.
        let Some(entry) = judged(&model.architecture, &package.name) else {
            continue;
        };
        for dependency in &package.dependencies {
            match workspace_target(model, dependency) {
                Some(target) => internal(model, package, entry, dependency, target, findings),
                None => external(model, package, entry, dependency, findings),
            }
        }
    }
}

/// XTASK-R032 and R036: a dependency on another workspace crate.
fn internal(
    model: &ArchModel,
    package: &Package,
    entry: &CrateEntry,
    dependency: &Dependency,
    target: &Package,
    findings: &mut Vec<Finding>,
) {
    let Some(target_entry) = judged(&model.architecture, &target.name) else {
        return; // R031 reports an unregistered target, R030 an invalid layer.
    };
    let (from, to) = (entry.layer, target_entry.layer);
    let kind = dependency.kind.as_str();
    let (name, target_name) = (&package.name, &target.name);
    if from < TOOLS_LAYER && to == TOOLS_LAYER {
        // R036 is the specific rule; R032 does not report it again (spec "`check-arch` details").
        findings.push(Finding::error(
            "XTASK-R036",
            manifest(package),
            format!(
                "crate `{name}` (layer {from}) has a {kind} dependency on `{target_name}`, a \
                 layer 5 tool crate"
            ),
            "Tool crates never ship in games: remove the dependency, or move the code it needs \
             into a crate of layer 0–4",
        ));
    } else if to >= from && !entry.same_layer_deps.contains(target_name) {
        findings.push(Finding::error(
            "XTASK-R032",
            manifest(package),
            format!(
                "crate `{name}` (layer {from}) has a {kind} dependency on `{target_name}` (layer \
                 {to}); allowed are lower layers and the crate's same_layer_deps"
            ),
            "Remove the dependency or move the shared code to a lower layer; for a same-layer \
             crate, add it to same_layer_deps in architecture.toml (an architectural change, \
             ADR-0004)",
        ));
    }
}

/// XTASK-R033 and R034: a dependency that is not a workspace crate.
fn external(
    model: &ArchModel,
    package: &Package,
    entry: &CrateEntry,
    dependency: &Dependency,
    findings: &mut Vec<Finding>,
) {
    let (name, dep) = (&package.name, &dependency.name);
    let kind = dependency.kind;
    let (rule, list) = match kind {
        DependencyKind::Normal | DependencyKind::Build => ("XTASK-R033", &entry.external),
        DependencyKind::Dev => ("XTASK-R034", &entry.dev_external),
    };
    let list_name = if kind == DependencyKind::Dev {
        "dev_external"
    } else {
        "external"
    };
    if !list.contains(dep) {
        findings.push(Finding::error(
            rule,
            manifest(package),
            format!(
                "crate `{name}` has an external {} dependency on `{dep}`, which is not in its \
                 `{list_name}` list",
                kind.as_str()
            ),
            format!(
                "Approve `{dep}` with an ADR, add [external.{dep}] and add it to the crate's \
                 `{list_name}` list in architecture.toml (docs/process/DEPENDENCIES.md), or \
                 remove the dependency"
            ),
        ));
        return;
    }
    let Some(approval) = model.architecture.external.get(dep) else {
        return; // R030 reports the missing [external.*] entry.
    };
    let scope = approval.scope.as_str();
    if !SCOPES.contains(&scope) {
        return; // R030 reports the invalid scope.
    }
    let tools = entry.layer == TOOLS_LAYER;
    let allowed = match kind {
        DependencyKind::Dev => scope == "dev" || (tools && scope == "tool"),
        DependencyKind::Normal | DependencyKind::Build => match scope {
            "runtime" => true,
            "build" => tools || kind == DependencyKind::Build || entry.kind == "proc-macro",
            "tool" => tools,
            _ => false,
        },
    };
    if !allowed {
        let why = match (kind, scope) {
            (DependencyKind::Dev, _) => {
                "dev-dependencies must have scope `dev` (layer 5 crates may also use `tool`)"
            }
            (_, "build") => {
                "`build` scope is allowed only as a build-dependency or from a proc-macro crate \
                 (layers 0–4)"
            }
            (_, "tool") => "`tool` scope is for layer 5 crates only",
            _ => "`dev` scope is for dev-dependencies only",
        };
        findings.push(Finding::error(
            rule,
            manifest(package),
            format!(
                "crate `{name}` (layer {}, kind {}) has a {} dependency on `{dep}` with scope \
                 `{scope}`: {why}",
                entry.layer,
                entry.kind,
                kind.as_str()
            ),
            format!(
                "Move `{dep}` to the dependency section its scope allows, or write an ADR that \
                 changes its scope in [external.{dep}]"
            ),
        ));
    }
}

/// XTASK-R035: [`judged`] crates with `unsafe = "forbid"` contain `#![forbid(unsafe_code)]` in
/// every crate root file.
pub(super) fn forbid_unsafe(model: &ArchModel, findings: &mut Vec<Finding>) {
    for package in &model.packages {
        let Some(entry) = judged(&model.architecture, &package.name) else {
            continue;
        };
        if entry.unsafe_policy != "forbid" {
            continue;
        }
        for root in crate_roots(package) {
            let forbids = model
                .sources
                .get(root)
                .is_some_and(|source| forbids_unsafe_code(source));
            if !forbids {
                findings.push(Finding::error(
                    "XTASK-R035",
                    root,
                    format!(
                        "crate `{}` has unsafe = \"forbid\", but its crate root `{root}` does not \
                         start with `#![forbid(unsafe_code)]`",
                        package.name
                    ),
                    "Add `#![forbid(unsafe_code)]` to the crate root's leading inner attributes \
                     (after the `//!` docs, before any item), or change the policy with an ADR \
                     (ADR-0018)",
                ));
            }
        }
    }
}
