//! A minimal Rust lexer shared by the checks that read `.rs` files: `check-arch` (XTASK-R035),
//! `trace` (XTASK-R041) and `hygiene` (XTASK-R050). Not a parser: it splits source text into
//! identifiers, punctuation, literals and comments, each with its line, so that a check can tell
//! code from comments and string contents without reading Rust grammar (no `syn` in xtask).

/// What a token is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Kind {
    /// An identifier or keyword, also a raw identifier without its `r#`.
    Ident(String),
    /// A single punctuation character.
    Punct(char),
    /// A string literal of any form (`"…"`, `b"…"`, `c"…"`, `r#"…"#`): its text between the
    /// quotes as written, escapes not processed.
    Str(String),
    /// A character, byte or numeric literal (contents are not needed).
    Literal,
    /// A line comment (`//…`, without the line break) or a block comment (`/*…*/`, nested), as
    /// written, delimiters included.
    Comment(String),
}

/// One token and the 1-based line it starts on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Token {
    /// What the token is.
    pub kind: Kind,
    /// 1-based line of the token's first character.
    pub line: usize,
    /// Whether no other token starts earlier on the same line.
    pub first_on_line: bool,
}

/// The tokens of `source`, comments included, after a byte order mark and a shebang line.
///
/// Stops at the first malformed construct (an unterminated block comment, string or raw string):
/// the tokens before it are returned, and the rest of the file is not tokenized. Such a file does
/// not compile.
pub(crate) fn tokens(source: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(source);
    lexer.skip_shebang();
    let mut tokens = Vec::new();
    let mut last_line = 0;
    while let Some((kind, line)) = lexer.next() {
        tokens.push(Token {
            kind,
            line,
            first_on_line: line != last_line,
        });
        last_line = line;
    }
    tokens
}

/// A tokenizer over one source text.
struct Lexer<'a> {
    source: &'a str,
    /// Byte offset of the next character.
    position: usize,
    /// 1-based line of the next character.
    line: usize,
    /// Whether the last token was the punctuation `.`.
    after_dot: bool,
    /// Whether the last token was a `.` that is not the second of `..`: a numeric literal after
    /// it is a tuple field index (`x.0.1`) and has no fraction.
    after_field_dot: bool,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        let source = source.strip_prefix('\u{feff}').unwrap_or(source);
        Self {
            source,
            position: 0,
            line: 1,
            after_dot: false,
            after_field_dot: false,
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
        if c == '\n' {
            self.line += 1;
        }
        Some(c)
    }

    /// Advances by `bytes` bytes, counting line breaks.
    fn advance(&mut self, bytes: usize) {
        let skipped = &self.source[self.position..self.position + bytes];
        self.line += skipped.matches('\n').count();
        self.position += bytes;
    }

    /// A first line `#!…` is a shebang unless the next token after `#!` is `[` (Rust reference,
    /// "Shebang"). Comments between `#!` and `[` do not change that.
    fn skip_shebang(&mut self) {
        if !self.rest().starts_with("#!") {
            return;
        }
        let mut probe = Lexer {
            source: self.source,
            position: 2,
            line: 1,
            after_dot: false,
            after_field_dot: false,
        };
        let is_attribute = loop {
            match probe.next() {
                Some((Kind::Comment(_), _)) => {}
                Some((Kind::Punct('['), _)) => break true,
                _ => break false,
            }
        };
        if !is_attribute {
            self.position = self.source.find('\n').unwrap_or(self.source.len());
        }
    }

    /// The next token and its line, or `None` at the end of input or on malformed input.
    fn next(&mut self) -> Option<(Kind, usize)> {
        let (kind, line) = self.next_token()?;
        let is_dot = kind == Kind::Punct('.');
        self.after_field_dot = is_dot && !self.after_dot;
        self.after_dot = is_dot;
        Some((kind, line))
    }

    /// [`Self::next`] without updating the `.` state.
    fn next_token(&mut self) -> Option<(Kind, usize)> {
        while self.peek().is_some_and(char::is_whitespace) {
            self.bump();
        }
        let line = self.line;
        let rest = self.rest();
        if rest.starts_with("//") {
            let end = rest.find('\n').unwrap_or(rest.len());
            self.position += end;
            return Some((Kind::Comment(rest[..end].to_string()), line));
        }
        if rest.starts_with("/*") {
            return self.block_comment().map(|kind| (kind, line));
        }
        let c = self.peek()?;
        let kind = if c == '_' || c.is_alphabetic() {
            let ident = self.identifier();
            match (ident.as_str(), self.peek()) {
                ("r" | "br" | "cr", Some('"' | '#')) => self.raw_string()?,
                ("b" | "c", Some('"')) => {
                    self.bump();
                    self.string('"')?
                }
                ("b", Some('\'')) => {
                    self.bump();
                    self.string('\'')?;
                    Kind::Literal
                }
                _ => Kind::Ident(ident),
            }
        } else if c.is_ascii_digit() {
            self.number();
            Kind::Literal
        } else {
            self.bump();
            match c {
                '"' => self.string('"')?,
                '\'' => self.char_or_lifetime()?,
                c => Kind::Punct(c),
            }
        };
        Some((kind, line))
    }

    /// A numeric literal at its first digit: digits, letters and `_` (radix prefix, hexadecimal
    /// digits, exponent, suffix); a sign directly after a decimal exponent `e`/`E` that comes
    /// before any other letter and is followed by a digit (`1e-3`); and one `.` (fraction) in a
    /// decimal literal before any letter, unless the `.` is followed by `.`, `_` or a letter
    /// (`0..n`, `1.max(2)`) or the literal is a tuple field index (`x.0.1`).
    fn number(&mut self) {
        let radix = matches!(self.rest().get(..2), Some("0x" | "0b" | "0o"));
        let mut letter_seen = false;
        let mut fraction_allowed = !radix && !self.after_field_dot;
        loop {
            let mut chars = self.rest().chars();
            let Some(c) = chars.next() else {
                return;
            };
            let next = chars.next();
            if c == '_' || c.is_alphanumeric() {
                self.bump();
                let exponent = !radix && !letter_seen && matches!(c, 'e' | 'E');
                if exponent
                    && matches!(next, Some('+' | '-'))
                    && chars.next().is_some_and(|d| d.is_ascii_digit())
                {
                    self.bump();
                }
                if c.is_alphabetic() {
                    letter_seen = true;
                    fraction_allowed = false;
                }
            } else if c == '.'
                && fraction_allowed
                && !next.is_some_and(|n| n == '.' || n == '_' || n.is_alphabetic())
            {
                self.bump();
                fraction_allowed = false;
            } else {
                return;
            }
        }
    }

    /// A block comment at `/*`; block comments nest. `None` if unterminated.
    fn block_comment(&mut self) -> Option<Kind> {
        let start = self.position;
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
                    return Some(Kind::Comment(self.source[start..self.position].to_string()));
                }
            } else {
                self.bump()?;
            }
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

    /// The rest of a quoted literal after its opening `quote`, with `\` escapes: its text up to
    /// the closing quote, as [`Kind::Str`]. `None` if unterminated.
    fn string(&mut self, quote: char) -> Option<Kind> {
        let start = self.position;
        loop {
            match self.bump()? {
                '\\' => {
                    self.bump()?;
                }
                c if c == quote => {
                    let end = self.position - quote.len_utf8();
                    return Some(Kind::Str(self.source[start..end].to_string()));
                }
                _ => {}
            }
        }
    }

    /// A raw string after its `r`/`br`/`cr` prefix: `#…#"…"#…#`. `None` if malformed.
    fn raw_string(&mut self) -> Option<Kind> {
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
        let text = self.rest()[..end].to_string();
        self.advance(end + close.len());
        Some(Kind::Str(text))
    }

    /// After a `'`: a character literal (`'x'`, `'\n'`) or a lifetime (`'a`, returned as the
    /// punctuation `'`; its name is consumed).
    fn char_or_lifetime(&mut self) -> Option<Kind> {
        let rest = self.rest();
        let mut chars = rest.chars();
        match (chars.next()?, chars.next()) {
            ('\\', _) => {
                self.string('\'')?;
                Some(Kind::Literal)
            }
            (c, Some('\'')) => {
                self.position += c.len_utf8() + 1;
                Some(Kind::Literal)
            }
            _ => {
                self.identifier();
                Some(Kind::Punct('\''))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<(Kind, usize)> {
        tokens(source)
            .into_iter()
            .map(|token| (token.kind, token.line))
            .collect()
    }

    fn ident(name: &str) -> Kind {
        Kind::Ident(name.to_string())
    }

    #[test]
    fn tokens_carry_their_line_and_comments_their_text() {
        let source = "// one\nfn f() { /* a\n b */ x }\n/// doc\n";
        assert_eq!(
            kinds(source),
            [
                (Kind::Comment("// one".to_string()), 1),
                (ident("fn"), 2),
                (ident("f"), 2),
                (Kind::Punct('('), 2),
                (Kind::Punct(')'), 2),
                (Kind::Punct('{'), 2),
                (Kind::Comment("/* a\n b */".to_string()), 2),
                (ident("x"), 3),
                (Kind::Punct('}'), 3),
                (Kind::Comment("/// doc".to_string()), 4),
            ]
        );
        let first: Vec<bool> = tokens(source).iter().map(|t| t.first_on_line).collect();
        assert_eq!(
            first,
            [
                true, true, false, false, false, false, false, true, false, true
            ]
        );
    }

    #[test]
    fn string_literals_keep_their_text_and_count_lines() {
        let source = "a(\"x // y\", r#\"two\n\"lines\"#, b\"\\\"q\", c\"c\", '\\'', 'z', b'\\n', 1.5e3)\nend";
        assert_eq!(
            kinds(source),
            [
                (ident("a"), 1),
                (Kind::Punct('('), 1),
                (Kind::Str("x // y".to_string()), 1),
                (Kind::Punct(','), 1),
                (Kind::Str("two\n\"lines".to_string()), 1),
                (Kind::Punct(','), 2),
                (Kind::Str("\\\"q".to_string()), 2),
                (Kind::Punct(','), 2),
                (Kind::Str("c".to_string()), 2),
                (Kind::Punct(','), 2),
                (Kind::Literal, 2),
                (Kind::Punct(','), 2),
                (Kind::Literal, 2),
                (Kind::Punct(','), 2),
                (Kind::Literal, 2),
                (Kind::Punct(','), 2),
                (Kind::Literal, 2),
                (Kind::Punct(')'), 2),
                (ident("end"), 3),
            ]
        );
    }

    /// Verifies: XTASK-R050
    #[test]
    fn numeric_literals_end_at_ranges_method_calls_and_tuple_fields() {
        let only = |source: &str| -> Vec<Kind> {
            tokens(source).into_iter().map(|token| token.kind).collect()
        };
        let dot = || Kind::Punct('.');
        for whole in [
            "1.5e3", "1_000u32", "0x1F", "1e-3", "2.5E+10", "1.", "1.0f64", "0b1010", "0o17", "7",
        ] {
            assert_eq!(only(whole), [Kind::Literal], "{whole}");
        }
        assert_eq!(only("0..n"), [Kind::Literal, dot(), dot(), ident("n")]);
        assert_eq!(
            only("0..=9"),
            [Kind::Literal, dot(), dot(), Kind::Punct('='), Kind::Literal]
        );
        assert_eq!(only("1..2.5"), [Kind::Literal, dot(), dot(), Kind::Literal]);
        assert_eq!(
            only("1.max(2)"),
            [
                Kind::Literal,
                dot(),
                ident("max"),
                Kind::Punct('('),
                Kind::Literal,
                Kind::Punct(')')
            ]
        );
        assert_eq!(only("1._x"), [Kind::Literal, dot(), ident("_x")]);
        assert_eq!(
            only("x.0.1"),
            [ident("x"), dot(), Kind::Literal, dot(), Kind::Literal]
        );
        // A hexadecimal `E` is a digit, not an exponent: `0x1E-3` is a subtraction.
        assert_eq!(
            only("0x1E-3"),
            [Kind::Literal, Kind::Punct('-'), Kind::Literal]
        );
        assert_eq!(only("1e-x"), [Kind::Literal, Kind::Punct('-'), ident("x")]);
    }

    #[test]
    fn lifetimes_raw_identifiers_and_nested_block_comments() {
        assert_eq!(
            kinds("&'a r#fn /* x /* y */ z */ q"),
            [
                (Kind::Punct('&'), 1),
                (Kind::Punct('\''), 1),
                (ident("fn"), 1),
                (Kind::Comment("/* x /* y */ z */".to_string()), 1),
                (ident("q"), 1),
            ]
        );
    }

    #[test]
    fn malformed_input_ends_the_token_list() {
        assert_eq!(kinds("a /* open"), [(ident("a"), 1)]);
        assert_eq!(kinds("a \"open"), [(ident("a"), 1)]);
        assert_eq!(kinds("a r#\"open\""), [(ident("a"), 1)]);
        assert_eq!(kinds("a r#x"), [(ident("a"), 1), (ident("x"), 1)]);
    }

    #[test]
    fn byte_order_mark_and_shebang_are_skipped() {
        assert_eq!(kinds("\u{feff}x"), [(ident("x"), 1)]);
        assert_eq!(kinds("#!/bin/run x\ny"), [(ident("y"), 2)]);
        assert_eq!(
            kinds("#! /* c */ [a]")
                .into_iter()
                .map(|(kind, _)| kind)
                .collect::<Vec<_>>(),
            [
                Kind::Punct('#'),
                Kind::Punct('!'),
                Kind::Comment("/* c */".to_string()),
                Kind::Punct('['),
                ident("a"),
                Kind::Punct(']'),
            ]
        );
    }
}
