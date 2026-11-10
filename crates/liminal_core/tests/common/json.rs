//! Test-local strict JSON decoder: the oracle for `render_json` (spec §10). Written from
//! RFC 8259, independent of the renderer. It accepts only what `render_json` may produce at the
//! structural level (objects, strings, numbers, `true`, `false`, `null`; no arrays) and rejects
//! any whitespace outside strings.

/// A decoded JSON value. Numbers keep their text; object members keep their order.
#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(String),
    Text(String),
    Object(Vec<(String, Json)>),
}

/// Decodes `text` as one JSON value with nothing before or after it.
pub fn parse(text: &str) -> Result<Json, String> {
    let mut parser = Parser {
        chars: text.chars().collect(),
        at: 0,
    };
    let value = parser.value()?;
    if parser.at != parser.chars.len() {
        return Err(format!("trailing input at character {}", parser.at));
    }
    Ok(value)
}

struct Parser {
    chars: Vec<char>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    fn next(&mut self) -> Option<char> {
        let next = self.peek();
        self.at += 1;
        next
    }

    fn expect(&mut self, expected: char) -> Result<(), String> {
        match self.next() {
            Some(found) if found == expected => Ok(()),
            found => Err(format!(
                "expected {expected:?}, found {found:?} at character {}",
                self.at - 1
            )),
        }
    }

    fn literal(&mut self, word: &str, value: Json) -> Result<Json, String> {
        for expected in word.chars() {
            self.expect(expected)?;
        }
        Ok(value)
    }

    fn value(&mut self) -> Result<Json, String> {
        match self.peek() {
            Some('{') => self.object(),
            Some('"') => self.string().map(Json::Text),
            Some('n') => self.literal("null", Json::Null),
            Some('t') => self.literal("true", Json::Bool(true)),
            Some('f') => self.literal("false", Json::Bool(false)),
            Some('-' | '0'..='9') => self.number(),
            found => Err(format!("unexpected {found:?} at character {}", self.at)),
        }
    }

    fn object(&mut self) -> Result<Json, String> {
        self.expect('{')?;
        let mut members = Vec::new();
        if self.peek() == Some('}') {
            self.at += 1;
            return Ok(Json::Object(members));
        }
        loop {
            let key = self.string()?;
            self.expect(':')?;
            members.push((key, self.value()?));
            match self.next() {
                Some(',') => {}
                Some('}') => return Ok(Json::Object(members)),
                found => return Err(format!("expected ',' or '}}', found {found:?}")),
            }
        }
    }

    /// RFC 8259 §7. Unescaped U+0000 to U+001F are rejected, as the RFC requires.
    fn string(&mut self) -> Result<String, String> {
        self.expect('"')?;
        let mut text = String::new();
        loop {
            match self.next() {
                None => return Err("unterminated string".to_string()),
                Some('"') => return Ok(text),
                Some('\\') => text.push(self.escape()?),
                Some(control) if u32::from(control) < 0x20 => {
                    return Err(format!("unescaped control character {control:?}"));
                }
                Some(other) => text.push(other),
            }
        }
    }

    fn escape(&mut self) -> Result<char, String> {
        match self.next() {
            Some('"') => Ok('"'),
            Some('\\') => Ok('\\'),
            Some('/') => Ok('/'),
            Some('b') => Ok('\u{08}'),
            Some('f') => Ok('\u{0C}'),
            Some('n') => Ok('\n'),
            Some('r') => Ok('\r'),
            Some('t') => Ok('\t'),
            Some('u') => {
                let mut code = 0_u32;
                for _ in 0..4 {
                    let digit = self.next().and_then(|c| c.to_digit(16));
                    code = code * 16 + digit.ok_or("bad \\u escape")?;
                }
                // Surrogate pairs are valid JSON, but `render_json` never writes them.
                char::from_u32(code).ok_or_else(|| format!("surrogate escape \\u{code:04x}"))
            }
            found => Err(format!("unknown escape {found:?}")),
        }
    }

    /// RFC 8259 §6: `-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?`.
    fn number(&mut self) -> Result<Json, String> {
        let start = self.at;
        if self.peek() == Some('-') {
            self.at += 1;
        }
        match self.next() {
            Some('0') => {}
            Some('1'..='9') => self.digits(),
            found => return Err(format!("bad number start {found:?}")),
        }
        if self.peek() == Some('.') {
            self.at += 1;
            self.one_or_more_digits()?;
        }
        if matches!(self.peek(), Some('e' | 'E')) {
            self.at += 1;
            if matches!(self.peek(), Some('+' | '-')) {
                self.at += 1;
            }
            self.one_or_more_digits()?;
        }
        Ok(Json::Number(self.chars[start..self.at].iter().collect()))
    }

    fn digits(&mut self) {
        while matches!(self.peek(), Some('0'..='9')) {
            self.at += 1;
        }
    }

    fn one_or_more_digits(&mut self) -> Result<(), String> {
        let start = self.at;
        self.digits();
        if self.at == start {
            return Err(format!("expected a digit at character {start}"));
        }
        Ok(())
    }
}
