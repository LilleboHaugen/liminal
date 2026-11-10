//! Unit tests of `trace`. Inputs with annotations that name unknown or fixture-only requirement
//! IDs are files under `xtask/tests/fixtures/trace/` (M0-T005 notes: never inline string
//! literals), read with `include_str!` or loaded as fixture repositories.

use std::path::{Path, PathBuf};

use super::annotations::Annotation;
use super::*;
use crate::test_support::{TempRoot, copy_dir};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/trace")
}

fn valid() -> SourceModel {
    sources::load(&fixtures().join("valid")).unwrap()
}

fn verifies(line: usize, ids: &[&str], function: Option<(&str, usize)>) -> Annotation {
    Annotation {
        line,
        content: Content::Verifies {
            ids: ids.iter().map(ToString::to_string).collect(),
            function: function.map(|(name, line)| (name.to_string(), line)),
        },
    }
}

fn malformed(line: usize) -> Annotation {
    Annotation {
        line,
        content: Content::Malformed,
    }
}

/// Verifies: XTASK-R041
#[test]
fn annotations_are_attached_to_the_next_function() {
    let found = annotations(include_str!(
        "../../tests/fixtures/trace/cases/annotations.rs"
    ));
    let (r1, r2, r3) = ("DEMO-R001", "DEMO-R002", "DEMO-R003");
    assert_eq!(
        found,
        [
            verifies(1, &[r1], Some(("a", 3))),
            verifies(6, &[r1, r2], Some(("b", 13))),
            verifies(15, &[r1, r2, r1], Some(("c", 16))),
            malformed(18),
            malformed(19),
            malformed(20),
            malformed(21),
            verifies(26, &[r1], None),
            verifies(33, &[r3], Some(("f", 34))),
            verifies(35, &[r1], None),
            verifies(37, &[r1], Some(("h", 38))),
            verifies(39, &[r1], None),
            verifies(41, &[r1], None),
        ]
    );
}

/// Verifies: XTASK-R041
#[test]
fn malformed_and_unattached_annotations_are_r041_findings() {
    let mut model = valid();
    model.sources.push(sources::SourceFile {
        path: "crates/demo/src/case.rs".to_string(),
        crate_name: "demo".to_string(),
        text: include_str!("../../tests/fixtures/trace/cases/annotations.rs").to_string(),
    });
    let result = trace(&model);
    let found: Vec<(&str, &str)> = result
        .findings
        .iter()
        .map(|(_, f)| (f.rule.as_str(), f.location.as_str()))
        .collect();
    let at = |line: usize| format!("crates/demo/src/case.rs:{line}");
    let expected: Vec<(&str, String)> = vec![
        ("XTASK-R041", at(18)),
        ("XTASK-R041", at(19)),
        ("XTASK-R041", at(20)),
        ("XTASK-R041", at(21)),
        ("XTASK-R041", at(26)),
        ("XTASK-R043", at(33)),
        ("XTASK-R041", at(35)),
        ("XTASK-R041", at(39)),
        ("XTASK-R041", at(41)),
    ];
    let expected: Vec<(&str, &str)> = expected.iter().map(|(r, l)| (*r, l.as_str())).collect();
    assert_eq!(found, expected);
    // Line 35: a string literal not directly after `extern` (`pub "x" fn g`) leaves the
    // annotation unattached (M1-T004).
    // Attached annotations count: `a`, `b`, `c` and `h` verify DEMO-R001; `b` and `c` DEMO-R002.
    let tests = |id: &str| -> Vec<String> {
        result
            .rows
            .iter()
            .find(|row| row.id == id)
            .unwrap()
            .tests
            .iter()
            .map(|t| format!("{} {}", t.location, t.function))
            .collect()
    };
    assert_eq!(
        tests("DEMO-R002"),
        [
            "crates/demo/src/case.rs:13 b",
            "crates/demo/src/case.rs:16 c"
        ]
    );
    // `c` names DEMO-R001 twice and is listed once.
    assert_eq!(tests("DEMO-R001").len(), 6);
}

/// Verifies: XTASK-R042
#[test]
fn the_matrix_lists_tests_per_requirement_and_counts_per_crate() {
    let result = trace(&valid());
    assert_eq!(result.findings, Vec::new());
    let rows: Vec<(&str, &str, bool, Vec<String>)> = result
        .rows
        .iter()
        .map(|row| {
            (
                row.id.as_str(),
                row.level,
                row.withdrawn,
                row.tests
                    .iter()
                    .map(|t| format!("{} {}", t.location, t.function))
                    .collect(),
            )
        })
        .collect();
    let lib = "crates/demo/src/lib.rs";
    assert_eq!(
        rows,
        [
            (
                "DEMO-R001",
                "MUST",
                false,
                vec![format!("{lib}:8 first"), format!("{lib}:12 second")]
            ),
            ("DEMO-R002", "SHOULD", false, vec![]),
            ("DEMO-R003", "MUST", true, vec![]),
            ("DEMO-R004", "MAY", false, vec![format!("{lib}:12 second")]),
        ]
    );
    assert_eq!(result.rows[0].spec, "docs/specs/demo.md:17");
    assert_eq!(result.crates.len(), 1);
    let summary = &result.crates[0];
    assert_eq!(
        (
            summary.crate_name.as_str(),
            summary.maturity.as_deref(),
            summary.gated
        ),
        ("demo", Some("usable"), true)
    );
    let count = |tested, untested| Count { tested, untested };
    assert_eq!(
        summary.levels,
        BTreeMap::from([
            ("MUST", count(1, 0)),
            ("SHOULD", count(0, 1)),
            ("MAY", count(1, 0)),
        ])
    );
}

/// Verifies: XTASK-R044
#[test]
fn untested_musts_are_errors_from_usable_and_information_below() {
    let root = TempRoot::new("trace-r044-maturities");
    copy_dir(&fixtures().join("valid"), root.path());
    copy_dir(&fixtures().join("r044_untested_must"), root.path());
    let base = sources::load(root.path()).unwrap();
    let spec = "docs/specs/demo.md";
    for (maturity, gated) in [
        ("planned", false),
        ("experimental", false),
        ("usable", true),
        ("stable", true),
        ("optimized", true),
    ] {
        let mut model = base.clone();
        let text =
            model.repo.markdown[spec].replace("maturity: usable", &format!("maturity: {maturity}"));
        model.repo.markdown.insert(spec.to_string(), text);
        let result = trace(&model);
        let found: Vec<(&str, &str)> = result
            .findings
            .iter()
            .map(|(_, f)| (f.rule.as_str(), f.location.as_str()))
            .collect();
        if gated {
            assert_eq!(
                found,
                [("XTASK-R044", "docs/specs/demo.md:21")],
                "{maturity}"
            );
        } else {
            assert!(found.is_empty(), "{maturity}: {found:?}");
            // Information only: the matrix still shows the requirement as untested.
            let row = result.rows.iter().find(|r| r.id == "DEMO-R005").unwrap();
            assert_eq!(row.tests, Vec::new());
            assert_eq!(
                result.crates[0].levels["MUST"],
                Count {
                    tested: 1,
                    untested: 1
                }
            );
        }
    }
}

/// Verifies: XTASK-R043
#[test]
fn malformed_rows_are_known_ids_for_annotations() {
    // DEMO-R005 is a malformed row (R040). An annotation naming it is not also an R043 finding.
    let root = TempRoot::new("trace-r043-malformed-row");
    copy_dir(&fixtures().join("valid"), root.path());
    copy_dir(&fixtures().join("r040_bad_row"), root.path());
    let mut model = sources::load(root.path()).unwrap();
    let lib = model
        .sources
        .iter_mut()
        .find(|s| s.path == "crates/demo/src/lib.rs")
        .unwrap();
    lib.text = lib.text.replace("DEMO-R001, DEMO-R004", "DEMO-R005");
    let result = trace(&model);
    let rules: Vec<&str> = result
        .findings
        .iter()
        .map(|(_, f)| f.rule.as_str())
        .collect();
    assert_eq!(rules, ["XTASK-R040"]);
}

fn run(args: &[&str], root: &Path) -> (Result<Exit, CommandError>, String) {
    let args: Vec<String> = args.iter().map(ToString::to_string).collect();
    let mut out = Vec::new();
    let result = execute(&args, &mut out, root);
    (result, String::from_utf8(out).unwrap())
}

/// Verifies: XTASK-R042
#[test]
fn human_output_is_the_matrix_then_the_findings() {
    let (result, out) = run(&[], &fixtures().join("valid"));
    assert!(matches!(result, Ok(Exit::Success)));
    assert_eq!(
        out,
        "demo (docs/specs/demo.md, maturity usable): untested MUST requirements are errors \
         (XTASK-R044)\n  \
         DEMO-R001    MUST   crates/demo/src/lib.rs:8 first\n                      \
         crates/demo/src/lib.rs:12 second\n  \
         DEMO-R002    SHOULD untested\n  \
         DEMO-R003    MUST   withdrawn\n  \
         DEMO-R004    MAY    crates/demo/src/lib.rs:12 second\n  \
         tested: MUST 1/1, SHOULD 0/1, MAY 1/1\n\ntrace: ok\n"
    );
}

/// Verifies: XTASK-R009, XTASK-R042
#[test]
fn json_output_has_findings_requirements_and_crates() {
    let root = TempRoot::new("trace-json");
    copy_dir(&fixtures().join("valid"), root.path());
    copy_dir(&fixtures().join("r044_untested_must"), root.path());
    let (result, out) = run(&["--json"], root.path());
    assert!(matches!(result, Ok(Exit::Failure)));
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(value["version"], 1);
    assert_eq!(value["ok"], false);
    assert_eq!(value["findings"].as_array().unwrap().len(), 1);
    assert_eq!(value["findings"][0]["rule"], "XTASK-R044");
    assert_eq!(value["findings"][0]["severity"], "error");
    assert_eq!(
        value["requirements"][0],
        serde_json::json!({
            "id": "DEMO-R001", "level": "MUST", "crate": "demo",
            "spec": "docs/specs/demo.md:17", "withdrawn": false,
            "tests": [
                { "location": "crates/demo/src/lib.rs:8", "function": "first" },
                { "location": "crates/demo/src/lib.rs:12", "function": "second" }
            ]
        })
    );
    assert_eq!(
        value["crates"][0],
        serde_json::json!({
            "crate": "demo", "spec": "docs/specs/demo.md", "maturity": "usable", "gated": true,
            "levels": {
                "MAY": { "tested": 1, "untested": 0 },
                "MUST": { "tested": 1, "untested": 1 },
                "SHOULD": { "tested": 0, "untested": 1 }
            }
        })
    );
}

#[test]
fn crate_option_limits_the_report_to_one_spec() {
    let root = TempRoot::new("trace-crate-option");
    copy_dir(&fixtures().join("valid"), root.path());
    copy_dir(&fixtures().join("r044_untested_must"), root.path());
    let other = std::fs::read_to_string(root.path().join("docs/specs/demo.md"))
        .unwrap()
        .replace("crate: demo", "crate: other")
        .replace("DEMO", "OTHER");
    std::fs::write(root.path().join("docs/specs/other.md"), other).unwrap();
    // demo has one untested MUST at `usable` (R005); other has two (R001 and R005: no test
    // names OTHER IDs). `--crate demo` reports only demo's.
    let (_, all) = run(&["--json"], root.path());
    let all: serde_json::Value = serde_json::from_str(&all).unwrap();
    assert_eq!(all["findings"].as_array().unwrap().len(), 3);
    for args in [
        &["--json", "--crate", "demo"][..],
        &["--crate=demo", "--json"][..],
    ] {
        let (result, out) = run(args, root.path());
        assert!(matches!(result, Ok(Exit::Failure)));
        let value: serde_json::Value = serde_json::from_str(&out).unwrap();
        let locations: Vec<&str> = value["findings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["location"].as_str().unwrap())
            .collect();
        assert_eq!(locations, ["docs/specs/demo.md:21"]);
        assert!(
            value["requirements"]
                .as_array()
                .unwrap()
                .iter()
                .all(|r| r["crate"] == "demo")
        );
        assert_eq!(value["crates"].as_array().unwrap().len(), 1);
    }
    for args in [
        &["--crate"][..],
        &["--crate", "nope"][..],
        &["--crate", "demo", "--crate", "other"][..],
        &["--crate=demo", "--crate=other"][..],
        &["--verbose"][..],
    ] {
        assert!(
            matches!(run(args, root.path()).0, Err(CommandError::Usage(_))),
            "{args:?}"
        );
    }
}

#[test]
fn the_real_repository_passes() {
    let findings = run_check(&repo::root()).unwrap();
    assert!(findings.is_empty(), "{}", check::finding_lines(&findings));
}
