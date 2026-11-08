//! Frontmatter parser tests: examples, every rejection rule, and property tests (spec §10).

use std::fmt::Write as _;

use proptest::prelude::*;

use super::*;

fn scalar(text: &str) -> Value {
    Value::Scalar(text.to_string())
}

fn list(items: &[&str]) -> Value {
    Value::List(items.iter().map(ToString::to_string).collect())
}

fn parse_err(text: &str) -> ParseError {
    match parse(text) {
        Err(error) => error,
        Ok(frontmatter) => panic!("expected an error for {text:?}, got {frontmatter:?}"),
    }
}

/// Verifies: XTASK-R020
#[test]
fn parses_the_documented_examples() {
    let text = "---\nid: ADR-0006\ntitle: Own GPU abstraction over Vulkan 1.3 and Metal\n\
                status: accepted            # proposed | accepted | superseded | rejected\n\
                date: 2026-11-08\nsupersedes: []\nsuperseded_by: [ADR-0019, ADR-0020]\n---\n# Body\n";
    let frontmatter = parse(text).unwrap();
    let pairs: Vec<(&str, &Value, usize)> = frontmatter
        .fields
        .iter()
        .map(|field| (field.key.as_str(), &field.value, field.line))
        .collect();
    assert_eq!(
        pairs,
        [
            ("id", &scalar("ADR-0006"), 2),
            (
                "title",
                &scalar("Own GPU abstraction over Vulkan 1.3 and Metal"),
                3
            ),
            ("status", &scalar("accepted"), 4),
            ("date", &scalar("2026-11-08"), 5),
            ("supersedes", &list(&[]), 6),
            ("superseded_by", &list(&["ADR-0019", "ADR-0020"]), 7),
        ]
    );
    assert_eq!(frontmatter.get("status").unwrap().line, 4);
    assert_eq!(frontmatter.get("missing"), None);
}

/// Verifies: XTASK-R020
#[test]
fn accepts_crlf_tabs_hash_without_space_and_body_after_close() {
    let frontmatter =
        parse("---\r\nlang: C#\r\nitems: [ a ,b\t]\t# note\r\nempty: [ ]\r\n---\r\nbody: [x\n")
            .unwrap();
    assert_eq!(frontmatter.get("lang").unwrap().value, scalar("C#"));
    assert_eq!(frontmatter.get("items").unwrap().value, list(&["a", "b"]));
    assert_eq!(frontmatter.get("empty").unwrap().value, list(&[]));
    assert_eq!(frontmatter.fields.len(), 3);
}

/// Verifies: XTASK-R020
#[test]
fn accepts_indicator_characters_that_yaml_reads_as_plain_text() {
    let frontmatter =
        parse("---\nneg: -1\nflag: --all\nq: ?x\nurl: a:b\nitems: [-x, a:b, ?y]\n---\n").unwrap();
    assert_eq!(frontmatter.get("neg").unwrap().value, scalar("-1"));
    assert_eq!(frontmatter.get("flag").unwrap().value, scalar("--all"));
    assert_eq!(frontmatter.get("q").unwrap().value, scalar("?x"));
    assert_eq!(frontmatter.get("url").unwrap().value, scalar("a:b"));
    assert_eq!(
        frontmatter.get("items").unwrap().value,
        list(&["-x", "a:b", "?y"])
    );
}

/// Verifies: XTASK-R020
#[test]
fn only_space_and_tab_are_trimmed_and_other_printable_characters_are_text() {
    // YAML's whitespace is space and tab only: U+00A0 and U+3000 are text, also at the edges and
    // before `#`, and `-`/`?` followed by U+00A0 are plain text. A tab inside a value is text.
    let frontmatter = parse(
        "---\ntrail: x\u{a0}\nlead: \u{a0}x\nideo: x\u{3000}  # c\nhash: x\u{a0}#y\n\
         dash: -\u{a0}x\nq: ?\u{a0}x\ntab: a\tb\nbom: a\u{feff}b\nitems: [a\u{a0}, \u{3000}b, \u{e9}]\n---\n",
    )
    .unwrap();
    let expected = [
        ("trail", scalar("x\u{a0}")),
        ("lead", scalar("\u{a0}x")),
        ("ideo", scalar("x\u{3000}")),
        ("hash", scalar("x\u{a0}#y")),
        ("dash", scalar("-\u{a0}x")),
        ("q", scalar("?\u{a0}x")),
        ("tab", scalar("a\tb")),
        ("bom", scalar("a\u{feff}b")),
        ("items", list(&["a\u{a0}", "\u{3000}b", "\u{e9}"])),
    ];
    for (key, value) in expected {
        assert_eq!(frontmatter.get(key).unwrap().value, value, "{key}");
    }
}

/// Verifies: XTASK-R020
#[test]
fn rejects_every_construct_outside_the_subset_with_its_line() {
    let cases: &[(&str, usize, &str)] = &[
        ("", 1, "line 1 must be `---`"),
        ("# Title\n", 1, "line 1 must be `---`"),
        ("---\nkey: value\n", 1, "never closed"),
        ("---\n\nkey: value\n---\n", 2, "blank lines"),
        ("---\nkey: v\n  nested: x\n---\n", 3, "indented line"),
        ("---\n# comment\n---\n", 2, "comment lines"),
        ("---\nno colon here\n---\n", 2, "expected `key: value`"),
        ("---\nCamelCase: x\n---\n", 2, "not snake_case"),
        ("---\n9key: x\n---\n", 2, "not snake_case"),
        ("---\nkey : x\n---\n", 2, "not snake_case"),
        ("---\n- item\n---\n", 2, "expected `key: value`"),
        ("---\nkey:value\n---\n", 2, "expected a space after `key:`"),
        ("---\nkey:\n---\n", 2, "has no value"),
        ("---\nkey:   # only a comment\n---\n", 2, "has no value"),
        (
            "---\na: 1\na: 2\n---\n",
            3,
            "duplicate key `a` (first on line 2)",
        ),
        ("---\nkey: [a, b\n---\n", 2, "must end with `]`"),
        ("---\nkey: [a, [b]]\n---\n", 2, "nested lists"),
        ("---\nkey: [a,, b]\n---\n", 2, "empty list item"),
        ("---\nkey: [a, ]\n---\n", 2, "empty list item"),
        ("---\nkey: [a #c, b]\n---\n", 2, "must end with `]`"),
        ("---\nkey: \"quoted\"\n---\n", 2, "starts with `\"`"),
        ("---\nkey: 'quoted'\n---\n", 2, "starts with `'`"),
        ("---\nkey: |\n---\n", 2, "starts with `|`"),
        ("---\nkey: >\n---\n", 2, "starts with `>`"),
        ("---\nkey: &anchor x\n---\n", 2, "starts with `&`"),
        ("---\nkey: *alias\n---\n", 2, "starts with `*`"),
        ("---\nkey: !tag x\n---\n", 2, "starts with `!`"),
        ("---\nkey: {a: b}\n---\n", 2, "starts with `{`"),
        ("---\nkey: [\"a\"]\n---\n", 2, "starts with `\"`"),
        ("---\nkey: a: b\n---\n", 2, "contains `: `"),
        ("---\nkey: [a: b]\n---\n", 2, "contains `: `"),
        ("---\nkey: - item\n---\n", 2, "starts with `- `"),
        ("---\nkey: -\n---\n", 2, "starts with `- `"),
        ("---\nkey: ? question\n---\n", 2, "starts with `? `"),
        ("---\nkey: ?\tq\n---\n", 2, "starts with `? `"),
        ("---\nkey: : x\n---\n", 2, "starts with `: `"),
        ("---\nkey: Ends with colon:\n---\n", 2, "ends with `:`"),
        ("---\nkey: x: # comment\n---\n", 2, "ends with `:`"),
        ("---\nkey: [a:, b]\n---\n", 2, "ends with `:`"),
        ("---\nkey: [- a]\n---\n", 2, "starts with `- `"),
        ("---\nkey: [a, ? b]\n---\n", 2, "starts with `? `"),
        ("---\nkey: [:]\n---\n", 2, "starts with `: `"),
        ("---\nkey: ]x\n---\n", 2, "starts with `]`"),
        ("---\nkey: ,x\n---\n", 2, "starts with `,`"),
        ("---\nkey: [a{b}]\n---\n", 2, "must not contain"),
        ("---\nkey: [a, b}]\n---\n", 2, "must not contain"),
        ("---\nkey: [{a}]\n---\n", 2, "must not contain"),
        ("---\nkey: a\u{1}b\n---\n", 2, "U+0001"),
        ("---\nkey: a\u{0}b\n---\n", 2, "U+0000"),
        ("---\nkey: x\u{b}\n---\n", 2, "U+000B"),
        ("---\nkey: a\u{c}b\n---\n", 2, "U+000C"),
        ("---\nkey: a\rb\n---\n", 2, "U+000D"),
        ("---\nkey: a\r \n---\n", 2, "U+000D"),
        ("---\nkey: a\u{7f}b\n---\n", 2, "U+007F"),
        ("---\nkey: a\u{85}b\n---\n", 2, "U+0085"),
        ("---\nkey: a\u{9f}b\n---\n", 2, "U+009F"),
        ("---\nkey: a\u{2028}b\n---\n", 2, "U+2028"),
        ("---\nkey: [a, b\u{2029}]\n---\n", 2, "U+2029"),
        ("---\nkey: a\u{fffe}b\n---\n", 2, "U+FFFE"),
        ("---\nkey: a\u{ffff}\n---\n", 2, "U+FFFF"),
        ("---\nkey: a  # c\u{1}\n---\n", 2, "U+0001"),
        ("---\nk\u{1}ey: a\n---\n", 2, "U+0001"),
        ("---\n\u{a0}key: a\n---\n", 2, "not snake_case"),
        ("---\nkey: x\n\u{a0}\n---\n", 3, "expected `key: value`"),
        (
            "---\nkey:\u{a0}x\n---\n",
            2,
            "expected a space after `key:`",
        ),
    ];
    for (text, line, message) in cases {
        let error = parse_err(text);
        assert_eq!(error.line, *line, "{text:?}: {error}");
        assert!(error.message.contains(message), "{text:?}: {error}");
    }
}

#[test]
fn integer_and_date_forms() {
    assert!(is_integer("0") && is_integer("42"));
    assert!(!is_integer("") && !is_integer("-1") && !is_integer("1.0") && !is_integer("x"));
    assert!(is_date("2026-11-08") && is_date("2026-13-99"));
    for text in ["YYYY-MM-DD", "2026-1-08", "2026/11/08", "2026-11-080", ""] {
        assert!(!is_date(text), "{text}");
    }
}

// ---------------------------------------------------------------- properties

/// Text allowed as a scalar or list item by construction: no `#` after a space, no `: `, no
/// forbidden first character, no commas or brackets, no space or tab at the edges. Non-ASCII
/// whitespace (U+00A0, U+3000) and other non-ASCII text may appear anywhere, edges included.
fn plain_text() -> impl Strategy<Value = String> {
    "[A-Za-z0-9\u{a0}\u{3000}\u{e9}][A-Za-z0-9 ._()/#:=+\u{a0}\u{3000}\u{e9}\u{1f600}-]{0,24}"
        .prop_filter_map("not representable", |text| {
            let text = text.trim_end_matches(' ').to_string();
            let representable =
                !text.contains(" #") && !text.contains(": ") && !text.ends_with(':');
            representable.then_some(text)
        })
}

fn value() -> impl Strategy<Value = Value> {
    prop_oneof![
        plain_text().prop_map(Value::Scalar),
        proptest::collection::vec(plain_text(), 0..5).prop_map(Value::List),
    ]
}

fn fields() -> impl Strategy<Value = Vec<(String, Value)>> {
    proptest::collection::btree_map("[a-z][a-z0-9_]{0,12}", value(), 0..8)
        .prop_map(|map| map.into_iter().collect())
}

/// Renders fields as frontmatter text, with the given spacing and optional comments.
fn render(fields: &[(String, Value)], spacing: &[(usize, bool)]) -> String {
    let mut text = String::from("---\n");
    for (index, (key, value)) in fields.iter().enumerate() {
        let (spaces, comment) = spacing.get(index).copied().unwrap_or((1, false));
        let rendered = match value {
            Value::Scalar(text) => text.clone(),
            Value::List(items) => format!("[{}]", items.join(&format!(",{}", " ".repeat(spaces)))),
        };
        let _ = write!(text, "{key}:{}{rendered}", " ".repeat(spaces));
        if comment {
            text.push_str("  # a comment: with [brackets], quotes \" and #");
        }
        text.push('\n');
    }
    text.push_str("---\nBody text.\n");
    text
}

/// Lines outside the subset, one per rejection rule.
fn invalid_line() -> impl Strategy<Value = String> {
    let key = "[a-z][a-z0-9_]{0,8}";
    prop_oneof![
        Just(String::new()),
        key.prop_map(|key| format!("  {key}: x")),
        key.prop_map(|key| format!("# {key}")),
        key.prop_map(|key| format!("{key} no colon")),
        "[A-Z][A-Za-z]{0,8}".prop_map(|key| format!("{key}: x")),
        key.prop_map(|key| format!("{key}:x")),
        key.prop_map(|key| format!("{key}:")),
        key.prop_map(|key| format!("{key}: [a, b")),
        key.prop_map(|key| format!("{key}: [a, [b]]")),
        key.prop_map(|key| format!("{key}: [a,,b]")),
        (key, proptest::sample::select(FORBIDDEN_START.to_vec()))
            .prop_map(|(key, c)| format!("{key}: {c}x")),
        key.prop_map(|key| format!("{key}: a: b")),
        key.prop_map(|key| format!("- {key}")),
        (key, proptest::sample::select(vec!['-', '?']))
            .prop_map(|(key, c)| format!("{key}: {c} x")),
        (key, proptest::sample::select(vec!['-', '?', ':']))
            .prop_map(|(key, c)| format!("{key}: [a, {c}]")),
        key.prop_map(|key| format!("{key}: x:")),
        key.prop_map(|key| format!("{key}: [a:, b]")),
        (key, proptest::sample::select(vec!['{', '}']))
            .prop_map(|(key, c)| format!("{key}: [a, b{c}c]")),
        (key, proptest::sample::select(REJECTED_CHARACTERS.to_vec()))
            .prop_map(|(key, c)| format!("{key}: a{c}b")),
        (key, proptest::sample::select(REJECTED_CHARACTERS.to_vec()))
            .prop_map(|(key, c)| format!("{key}: [a, b{c}]")),
    ]
}

/// One sample per class of character YAML rejects or reads as a line break: C0 controls (not
/// tab, not LF), DEL, C1 controls (U+0085 included), U+2028, U+2029, U+FFFE, U+FFFF.
const REJECTED_CHARACTERS: &[char] = &[
    '\u{0}', '\u{1}', '\u{b}', '\u{c}', '\r', '\u{1b}', '\u{7f}', '\u{80}', '\u{85}', '\u{9f}',
    '\u{2028}', '\u{2029}', '\u{fffe}', '\u{ffff}',
];

/// Input for the arbitrary-input property: any text, or text dense in the characters that matter
/// (indicators, ASCII and non-ASCII whitespace, rejected characters).
fn arbitrary_line() -> impl Strategy<Value = String> {
    let dense = "[a-z:#,\\[\\]{}?\\- \t\u{a0}\u{3000}\u{0}\u{1}\u{b}\u{c}\r\u{7f}\u{85}\u{9f}\u{2028}\u{2029}\u{fffe}\u{ffff}]{0,24}";
    prop_oneof![
        ".{0,40}",
        "[a-z]{1,4}: "
            .prop_flat_map(move |prefix| { dense.prop_map(move |rest| format!("{prefix}{rest}")) })
    ]
}

proptest! {
    /// Verifies: XTASK-R020
    #[test]
    fn generated_valid_frontmatter_parses_back_to_the_same_data(
        fields in fields(),
        spacing in proptest::collection::vec((1usize..4, any::<bool>()), 8),
    ) {
        let text = render(&fields, &spacing);
        let parsed = parse(&text).map_err(|error| TestCaseError::fail(format!("{error}\n{text}")))?;
        let parsed: Vec<(String, Value)> =
            parsed.fields.into_iter().map(|field| (field.key, field.value)).collect();
        prop_assert_eq!(parsed, fields);
    }

    /// Verifies: XTASK-R020
    #[test]
    fn an_invalid_line_is_rejected_at_its_line_never_misparsed(
        fields in fields(),
        line in invalid_line(),
        position in any::<proptest::sample::Index>(),
    ) {
        let mut lines: Vec<String> = render(&fields, &[]).lines().map(String::from).collect();
        // Insert between the opening `---` (index 0) and the closing `---`.
        let closing = lines.iter().skip(1).position(|l| l == "---").unwrap() + 1;
        let at = 1 + position.index(closing);
        lines.insert(at, line.clone());
        let text = lines.join("\n");
        match parse(&text) {
            Err(error) => prop_assert_eq!(error.line, at + 1, "{} in\n{}", error, text),
            Ok(parsed) => prop_assert!(false, "accepted {:?} as {:?}", line, parsed),
        }
    }

    /// Arbitrary input never panics, and accepted fields satisfy the subset's key and text rules.
    #[test]
    fn arbitrary_lines_never_panic_and_accepted_output_is_in_the_subset(line in arbitrary_line()) {
        if let Ok(parsed) = parse(&format!("---\n{line}\n---\n")) {
            for field in parsed.fields {
                prop_assert!(is_key(&field.key));
                let texts = match &field.value {
                    Value::Scalar(text) => vec![text.clone()],
                    Value::List(items) => items.clone(),
                };
                for text in texts {
                    prop_assert!(!text.is_empty(), "{:?}", text);
                    prop_assert!(
                        !text.starts_with([' ', '\t']) && !text.ends_with([' ', '\t']),
                        "{:?}", text
                    );
                    // Rust's `char::is_control` is general category Cc: C0, DEL and C1.
                    prop_assert!(
                        !text.chars().any(|c| (c.is_control() && c != '\t')
                            || matches!(c, '\u{2028}' | '\u{2029}' | '\u{fffe}' | '\u{ffff}')),
                        "{:?}", text
                    );
                    prop_assert!(!text.contains(": ") && !text.contains(" #"));
                    prop_assert!(!text.starts_with(FORBIDDEN_START));
                    if matches!(field.value, Value::List(_)) {
                        prop_assert!(!text.contains(['[', ']', '{', '}', ',']), "{:?}", text);
                    }
                    prop_assert!(!text.ends_with(':'));
                    let mut chars = text.chars();
                    let first = chars.next();
                    let second = chars.next();
                    prop_assert!(
                        !(matches!(first, Some('-' | '?' | ':'))
                            && second.is_none_or(|c| c == ' ' || c == '\t')),
                        "{:?}", text
                    );
                }
            }
        }
    }
}
