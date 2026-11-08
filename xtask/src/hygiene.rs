//! The `hygiene` command and `ci` step (`docs/specs/xtask.md` XTASK-R050 … R052 and the "`trace`
//! and `hygiene` details").
//!
//! [`check`] is a pure function of the [`SourceModel`]. Only [`run_check`] reads files, through the
//! shared loader in [`crate::repo`].
//!
//! Files are read as tokens ([`crate::rust_lexer`]): forbidden macros are found in code, task
//! markers in comments, and ignore reasons in attribute string literals. The contents of string
//! literals are never searched for macros or markers. That is how this module, its tests and the
//! `ci` tests can name the patterns in string literals (the pattern tables below, messages, test
//! data) and still pass: no file of xtask's source is excluded, so a real forbidden macro or
//! untracked marker anywhere in xtask is still reported, and the patterns need no obfuscation.
//! Comments in xtask must not contain a marker word themselves; they say "task marker" instead.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

use crate::check;
use crate::cli::{CommandError, Exit};
use crate::finding::Finding;
use crate::frontmatter::{self, Value};
use crate::repo::sources::{self, CONFIG_PATH, Exclusion, SourceModel};
use crate::repo::{LoadError, RepoModel, file_name};
use crate::rust_lexer::{self, Kind, Token};

/// Command and step name.
pub(crate) const NAME: &str = "hygiene";

/// Macros that mark unfinished code (XTASK-R050), found as `name!` followed by `(`, `[` or `{`.
const MACROS: &[&str] = &["todo", "unimplemented", "dbg"];

/// Task marker words in comments (XTASK-R050). Each must be immediately followed by
/// `(M<n>-T<nnn>)`.
const MARKERS: &[&str] = &["TODO", "FIXME", "XXX", "HACK"];

/// The `hygiene` command as registered in the command table.
pub(crate) fn command(args: &[String], out: &mut dyn Write) -> Result<Exit, CommandError> {
    check::command(NAME, args, out, run_check)
}

/// Loads the repository at `root` and checks it (the [`check::CheckFn`] of this check).
///
/// # Errors
///
/// [`LoadError`] if the repository, `ci/hygiene.toml` or a source file cannot be read.
pub(crate) fn run_check(root: &Path) -> Result<Vec<Finding>, LoadError> {
    Ok(check(&sources::load(root)?))
}

/// Every `hygiene` finding for `model`, sorted (spec "Check command output").
pub(crate) fn check(model: &SourceModel) -> Vec<Finding> {
    let mut findings = Vec::new();
    exclusions(model, &mut findings);
    let tasks = task_statuses(&model.repo);
    for file in &model.sources {
        if model
            .config
            .hygiene
            .iter()
            .any(|exclusion| exclusion.covers(&file.path))
        {
            continue;
        }
        scan(&file.path, &file.text, &tasks, &mut findings);
    }
    check::sort(&mut findings);
    findings
}

/// Task ID → status (`None` if the task file's frontmatter cannot be read). The ID is the task
/// file name's `M<n>-T<nnn>` prefix (XTASK-R023 requires `id` to equal it).
fn task_statuses(model: &RepoModel) -> BTreeMap<String, Option<String>> {
    let mut tasks = BTreeMap::new();
    for (path, text) in model.task_files() {
        let name = file_name(path);
        let Some(id) = task_ids(name)
            .into_iter()
            .next()
            .filter(|id| name.starts_with(id.as_str()))
        else {
            continue;
        };
        let status = frontmatter::parse(text)
            .ok()
            .and_then(|fm| match fm.get("status") {
                Some(field) => match &field.value {
                    Value::Scalar(status) => Some(status.clone()),
                    Value::List(_) => None,
                },
                None => None,
            });
        tasks.insert(id, status);
    }
    tasks
}

/// Every task ID (`M<n>-T<nnn>`, not part of a longer word or number) in `text`, in order.
pub(crate) fn task_ids(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let word = |index: usize| bytes.get(index).is_some_and(u8::is_ascii_alphanumeric);
    let mut ids = Vec::new();
    for (start, _) in text.match_indices('M') {
        if start > 0 && word(start - 1) {
            continue;
        }
        let mut index = start + 1;
        let digits = bytes[index..]
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count();
        index += digits;
        if digits == 0 || !text[index..].starts_with("-T") {
            continue;
        }
        index += 2;
        let number = bytes[index..]
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count();
        if number == 3 && !word(index + 3) {
            ids.push(text[start..index + 3].to_string());
        }
    }
    ids
}

/// XTASK-R052: every exclusion (both sections) has a non-empty reason and names an existing
/// repository-relative path.
fn exclusions(model: &SourceModel, findings: &mut Vec<Finding>) {
    let sections = [
        ("hygiene.exclude", &model.config.hygiene),
        ("trace.exclude", &model.config.trace),
    ];
    for (section, entries) in sections {
        for Exclusion { path, reason, line } in entries {
            let location = format!("{CONFIG_PATH}:{line}");
            let mut report = |message: String, help: &str| {
                findings.push(Finding::error("XTASK-R052", &location, message, help));
            };
            if reason.trim().is_empty() {
                report(
                    format!("`{section}` entry `{path}` has no reason"),
                    "Say in `reason` why the path must not be scanned",
                );
            }
            let trimmed = path.trim_end_matches('/');
            let well_formed = !trimmed.is_empty()
                && !path.starts_with('/')
                && !path.contains('\\')
                && trimmed
                    .split('/')
                    .all(|part| !matches!(part, "" | "." | ".."));
            if !well_formed {
                report(
                    format!("`{section}` path `{path}` is not a repository-relative path"),
                    "Write the path relative to the repository root with `/` separators, without \
                     `.` or `..` components",
                );
            } else if !model.repo.paths.contains(trimmed) {
                report(
                    format!("`{section}` path `{path}` does not exist"),
                    "Remove the stale entry, or correct the path",
                );
            }
        }
    }
}

/// XTASK-R050 and R051 for one file.
fn scan(
    path: &str,
    text: &str,
    tasks: &BTreeMap<String, Option<String>>,
    findings: &mut Vec<Finding>,
) {
    let tokens = rust_lexer::tokens(text);
    for token in &tokens {
        if let Kind::Comment(comment) = &token.kind {
            markers(path, token.line, comment, tasks, findings);
        }
    }
    let code: Vec<&Token> = tokens
        .iter()
        .filter(|token| !matches!(token.kind, Kind::Comment(_)))
        .collect();
    macros(path, &code, findings);
    ignores(path, &code, tasks, findings);
}

fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Task markers in one comment: a marker word (optionally with a plural `s`) that is a whole
/// word and not immediately followed by `(M<n>-T<nnn>)` is XTASK-R050; a task it names that does
/// not exist or is `done` is XTASK-R051.
fn markers(
    path: &str,
    first_line: usize,
    comment: &str,
    tasks: &BTreeMap<String, Option<String>>,
    findings: &mut Vec<Finding>,
) {
    let bytes = comment.as_bytes();
    for marker in MARKERS {
        for (start, _) in comment.match_indices(marker) {
            let mut end = start + marker.len();
            let plural = bytes.get(end) == Some(&b's');
            let after_word = if plural { end + 1 } else { end };
            let whole = (start == 0 || !is_word_byte(bytes[start - 1]))
                && bytes.get(after_word).is_none_or(|b| !is_word_byte(*b));
            if !whole {
                continue;
            }
            let line = first_line + comment[..start].matches('\n').count();
            let location = format!("{path}:{line}");
            let task = (!plural).then(|| tracked_task(&comment[end..])).flatten();
            let Some(task) = task else {
                if plural {
                    end += 1;
                }
                findings.push(Finding::error(
                    "XTASK-R050",
                    location,
                    format!(
                        "`{}` comment without a task: write `{marker}(M<n>-T<nnn>): …`",
                        &comment[start..end]
                    ),
                    "Name the task that will resolve it, e.g. `(M1-T014)` directly after the \
                     marker, or resolve it now",
                ));
                continue;
            };
            match tasks.get(&task) {
                None => findings.push(Finding::error(
                    "XTASK-R051",
                    location,
                    format!("`{marker}({task})` names a task that does not exist in docs/tasks/"),
                    "Name an existing task, or create the task file first",
                )),
                Some(Some(status)) if status == "done" => findings.push(Finding::error(
                    "XTASK-R051",
                    location,
                    format!("`{marker}({task})` names a task that is `done`: the comment is stale"),
                    "Resolve the comment, or move it to the task that now owns the work",
                )),
                Some(_) => {}
            }
        }
    }
}

/// The task ID if `rest` starts with `(M<n>-T<nnn>)`.
fn tracked_task(rest: &str) -> Option<String> {
    let inner = rest.strip_prefix('(')?;
    let close = inner.find(')')?;
    let id = &inner[..close];
    (task_ids(id).first().map(String::as_str) == Some(id)).then(|| id.to_string())
}

/// Forbidden macro invocations: `name ! (`, `name ! [` or `name ! {` (XTASK-R050).
fn macros(path: &str, code: &[&Token], findings: &mut Vec<Finding>) {
    for window in code.windows(3) {
        let [name, bang, open] = window else { continue };
        let Kind::Ident(name_text) = &name.kind else {
            continue;
        };
        if MACROS.contains(&name_text.as_str())
            && bang.kind == Kind::Punct('!')
            && matches!(open.kind, Kind::Punct('(' | '[' | '{'))
        {
            findings.push(Finding::error(
                "XTASK-R050",
                format!("{path}:{}", name.line),
                format!("`{name_text}!` must not be committed"),
                "Finish the code, or leave the task `in-progress` with a handoff note saying \
                 what is missing (AGENTS.md rule 1); remove debugging output",
            ));
        }
    }
}

/// `ignore` in an attribute (`#[…]` or `#![…]`, also inside `cfg_attr`) without a reason string
/// that names a task is XTASK-R050; a named task that does not exist is XTASK-R051.
fn ignores(
    path: &str,
    code: &[&Token],
    tasks: &BTreeMap<String, Option<String>>,
    findings: &mut Vec<Finding>,
) {
    let mut index = 0;
    while index < code.len() {
        if code[index].kind != Kind::Punct('#') {
            index += 1;
            continue;
        }
        let mut open = index + 1;
        if code.get(open).is_some_and(|t| t.kind == Kind::Punct('!')) {
            open += 1;
        }
        if code.get(open).is_none_or(|t| t.kind != Kind::Punct('[')) {
            index += 1;
            continue;
        }
        // The attribute's tokens up to the matching `]`.
        let mut depth = 0usize;
        let mut end = open;
        while let Some(token) = code.get(end) {
            match token.kind {
                Kind::Punct('(' | '[' | '{') => depth += 1,
                Kind::Punct(')' | ']' | '}') => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            end += 1;
        }
        for at in open + 1..end.min(code.len()) {
            let token = code[at];
            if token.kind != Kind::Ident("ignore".to_string())
                || !matches!(code[at - 1].kind, Kind::Punct('[' | ',' | '('))
            {
                continue;
            }
            let location = format!("{path}:{}", token.line);
            let reason = match (code.get(at + 1), code.get(at + 2)) {
                (Some(eq), Some(value)) if eq.kind == Kind::Punct('=') => match &value.kind {
                    Kind::Str(text) => Some(text.as_str()),
                    _ => None,
                },
                (Some(next), _) if matches!(next.kind, Kind::Punct(']' | ',' | ')')) => None,
                _ => continue, // Not an ignore attribute (e.g. a path `ignore::x`).
            };
            let ids = reason.map(task_ids).unwrap_or_default();
            if ids.is_empty() {
                findings.push(Finding::error(
                    "XTASK-R050",
                    location,
                    "`ignore` attribute without a reason that names a task".to_string(),
                    "Write `#[ignore = \"M<n>-T<nnn>: why\"]` with the task that tracks it",
                ));
                continue;
            }
            for id in ids {
                if !tasks.contains_key(&id) {
                    findings.push(Finding::error(
                        "XTASK-R051",
                        location.clone(),
                        format!(
                            "`ignore` reason names `{id}`, which does not exist in docs/tasks/"
                        ),
                        "Name an existing task in the reason",
                    ));
                }
            }
        }
        index = end + 1;
    }
}

#[cfg(test)]
mod fixture_tests;
#[cfg(test)]
mod tests;
