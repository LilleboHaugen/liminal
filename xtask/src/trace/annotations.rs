//! `Verifies:` annotations in a `.rs` file and the function each one is attached to
//! (`docs/specs/xtask.md` XTASK-R041 and the "`trace` and `hygiene` details").

use crate::requirements::is_requirement_id;
use crate::rust_lexer::{self, Kind, Token};

/// One annotation line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Annotation {
    /// 1-based line of the annotation.
    pub line: usize,
    /// What the line says.
    pub content: Content,
}

/// What an annotation line says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Content {
    /// A well-formed annotation: its requirement IDs and the function it is attached to (`None`
    /// if the next item is not a function).
    Verifies {
        /// The requirement IDs, in order.
        ids: Vec<String>,
        /// Name and 1-based line of the function the annotation is attached to.
        function: Option<(String, usize)>,
    },
    /// A `///` line whose text starts with `Verifies:` but does not match the annotation form.
    Malformed,
}

/// Every annotation in `source`. An annotation is a `///` line comment that is the first token on
/// its line and whose text (after `///` and whitespace) starts with `Verifies:`.
pub(super) fn annotations(source: &str) -> Vec<Annotation> {
    let tokens = rust_lexer::tokens(source);
    let mut found = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        let Kind::Comment(text) = &token.kind else {
            continue;
        };
        let Some(body) = text.strip_prefix("///") else {
            continue;
        };
        if body.starts_with('/') || !token.first_on_line {
            continue;
        }
        let Some(list) = body.trim_start().strip_prefix("Verifies:") else {
            continue;
        };
        let content = match ids(list) {
            Some(ids) => Content::Verifies {
                ids,
                function: next_function(&tokens[index + 1..]),
            },
            None => Content::Malformed,
        };
        found.push(Annotation {
            line: token.line,
            content,
        });
    }
    found
}

/// The IDs of `\s*ID(\s*,\s*ID)*\s*`, or `None` if `list` does not have that form.
fn ids(list: &str) -> Option<Vec<String>> {
    let ids: Vec<String> = list.split(',').map(|id| id.trim().to_string()).collect();
    ids.iter().all(|id| is_requirement_id(id)).then_some(ids)
}

/// Words that may come before `fn` in a function header.
const QUALIFIERS: &[&str] = &[
    "pub", "const", "async", "unsafe", "extern", "default", "safe",
];

/// The name and line of the function whose header starts the item after the annotation: comments
/// and outer attributes are skipped; then qualifiers (`pub`, `pub(…)`, `const`, `async`,
/// `unsafe`, `extern "ABI"`, …; a string literal only directly after `extern`) and `fn name`. `None` if the next item is something else.
fn next_function(tokens: &[Token]) -> Option<(String, usize)> {
    let mut index = 0;
    // Comments and outer attributes.
    loop {
        match &tokens.get(index)?.kind {
            Kind::Comment(_) => index += 1,
            Kind::Punct('#') if tokens.get(index + 1)?.kind == Kind::Punct('[') => {
                index = after_group(tokens, index + 1)?;
            }
            _ => break,
        }
    }
    // Qualifiers.
    loop {
        match &tokens.get(index)?.kind {
            Kind::Comment(_) => index += 1,
            Kind::Ident(word) if QUALIFIERS.contains(&word.as_str()) => {
                index += 1;
                if word == "pub" && tokens.get(index)?.kind == Kind::Punct('(') {
                    index = after_group(tokens, index)?;
                }
                if word == "extern" {
                    // The optional ABI string, after comments; a string anywhere else ends the
                    // header.
                    while matches!(tokens.get(index)?.kind, Kind::Comment(_)) {
                        index += 1;
                    }
                    if matches!(tokens.get(index)?.kind, Kind::Str(_)) {
                        index += 1;
                    }
                }
            }
            _ => break,
        }
    }
    let is_fn = tokens.get(index)?.kind == Kind::Ident("fn".to_string());
    match &tokens.get(index + 1)?.kind {
        Kind::Ident(name) if is_fn => Some((name.clone(), tokens[index + 1].line)),
        _ => None,
    }
}

/// The index after the group that opens at `open` (`(`, `[` or `{`) and its matching closer.
fn after_group(tokens: &[Token], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(open) {
        match token.kind {
            Kind::Punct('(' | '[' | '{') => depth += 1,
            Kind::Punct(')' | ']' | '}') => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index + 1);
                }
            }
            Kind::Comment(_) => {}
            _ if depth == 0 => return None,
            _ => {}
        }
    }
    None
}
