//! `architecture.toml`, deserialized in full (`docs/specs/xtask.md` XTASK-R030 lists the fields).
//!
//! This module checks the TOML shape: field names and TOML types. A missing or unknown field, or a
//! value of the wrong TOML type, is internal error LIM-XTASK-0003 with its line, because no model
//! can be built without it. Value rules (valid layers, kinds, scopes, references) belong to
//! `check-arch`, which reports them as XTASK-R030 findings.

use std::collections::BTreeMap;

use serde::Deserialize;
use toml::Spanned;

use super::LoadError;

/// The whole file.
#[derive(Debug, Clone)]
pub(crate) struct Architecture {
    /// Format version; `check-arch` requires 1.
    pub schema_version: i64,
    /// Layer number (as a string key) → layer name.
    pub layers: BTreeMap<String, String>,
    /// Every `[[crate]]` entry, in file order.
    pub crates: Vec<CrateEntry>,
    /// Every `[external.<name>]` entry.
    pub external: BTreeMap<String, ExternalEntry>,
}

/// One `[[crate]]` entry.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CrateEntry {
    /// Crate name, e.g. `liminal_ecs`.
    pub name: String,
    /// Manifest directory relative to the repository root.
    pub path: String,
    /// Layer 0–5 (range checked by `check-arch`).
    pub layer: i64,
    /// `lib`, `proc-macro` or `bin`.
    pub kind: String,
    /// `forbid` or `allow`.
    #[serde(rename = "unsafe")]
    pub unsafe_policy: String,
    /// Whether ADR-0009 determinism rules apply.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "XTASK-R030 checks only its TOML type (bool), at load; no rule reads the value yet"
        )
    )]
    pub deterministic: bool,
    /// Same-layer crates this crate may depend on.
    pub same_layer_deps: Vec<String>,
    /// Approved external normal and build dependencies.
    pub external: Vec<String>,
    /// Approved external dev-dependencies.
    pub dev_external: Vec<String>,
    /// 1-based line of the entry in `architecture.toml` (not a TOML field).
    #[serde(skip)]
    pub line: usize,
}

/// One `[external.<name>]` entry.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExternalEntry {
    /// `runtime`, `build`, `dev` or `tool`.
    pub scope: String,
    /// The ADR approving the dependency.
    pub adr: String,
    /// Why it is needed.
    pub reason: String,
    /// 1-based line of the entry in `architecture.toml` (not a TOML field).
    #[serde(skip)]
    pub line: usize,
}

/// The file as deserialized, with the spans that give each entry its line.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawArchitecture {
    schema_version: i64,
    layers: BTreeMap<String, String>,
    #[serde(rename = "crate", default)]
    crates: Vec<Spanned<CrateEntry>>,
    #[serde(default)]
    external: BTreeMap<String, Spanned<ExternalEntry>>,
}

/// Deserializes the contents of `architecture.toml`.
///
/// # Errors
///
/// LIM-XTASK-0003 if the text is not valid TOML or does not have the expected shape.
pub(crate) fn parse(text: &str) -> Result<Architecture, LoadError> {
    let raw: RawArchitecture = toml::from_str(text).map_err(|error| LoadError {
        code: "LIM-XTASK-0003",
        path: "architecture.toml".to_string(),
        detail: format!("is not valid: {}", error.to_string().trim_end()),
        help: "Fix the syntax at the reported line",
    })?;
    let line = |offset: usize| text[..offset].matches('\n').count() + 1;
    Ok(Architecture {
        schema_version: raw.schema_version,
        layers: raw.layers,
        crates: raw
            .crates
            .into_iter()
            .map(|entry| {
                let line = line(entry.span().start);
                CrateEntry {
                    line,
                    ..entry.into_inner()
                }
            })
            .collect(),
        external: raw
            .external
            .into_iter()
            .map(|(name, entry)| {
                let line = line(entry.span().start);
                (
                    name,
                    ExternalEntry {
                        line,
                        ..entry.into_inner()
                    },
                )
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL: &str = r#"
schema_version = 1
[layers]
"5" = "tools"
[[crate]]
name = "demo"
path = "tools/demo"
layer = 5
kind = "bin"
unsafe = "forbid"
deterministic = false
same_layer_deps = []
external = ["serde"]
dev_external = []
[external.serde]
scope = "tool"
adr = "ADR-0003"
reason = "test"
"#;

    #[test]
    fn deserializes_every_field() {
        let architecture = parse(MINIMAL).unwrap();
        assert_eq!(architecture.schema_version, 1);
        assert_eq!(architecture.layers["5"], "tools");
        let demo = &architecture.crates[0];
        assert_eq!(
            (demo.name.as_str(), demo.path.as_str(), demo.layer),
            ("demo", "tools/demo", 5)
        );
        assert_eq!(
            (demo.kind.as_str(), demo.unsafe_policy.as_str()),
            ("bin", "forbid")
        );
        assert!(!demo.deterministic);
        assert_eq!(demo.external, ["serde"]);
        assert!(demo.same_layer_deps.is_empty() && demo.dev_external.is_empty());
        let serde = &architecture.external["serde"];
        assert_eq!(
            (
                serde.scope.as_str(),
                serde.adr.as_str(),
                serde.reason.as_str()
            ),
            ("tool", "ADR-0003", "test")
        );
    }

    #[test]
    fn entries_carry_their_line() {
        let architecture = parse(MINIMAL).unwrap();
        // MINIMAL starts with a newline: `[[crate]]` is line 5, `[external.serde]` line 15.
        assert_eq!(architecture.crates[0].line, 5);
        assert_eq!(architecture.external["serde"].line, 15);
        let real = std::fs::read_to_string(crate::repo::root().join("architecture.toml")).unwrap();
        let architecture = parse(&real).unwrap();
        for entry in &architecture.crates {
            let text = real.lines().nth(entry.line - 1).unwrap();
            assert_eq!(text, "[[crate]]", "{}", entry.name);
        }
        for (name, entry) in &architecture.external {
            let text = real.lines().nth(entry.line - 1).unwrap();
            assert_eq!(text, format!("[external.{name}]"));
        }
    }

    /// Verifies: XTASK-R030
    #[test]
    fn invalid_toml_unknown_and_missing_fields_are_lim_xtask_0003() {
        for text in [
            "schema_version = ",
            &MINIMAL.replace("kind = \"bin\"", "kind = \"bin\"\nkinds = []"),
            &MINIMAL.replace("deterministic = false\n", ""),
            &MINIMAL.replace("layer = 5", "layer = \"five\""),
            &MINIMAL.replace("schema_version = 1", "schema_version = 1\nversion = 2"),
            &MINIMAL.replace("reason = \"test\"", ""),
            &MINIMAL.replace("adr = \"ADR-0003\"", "adr = \"ADR-0003\"\nurl = \"x\""),
            &MINIMAL.replace("[layers]\n\"5\" = \"tools\"\n", ""),
        ] {
            let error = parse(text).unwrap_err();
            assert_eq!(error.code, "LIM-XTASK-0003", "{text}");
            assert_eq!(error.path, "architecture.toml");
            assert!(error.detail.contains("line "), "{}", error.detail);
        }
    }

    #[test]
    fn the_real_architecture_toml_deserializes() {
        let text = std::fs::read_to_string(crate::repo::root().join("architecture.toml")).unwrap();
        let architecture = parse(&text).unwrap();
        assert!(architecture.crates.len() > 30);
    }
}
