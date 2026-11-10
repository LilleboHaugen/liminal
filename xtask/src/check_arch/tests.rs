//! Unit tests of the `check-arch` rules on hand-built models (spec §10: "unit tests on a
//! hand-built repo model"). Fixture repositories are in `fixture_tests.rs`.

use std::collections::BTreeMap;

use super::*;
use crate::repo::architecture;
use crate::repo::metadata::{Dependency, DependencyKind, Package, Target};

/// Approved externals, one per scope, plus the header. Crate entries are appended by tests.
const HEADER: &str = r#"schema_version = 1
[layers]
"0" = "foundation"
"1" = "runtime"
"2" = "platform-gpu"
"3" = "subsystems"
"4" = "agent-interface-integration"
"5" = "tools"

[external.rt]
scope = "runtime"
adr = "ADR-0001"
reason = "runtime scope"

[external.bld]
scope = "build"
adr = "ADR-0001"
reason = "build scope"

[external.dv]
scope = "dev"
adr = "ADR-0001"
reason = "dev scope"

[external.tl]
scope = "tool"
adr = "ADR-0001"
reason = "tool scope"
"#;

/// A `[[crate]]` entry. `lists` are `same_layer_deps`, `external`, `dev_external`.
fn entry(name: &str, layer: i64, kind: &str, lists: [&[&str]; 3]) -> String {
    let list = |items: &[&str]| {
        let items: Vec<String> = items.iter().map(|item| format!("\"{item}\"")).collect();
        format!("[{}]", items.join(", "))
    };
    format!(
        "\n[[crate]]\nname = \"{name}\"\npath = \"crates/{name}\"\nlayer = {layer}\n\
         kind = \"{kind}\"\nunsafe = \"forbid\"\ndeterministic = false\n\
         same_layer_deps = {}\nexternal = {}\ndev_external = {}\n",
        list(lists[0]),
        list(lists[1]),
        list(lists[2])
    )
}

const NONE: [&[&str]; 3] = [&[], &[], &[]];

/// A package at `crates/<name>` with one target per kind (`lib` → `src/lib.rs`, `bin` →
/// `src/main.rs`, `proc-macro` → `src/lib.rs`).
fn package(name: &str, kinds: &[&str], dependencies: Vec<Dependency>) -> Package {
    Package {
        name: name.to_string(),
        manifest_dir: format!("crates/{name}"),
        targets: kinds
            .iter()
            .map(|kind| Target {
                kinds: vec![(*kind).to_string()],
                src_path: format!(
                    "crates/{name}/src/{}",
                    if *kind == "bin" { "main.rs" } else { "lib.rs" }
                ),
            })
            .collect(),
        dependencies,
        features: Vec::new(),
    }
}

/// A dependency on the workspace crate `name`.
fn on_crate(name: &str, kind: DependencyKind) -> Dependency {
    Dependency {
        name: name.to_string(),
        kind,
        path: Some(format!("crates/{name}")),
    }
}

/// A dependency on the external crate `name`.
fn on_external(name: &str, kind: DependencyKind) -> Dependency {
    Dependency {
        name: name.to_string(),
        kind,
        path: None,
    }
}

/// A model whose crate roots all forbid unsafe code.
fn model(architecture: &str, packages: Vec<Package>) -> ArchModel {
    let mut sources = BTreeMap::new();
    for package in &packages {
        for root in crate::repo::crate_roots(package) {
            sources.insert(root.to_string(), "#![forbid(unsafe_code)]\n".to_string());
        }
    }
    ArchModel {
        architecture: architecture::parse(architecture).unwrap(),
        packages,
        sources,
    }
}

/// `(rule, location, message)` of every finding.
fn summary(findings: &[Finding]) -> Vec<(String, String, String)> {
    findings
        .iter()
        .map(|f| (f.rule.clone(), f.location.clone(), f.message.clone()))
        .collect()
}

/// Asserts exactly one finding, of `rule`, whose message contains `fragment`.
fn one(findings: &[Finding], rule: &str, fragment: &str) {
    assert_eq!(findings.len(), 1, "{:#?}", summary(findings));
    assert_eq!(findings[0].rule, rule, "{:#?}", summary(findings));
    assert!(
        findings[0].message.contains(fragment),
        "expected `{fragment}` in: {}",
        findings[0].message
    );
}

use DependencyKind::{Build, Dev, Normal};

#[test]
fn a_consistent_model_has_no_findings() {
    let text = format!(
        "{HEADER}{}{}{}{}",
        entry("base", 0, "lib", [&[], &[], &["dv"]]),
        entry("derive", 0, "proc-macro", [&[], &["bld"], &[]]),
        entry("app", 1, "lib", [&[], &["rt", "bld"], &["dv"]]),
        entry("tool", 5, "bin", [&[], &["tl"], &["tl", "dv"]])
    );
    let packages = vec![
        package("base", &["lib"], vec![on_external("dv", Dev)]),
        package("derive", &["proc-macro"], vec![on_external("bld", Normal)]),
        package(
            "app",
            &["lib"],
            vec![
                on_crate("base", Normal),
                on_crate("derive", Normal),
                on_external("rt", Normal),
                on_external("bld", Build),
                on_external("dv", Dev),
            ],
        ),
        package(
            "tool",
            &["lib", "bin"],
            vec![
                on_crate("app", Normal),
                on_external("tl", Normal),
                on_external("tl", Dev),
                on_external("dv", Dev),
            ],
        ),
    ];
    let findings = check(&model(&text, packages));
    assert!(findings.is_empty(), "{:#?}", summary(&findings));
}

/// R030 cases: `(text to replace in the base, replacement, location, message fragment)`.
const R030_CASES: &[(&str, &str, &str, &str)] = &[
    (
        "schema_version = 1",
        "schema_version = 2",
        "architecture.toml",
        "schema_version is 2",
    ),
    (
        "schema_version = 1",
        "schema_version = 0",
        "architecture.toml",
        "schema_version is 0",
    ),
    (
        "\"5\" = \"tools\"\n",
        "",
        "architecture.toml",
        "[layers] has keys \"0\", \"1\", \"2\", \"3\", \"4\"",
    ),
    (
        "\"5\" = \"tools\"\n",
        "\"6\" = \"extra\"\n",
        "architecture.toml",
        "it must have exactly",
    ),
    (
        "name = \"two\"\npath = \"crates/two\"\nlayer = 1",
        "name = \"two\"\npath = \"crates/two\"\nlayer = 6",
        "architecture.toml:41",
        "layer 6",
    ),
    (
        "name = \"two\"\npath = \"crates/two\"\nlayer = 1",
        "name = \"two\"\npath = \"crates/two\"\nlayer = -1",
        "architecture.toml:41",
        "layer -1",
    ),
    (
        "layer = 1\nkind = \"lib\"\nunsafe = \"forbid\"\ndeterministic = false\nsame_layer_deps = []\nexternal = []\ndev_external = []\n\n[[crate]]\nname = \"two\"",
        "layer = 1\nkind = \"library\"\nunsafe = \"forbid\"\ndeterministic = false\nsame_layer_deps = []\nexternal = []\ndev_external = []\n\n[[crate]]\nname = \"two\"",
        "architecture.toml:30",
        "kind `library`",
    ),
    (
        "kind = \"lib\"\nunsafe = \"forbid\"\ndeterministic = false\nsame_layer_deps = []\nexternal = []\ndev_external = []\n\n[[crate]]\nname = \"two\"",
        "kind = \"lib\"\nunsafe = \"deny\"\ndeterministic = false\nsame_layer_deps = []\nexternal = []\ndev_external = []\n\n[[crate]]\nname = \"two\"",
        "architecture.toml:30",
        "unsafe = `deny`",
    ),
    (
        "name = \"two\"",
        "name = \"one\"",
        "architecture.toml:41",
        "crate `one` is registered twice (first at line 30)",
    ),
    (
        "same_layer_deps = []\nexternal = []\ndev_external = []\n\n[[crate]]\nname = \"two\"",
        "same_layer_deps = [\"three\"]\nexternal = []\ndev_external = []\n\n[[crate]]\nname = \"two\"",
        "architecture.toml:30",
        "same_layer_deps entry `three`, which is not a registered crate",
    ),
    (
        "external = []\ndev_external = []\n\n[[crate]]\nname = \"two\"",
        "external = [\"nope\"]\ndev_external = []\n\n[[crate]]\nname = \"two\"",
        "architecture.toml:30",
        "external entry `nope`, which has no [external.nope] entry",
    ),
    (
        "dev_external = []\n\n[[crate]]\nname = \"two\"",
        "dev_external = [\"nope\"]\n\n[[crate]]\nname = \"two\"",
        "architecture.toml:30",
        "dev_external entry `nope`",
    ),
    (
        "scope = \"runtime\"",
        "scope = \"shipped\"",
        "architecture.toml:10",
        "scope `shipped`",
    ),
    (
        "adr = \"ADR-0001\"\nreason = \"dev scope\"",
        "adr = \"0001\"\nreason = \"dev scope\"",
        "architecture.toml:20",
        "adr `0001`",
    ),
    (
        "adr = \"ADR-0001\"\nreason = \"dev scope\"",
        "adr = \"ADR-00001\"\nreason = \"dev scope\"",
        "architecture.toml:20",
        "adr `ADR-00001`",
    ),
    (
        "reason = \"dev scope\"",
        "reason = \" \"",
        "architecture.toml:20",
        "empty reason",
    ),
];

/// Verifies: XTASK-R030
#[test]
fn r030_every_invalid_value_and_reference_is_one_finding_at_its_line() {
    let base = format!(
        "{HEADER}{}{}",
        entry("one", 1, "lib", NONE),
        entry("two", 1, "lib", NONE)
    );
    // The first [[crate]] entry is on line 30 of `base`, the second on line 41, [external.rt] on
    // line 10, [external.dv] on line 20.
    let cases = R030_CASES;
    assert_eq!(check(&model(&base, Vec::new())), []);
    for (index, (from, to, location, fragment)) in cases.iter().enumerate() {
        let text = base.replacen(from, to, 1);
        assert_ne!(text, base, "case {index}: `{from}` not found");
        let findings = check(&model(&text, Vec::new()));
        one(&findings, "XTASK-R030", fragment);
        assert_eq!(findings[0].location, *location, "case {index}");
    }
    // `one` (layer 1) lists `two` (layer 2) in same_layer_deps.
    let text = base
        .replacen("same_layer_deps = []", "same_layer_deps = [\"two\"]", 1)
        .replacen(
            "path = \"crates/two\"\nlayer = 1",
            "path = \"crates/two\"\nlayer = 2",
            1,
        );
    let findings = check(&model(&text, Vec::new()));
    one(
        &findings,
        "XTASK-R030",
        "crate `one` (layer 1) lists same_layer_deps entry `two`, which is in layer 2",
    );
    assert_eq!(findings[0].location, "architecture.toml:30");
    // A lower layer is not a same-layer crate either.
    let text = format!(
        "{HEADER}{}{}",
        entry("one", 0, "lib", NONE),
        entry("two", 1, "lib", [&["one"], &[], &[]])
    );
    one(
        &check(&model(&text, Vec::new())),
        "XTASK-R030",
        "crate `two` (layer 1) lists same_layer_deps entry `one`, which is in layer 0",
    );
}

/// Verifies: XTASK-R031
#[test]
fn r031_members_are_registered_with_their_path_and_kind() {
    let text = format!(
        "{HEADER}{}{}",
        entry("app", 1, "lib", NONE),
        entry("derive", 1, "proc-macro", NONE)
    );
    // Unregistered.
    let findings = check(&model(&text, vec![package("ghost", &["lib"], Vec::new())]));
    one(&findings, "XTASK-R031", "`ghost` is not registered");
    assert_eq!(findings[0].location, "crates/ghost/Cargo.toml");
    // Path mismatch.
    let mut moved = package("app", &["lib"], Vec::new());
    moved.manifest_dir = "engine/app".to_string();
    one(
        &check(&model(&text, vec![moved])),
        "XTASK-R031",
        "is at `engine/app`, but architecture.toml line 30 says path = `crates/app`",
    );
    // Kind mismatches: a lib+bin package is `bin`; a proc-macro is `proc-macro`.
    one(
        &check(&model(
            &text,
            vec![package("app", &["lib", "bin"], Vec::new())],
        )),
        "XTASK-R031",
        "is a `bin` crate by its targets, but architecture.toml line 30 says kind = `lib`",
    );
    one(
        &check(&model(&text, vec![package("derive", &["lib"], Vec::new())])),
        "XTASK-R031",
        "is a `lib` crate",
    );
    let proc_macro_registered_as_lib = text.replacen("kind = \"proc-macro\"", "kind = \"bin\"", 1);
    one(
        &check(&model(
            &proc_macro_registered_as_lib,
            vec![package("derive", &["proc-macro"], Vec::new())],
        )),
        "XTASK-R031",
        "is a `proc-macro` crate",
    );
    // A proc-macro target decides before a binary target.
    assert_eq!(
        check(&model(
            &text,
            vec![package("derive", &["proc-macro", "bin"], Vec::new())]
        )),
        []
    );
    // Other target kinds (tests, examples, build scripts) do not make a crate `bin`.
    let mut with_extras = package("app", &["lib"], Vec::new());
    for kind in ["test", "example", "bench", "custom-build"] {
        with_extras.targets.push(Target {
            kinds: vec![kind.to_string()],
            src_path: format!("crates/app/{kind}.rs"),
        });
    }
    assert_eq!(check(&model(&text, vec![with_extras])), []);
    // An invalid registered kind is R030 only, not also an R031 mismatch.
    let invalid = text.replacen("kind = \"lib\"", "kind = \"library\"", 1);
    let findings = check(&model(&invalid, vec![package("app", &["lib"], Vec::new())]));
    one(&findings, "XTASK-R030", "kind `library`");
}

/// Verifies: XTASK-R032
#[test]
fn r032_workspace_dependencies_go_to_lower_layers_or_listed_same_layer_crates() {
    let text = format!(
        "{HEADER}{}{}{}{}",
        entry("low", 0, "lib", NONE),
        entry("mid", 1, "lib", [&["peer"], &[], &[]]),
        entry("peer", 1, "lib", NONE),
        entry("high", 2, "lib", NONE)
    );
    let packages = |dependencies: Vec<Dependency>| {
        vec![
            package("low", &["lib"], Vec::new()),
            package("mid", &["lib"], dependencies),
            package("peer", &["lib"], Vec::new()),
            package("high", &["lib"], Vec::new()),
        ]
    };
    for kind in [Normal, Build, Dev] {
        // Lower layer and listed same-layer crate: allowed.
        let allowed = vec![on_crate("low", kind), on_crate("peer", kind)];
        assert_eq!(check(&model(&text, packages(allowed))), []);
        // Higher layer: not allowed, whatever the dependency kind.
        let findings = check(&model(&text, packages(vec![on_crate("high", kind)])));
        one(
            &findings,
            "XTASK-R032",
            &format!(
                "crate `mid` (layer 1) has a {} dependency on `high` (layer 2)",
                kind.as_str()
            ),
        );
        assert_eq!(findings[0].location, "crates/mid/Cargo.toml");
    }
    // Same layer, not listed: `peer` → `mid`.
    let mut packages = packages(Vec::new());
    packages[2].dependencies.push(on_crate("mid", Normal));
    one(
        &check(&model(&text, packages)),
        "XTASK-R032",
        "crate `peer` (layer 1) has a normal dependency on `mid` (layer 1)",
    );
}

/// Verifies: XTASK-R032
#[test]
fn r032_a_path_dependency_on_a_root_package_is_a_workspace_dependency() {
    // `cargo metadata` output: the root package `high` (layer 2, manifest `/w/Cargo.toml`) and
    // `mid` (layer 1) with a path dependency on the workspace root `/w`.
    let json = r#"{
  "packages": [
    { "name": "high", "id": "high-id", "manifest_path": "/w/Cargo.toml",
      "targets": [ { "kind": ["lib"], "src_path": "/w/src/lib.rs" } ], "dependencies": [],
      "features": {} },
    { "name": "mid", "id": "mid-id", "manifest_path": "/w/crates/mid/Cargo.toml",
      "targets": [ { "kind": ["lib"], "src_path": "/w/crates/mid/src/lib.rs" } ],
      "dependencies": [ { "name": "high", "kind": null, "path": "/w" } ], "features": {} }
  ],
  "workspace_members": ["high-id", "mid-id"],
  "workspace_root": "/w"
}"#;
    let packages = crate::repo::metadata::parse(json).unwrap();
    let text = format!(
        "{HEADER}{}{}",
        entry("high", 2, "lib", NONE).replace("path = \"crates/high\"", "path = \"\""),
        entry("mid", 1, "lib", NONE)
    );
    let findings = check(&model(&text, packages));
    one(
        &findings,
        "XTASK-R032",
        "crate `mid` (layer 1) has a normal dependency on `high` (layer 2)",
    );
    assert_eq!(findings[0].location, "crates/mid/Cargo.toml");
}

/// The R033/R034 scope table of `docs/specs/xtask.md` §6 "`check-arch` details", copied row
/// by row from the spec text (not from the implementation): `(layer group, crate kinds,
/// dependency kinds, [runtime, build, dev, tool])`.
type SpecScopeRow = (
    &'static str,
    &'static [&'static str],
    &'static [&'static str],
    [bool; 4],
);

const SPEC_SCOPE_ROWS: &[SpecScopeRow] = &[
    (
        "0-4",
        &["lib", "bin"],
        &["normal"],
        [true, false, false, false],
    ),
    (
        "0-4",
        &["proc-macro"],
        &["normal"],
        [true, true, false, false],
    ),
    (
        "0-4",
        &["lib", "bin", "proc-macro"],
        &["build"],
        [true, true, false, false],
    ),
    (
        "5",
        &["lib", "bin", "proc-macro"],
        &["normal", "build"],
        [true, true, false, true],
    ),
    (
        "0-4",
        &["lib", "bin", "proc-macro"],
        &["dev"],
        [false, false, true, false],
    ),
    (
        "5",
        &["lib", "bin", "proc-macro"],
        &["dev"],
        [false, false, true, true],
    ),
];

const SCOPES: [&str; 4] = ["runtime", "build", "dev", "tool"];

/// The full product layer group × crate kind × dependency kind × scope (2 × 3 × 3 × 4 = 72
/// rows), each with the value of the one spec row that covers it.
fn scope_table() -> Vec<(&'static str, &'static str, &'static str, &'static str, bool)> {
    let mut table = Vec::new();
    for layers in ["0-4", "5"] {
        for crate_kind in ["lib", "bin", "proc-macro"] {
            for dep_kind in ["normal", "build", "dev"] {
                let rows: Vec<_> = SPEC_SCOPE_ROWS
                    .iter()
                    .filter(|(group, kinds, deps, _)| {
                        *group == layers && kinds.contains(&crate_kind) && deps.contains(&dep_kind)
                    })
                    .collect();
                assert_eq!(rows.len(), 1, "{layers} {crate_kind} {dep_kind}: {rows:?}");
                for (scope, allowed) in SCOPES.iter().zip(rows[0].3) {
                    table.push((layers, crate_kind, dep_kind, *scope, allowed));
                }
            }
        }
    }
    table
}

#[test]
fn the_scope_table_has_every_combination_once() {
    let table = scope_table();
    assert_eq!(table.len(), 72);
    let keys: std::collections::BTreeSet<_> = table
        .iter()
        .map(|(l, c, d, s, _)| (*l, *c, *d, *s))
        .collect();
    assert_eq!(keys.len(), 72);
}

/// Verifies: XTASK-R033, XTASK-R034
#[test]
fn r033_r034_scope_rules_for_every_layer_group_crate_kind_dependency_kind_and_scope() {
    let scope_crate = |scope: &str| match scope {
        "runtime" => "rt",
        "build" => "bld",
        "dev" => "dv",
        _ => "tl",
    };
    let mut runs = 0;
    for (layers, crate_kind, dep_kind, scope, allowed) in scope_table() {
        let external = scope_crate(scope);
        let kind = match dep_kind {
            "normal" => Normal,
            "build" => Build,
            _ => Dev,
        };
        // Listed in both lists, so only the scope decides.
        let lists: [&[&str]; 3] = [&[], &[external], &[external]];
        for layer in if layers == "5" { vec![5] } else { vec![0, 4] } {
            runs += 1;
            let text = format!("{HEADER}{}", entry("c", layer, crate_kind, lists));
            let targets: &[&str] = match crate_kind {
                "bin" => &["bin"],
                "proc-macro" => &["proc-macro"],
                _ => &["lib"],
            };
            let packages = vec![package("c", targets, vec![on_external(external, kind)])];
            let findings = check(&model(&text, packages));
            let case = format!("layer {layer} {crate_kind}, {dep_kind} dependency, scope {scope}");
            if allowed {
                assert!(findings.is_empty(), "{case}: {:#?}", summary(&findings));
            } else {
                let rule = if kind == Dev {
                    "XTASK-R034"
                } else {
                    "XTASK-R033"
                };
                one(&findings, rule, &format!("with scope `{scope}`"));
            }
        }
    }
    // 36 combinations of layers 0–4 at layers 0 and 4, 36 of layer 5 at layer 5.
    assert_eq!(runs, 36 * 2 + 36);
}

/// Verifies: XTASK-R033, XTASK-R034
#[test]
fn r033_r034_external_dependencies_must_be_in_the_matching_list() {
    // `rt` only in dev_external, `dv` only in external: each used in the other section fails.
    let text = format!("{HEADER}{}", entry("c", 1, "lib", [&[], &["dv"], &["rt"]]));
    let findings = check(&model(
        &text,
        vec![package("c", &["lib"], vec![on_external("rt", Normal)])],
    ));
    one(
        &findings,
        "XTASK-R033",
        "crate `c` has an external normal dependency on `rt`, which is not in its `external` list",
    );
    let findings = check(&model(
        &text,
        vec![package("c", &["lib"], vec![on_external("rt", Build)])],
    ));
    one(&findings, "XTASK-R033", "external build dependency on `rt`");
    let findings = check(&model(
        &text,
        vec![package("c", &["lib"], vec![on_external("dv", Dev)])],
    ));
    one(
        &findings,
        "XTASK-R034",
        "external dev dependency on `dv`, which is not in its `dev_external` list",
    );
    // Not approved at all and not listed: the dependency rule reports it.
    let findings = check(&model(
        &text,
        vec![package("c", &["lib"], vec![on_external("unknown", Dev)])],
    ));
    one(&findings, "XTASK-R034", "`unknown`");
}

#[test]
fn listed_but_unapproved_or_invalid_scope_is_r030_only() {
    let text = format!(
        "{HEADER}{}",
        entry("c", 1, "lib", [&[], &["nope", "rt"], &[]])
    )
    .replacen("scope = \"runtime\"", "scope = \"shipped\"", 1);
    let packages = vec![package(
        "c",
        &["lib"],
        vec![on_external("nope", Normal), on_external("rt", Normal)],
    )];
    let findings = check(&model(&text, packages));
    let rules: Vec<&str> = findings.iter().map(|f| f.rule.as_str()).collect();
    assert_eq!(
        rules,
        ["XTASK-R030", "XTASK-R030"],
        "{:#?}",
        summary(&findings)
    );
}

/// Verifies: XTASK-R035
#[test]
fn r035_every_crate_root_of_a_forbid_crate_forbids_unsafe_code() {
    let text = format!(
        "{HEADER}{}{}",
        entry("safe", 1, "bin", NONE),
        entry("fast", 1, "lib", NONE).replace("unsafe = \"forbid\"", "unsafe = \"allow\"")
    );
    let packages = vec![
        package("safe", &["lib", "bin"], Vec::new()),
        package("fast", &["lib"], Vec::new()),
    ];
    let mut model = model(&text, packages);
    assert_eq!(check(&model), []);
    // An `allow` crate needs no attribute.
    model.sources.insert(
        "crates/fast/src/lib.rs".to_string(),
        "pub fn f() {}\n".to_string(),
    );
    assert_eq!(check(&model), []);
    // Every root of a `forbid` crate is checked: the binary root too.
    model.sources.insert(
        "crates/safe/src/main.rs".to_string(),
        "// #![forbid(unsafe_code)]\nfn main() {}\n".to_string(),
    );
    let findings = check(&model);
    one(
        &findings,
        "XTASK-R035",
        "crate `safe` has unsafe = \"forbid\", but its crate root `crates/safe/src/main.rs`",
    );
    assert_eq!(findings[0].location, "crates/safe/src/main.rs");
}

/// Verifies: XTASK-R036
#[test]
fn r036_no_layer_0_to_4_crate_depends_on_a_tool_crate() {
    let text = format!(
        "{HEADER}{}{}{}",
        entry("facade", 4, "lib", NONE),
        entry("tool", 5, "bin", NONE),
        entry("other_tool", 5, "lib", [&["tool"], &[], &[]])
    );
    for kind in [Normal, Build, Dev] {
        let packages = vec![
            package("facade", &["lib"], vec![on_crate("tool", kind)]),
            package("tool", &["bin"], Vec::new()),
        ];
        let findings = check(&model(&text, packages));
        // R036 only: R032 does not report the same dependency again.
        one(
            &findings,
            "XTASK-R036",
            &format!(
                "crate `facade` (layer 4) has a {} dependency on `tool`, a layer 5 tool crate",
                kind.as_str()
            ),
        );
    }
    // Between tool crates, R032 applies as usual.
    let packages = vec![
        package("tool", &["bin"], vec![on_crate("other_tool", Normal)]),
        package("other_tool", &["lib"], vec![on_crate("tool", Normal)]),
    ];
    one(
        &check(&model(&text, packages)),
        "XTASK-R032",
        "crate `tool` (layer 5) has a normal dependency on `other_tool` (layer 5)",
    );
}

/// Verifies: XTASK-R037
#[test]
fn r037_every_finding_names_the_crate_the_dependency_and_has_help() {
    let text = format!(
        "{HEADER}{}{}{}",
        entry("app", 1, "lib", [&["ghost_peer"], &["tl"], &[]]),
        entry("high", 2, "lib", NONE),
        entry("tool", 5, "lib", NONE)
    );
    let packages = vec![
        package("unlisted", &["lib"], Vec::new()),
        package(
            "app",
            &["lib"],
            vec![
                on_crate("high", Normal),
                on_crate("tool", Dev),
                on_external("tl", Normal),
                on_external("missing", Dev),
            ],
        ),
        package("high", &["lib"], Vec::new()),
        package("tool", &["lib"], Vec::new()),
    ];
    let mut model = model(&text, packages);
    model
        .sources
        .insert("crates/high/src/lib.rs".to_string(), String::new());
    let findings = check(&model);
    let mut rules: Vec<&str> = findings.iter().map(|f| f.rule.as_str()).collect();
    rules.sort_unstable();
    rules.dedup();
    assert_eq!(
        rules,
        [
            "XTASK-R030",
            "XTASK-R031",
            "XTASK-R032",
            "XTASK-R033",
            "XTASK-R034",
            "XTASK-R035",
            "XTASK-R036"
        ],
        "{:#?}",
        summary(&findings)
    );
    let dependency_of = |rule: &str| match rule {
        "XTASK-R032" => Some("`high`"),
        "XTASK-R033" => Some("`tl`"),
        "XTASK-R034" => Some("`missing`"),
        "XTASK-R036" => Some("`tool`"),
        _ => None,
    };
    for finding in &findings {
        let crate_name = match finding.rule.as_str() {
            "XTASK-R031" => "`unlisted`",
            "XTASK-R035" => "`high`",
            _ => "`app`",
        };
        assert!(
            finding.message.contains(&format!("crate {crate_name}"))
                || finding
                    .message
                    .contains(&format!("workspace crate {crate_name}")),
            "{finding:?}"
        );
        if let Some(dependency) = dependency_of(&finding.rule) {
            assert!(finding.message.contains(dependency), "{finding:?}");
        }
        assert!(finding.help.len() > 20, "{finding:?}");
    }
}

#[test]
fn unregistered_crates_have_no_dependency_or_unsafe_findings() {
    let text = format!("{HEADER}{}", entry("high", 2, "lib", NONE));
    let packages = vec![
        package(
            "ghost",
            &["lib"],
            vec![on_crate("high", Normal), on_external("anything", Normal)],
        ),
        package("high", &["lib"], Vec::new()),
    ];
    let mut model = model(&text, packages);
    model
        .sources
        .insert("crates/ghost/src/lib.rs".to_string(), String::new());
    one(&check(&model), "XTASK-R031", "`ghost` is not registered");
}

#[test]
fn duplicate_declarations_give_one_finding() {
    // The same dependency under two `cfg` targets appears twice in cargo metadata.
    let text = format!("{HEADER}{}", entry("c", 1, "lib", NONE));
    let packages = vec![package(
        "c",
        &["lib"],
        vec![on_external("x", Normal), on_external("x", Normal)],
    )];
    one(&check(&model(&text, packages)), "XTASK-R033", "`x`");
}

/// Verifies: XTASK-R032, XTASK-R033, XTASK-R034, XTASK-R035, XTASK-R036
#[test]
fn a_crate_with_an_invalid_layer_gets_only_the_r030_finding() {
    // Spec "`check-arch` details": R032–R036 skip a layer outside 0–5; R030 reports it.
    // `bad` (layer 7) would otherwise break R033 twice (a `tool`-scope normal dependency, which
    // the scope table judges by the layer, and an unlisted one), R034 (unlisted
    // dev-dependency), R035 (no attribute) and, as a dependent or a target, R032/R036.
    let text = format!(
        "{HEADER}{}{}{}",
        entry("bad", 7, "lib", [&[], &["tl"], &[]]),
        entry("low", 0, "lib", NONE),
        entry("tool", 5, "lib", NONE)
    );
    let packages = vec![
        package(
            "bad",
            &["lib"],
            vec![
                on_external("tl", Normal),
                on_external("unlisted", Normal),
                on_external("unlisted_dev", Dev),
                on_crate("low", Normal),
                on_crate("tool", Normal),
            ],
        ),
        package("low", &["lib"], vec![on_crate("bad", Normal)]),
        package("tool", &["lib"], vec![on_crate("bad", Dev)]),
    ];
    let mut model = model(&text, packages);
    model.sources.insert(
        "crates/bad/src/lib.rs".to_string(),
        "pub fn f() {}\n".to_string(),
    );
    let findings = check(&model);
    one(
        &findings,
        "XTASK-R030",
        "crate `bad` has layer 7; layers are 0–5",
    );
    assert_eq!(findings[0].location, "architecture.toml:30");
}

/// Verifies: XTASK-R031, XTASK-R032, XTASK-R036
#[test]
fn a_dependency_on_an_unregistered_workspace_crate_is_reported_by_r031_only() {
    // `ghost` is a workspace crate without a [[crate]] entry. Dependencies on it from a low layer
    // and from a tool-layer crate are not judged by R032/R036, and not as external (R033/R034).
    let text = format!(
        "{HEADER}{}{}",
        entry("low", 0, "lib", NONE),
        entry("tool", 5, "lib", NONE)
    );
    let packages = vec![
        package("ghost", &["lib"], Vec::new()),
        package(
            "low",
            &["lib"],
            vec![on_crate("ghost", Normal), on_crate("ghost", Dev)],
        ),
        package("tool", &["lib"], vec![on_crate("ghost", Build)]),
    ];
    let findings = check(&model(&text, packages));
    one(&findings, "XTASK-R031", "`ghost` is not registered");
    assert_eq!(findings[0].location, "crates/ghost/Cargo.toml");
}
