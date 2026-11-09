//! Tests of `status` (spec §10: "snapshot test of generated STATUS.md for a fixture"). The fixture
//! is `xtask/tests/fixtures/status/valid/`; the approved output is `xtask/tests/snapshots/STATUS.md`.

use std::path::{Path, PathBuf};

use super::*;
use crate::test_support::{TempRoot, copy_dir};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/status/valid")
}

const SNAPSHOT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snapshots/STATUS.md");

fn snapshot() -> String {
    std::fs::read_to_string(SNAPSHOT_PATH).unwrap()
}

/// A temporary copy of the fixture.
fn temp_fixture(name: &str) -> TempRoot {
    let root = TempRoot::new(&format!("status-{name}"));
    copy_dir(&fixture(), root.path());
    root
}

/// Runs `status` with `args` on `root`; returns the result and the output.
fn run(root: &Path, args: &[&str]) -> (Result<Exit, CommandError>, String) {
    let args: Vec<String> = args.iter().map(ToString::to_string).collect();
    let mut out = Vec::new();
    let result = execute(&args, &mut out, root);
    (result, String::from_utf8(out).unwrap())
}

fn status_file(root: &Path) -> Option<String> {
    std::fs::read_to_string(root.join(PATH)).ok()
}

/// Replaces `from` with `to` in the repository file `path` (which must contain `from`).
fn edit(root: &Path, path: &str, from: &str, to: &str) {
    let full = root.join(path);
    let text = std::fs::read_to_string(&full).unwrap();
    assert!(text.contains(from), "{path} does not contain {from:?}");
    std::fs::write(full, text.replacen(from, to, 1)).unwrap();
}

/// Verifies: XTASK-R070, XTASK-R071
#[test]
fn fixture_output_matches_the_approved_snapshot() {
    let model = repo::load(&fixture()).unwrap();
    let generated = generate(&model).unwrap();
    assert_eq!(
        generated,
        snapshot(),
        "output differs from {SNAPSHOT_PATH}; update the snapshot only for an intended format \
         change, and say why in the journal (AGENTS.md rule 5)"
    );
}

/// Verifies: XTASK-R071
#[test]
fn running_status_twice_gives_identical_files_and_ignores_the_old_file() {
    let root = temp_fixture("twice");
    // Whatever the file held before has no influence on the output.
    std::fs::write(root.path().join(PATH), "hand-edited\n").unwrap();
    let (result, out) = run(root.path(), &[]);
    assert!(matches!(result, Ok(Exit::Success)));
    assert_eq!(out, "status: wrote docs/STATUS.md\n");
    let first = status_file(root.path()).unwrap();
    assert_eq!(first, snapshot());

    let (result, out) = run(root.path(), &[]);
    assert!(matches!(result, Ok(Exit::Success)));
    assert_eq!(out, "status: docs/STATUS.md is up to date\n");
    assert_eq!(status_file(root.path()).unwrap(), first);
}

/// Verifies: XTASK-R072, XTASK-R010
#[test]
fn check_fails_after_a_task_status_is_edited_without_regenerating() {
    let root = temp_fixture("check-edit");
    assert!(matches!(run(root.path(), &[]).0, Ok(Exit::Success)));
    let (result, out) = run(root.path(), &["--check"]);
    assert!(matches!(result, Ok(Exit::Success)));
    assert_eq!(out, "status: ok\n");

    edit(
        root.path(),
        "docs/tasks/M2-T002-second.md",
        "status: in-progress",
        "status: review",
    );
    let before = status_file(root.path()).unwrap();
    let (result, out) = run(root.path(), &["--check"]);
    assert!(matches!(result, Ok(Exit::Failure)));
    // Line 17 is the M2-T002 row of the current-milestone table.
    assert!(
        out.starts_with("error XTASK-R072 docs/STATUS.md:17: "),
        "{out}"
    );
    assert!(
        out.contains("help: Run cargo xtask status and commit docs/STATUS.md"),
        "{out}"
    );
    assert!(out.ends_with("status: 1 finding\n"), "{out}");
    assert_eq!(status_file(root.path()).unwrap(), before, "--check wrote");

    assert!(matches!(run(root.path(), &[]).0, Ok(Exit::Success)));
    assert_eq!(run(root.path(), &["--check"]).1, "status: ok\n");
}

/// Verifies: XTASK-R072
#[test]
fn check_fails_when_the_file_is_missing_or_out_of_date() {
    let root = temp_fixture("check-missing");
    let findings = run_check(root.path()).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule, "XTASK-R072");
    assert_eq!(findings[0].location, "docs/STATUS.md");
    assert!(status_file(root.path()).is_none(), "--check wrote");

    // Only the final newline is missing.
    let mut text = snapshot();
    text.pop();
    std::fs::write(root.path().join(PATH), &text).unwrap();
    let findings = run_check(root.path()).unwrap();
    let last_line = snapshot().lines().count();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].location, format!("docs/STATUS.md:{last_line}"));

    std::fs::write(root.path().join(PATH), snapshot()).unwrap();
    assert_eq!(run_check(root.path()).unwrap(), Vec::new());
}

#[test]
fn first_difference_is_the_first_line_that_differs_including_its_line_ending() {
    assert_eq!(first_difference("a\nb\n", "a\nb\n"), None);
    assert_eq!(first_difference("a\nb\n", "a\nc\n"), Some(2));
    assert_eq!(first_difference("a\nb", "a\nb\n"), Some(2));
    assert_eq!(first_difference("a\n", "a\nb\n"), Some(2));
    assert_eq!(first_difference("a\nb\n", "a\n"), Some(2));
    assert_eq!(first_difference("", "a\n"), Some(1));
    assert_eq!(first_difference("a\r\n", "a\n"), Some(1));
}

/// Verifies: XTASK-R070
#[test]
fn unreadable_task_or_spec_input_is_reported_and_nothing_is_written() {
    let cases: &[(&str, &str, &str, &str, &str)] = &[
        (
            "status-value",
            "docs/tasks/M2-T002-second.md",
            "status: in-progress",
            "status: wip",
            "docs/tasks/M2-T002-second.md:5",
        ),
        (
            "syntax",
            "docs/tasks/M2-T003-third.md",
            "status: review",
            "status:review",
            "docs/tasks/M2-T003-third.md:5",
        ),
        (
            "missing-title",
            "docs/tasks/M10-T002-later.md",
            "title: Later\n",
            "",
            "docs/tasks/M10-T002-later.md",
        ),
        (
            "milestone-mismatch",
            "docs/tasks/M10-T002-later.md",
            "milestone: M10",
            "milestone: M2",
            "docs/tasks/M10-T002-later.md:4",
        ),
        (
            "depends-not-list",
            "docs/tasks/M2-T002-second.md",
            "depends_on: [M2-T001]",
            "depends_on: M2-T001",
            "docs/tasks/M2-T002-second.md:7",
        ),
        (
            "duplicate-id",
            "docs/tasks/M2-T003-third.md",
            "id: M2-T003",
            "id: M2-T002",
            // `M2-T002-second.md` sorts first, so the later file is reported.
            "docs/tasks/M2-T003-third.md:2",
        ),
        (
            "spec-maturity",
            "docs/specs/demo.md",
            "maturity: experimental",
            "maturity: beta",
            "docs/specs/demo.md:5",
        ),
        (
            "spec-crate-missing",
            "docs/specs/alpha_lib.md",
            "crate: alpha_lib\n",
            "",
            "docs/specs/alpha_lib.md",
        ),
    ];
    for (name, path, from, to, location) in cases {
        let root = temp_fixture(&format!("unreadable-{name}"));
        edit(root.path(), path, from, to);
        let (result, out) = run(root.path(), &[]);
        assert!(matches!(result, Ok(Exit::Failure)), "{name}: {out}");
        assert!(
            out.starts_with(&format!("error XTASK-R070 {location}: ")),
            "{name}: {out}"
        );
        assert!(out.ends_with("status: 1 finding\n"), "{name}: {out}");
        assert!(status_file(root.path()).is_none(), "{name}: wrote a file");

        let findings = run_check(root.path()).unwrap();
        let rules: Vec<&str> = findings.iter().map(|f| f.rule.as_str()).collect();
        assert_eq!(rules, ["XTASK-R070"], "{name}");
    }
}

/// Verifies: XTASK-R070
#[test]
fn without_open_tasks_there_is_no_current_milestone_table() {
    let root = temp_fixture("all-done");
    for (path, from) in [
        ("docs/tasks/M2-T002-second.md", "status: in-progress"),
        ("docs/tasks/M2-T003-third.md", "status: review"),
        ("docs/tasks/M2-T010-tenth.md", "status: todo"),
        ("docs/tasks/M10-T001-stuck.md", "status: blocked"),
        ("docs/tasks/M10-T002-later.md", "status: todo"),
    ] {
        edit(root.path(), path, from, "status: done");
    }
    let text = generate(&repo::load(root.path()).unwrap()).unwrap();
    assert!(
        text.contains(
            "## Current milestone\n\nNo milestone has open tasks.\n\n## Blocked tasks\n\nNone.\n\n"
        ),
        "{text}"
    );
    assert!(text.contains("| M10 | 2 | 0 | 0 | 0 | 0 | 2 |\n"), "{text}");
}

/// Verifies: XTASK-R070
#[test]
fn a_milestone_heading_without_the_dash_form_gives_the_id_alone() {
    assert_eq!(
        milestone_names("## M1 — One\n## M2 Two\n##  M3 — x\n## M4 —\n## Notes — x\n## M1 — Dup\n"),
        BTreeMap::from([(1, "One".to_string())])
    );
}

#[test]
fn the_real_repository_has_readable_status_input() {
    let model = repo::load(&repo::root()).unwrap();
    if let Err(findings) = generate(&model) {
        panic!("{}", check::finding_lines(&findings));
    }
}

#[test]
fn unknown_arguments_are_usage_errors() {
    let root = temp_fixture("usage");
    for args in [&["--json"][..], &["extra"][..]] {
        assert!(matches!(
            run(root.path(), args).0,
            Err(CommandError::Usage(_))
        ));
    }
    assert!(status_file(root.path()).is_none());
}
