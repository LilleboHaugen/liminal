//! Unit tests for each sub-rule of R020–R027. Each test loads the `valid` fixture once, edits the
//! model in memory, and runs the pure [`check`].

use std::path::Path;

use super::*;

fn valid() -> RepoModel {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/check_docs/valid");
    repo::load(&root).unwrap()
}

/// Sets `path` to `text` in the model (adding it and its parent directories to `paths`).
fn put(model: &mut RepoModel, path: &str, text: &str) {
    let mut prefix = String::new();
    for part in path.split('/') {
        if !prefix.is_empty() {
            prefix.push('/');
        }
        prefix.push_str(part);
        model.paths.insert(prefix.clone());
    }
    model.markdown.insert(path.to_string(), text.to_string());
}

/// Replaces `from` with `to` in `path`, which must contain `from`.
fn edit(model: &mut RepoModel, path: &str, from: &str, to: &str) {
    let text = &model.markdown[path];
    assert!(text.contains(from), "{path} does not contain {from:?}");
    let text = text.replace(from, to);
    model.markdown.insert(path.to_string(), text);
}

fn remove(model: &mut RepoModel, path: &str) {
    model.markdown.remove(path);
    model.paths.remove(path);
}

/// `(rule, location, message)` of every finding.
fn findings_of(model: &RepoModel) -> Vec<(String, String, String)> {
    check(model)
        .into_iter()
        .map(|f| (f.rule, f.location, f.message))
        .collect()
}

/// Asserts exactly one finding, of `rule` at `location`, whose message contains `message`.
fn assert_one(model: &RepoModel, rule: &str, location: &str, message: &str) {
    let found = findings_of(model);
    assert_eq!(found.len(), 1, "{found:#?}");
    let (r, l, m) = &found[0];
    assert_eq!((r.as_str(), l.as_str()), (rule, location), "{m}");
    assert!(m.contains(message), "message: {m}");
}

const ADR1: &str = "docs/adr/0001-first.md";
const ADR2: &str = "docs/adr/0002-second.md";
const SPEC: &str = "docs/specs/demo.md";
const T1: &str = "docs/tasks/M0-T001-first.md";
const T2: &str = "docs/tasks/M0-T002-second.md";

// ---------------------------------------------------------------- R020

/// Verifies: XTASK-R020
#[test]
fn r020_missing_or_broken_frontmatter_in_each_kind_of_file() {
    for path in [ADR2, SPEC, T2] {
        let mut model = valid();
        let text = model.markdown[path].replacen("---\n", "", 1);
        put(&mut model, path, &text);
        let found = findings_of(&model);
        assert!(
            found.iter().any(|(rule, location, _)| rule == "XTASK-R020" && location == &format!("{path}:1")),
            "{path}: {found:#?}"
        );
    }
}

/// Verifies: XTASK-R020
#[test]
fn r020_templates_and_non_document_files_are_not_parsed() {
    let mut model = valid();
    put(&mut model, "docs/adr/notes.md", "no frontmatter");
    put(&mut model, "docs/tasks/README.md", "no frontmatter");
    put(&mut model, "docs/specs/sub/deep.md", "no frontmatter");
    put(&mut model, "docs/process/GUIDE.md", "no frontmatter");
    assert_eq!(findings_of(&model), []);
}

// ---------------------------------------------------------------- R021

/// Verifies: XTASK-R021
#[test]
fn r021_missing_unknown_and_list_valued_fields() {
    let mut model = valid();
    edit(&mut model, ADR2, "date: 2026-11-08\n", "");
    assert_one(
        &model,
        "XTASK-R021",
        ADR2,
        "required field `date` is missing",
    );

    let mut model = valid();
    edit(
        &mut model,
        ADR2,
        "superseded_by: []\n",
        "superseded_by: []\nsuperseeded_by: []\n",
    );
    assert_one(
        &model,
        "XTASK-R021",
        &format!("{ADR2}:8"),
        "unknown field `superseeded_by`",
    );

    let mut model = valid();
    edit(
        &mut model,
        ADR2,
        "title: Second decision",
        "title: [Second]",
    );
    assert_one(
        &model,
        "XTASK-R021",
        &format!("{ADR2}:3"),
        "`title` must be a single value",
    );

    let mut model = valid();
    edit(
        &mut model,
        ADR2,
        "supersedes: [ADR-0001]",
        "supersedes: ADR-0001",
    );
    let found = findings_of(&model);
    assert!(
        found
            .iter()
            .any(|(_, _, m)| m.contains("`supersedes` must be a list")),
        "{found:#?}"
    );
}

/// Verifies: XTASK-R021
#[test]
fn r021_id_must_match_the_file_number() {
    let mut model = valid();
    edit(&mut model, ADR2, "id: ADR-0002", "id: ADR-0003");
    assert_one(
        &model,
        "XTASK-R021",
        &format!("{ADR2}:2"),
        "the file number says `ADR-0002`",
    );
}

/// Verifies: XTASK-R021
#[test]
fn r021_status_and_date_values() {
    let mut model = valid();
    edit(&mut model, ADR2, "date: 2026-11-08", "date: 8 Nov 2026");
    assert_one(
        &model,
        "XTASK-R021",
        &format!("{ADR2}:5"),
        "`date` is `8 Nov 2026`",
    );
}

/// Verifies: XTASK-R021
#[test]
fn r021_supersession_references_must_exist_and_be_symmetric() {
    let mut model = valid();
    edit(
        &mut model,
        ADR2,
        "supersedes: [ADR-0001]",
        "supersedes: [ADR-0001, ADR-0009]",
    );
    assert_one(
        &model,
        "XTASK-R021",
        &format!("{ADR2}:6"),
        "`ADR-0009`, which is not an existing ADR",
    );

    // ADR-0002 no longer claims to supersede ADR-0001, which still says it is superseded by it.
    let mut model = valid();
    edit(&mut model, ADR2, "supersedes: [ADR-0001]", "supersedes: []");
    assert_one(
        &model,
        "XTASK-R021",
        &format!("{ADR1}:7"),
        "ADR-0002 does not list ADR-0001 in `supersedes`",
    );

    // ADR-0001 no longer names its successor: asymmetric, and superseded without successor.
    let mut model = valid();
    edit(
        &mut model,
        ADR1,
        "superseded_by: [ADR-0002]",
        "superseded_by: []",
    );
    let found = findings_of(&model);
    let messages: Vec<&str> = found.iter().map(|(_, _, m)| m.as_str()).collect();
    assert_eq!(found.len(), 3, "{found:#?}");
    assert!(messages.contains(&"ADR-0001 has status superseded but an empty `superseded_by`"));
    assert!(messages.contains(
        &"ADR-0002 supersedes ADR-0001, but ADR-0001 does not list ADR-0002 in `superseded_by`"
    ));
    // The index row still says "superseded by ADR-0002"; the index now expects "superseded".
    assert!(found.iter().any(|(rule, _, _)| rule == "XTASK-R025"));
}

/// Verifies: XTASK-R021
#[test]
fn r021_superseded_by_requires_status_superseded() {
    let mut model = valid();
    edit(&mut model, ADR1, "status: superseded", "status: accepted");
    let found = findings_of(&model);
    assert!(
        found.contains(&(
            "XTASK-R021".to_string(),
            format!("{ADR1}:4"),
            "ADR-0001 lists `superseded_by` but its status is not superseded".to_string()
        )),
        "{found:#?}"
    );
}

// ---------------------------------------------------------------- R022

/// Verifies: XTASK-R022
#[test]
fn r022_crate_must_be_registered_and_match_the_file_name() {
    let mut model = valid();
    edit(&mut model, SPEC, "crate: demo", "crate: unknown_crate");
    let found = findings_of(&model);
    let messages: Vec<&str> = found.iter().map(|(_, _, m)| m.as_str()).collect();
    assert_eq!(
        messages,
        [
            "`crate` is `unknown_crate`, but the file is demo.md",
            "crate `unknown_crate` is not registered in architecture.toml"
        ]
    );
}

/// Verifies: XTASK-R022
#[test]
fn r022_value_forms() {
    let cases = [
        ("layer: 5", "layer: five", 4, "`layer` is `five`"),
        (
            "maturity: planned",
            "maturity: beta",
            5,
            "`maturity` is `beta`",
        ),
        (
            "spec_version: 1",
            "spec_version: 0",
            6,
            "`spec_version` is `0`",
        ),
        (
            "last_reviewed: 2026-11-08",
            "last_reviewed: soon",
            7,
            "`last_reviewed` is `soon`",
        ),
        (
            "req_prefix: DEMO",
            "req_prefix: Demo",
            3,
            "`req_prefix` is `Demo`",
        ),
        (
            "adrs: [ADR-0002]",
            "adrs: [ADR-0042]",
            8,
            "`ADR-0042`, which is not an existing ADR",
        ),
    ];
    for (from, to, line, message) in cases {
        let mut model = valid();
        edit(&mut model, SPEC, from, to);
        let found = findings_of(&model);
        let r022: Vec<_> = found
            .iter()
            .filter(|(rule, _, _)| rule == "XTASK-R022")
            .collect();
        assert_eq!(r022.len(), 1, "{to}: {found:#?}");
        assert_eq!(r022[0].1, format!("{SPEC}:{line}"), "{to}");
        assert!(r022[0].2.contains(message), "{to}: {}", r022[0].2);
    }
}

/// Verifies: XTASK-R022
#[test]
fn r022_req_prefix_is_unique_across_specs() {
    let mut model = valid();
    let text = model.markdown[SPEC]
        .replace("crate: demo", "crate: planned_lib")
        .replace("layer: 5", "layer: 1");
    put(&mut model, "docs/specs/planned_lib.md", &text);
    edit(
        &mut model,
        "docs/specs/README.md",
        "| `planned` |\n",
        "| `planned` |\n| [planned_lib.md](planned_lib.md) | `planned_lib` | DEMO | planned |\n",
    );
    assert_one(
        &model,
        "XTASK-R022",
        "docs/specs/planned_lib.md:3",
        "`req_prefix` DEMO is already used by docs/specs/demo.md",
    );
}

// ---------------------------------------------------------------- R023

/// Verifies: XTASK-R023
#[test]
fn r023_id_form_file_name_and_uniqueness() {
    let mut model = valid();
    edit(&mut model, T2, "id: M0-T002", "id: M0-T2");
    assert_one(
        &model,
        "XTASK-R023",
        &format!("{T2}:2"),
        "`id` is `M0-T2`; expected a task ID",
    );

    let mut model = valid();
    edit(&mut model, T2, "id: M0-T002", "id: M0-T003");
    assert_one(
        &model,
        "XTASK-R023",
        &format!("{T2}:2"),
        "the file name M0-T002-second.md does not start with it",
    );

    let mut model = valid();
    let text = model.markdown[T2].clone();
    put(&mut model, "docs/tasks/M0-T002-copy.md", &text);
    assert_one(
        &model,
        "XTASK-R023",
        "docs/tasks/M0-T002-second.md:2",
        "task ID M0-T002 is already used by docs/tasks/M0-T002-copy.md",
    );
}

/// Verifies: XTASK-R023
#[test]
fn r023_milestone_status_and_crates() {
    let cases = [
        (
            "milestone: M0",
            "milestone: M1",
            4,
            "the task ID M0-T002 is in another milestone",
        ),
        (
            "milestone: M0",
            "milestone: zero",
            4,
            "`milestone` is `zero`",
        ),
        (
            "status: todo",
            "status: started",
            5,
            "`status` is `started`",
        ),
        (
            "crates: [demo, planned_lib]",
            "crates: [demo, liminal_nope]",
            6,
            "`liminal_nope`, which is not registered",
        ),
    ];
    for (from, to, line, message) in cases {
        let mut model = valid();
        edit(&mut model, T2, from, to);
        assert_one(&model, "XTASK-R023", &format!("{T2}:{line}"), message);
    }
}

/// Verifies: XTASK-R023
#[test]
fn r023_dependencies_must_exist_and_have_no_cycles() {
    let mut model = valid();
    edit(
        &mut model,
        T2,
        "depends_on: [M0-T001]",
        "depends_on: [M0-T001, M0-T099]",
    );
    assert_one(
        &model,
        "XTASK-R023",
        &format!("{T2}:7"),
        "M0-T002 depends on M0-T099, which is not an existing task",
    );

    let mut model = valid();
    edit(&mut model, T1, "depends_on: []", "depends_on: [M0-T001]");
    assert_one(
        &model,
        "XTASK-R023",
        &format!("{T1}:7"),
        "dependency cycle: M0-T001 → M0-T001",
    );

    // A three-task cycle is reported once, starting at its smallest ID.
    let mut model = valid();
    let text = model.markdown[T2]
        .replace("M0-T002", "M0-T003")
        .replace("depends_on: [M0-T001]", "depends_on: [M0-T002]");
    put(&mut model, "docs/tasks/M0-T003-third.md", &text);
    edit(&mut model, T1, "depends_on: []", "depends_on: [M0-T003]");
    assert_one(
        &model,
        "XTASK-R023",
        &format!("{T1}:7"),
        "dependency cycle: M0-T001 → M0-T003 → M0-T002 → M0-T001",
    );
}

/// Verifies: XTASK-R023
#[test]
fn r023_a_task_without_a_valid_id_still_exists_by_its_file_name() {
    // Broken frontmatter in M0-T001: only the R020 finding, no "M0-T002 depends on M0-T001, which
    // is not an existing task".
    let mut model = valid();
    edit(&mut model, T1, "status: done", "status:done");
    let found = findings_of(&model);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].0, "XTASK-R020");

    // Readable frontmatter with an invalid `id`: only the finding on that field.
    let mut model = valid();
    edit(&mut model, T1, "id: M0-T001", "id: M0-T1");
    assert_one(&model, "XTASK-R023", &format!("{T1}:2"), "`id` is `M0-T1`");

    // A file name without a task ID prefix gives no ID: the dependency is reported.
    let mut model = valid();
    let text = model.markdown[T1].replace("status: done", "status:done");
    remove(&mut model, T1);
    put(&mut model, "docs/tasks/M0-T1-first.md", &text);
    // The rename also breaks M0-T002's link to the file (R024), which is not under test here.
    let found: Vec<_> = findings_of(&model)
        .into_iter()
        .filter(|f| f.0 != "XTASK-R024")
        .collect();
    let rules: Vec<&str> = found.iter().map(|f| f.0.as_str()).collect();
    // Sorted by path: M0-T002-second.md before M0-T1-first.md.
    assert_eq!(rules, ["XTASK-R023", "XTASK-R020"], "{found:#?}");
    assert!(
        found[0]
            .2
            .contains("depends on M0-T001, which is not an existing task")
    );
}

/// Verifies: XTASK-R023
#[test]
fn r023_a_task_with_a_duplicate_id_does_not_exist_by_its_file_name() {
    // M0-T003-third.md declares `id: M0-T001`, which M0-T001-first.md already has. The file is
    // known by its `id`, so it does not also exist as M0-T003: depending on M0-T003 is reported.
    let mut model = valid();
    let text = model.markdown[T1].clone();
    put(&mut model, "docs/tasks/M0-T003-third.md", &text);
    edit(
        &mut model,
        T2,
        "depends_on: [M0-T001]",
        "depends_on: [M0-T001, M0-T003]",
    );
    let found = findings_of(&model);
    assert!(
        found
            .iter()
            .any(|(rule, location, message)| rule == "XTASK-R023"
                && location == &format!("{T2}:7")
                && message.contains("depends on M0-T003, which is not an existing task")),
        "{found:#?}"
    );
    assert!(
        found
            .iter()
            .any(|(_, _, message)| message.contains("task ID M0-T001 is already used")),
        "{found:#?}"
    );
}

// ---------------------------------------------------------------- R024

/// Verifies: XTASK-R024
#[test]
fn r024_links_resolve_relative_to_the_file_or_the_root() {
    let mut model = valid();
    put(
        &mut model,
        "docs/process/GUIDE.md",
        "[ok](../adr/README.md) [ok](/README.md#top) [ok](../../tools/demo/) [ok](..)\n\
         [bad](README.md)\n[bad](../../../outside.md)\n[bad]: missing-ref.md\n",
    );
    let found = findings_of(&model);
    let summary: Vec<(&str, &str)> = found
        .iter()
        .map(|(_, l, m)| (l.as_str(), m.as_str()))
        .collect();
    assert_eq!(
        summary,
        [
            (
                "docs/process/GUIDE.md:2",
                "link target `README.md` does not exist (resolved to `docs/process/README.md`)"
            ),
            (
                "docs/process/GUIDE.md:3",
                "link target `../../../outside.md` leaves the repository"
            ),
            (
                "docs/process/GUIDE.md:4",
                "link target `missing-ref.md` does not exist (resolved to `docs/process/missing-ref.md`)"
            ),
        ]
    );
}

/// Verifies: XTASK-R024
#[test]
fn r024_only_root_and_docs_markdown_is_checked() {
    let mut model = valid();
    // Not in the model's markdown: files outside the root and docs/ are not loaded.
    assert!(!model.markdown.keys().any(|path| path.starts_with("tools/")));
    put(&mut model, "docs/deep/er/x.md", "[bad](nope.md)");
    assert_one(
        &model,
        "XTASK-R024",
        "docs/deep/er/x.md:1",
        "`nope.md` does not exist",
    );
}

#[test]
fn resolve_and_scheme_rules() {
    assert_eq!(
        resolve("docs/a/b.md", "c.md#x"),
        Some("docs/a/c.md".to_string())
    );
    assert_eq!(
        resolve("docs/a/b.md", "./c.md?q=1"),
        Some("docs/a/c.md".to_string())
    );
    assert_eq!(resolve("docs/a/b.md", "../../x"), Some("x".to_string()));
    assert_eq!(resolve("docs/a/b.md", "../../.."), None);
    assert_eq!(resolve("README.md", "/docs/"), Some("docs".to_string()));
    assert_eq!(resolve("README.md", "."), Some(String::new()));
    for target in ["https://x", "mailto:a@b", "ftp+x.y-z:abc", "C:/x"] {
        assert!(has_scheme(target), "{target}");
    }
    for target in ["a.md", "a.md#x:y", "1http://x", "dir/a:b"] {
        assert!(!has_scheme(target), "{target}");
    }
}

// ---------------------------------------------------------------- R025

/// Verifies: XTASK-R025
#[test]
fn r025_status_must_match_including_superseded_by() {
    let mut model = valid();
    edit(
        &mut model,
        "docs/adr/README.md",
        "| Second decision | accepted |",
        "| Second decision | proposed |",
    );
    assert_one(
        &model,
        "XTASK-R025",
        "docs/adr/README.md:8",
        "listed with status `proposed`, but its frontmatter says `accepted`",
    );

    let mut model = valid();
    edit(
        &mut model,
        "docs/adr/README.md",
        "superseded by ADR-0002",
        "superseded",
    );
    assert_one(
        &model,
        "XTASK-R025",
        "docs/adr/README.md:7",
        "its frontmatter says `superseded by ADR-0002`",
    );
}

/// Verifies: XTASK-R025
#[test]
fn r025_adr_status_cell_must_not_be_in_backticks() {
    let mut model = valid();
    edit(
        &mut model,
        "docs/adr/README.md",
        "| Second decision | accepted |",
        "| Second decision | `accepted` |",
    );
    assert_one(
        &model,
        "XTASK-R025",
        "docs/adr/README.md:8",
        "listed with status ``accepted``",
    );
}

/// Verifies: XTASK-R025
#[test]
fn r025_duplicate_foreign_and_missing_rows() {
    let row = "| [0002](0002-second.md) | Second decision | accepted |\n";
    let mut model = valid();
    edit(
        &mut model,
        "docs/adr/README.md",
        row,
        &format!("{row}{row}"),
    );
    assert_one(
        &model,
        "XTASK-R025",
        "docs/adr/README.md:9",
        "ADR-0002 is listed 2 times (lines 8, 9)",
    );

    let mut model = valid();
    edit(
        &mut model,
        "docs/adr/README.md",
        row,
        &format!("{row}| [README](README.md) | x | accepted |\n| no link | y | accepted |\n"),
    );
    let found = findings_of(&model);
    let locations: Vec<&str> = found.iter().map(|(_, l, _)| l.as_str()).collect();
    assert_eq!(
        locations,
        ["docs/adr/README.md:9", "docs/adr/README.md:10"],
        "{found:#?}"
    );

    let mut model = valid();
    edit(
        &mut model,
        "docs/adr/README.md",
        "| ADR | Title | Status |",
        "| Number | Title | Status |",
    );
    assert_one(&model, "XTASK-R025", "docs/adr/README.md", "no index table");

    let mut model = valid();
    remove(&mut model, "docs/adr/README.md");
    let found = findings_of(&model);
    // The README of the fixture root links to the index, so R024 fires too.
    assert!(
        found.contains(&(
            "XTASK-R025".to_string(),
            "docs/adr/README.md".to_string(),
            "the ADR index docs/adr/README.md does not exist".to_string()
        )),
        "{found:#?}"
    );
}

/// Verifies: XTASK-R025
#[test]
fn r025_unreadable_adr_must_still_be_listed() {
    let mut model = valid();
    edit(&mut model, ADR2, "status: accepted ", "status:accepted ");
    edit(
        &mut model,
        "docs/adr/README.md",
        "| [0002](0002-second.md) | Second decision | accepted |\n",
        "",
    );
    let rules: Vec<String> = findings_of(&model)
        .into_iter()
        .map(|(rule, _, _)| rule)
        .collect();
    // R020 for the syntax error and R025 for the missing row. The cross-file R021 rules skip the
    // unreadable ADR-0002, so ADR-0001's `superseded_by: [ADR-0002]` raises nothing.
    assert_eq!(rules, ["XTASK-R020", "XTASK-R025"]);
}

// ---------------------------------------------------------------- R026

/// Verifies: XTASK-R026
#[test]
fn r026_every_member_needs_a_spec() {
    let mut model = valid();
    remove(&mut model, SPEC);
    let found = findings_of(&model);
    assert!(
        found.contains(&(
            "XTASK-R026".to_string(),
            "tools/demo/Cargo.toml".to_string(),
            "workspace member `demo` has no spec at docs/specs/demo.md".to_string()
        )),
        "{found:#?}"
    );
}

// ---------------------------------------------------------------- R027

/// Verifies: XTASK-R027
#[test]
fn r027_spec_index_lists_every_spec_with_its_maturity() {
    let mut model = valid();
    edit(
        &mut model,
        "docs/specs/README.md",
        "| `planned` |",
        "| planned |",
    );
    assert_eq!(findings_of(&model), [], "backticks are optional");

    let mut model = valid();
    edit(
        &mut model,
        "docs/specs/README.md",
        "| [demo.md](demo.md) | `demo` | DEMO | `planned` |\n",
        "",
    );
    assert_one(
        &model,
        "XTASK-R027",
        "docs/specs/README.md",
        "demo.md (docs/specs/demo.md) is not listed",
    );
}

// ---------------------------------------------------------------- R028

/// Verifies: XTASK-R028
#[test]
fn r028_task_requirements_must_exist_in_some_spec() {
    let mut model = valid();
    edit(
        &mut model,
        T2,
        "requirements: []",
        "requirements: [DEMO-R001, DEMO-R002, DEMO-R003, DEMO-R009]",
    );
    // DEMO-R002 is withdrawn and DEMO-R003 is a malformed row: both exist in the spec.
    edit(
        &mut model,
        SPEC,
        "| DEMO-R001 | MUST | The demo crate exists. |",
        "| DEMO-R001 | MUST | The demo crate exists. |\n\
         | DEMO-R002 | MUST | Old (withdrawn in v2). |\n| DEMO-R003 | must | Malformed. |",
    );
    assert_one(
        &model,
        "XTASK-R028",
        "docs/tasks/M0-T002-second.md:8",
        "`requirements` names `DEMO-R009`, which is not a requirement in any spec",
    );
}

/// Verifies: XTASK-R028
#[test]
fn r028_skips_tasks_with_broken_frontmatter_or_a_scalar_requirements_field() {
    let mut model = valid();
    edit(
        &mut model,
        T2,
        "requirements: []",
        "requirements: DEMO-R009",
    );
    let found = findings_of(&model);
    assert!(
        found.iter().all(|(rule, _, _)| rule == "XTASK-R023"),
        "{found:#?}"
    );
    let mut model = valid();
    edit(
        &mut model,
        T2,
        "requirements: []",
        "requirements: [DEMO-R009]",
    );
    edit(&mut model, T2, "status: todo", "status:todo");
    let found = findings_of(&model);
    assert!(
        found.iter().all(|(rule, _, _)| rule == "XTASK-R020"),
        "{found:#?}"
    );
}
