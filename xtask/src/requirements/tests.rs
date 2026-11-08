//! Unit tests of requirement extraction on the `check-docs` `valid` fixture with specs added in
//! memory.

use std::path::Path;

use super::*;
use crate::repo;

fn model_with(specs: &[(&str, &str)]) -> RepoModel {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/check_docs/valid");
    let mut model = repo::load(&root).unwrap();
    for (path, text) in specs {
        model
            .markdown
            .insert((*path).to_string(), (*text).to_string());
    }
    model
}

fn spec(prefix: &str, maturity: &str, rows: &str) -> String {
    format!(
        "---\ncrate: x\nreq_prefix: {prefix}\nlayer: 5\nmaturity: {maturity}\nspec_version: 1\n\
         last_reviewed: 2026-11-08\nadrs: []\n---\n\n# Spec\n\n## 6. Requirements\n\n\
         ### Group\n\n| ID | Level | Requirement |\n|---|---|---|\n{rows}\n## 7. Invariants\n"
    )
}

fn summary(findings: &[Finding]) -> Vec<(&str, &str, &str)> {
    let mut list: Vec<(&str, &str, &str)> = findings
        .iter()
        .map(|f| (f.rule.as_str(), f.location.as_str(), f.message.as_str()))
        .collect();
    list.sort_unstable();
    list
}

/// Verifies: XTASK-R040
#[test]
fn rows_in_section_6_tables_are_requirements_with_level_and_withdrawn_flag() {
    let text = spec(
        "ALPHA",
        "usable",
        "| ALPHA-R001 | MUST | First. |\n| ALPHA-R002 | SHOULD | Second (withdrawn in v2). |\n\
         | ALPHA-R003 | MAY | Third. |\n| not an id | MUST | ignored |\n| `ALPHA-R004` | MUST | ignored |\n",
    );
    let extracted = extract(&model_with(&[("docs/specs/alpha.md", &text)]));
    assert_eq!(summary(&extracted.findings), []);
    let alpha = extracted
        .specs
        .iter()
        .position(|s| s.path == "docs/specs/alpha.md")
        .unwrap();
    assert_eq!(
        extracted.specs[alpha],
        Spec {
            path: "docs/specs/alpha.md".to_string(),
            crate_name: "alpha".to_string(),
            maturity: Some("usable".to_string()),
        }
    );
    assert!(extracted.specs[alpha].gated());
    let rows: Vec<(&str, Level, bool, usize, usize)> = extracted
        .requirements
        .iter()
        .map(|r| (r.id.as_str(), r.level, r.withdrawn, r.spec, r.line))
        .collect();
    let demo = extracted
        .specs
        .iter()
        .position(|s| s.path == "docs/specs/demo.md")
        .unwrap();
    assert_eq!(
        rows,
        [
            ("ALPHA-R001", Level::Must, false, alpha, 19),
            ("ALPHA-R002", Level::Should, true, alpha, 20),
            ("ALPHA-R003", Level::May, false, alpha, 21),
            ("DEMO-R001", Level::Must, false, demo, 17),
        ]
    );
}

/// Verifies: XTASK-R040
#[test]
fn tables_outside_section_6_and_in_fenced_code_are_not_requirements() {
    let text = "---\ncrate: x\nreq_prefix: BETA\n---\n\n## 5. API\n\n| ID | Level | Requirement |\n\
                |---|---|---|\n| BETA-R001 | MUST | Before section 6. |\n\n## 6. Requirements\n\n\
                ```\n## 7. Not a heading\n```\n\n| ID | Level | Requirement |\n|---|---|---|\n\
                | BETA-R002 | MUST | In section 6. |\n\n## 7. Invariants\n\n| ID | Level | Requirement |\n\
                |---|---|---|\n| BETA-R003 | MUST | After section 6. |\n";
    let extracted = extract(&model_with(&[("docs/specs/beta.md", text)]));
    let ids: Vec<&str> = extracted
        .requirements
        .iter()
        .map(|r| r.id.as_str())
        .collect();
    assert_eq!(ids, ["BETA-R002", "DEMO-R001"]);
    assert_eq!(summary(&extracted.findings), []);
}

/// Verifies: XTASK-R040
#[test]
fn malformed_rows_and_foreign_prefixes_are_r040_findings() {
    let text = spec(
        "GAMMA",
        "planned",
        "| GAMMA-R001 | Must | Lower case level. |\n| GAMMA-R002 | MUST | |\n| GAMMA-R003 |\n\
         | OTHER-R004 | MUST | Foreign prefix. |\n",
    );
    let extracted = extract(&model_with(&[("docs/specs/gamma.md", &text)]));
    let found = summary(&extracted.findings);
    assert_eq!(
        found.iter().map(|(r, l, _)| (*r, *l)).collect::<Vec<_>>(),
        [
            ("XTASK-R040", "docs/specs/gamma.md:19"),
            ("XTASK-R040", "docs/specs/gamma.md:20"),
            ("XTASK-R040", "docs/specs/gamma.md:21"),
            ("XTASK-R040", "docs/specs/gamma.md:22"),
        ]
    );
    assert!(
        found[3]
            .2
            .contains("`OTHER-R004` does not use the spec's prefix")
    );
    // The foreign-prefix row is still a requirement; the malformed rows are known IDs only.
    assert!(extracted.get("OTHER-R004").is_some());
    for id in ["GAMMA-R001", "GAMMA-R002", "GAMMA-R003"] {
        assert!(
            extracted.get(id).is_none() && extracted.is_known(id),
            "{id}"
        );
    }
    assert!(!extracted.is_known("GAMMA-R009"));
}

/// Verifies: XTASK-R045
#[test]
fn duplicate_ids_within_and_across_specs_are_r045_findings() {
    let first = spec(
        "DELTA",
        "planned",
        "| DELTA-R001 | MUST | One. |\n| DELTA-R001 | SHOULD | Again. |\n",
    );
    let second = spec("DELTB", "planned", "| DELTA-R001 | MAY | Elsewhere. |\n");
    let extracted = extract(&model_with(&[
        ("docs/specs/delta.md", &first),
        ("docs/specs/deltb.md", &second),
    ]));
    let found = summary(&extracted.findings);
    let r045: Vec<(&str, &str)> = found
        .iter()
        .filter(|(rule, _, _)| *rule == "XTASK-R045")
        .map(|(_, location, message)| (*location, *message))
        .collect();
    assert_eq!(
        r045,
        [
            (
                "docs/specs/delta.md:20",
                "requirement ID `DELTA-R001` is already defined at docs/specs/delta.md:19"
            ),
            (
                "docs/specs/deltb.md:19",
                "requirement ID `DELTA-R001` is already defined at docs/specs/delta.md:19"
            ),
        ]
    );
    let delta: Vec<&Requirement> = extracted
        .requirements
        .iter()
        .filter(|r| r.id == "DELTA-R001")
        .collect();
    assert_eq!(delta.len(), 1);
    assert_eq!((delta[0].level, delta[0].line), (Level::Must, 19));
}

#[test]
fn broken_frontmatter_still_yields_requirements_but_no_maturity_or_prefix_check() {
    let text = "no frontmatter\n\n## 6. Requirements\n\n| ID | Level | Requirement |\n|---|---|---|\n\
                | ANY-R001 | MUST | Kept. |\n";
    let extracted = extract(&model_with(&[("docs/specs/eps.md", text)]));
    let eps = extracted
        .specs
        .iter()
        .find(|s| s.path == "docs/specs/eps.md")
        .unwrap();
    assert_eq!(eps.maturity, None);
    assert!(!eps.gated());
    assert!(extracted.get("ANY-R001").is_some());
    assert_eq!(summary(&extracted.findings), []);
}

#[test]
fn gated_maturities_are_usable_stable_and_optimized() {
    for (maturity, gated) in [
        ("planned", false),
        ("experimental", false),
        ("usable", true),
        ("stable", true),
        ("optimized", true),
    ] {
        let spec = Spec {
            path: String::new(),
            crate_name: String::new(),
            maturity: Some(maturity.to_string()),
        };
        assert_eq!(spec.gated(), gated, "{maturity}");
    }
}

#[test]
fn requirement_id_form() {
    for id in ["XTASK-R001", "A-R999"] {
        assert!(is_requirement_id(id), "{id}");
    }
    for id in [
        "xtask-R001",
        "-R001",
        "XTASK-R01",
        "XTASK-R0001",
        "XTASK-001",
        "X1-R001",
        "XTASK-R00a",
    ] {
        assert!(!is_requirement_id(id), "{id}");
    }
}

#[test]
fn the_real_xtask_spec_has_its_requirements() {
    let model = repo::load(&repo::root()).unwrap();
    let extracted = extract(&model);
    assert_eq!(summary(&extracted.findings), []);
    let xtask: Vec<&str> = extracted
        .requirements
        .iter()
        .filter(|r| extracted.specs[r.spec].crate_name == "xtask")
        .map(|r| r.id.as_str())
        .collect();
    assert!(xtask.contains(&"XTASK-R001") && xtask.contains(&"XTASK-R081"));
    assert!(xtask.len() >= 50, "{}", xtask.len());
}

/// Verifies: XTASK-R040
#[test]
fn the_withdrawn_marker_counts_only_outside_inline_code() {
    for (text, withdrawn) in [
        ("Old (withdrawn in v3).", true),
        ("Old (withdrawn).", true),
        ("Marked by `(withdrawn` in the text.", false),
        ("Marked by ``x `(withdrawn` y`` in code.", false),
        ("Code `x` then (withdrawn in v2).", true),
        ("Unmatched ` then (withdrawn).", true),
        ("Withdrawn without the parenthesis.", false),
    ] {
        let rows = format!("| ZETA-R001 | MUST | {text} |\n");
        let extracted = extract(&model_with(&[(
            "docs/specs/zeta.md",
            &spec("ZETA", "planned", &rows),
        )]));
        assert_eq!(
            extracted.get("ZETA-R001").unwrap().withdrawn,
            withdrawn,
            "{text}"
        );
    }
}
