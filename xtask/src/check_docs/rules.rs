//! Frontmatter field rules: ADRs (XTASK-R021), specs (R022) and tasks (R023).

use std::collections::{BTreeMap, BTreeSet};

use super::fields::{self, Fields, one_of};
use super::parse_frontmatter;
use crate::finding::Finding;
use crate::frontmatter::{Frontmatter, Value, is_date, is_integer};
use crate::repo::{RepoModel, file_name};

const ADR_FIELDS: &[&str] = &[
    "id",
    "title",
    "status",
    "date",
    "supersedes",
    "superseded_by",
];
const ADR_STATUSES: &[&str] = &["proposed", "accepted", "superseded", "rejected"];
const SPEC_FIELDS: &[&str] = &[
    "crate",
    "req_prefix",
    "layer",
    "maturity",
    "spec_version",
    "last_reviewed",
    "adrs",
];
pub(crate) const MATURITIES: &[&str] =
    &["planned", "experimental", "usable", "stable", "optimized"];
const TASK_FIELDS: &[&str] = &[
    "id",
    "title",
    "milestone",
    "status",
    "crates",
    "depends_on",
    "requirements",
];
pub(crate) const TASK_STATUSES: &[&str] = &["todo", "in-progress", "blocked", "review", "done"];

/// An ADR file, as far as the index check (R025) needs it.
#[derive(Debug)]
pub(super) struct Adr {
    /// Repository-relative path.
    pub path: String,
    /// `ADR-NNNN`, from the file name.
    pub id: String,
    /// The expected index status cell, if the frontmatter was readable and its status valid.
    pub index_status: Option<String>,
}

/// A spec file, as far as the index check (R027) needs it.
#[derive(Debug)]
pub(super) struct Spec {
    /// Repository-relative path.
    pub path: String,
    /// The frontmatter maturity, if readable and valid.
    pub maturity: Option<String>,
}

/// The frontmatter data of a readable ADR, for the cross-file rules.
struct AdrData {
    path: String,
    status_line: usize,
    status: Option<String>,
    supersedes: Vec<(String, usize)>,
    superseded_by: Vec<(String, usize)>,
}

/// XTASK-R020 and R021 for every ADR file. Returns every ADR file, readable or not.
pub(super) fn adrs(model: &RepoModel, findings: &mut Vec<Finding>) -> Vec<Adr> {
    let ids: BTreeSet<String> = model.adr_files().map(|(path, _)| adr_id(path)).collect();
    let mut adrs = Vec::new();
    let mut data: BTreeMap<String, AdrData> = BTreeMap::new();
    for (path, text) in model.adr_files() {
        let id = adr_id(path);
        adrs.push(Adr {
            path: path.to_string(),
            id: id.clone(),
            index_status: None,
        });
        let Some(frontmatter) = parse_frontmatter(path, text, findings) else {
            continue;
        };
        let mut fields = Fields::new("XTASK-R021", path, &frontmatter, ADR_FIELDS, findings);
        if let Some((value, line)) = fields.scalar("id")
            && value != id
        {
            fields.report(
                Some(line),
                format!("`id` is `{value}`, but the file number says `{id}`"),
                format!("Set `id: {id}` or rename the file"),
            );
        }
        fields.scalar("title");
        let status = fields.scalar_where(
            "status",
            one_of(ADR_STATUSES),
            "one of proposed, accepted, superseded, rejected",
        );
        fields.scalar_where("date", is_date, "a date `YYYY-MM-DD`");
        let mut adr_list = |key: &str| -> Vec<(String, usize)> {
            let Some((items, line)) = fields.list(key) else {
                return Vec::new();
            };
            let mut valid = Vec::new();
            for item in items {
                if ids.contains(item) {
                    valid.push((item.clone(), line));
                } else {
                    fields.report(
                        Some(line),
                        format!("`{key}` names `{item}`, which is not an existing ADR"),
                        "Use the ID of an ADR file in docs/adr/ (`ADR-NNNN`)",
                    );
                }
            }
            valid
        };
        let supersedes = adr_list("supersedes");
        let superseded_by = adr_list("superseded_by");
        data.insert(
            id,
            AdrData {
                path: path.to_string(),
                status_line: status.map_or(1, |(_, line)| line),
                status: status.map(|(status, _)| status.to_string()),
                supersedes,
                superseded_by,
            },
        );
    }
    supersession(&data, findings);
    for adr in &mut adrs {
        let Some(entry) = data.get(&adr.id) else {
            continue;
        };
        adr.index_status = entry.status.as_ref().map(|status| {
            if status == "superseded" && !entry.superseded_by.is_empty() {
                let ids: Vec<&str> = entry
                    .superseded_by
                    .iter()
                    .map(|(id, _)| id.as_str())
                    .collect();
                format!("superseded by {}", ids.join(", "))
            } else {
                status.clone()
            }
        });
    }
    adrs
}

/// `docs/adr/0012-x.md` → `ADR-0012`.
fn adr_id(path: &str) -> String {
    format!("ADR-{}", &file_name(path)[..4])
}

/// R021 cross-file rules: symmetric supersession links; `superseded` iff `superseded_by` is set.
fn supersession(data: &BTreeMap<String, AdrData>, findings: &mut Vec<Finding>) {
    let rule = "XTASK-R021";
    for (id, adr) in data {
        for (other, line) in &adr.supersedes {
            if let Some(target) = data.get(other)
                && !target.superseded_by.iter().any(|(back, _)| back == id)
            {
                findings.push(Finding::error(
                    rule,
                    format!("{}:{line}", adr.path),
                    format!(
                        "{id} supersedes {other}, but {other} does not list {id} in `superseded_by`"
                    ),
                    format!(
                        "Add {id} to `superseded_by` of {other} (and set its status to superseded)"
                    ),
                ));
            }
        }
        for (other, line) in &adr.superseded_by {
            if let Some(target) = data.get(other)
                && !target.supersedes.iter().any(|(back, _)| back == id)
            {
                findings.push(Finding::error(
                    rule,
                    format!("{}:{line}", adr.path),
                    format!("{id} is superseded by {other}, but {other} does not list {id} in `supersedes`"),
                    format!("Add {id} to `supersedes` of {other}"),
                ));
            }
        }
        let superseded = adr.status.as_deref() == Some("superseded");
        let has_successor = !adr.superseded_by.is_empty();
        if adr.status.is_some() && superseded != has_successor {
            let message = if superseded {
                format!("{id} has status superseded but an empty `superseded_by`")
            } else {
                format!("{id} lists `superseded_by` but its status is not superseded")
            };
            findings.push(Finding::error(
                rule,
                format!("{}:{}", adr.path, adr.status_line),
                message,
                "Status is superseded if and only if `superseded_by` names the superseding ADR",
            ));
        }
    }
}

/// XTASK-R020 and R022 for every spec file.
pub(super) fn specs(model: &RepoModel, adrs: &[Adr], findings: &mut Vec<Finding>) -> Vec<Spec> {
    let adr_ids: BTreeSet<&str> = adrs.iter().map(|adr| adr.id.as_str()).collect();
    let mut prefixes: BTreeMap<String, String> = BTreeMap::new();
    let mut specs = Vec::new();
    for (path, text) in model.spec_files() {
        let mut spec = Spec {
            path: path.to_string(),
            maturity: None,
        };
        if let Some(frontmatter) = parse_frontmatter(path, text, findings) {
            let mut fields = Fields::new("XTASK-R022", path, &frontmatter, SPEC_FIELDS, findings);
            let stem = file_name(path).trim_end_matches(".md");
            let entry = fields.scalar("crate").and_then(|(name, line)| {
                if name != stem {
                    fields.report(
                        Some(line),
                        format!("`crate` is `{name}`, but the file is {stem}.md"),
                        format!("A spec lives at docs/specs/<crate>.md; set `crate: {stem}` or rename the file"),
                    );
                }
                let entry = model.architecture.crates.iter().find(|c| c.name == name);
                if entry.is_none() {
                    fields.report(
                        Some(line),
                        format!("crate `{name}` is not registered in architecture.toml"),
                        "Register the crate in architecture.toml first (AGENTS.md rule 6)",
                    );
                }
                entry
            });
            let layer = fields.scalar_where("layer", is_integer, "an integer 0–5");
            if let (Some(entry), Some((layer, line))) = (entry, layer)
                && layer.parse::<i64>().ok() != Some(entry.layer)
            {
                fields.report(
                    Some(line),
                    format!(
                        "`layer` is {layer}, but architecture.toml puts `{}` in layer {}",
                        entry.name, entry.layer
                    ),
                    format!("Set `layer: {}`", entry.layer),
                );
            }
            if let Some((prefix, line)) = fields.scalar_where(
                "req_prefix",
                fields::is_req_prefix,
                "upper-case letters (`^[A-Z]+$`)",
            ) {
                if let Some(first) = prefixes.get(prefix) {
                    fields.report(
                        Some(line),
                        format!("`req_prefix` {prefix} is already used by {first}"),
                        "Use the prefix reserved for this crate in docs/specs/README.md",
                    );
                } else {
                    prefixes.insert(prefix.to_string(), path.to_string());
                }
            }
            let maturity = fields.scalar_where(
                "maturity",
                one_of(MATURITIES),
                "one of planned, experimental, usable, stable, optimized",
            );
            spec.maturity = maturity.map(|(maturity, _)| maturity.to_string());
            fields.scalar_where(
                "spec_version",
                |v| is_integer(v) && v.parse::<u64>().is_ok_and(|v| v >= 1),
                "an integer ≥ 1",
            );
            fields.scalar_where("last_reviewed", is_date, "a date `YYYY-MM-DD`");
            if let Some((items, line)) = fields.list("adrs") {
                for item in items {
                    if !adr_ids.contains(item.as_str()) {
                        fields.report(
                            Some(line),
                            format!("`adrs` names `{item}`, which is not an existing ADR"),
                            "Use the ID of an ADR file in docs/adr/ (`ADR-NNNN`)",
                        );
                    }
                }
            }
        }
        specs.push(spec);
    }
    specs
}

/// The frontmatter data of a readable task, for the cross-file rules.
struct TaskData {
    path: String,
    depends_on: Vec<String>,
    depends_line: usize,
}

/// XTASK-R020 and R023 for every task file.
pub(super) fn tasks(model: &RepoModel, findings: &mut Vec<Finding>) {
    let crates: BTreeSet<&str> = model
        .architecture
        .crates
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    let mut tasks: BTreeMap<String, TaskData> = BTreeMap::new();
    // Task files without a valid `id`, by the ID their file name starts with.
    let mut unreadable: BTreeSet<String> = BTreeSet::new();
    for (path, text) in model.task_files() {
        let Some(frontmatter) = parse_frontmatter(path, text, findings) else {
            unreadable.extend(file_task_id(path));
            continue;
        };
        let mut fields = Fields::new("XTASK-R023", path, &frontmatter, TASK_FIELDS, findings);
        let name = file_name(path);
        let id = fields
            .scalar_where("id", fields::is_task_id, "a task ID `M<n>-T<nnn>`")
            .and_then(|(id, line)| {
                let rest = name.strip_prefix(id).unwrap_or("x");
                if !(rest.starts_with('-') || rest == ".md") {
                    fields.report(
                        Some(line),
                        format!("`id` is `{id}`, but the file name {name} does not start with it"),
                        format!("Name the file `{id}-<slug>.md`"),
                    );
                }
                if let Some(first) = tasks.get(id) {
                    fields.report(
                        Some(line),
                        format!("task ID {id} is already used by {}", first.path),
                        "Give the task the next free ID in its milestone",
                    );
                    return None;
                }
                Some(id)
            });
        fields.scalar("title");
        if let Some((milestone, line)) =
            fields.scalar_where("milestone", fields::is_milestone, "a milestone `M<n>`")
            && let Some(id) = id
            && id.split_once('-').map(|(m, _)| m) != Some(milestone)
        {
            fields.report(
                Some(line),
                format!("`milestone` is {milestone}, but the task ID {id} is in another milestone"),
                "The milestone is the part of `id` before `-T`",
            );
        }
        fields.scalar_where(
            "status",
            one_of(TASK_STATUSES),
            "one of todo, in-progress, blocked, review, done",
        );
        if let Some((items, line)) = fields.list("crates") {
            for item in items {
                if !crates.contains(item.as_str()) {
                    fields.report(
                        Some(line),
                        format!(
                            "`crates` names `{item}`, which is not registered in architecture.toml"
                        ),
                        "Use crate names from architecture.toml",
                    );
                }
            }
        }
        fields.list("requirements");
        let depends = fields.list("depends_on");
        if id.is_none() && !fields_have_valid_id(&frontmatter) {
            unreadable.extend(file_task_id(path));
        }
        if let Some(id) = id {
            tasks.insert(
                id.to_string(),
                TaskData {
                    path: path.to_string(),
                    depends_on: depends.map(|(items, _)| items.to_vec()).unwrap_or_default(),
                    depends_line: depends.map_or(1, |(_, line)| line),
                },
            );
        }
    }
    dependencies(&tasks, &unreadable, findings);
}

/// The task ID a task file name starts with (`^M\d+-T\d{3}`), if any.
fn file_task_id(path: &str) -> Option<String> {
    let name = file_name(path).strip_suffix(".md")?;
    let mut parts = name.splitn(3, '-');
    let id = format!("{}-{}", parts.next()?, parts.next()?);
    fields::is_task_id(&id).then_some(id)
}

/// Whether the frontmatter has an `id` of the task ID form (then the file is in the task set, or
/// its ID is a duplicate of one that is).
fn fields_have_valid_id(frontmatter: &Frontmatter) -> bool {
    frontmatter
        .get("id")
        .is_some_and(|field| matches!(&field.value, Value::Scalar(id) if fields::is_task_id(id)))
}

/// R023 cross-file rules: every `depends_on` ID exists; the dependency graph has no cycles.
/// `unreadable` holds the file-name IDs of task files without a valid `id`. They exist for
/// `depends_on`, but their own dependencies are unknown.
fn dependencies(
    tasks: &BTreeMap<String, TaskData>,
    unreadable: &BTreeSet<String>,
    findings: &mut Vec<Finding>,
) {
    for (id, task) in tasks {
        for dependency in &task.depends_on {
            if !tasks.contains_key(dependency) && !unreadable.contains(dependency) {
                findings.push(Finding::error(
                    "XTASK-R023",
                    format!("{}:{}", task.path, task.depends_line),
                    format!("{id} depends on {dependency}, which is not an existing task"),
                    "Use the ID of a task file in docs/tasks/",
                ));
            }
        }
    }
    let mut reported: BTreeSet<Vec<String>> = BTreeSet::new();
    let mut done: BTreeSet<&str> = BTreeSet::new();
    for start in tasks.keys() {
        let mut stack: Vec<&str> = Vec::new();
        find_cycles(start, tasks, &mut stack, &mut done, &mut reported);
    }
    for cycle in reported {
        let first = &tasks[&cycle[0]];
        let mut path = cycle.clone();
        path.push(cycle[0].clone());
        findings.push(Finding::error(
            "XTASK-R023",
            format!("{}:{}", first.path, first.depends_line),
            format!("dependency cycle: {}", path.join(" → ")),
            "Remove one `depends_on` entry of the cycle",
        ));
    }
}

/// Depth-first search from `id`. Each cycle is recorded once, rotated to start at its smallest ID.
fn find_cycles<'a>(
    id: &'a str,
    tasks: &'a BTreeMap<String, TaskData>,
    stack: &mut Vec<&'a str>,
    done: &mut BTreeSet<&'a str>,
    reported: &mut BTreeSet<Vec<String>>,
) {
    if done.contains(id) {
        return;
    }
    if let Some(position) = stack.iter().position(|on_stack| *on_stack == id) {
        let cycle = &stack[position..];
        let smallest = (0..cycle.len()).min_by_key(|&i| cycle[i]).unwrap_or(0);
        let rotated: Vec<String> = cycle[smallest..]
            .iter()
            .chain(&cycle[..smallest])
            .map(ToString::to_string)
            .collect();
        reported.insert(rotated);
        return;
    }
    let Some(task) = tasks.get(id) else {
        return;
    };
    stack.push(id);
    for dependency in &task.depends_on {
        find_cycles(dependency, tasks, stack, done, reported);
    }
    stack.pop();
    done.insert(id);
}
