//! Unit tests of the `hygiene` scanner. Inputs that contain forbidden patterns are files under
//! `xtask/tests/fixtures/hygiene/cases/` (M0-T005 notes: never inline string literals), read with
//! `include_str!`.

use super::*;

/// The task statuses of the hygiene `valid` fixture.
fn tasks() -> BTreeMap<String, Option<String>> {
    BTreeMap::from([
        ("M0-T001".to_string(), Some("done".to_string())),
        ("M0-T003".to_string(), None),
        ("M0-T005".to_string(), Some("in-progress".to_string())),
    ])
}

/// `(rule, line)` of every finding of scanning `text` as `case.rs`, sorted.
fn scan_case(text: &str) -> Vec<(String, usize)> {
    let mut findings = Vec::new();
    scan("case.rs", text, &tasks(), &mut findings);
    check::sort(&mut findings);
    findings
        .iter()
        .map(|f| {
            let line = f
                .location
                .strip_prefix("case.rs:")
                .unwrap()
                .parse()
                .unwrap();
            (f.rule.clone(), line)
        })
        .collect()
}

fn expect(pairs: &[(&str, usize)]) -> Vec<(String, usize)> {
    pairs
        .iter()
        .map(|(rule, line)| ((*rule).to_string(), *line))
        .collect()
}

/// Verifies: XTASK-R050
#[test]
fn forbidden_macros_in_code_in_every_delimiter_and_spacing() {
    let found = scan_case(include_str!("../../tests/fixtures/hygiene/cases/macros.rs"));
    let r050 = "XTASK-R050";
    assert_eq!(
        found,
        expect(&[
            (r050, 2),
            (r050, 3),
            (r050, 4),
            (r050, 5),
            (r050, 6),
            (r050, 7),
            (r050, 13)
        ])
    );
}

/// Verifies: XTASK-R050, XTASK-R051
#[test]
fn task_markers_in_comments_need_an_existing_open_task() {
    let found = scan_case(include_str!(
        "../../tests/fixtures/hygiene/cases/markers.rs"
    ));
    let (r050, r051) = ("XTASK-R050", "XTASK-R051");
    assert_eq!(
        found,
        expect(&[
            (r050, 2),
            (r050, 3),
            (r050, 4),
            (r050, 5),
            (r051, 7),
            (r050, 9),
            (r050, 10),
            (r050, 11),
            (r051, 12),
            (r050, 17),
        ])
    );
}

/// Verifies: XTASK-R050, XTASK-R051
#[test]
fn ignore_attributes_need_a_reason_naming_an_existing_task() {
    let found = scan_case(include_str!(
        "../../tests/fixtures/hygiene/cases/ignores.rs"
    ));
    let (r050, r051) = ("XTASK-R050", "XTASK-R051");
    assert_eq!(
        found,
        expect(&[
            (r050, 2),
            (r050, 4),
            (r051, 7),
            (r050, 8),
            (r050, 11),
            (r050, 18),
            (r050, 19),
        ])
    );
}

/// Verifies: XTASK-R051
#[test]
fn messages_say_which_task_is_missing_or_done() {
    let mut findings = Vec::new();
    let text = include_str!("../../tests/fixtures/hygiene/cases/markers.rs");
    scan("case.rs", text, &tasks(), &mut findings);
    let r051: Vec<&str> = findings
        .iter()
        .filter(|f| f.rule == "XTASK-R051")
        .map(|f| f.message.as_str())
        .collect();
    assert_eq!(
        r051,
        [
            "`TODO(M0-T001)` names a task that is `done`: the comment is stale",
            "`TODO(M9-T999)` names a task that does not exist in docs/tasks/",
        ]
    );
}

#[test]
fn task_ids_are_whole_words_with_three_digit_numbers() {
    assert_eq!(
        task_ids("M0-T005: x, M12-T100 and (M1-T002)"),
        ["M0-T005", "M12-T100", "M1-T002"]
    );
    for text in [
        "M0-T05", "M0-T0050", "XM0-T005", "M-T005", "M0T005", "m0-t005", "M0-T005a",
    ] {
        assert_eq!(task_ids(text), Vec::<String>::new(), "{text}");
    }
    assert_eq!(tracked_task("(M0-T005): x"), Some("M0-T005".to_string()));
    for rest in [
        "(M0-T005 ): x",
        "( M0-T005)",
        "(M0-T005",
        ": x",
        "(M0-T005, M0-T006)",
    ] {
        assert_eq!(tracked_task(rest), None, "{rest}");
    }
}

#[test]
fn task_statuses_come_from_task_file_names_and_frontmatter() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hygiene/valid");
    let model = crate::repo::load(&root).unwrap();
    assert_eq!(task_statuses(&model), tasks());
}
