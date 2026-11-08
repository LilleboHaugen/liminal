//! Detects `#![forbid(unsafe_code)]` at a crate root (XTASK-R035) with a token-level scan, without
//! a Rust parser (the M0-T004 notes rule out `syn` in xtask). Tokens come from
//! [`crate::rust_lexer`].
//!
//! Only the crate's leading inner attributes count: the `#![…]` attributes before the first item,
//! with whitespace and comments (doc comments included) skipped between and inside them. An inner
//! attribute inside an inline `mod m { … }` applies to that module only, and text in comments or
//! string literals is not an attribute, so neither is found.

use crate::rust_lexer::{self, Kind};

/// One token of an attribute.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    /// An identifier or keyword, also a raw identifier without its `r#`.
    Ident(String),
    /// A single punctuation character.
    Punct(char),
    /// A string, character or numeric literal (contents are not needed).
    Literal,
}

/// Whether `source` (a crate root file) starts with inner attributes that include
/// `forbid(…)` with `unsafe_code` as one of its lints. `#![cfg_attr(…, forbid(unsafe_code))]` does
/// not count: it is conditional.
pub(crate) fn forbids_unsafe_code(source: &str) -> bool {
    leading_inner_attributes(source)
        .iter()
        .any(|tokens| is_forbid_unsafe_code(tokens))
}

/// `forbid ( … )` where one comma-separated item is exactly `unsafe_code`.
fn is_forbid_unsafe_code(tokens: &[Token]) -> bool {
    let [
        Token::Ident(name),
        Token::Punct('('),
        inner @ ..,
        Token::Punct(')'),
    ] = tokens
    else {
        return false;
    };
    if name != "forbid" {
        return false;
    }
    // Lint list items are paths or `reason = "…"`; a string is one token, so no item contains a
    // comma.
    inner
        .split(|token| *token == Token::Punct(','))
        .any(|item| matches!(item, [Token::Ident(lint)] if lint == "unsafe_code"))
}

/// The token lists of the leading inner attributes (the text between `#![` and the matching `]`).
/// Stops at the first token that does not start an inner attribute, or at malformed input.
fn leading_inner_attributes(source: &str) -> Vec<Vec<Token>> {
    // Comments are trivia here; string and other literals are one opaque token each.
    let mut lexer = rust_lexer::tokens(source)
        .into_iter()
        .filter_map(|token| match token.kind {
            Kind::Ident(name) => Some(Token::Ident(name)),
            Kind::Punct(c) => Some(Token::Punct(c)),
            Kind::Str(_) | Kind::Literal => Some(Token::Literal),
            Kind::Comment(_) => None,
        });
    let mut attributes = Vec::new();
    loop {
        let opens = lexer.next() == Some(Token::Punct('#'))
            && lexer.next() == Some(Token::Punct('!'))
            && lexer.next() == Some(Token::Punct('['));
        if !opens {
            return attributes;
        }
        let mut depth = 0usize;
        let mut tokens = Vec::new();
        loop {
            match lexer.next() {
                None => return attributes,
                Some(Token::Punct(']')) if depth == 0 => break,
                Some(token) => {
                    match token {
                        Token::Punct('(' | '[' | '{') => depth += 1,
                        Token::Punct(')' | ']' | '}') => depth = depth.saturating_sub(1),
                        _ => {}
                    }
                    tokens.push(token);
                }
            }
        }
        attributes.push(tokens);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies: XTASK-R035
    #[test]
    fn finds_forbid_unsafe_code_among_the_leading_inner_attributes() {
        for source in [
            "#![forbid(unsafe_code)]\n",
            "#![forbid(unsafe_code)]",
            "\u{feff}#![forbid(unsafe_code)]\nfn main() {}\n",
            "//! Crate docs.\n//! More.\n#![forbid(unsafe_code)]\n\nmod a;\n",
            "/*! Block docs /* nested */ */\n#![forbid(unsafe_code)]\n",
            "#![deny(missing_docs)]\n#![forbid(unsafe_code)]\n",
            "#![allow(\n    clippy::print_stdout,\n    reason = \"CLI ]) output\"\n)]\n#![forbid(unsafe_code)]\n",
            "#![forbid(missing_docs, unsafe_code)]\n",
            "#![forbid(unsafe_code, reason = \"policy\")]\n",
            "#![doc = concat![\"a\", \"b\"]]\n#![forbid(unsafe_code)]\n",
            "#![doc = \"a\\\"]\"]\n#![forbid(unsafe_code)]\n",
            "# ! [ forbid ( unsafe_code ) ]\n",
            "#!/usr/bin/env run-cargo-script\n#![forbid(unsafe_code)]\n",
            "#! [forbid(unsafe_code)]\n",
            "#![doc = r#\"has \"]\" inside\"#]\n#![forbid(unsafe_code)]\n",
            "#![doc = 'x']\n#![doc = '\\'']\n#![forbid(unsafe_code)]\n",
            "#![doc = b\"]\"]\n#![cfg_attr(test, allow(x))]\n#![forbid(unsafe_code)]\n",
            "#![doc = 1.5e3]\n#![forbid(unsafe_code)]\n",
        ] {
            assert!(forbids_unsafe_code(source), "{source:?}");
        }
    }

    /// Verifies: XTASK-R035
    #[test]
    fn ignores_comments_strings_other_lints_and_non_leading_attributes() {
        for source in [
            "",
            "fn main() {}\n",
            "// #![forbid(unsafe_code)]\n",
            "/* #![forbid(unsafe_code)] */\n",
            "/* unterminated #![forbid(unsafe_code)]\n",
            "//! Use `#![forbid(unsafe_code)]` in every crate.\n",
            "const S: &str = \"#![forbid(unsafe_code)]\";\n",
            "#![deny(unsafe_code)]\n",
            "#![allow(unsafe_code)]\n",
            "#![forbid(unsafe_op_in_unsafe_fn)]\n",
            "#![forbid(unsafe_code_x)]\n",
            "#![forbid(clippy::unsafe_code)]\n",
            "#![forbid(unsafe_code = 1)]\n",
            "#![forbid]\n",
            "#![cfg_attr(not(test), forbid(unsafe_code))]\n",
            "#[forbid(unsafe_code)]\nfn f() {}\n",
            "mod m {\n    #![forbid(unsafe_code)]\n}\n",
            "use std::io;\n#![forbid(unsafe_code)]\n",
            "#![doc = \"unterminated\n#![forbid(unsafe_code)]\n",
            "#![forbid(unsafe_code)\n",
            "#![doc = r#\"unterminated\n#![forbid(unsafe_code)]\n",
            "#!/bin/sh #![forbid(unsafe_code)]\n",
        ] {
            assert!(!forbids_unsafe_code(source), "{source:?}");
        }
    }

    #[test]
    fn the_real_crate_roots_forbid_unsafe_code() {
        let root = crate::repo::root();
        for file in ["xtask/src/lib.rs", "xtask/src/main.rs"] {
            let source = std::fs::read_to_string(root.join(file)).unwrap();
            assert!(forbids_unsafe_code(&source), "{file}");
        }
    }
}
