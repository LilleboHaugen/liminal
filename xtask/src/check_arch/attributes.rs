//! Detects `#![forbid(unsafe_code)]` at a crate root (XTASK-R035) with a token-level scan, without
//! a Rust parser (the M0-T004 notes rule out `syn` in xtask).
//!
//! Only the crate's leading inner attributes count: the `#![…]` attributes before the first item,
//! with whitespace and comments (doc comments included) skipped between and inside them. An inner
//! attribute inside an inline `mod m { … }` applies to that module only, and text in comments or
//! string literals is not an attribute, so neither is found.

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
    let mut lexer = Lexer::new(source);
    lexer.skip_shebang();
    let mut attributes = Vec::new();
    loop {
        let opens = lexer.next_token() == Some(Token::Punct('#'))
            && lexer.next_token() == Some(Token::Punct('!'))
            && lexer.next_token() == Some(Token::Punct('['));
        if !opens {
            return attributes;
        }
        let mut depth = 0usize;
        let mut tokens = Vec::new();
        loop {
            match lexer.next_token() {
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

/// A minimal Rust lexer: identifiers, punctuation and literals; whitespace and comments skipped.
struct Lexer<'a> {
    source: &'a str,
    /// Byte offset of the next character.
    position: usize,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        let source = source.strip_prefix('\u{feff}').unwrap_or(source);
        Self {
            source,
            position: 0,
        }
    }

    fn rest(&self) -> &'a str {
        &self.source[self.position..]
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.position += c.len_utf8();
        Some(c)
    }

    /// A first line `#!…` is a shebang unless the next token after `#!` is `[` (Rust reference,
    /// "Shebang").
    fn skip_shebang(&mut self) {
        if !self.rest().starts_with("#!") {
            return;
        }
        self.position = 2;
        let is_attribute = self.skip_trivia() && self.peek() == Some('[');
        self.position = if is_attribute {
            0
        } else {
            self.source.find('\n').unwrap_or(self.source.len())
        };
    }

    /// Skips whitespace and comments. Returns `false` on an unterminated block comment.
    fn skip_trivia(&mut self) -> bool {
        loop {
            let rest = self.rest();
            if rest.starts_with(char::is_whitespace) {
                self.bump();
            } else if rest.starts_with("//") {
                self.position += rest.find('\n').unwrap_or(rest.len());
            } else if rest.starts_with("/*") {
                // Block comments nest.
                let mut depth = 0usize;
                loop {
                    let rest = self.rest();
                    if rest.starts_with("/*") {
                        depth += 1;
                        self.position += 2;
                    } else if rest.starts_with("*/") {
                        depth -= 1;
                        self.position += 2;
                        if depth == 0 {
                            break;
                        }
                    } else if self.bump().is_none() {
                        return false;
                    }
                }
            } else {
                return true;
            }
        }
    }

    /// The next token, or `None` at the end of input or on malformed input.
    fn next_token(&mut self) -> Option<Token> {
        if !self.skip_trivia() {
            return None;
        }
        let c = self.peek()?;
        if c == '_' || c.is_alphabetic() {
            let ident = self.identifier();
            return match (ident.as_str(), self.peek()) {
                ("r" | "br" | "cr", Some('"' | '#')) => self.raw_string(),
                ("b" | "c", Some('"')) => {
                    self.bump();
                    self.string('"')
                }
                ("b", Some('\'')) => {
                    self.bump();
                    self.string('\'')
                }
                _ => Some(Token::Ident(ident)),
            };
        }
        if c.is_ascii_digit() {
            while self
                .peek()
                .is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.')
            {
                self.bump();
            }
            return Some(Token::Literal);
        }
        self.bump();
        match c {
            '"' => self.string('"'),
            '\'' => self.char_or_lifetime(),
            c => Some(Token::Punct(c)),
        }
    }

    /// An identifier; a raw identifier `r#name` yields `name`.
    fn identifier(&mut self) -> String {
        let take = |lexer: &mut Self| {
            let start = lexer.position;
            while lexer
                .peek()
                .is_some_and(|c| c == '_' || c.is_alphanumeric())
            {
                lexer.bump();
            }
            lexer.source[start..lexer.position].to_string()
        };
        let ident = take(self);
        if ident == "r" && self.rest().starts_with('#') {
            let after = &self.rest()[1..];
            if after.starts_with(|c: char| c == '_' || c.is_alphabetic()) {
                self.position += 1;
                return take(self);
            }
        }
        ident
    }

    /// The rest of a quoted literal after its opening `quote`, with `\` escapes.
    fn string(&mut self, quote: char) -> Option<Token> {
        loop {
            match self.bump()? {
                '\\' => {
                    self.bump()?;
                }
                c if c == quote => return Some(Token::Literal),
                _ => {}
            }
        }
    }

    /// A raw string after its `r`/`br`/`cr` prefix: `#…#"…"#…#`.
    fn raw_string(&mut self) -> Option<Token> {
        let mut hashes = 0;
        while self.peek() == Some('#') {
            self.bump();
            hashes += 1;
        }
        if self.bump()? != '"' {
            return None;
        }
        let close = format!("\"{}", "#".repeat(hashes));
        let end = self.rest().find(&close)?;
        self.position += end + close.len();
        Some(Token::Literal)
    }

    /// After a `'`: a character literal (`'x'`, `'\n'`) or a lifetime (`'a`).
    fn char_or_lifetime(&mut self) -> Option<Token> {
        let rest = self.rest();
        let mut chars = rest.chars();
        match (chars.next()?, chars.next()) {
            ('\\', _) => self.string('\''),
            (c, Some('\'')) => {
                self.position += c.len_utf8() + 1;
                Some(Token::Literal)
            }
            _ => {
                self.identifier();
                Some(Token::Punct('\''))
            }
        }
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
