//! Integration tests of `liminal_core::error` through the public API (spec §10, CORE-R001 to
//! CORE-R013). Oracles: a test-local predicate for the code grammar, a test-local JSON decoder
//! (`common/json.rs`), hand-written expected values, and the approved snapshot
//! `tests/snapshots/core_errors.txt`.

#[path = "common/json.rs"]
mod json;

use json::Json;
use liminal_core::error::{
    ContextValue, CoreError, EngineError, ErrorCode, ErrorContext, render_json, validate_error,
    write_display,
};
use proptest::prelude::*;

/// An error whose every part is set by the test, including a `Display` that may break the
/// contract.
#[derive(Clone, Debug)]
struct TestError {
    code: ErrorCode,
    message: String,
    help: Option<String>,
    context: ErrorContext,
    /// `None`: `Display` is `write_display`, as the contract requires.
    display: Option<String>,
}

impl TestError {
    fn new(message: &str) -> TestError {
        TestError {
            code: ErrorCode::new("TEST", 7),
            message: message.to_string(),
            help: None,
            context: ErrorContext::new(),
            display: None,
        }
    }

    fn help(mut self, help: &str) -> TestError {
        self.help = Some(help.to_string());
        self
    }

    fn context(mut self, context: ErrorContext) -> TestError {
        self.context = context;
        self
    }

    fn display(mut self, display: &str) -> TestError {
        self.display = Some(display.to_string());
        self
    }
}

impl EngineError for TestError {
    fn code(&self) -> ErrorCode {
        self.code
    }
    fn message(&self) -> String {
        self.message.clone()
    }
    fn help(&self) -> Option<String> {
        self.help.clone()
    }
    fn context(&self) -> ErrorContext {
        self.context.clone()
    }
}

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.display {
            Some(display) => f.write_str(display),
            None => write_display(self, f),
        }
    }
}

impl std::error::Error for TestError {}

/// One value of every `CoreError` variant, in code order. The unit test
/// `error::core_error::tests` checks that no variant is missing from its own list; this list
/// must match the snapshot.
fn core_errors() -> Vec<CoreError> {
    let code = ErrorCode::new("ECS", 3);
    vec![
        CoreError::InvalidErrorCode {
            text: "LIM-ecs-3".to_string(),
        },
        CoreError::InvalidContextKey {
            code,
            key: "Entity ID".to_string(),
        },
        CoreError::DuplicateContextKey {
            code,
            key: "entity".to_string(),
        },
        CoreError::EmptyMessage { code },
        CoreError::EmptyHelp { code },
        CoreError::DisplayMismatch {
            code,
            expected: "LIM-ECS-0003: component \"Health\" is not registered.".to_string(),
            actual: "not registered\n".to_string(),
        },
        CoreError::AreaMismatch {
            code,
            expected_area: "PHYS".to_string(),
        },
        CoreError::StaleHandle {
            index: 3,
            generation: 2,
        },
        CoreError::HandleSlotsExhausted { slot_limit: 4096 },
        CoreError::InvalidTickRate {
            ticks_per_second: 1001,
        },
        CoreError::ZeroRngState,
    ]
}

/// The CORE-R001 grammar, written without reference to the implementation.
fn is_valid_code_text(text: &str) -> bool {
    let Some(rest) = text.strip_prefix("LIM-") else {
        return false;
    };
    let parts: Vec<&str> = rest.split('-').collect();
    let [area, digits] = parts.as_slice() else {
        return false;
    };
    (1..=8).contains(&area.chars().count())
        && area.chars().all(|c| c.is_ascii_uppercase())
        && digits.chars().count() == 4
        && digits.chars().all(|c| c.is_ascii_digit())
        && *digits != "0000"
}

/// Strings that are near the code grammar: random mixes of its pieces and a few foreign
/// characters. About one in a few is valid, so both outcomes of `parse` are exercised.
fn code_like_text() -> impl Strategy<Value = String> {
    let piece = prop_oneof![
        4 => Just("LIM-".to_string()),
        4 => "[A-Z]{1,9}",
        4 => "[0-9]{3,5}",
        3 => Just("-".to_string()),
        1 => Just("0000".to_string()),
        1 => "[a-z ]{1,2}",
        1 => any::<char>().prop_map(|c| c.to_string()),
    ];
    prop_oneof![
        proptest::collection::vec(piece, 0..5).prop_map(|pieces| pieces.concat()),
        ("[A-Z]{0,9}", "[0-9]{4}").prop_map(|(area, digits)| format!("LIM-{area}-{digits}")),
    ]
}

/// Strings with every kind of character CORE-R009 distinguishes.
fn any_text() -> impl Strategy<Value = String> {
    let character = prop_oneof![
        4 => any::<char>(),
        2 => (0_u32..0x20).prop_map(|code| char::from_u32(code).expect("test setup: below 0x20 is a char")),
        1 => proptest::sample::select(vec!['"', '\\', '/', '\u{7F}', '\u{80}', '\u{2028}', 'a']),
    ];
    proptest::collection::vec(character, 0..24).prop_map(|chars| chars.into_iter().collect())
}

fn any_value() -> impl Strategy<Value = ContextValue> {
    prop_oneof![
        any::<bool>().prop_map(ContextValue::Bool),
        any::<i64>().prop_map(ContextValue::Int),
        any::<u64>().prop_map(ContextValue::UInt),
        any::<f64>().prop_map(ContextValue::Float),
        any::<u64>().prop_map(|bits| ContextValue::Float(f64::from_bits(bits))),
        any_text().prop_map(ContextValue::Text),
    ]
}

/// Context keys, including ones that break the contract: `render_json` renders them anyway.
const KEYS: [&str; 8] = [
    "index",
    "name",
    "slot_limit",
    "x9",
    "",
    "has space",
    "quote\"back\\slash",
    "line\nbreak\u{1}",
];

fn any_context() -> impl Strategy<Value = ErrorContext> {
    proptest::collection::vec((proptest::sample::select(KEYS.to_vec()), any_value()), 0..6)
        .prop_map(|fields| {
            fields
                .into_iter()
                .fold(ErrorContext::new(), |context, (key, value)| {
                    context.with(key, value)
                })
        })
}

/// The JSON value CORE-R010 requires for a context value, built without `render_json`.
fn expected_json(value: &ContextValue) -> Json {
    match value {
        ContextValue::Bool(flag) => Json::Bool(*flag),
        ContextValue::Int(int) => Json::Number(int.to_string()),
        ContextValue::UInt(uint) => Json::Number(uint.to_string()),
        ContextValue::Float(float) if float.is_finite() => Json::Number(float.to_string()),
        ContextValue::Float(_) => Json::Null,
        ContextValue::Text(text) => Json::Text(text.clone()),
    }
}

proptest! {
    /// Verifies: CORE-R001
    #[test]
    fn parse_accepts_exactly_the_grammar(text in code_like_text()) {
        let parsed = ErrorCode::parse(&text);
        if is_valid_code_text(&text) {
            let code = parsed.unwrap();
            prop_assert_eq!(code.to_string(), text);
        } else {
            prop_assert_eq!(parsed, Err(CoreError::InvalidErrorCode { text: text.clone() }));
        }
    }

    /// Verifies: CORE-R002
    #[test]
    fn display_and_parse_round_trip(area in "[A-Z]{1,8}", number in 1_u16..=9999) {
        let text = format!("LIM-{area}-{number:04}");
        let code = ErrorCode::parse(&text).unwrap();
        prop_assert_eq!(code.to_string(), text);
        prop_assert_eq!(code.area(), area.as_str());
        prop_assert_eq!(code.number(), number);
        prop_assert_eq!(ErrorCode::parse(&code.to_string()), Ok(code));
    }

    /// Verifies: CORE-R004
    #[test]
    fn order_is_area_bytes_then_number(
        a in ("[A-Z]{1,8}", 1_u16..=9999),
        b in ("[A-C]{1,3}", 1_u16..=3),
        c in ("[A-C]{1,3}", 1_u16..=3),
    ) {
        for (left, right) in [(&a, &b), (&b, &c), (&c, &a), (&b, &b)] {
            let left_code = ErrorCode::parse(&format!("LIM-{}-{:04}", left.0, left.1)).unwrap();
            let right_code = ErrorCode::parse(&format!("LIM-{}-{:04}", right.0, right.1)).unwrap();
            let expected = (left.0.as_bytes(), left.1).cmp(&(right.0.as_bytes(), right.1));
            prop_assert_eq!(left_code.cmp(&right_code), expected);
            prop_assert_eq!(left_code == right_code, expected.is_eq());
        }
    }

    /// Verifies: CORE-R006
    #[test]
    fn display_is_code_message_help(message in any_text(), help in proptest::option::of(any_text())) {
        let mut error = TestError::new(&message);
        error.help.clone_from(&help);
        let expected = match &help {
            Some(help) => format!("LIM-TEST-0007: {message} help: {help}"),
            None => format!("LIM-TEST-0007: {message}"),
        };
        prop_assert_eq!(error.to_string(), expected);
    }

    /// Verifies: CORE-R008, CORE-R009, CORE-R010
    #[test]
    fn json_decodes_to_the_error_parts(
        message in any_text(),
        help in proptest::option::of(any_text()),
        context in any_context(),
    ) {
        let mut error = TestError::new(&message).context(context.clone());
        error.help.clone_from(&help);
        let rendered = render_json(&error);

        // One line, and no control character anywhere (they must be escaped, CORE-R009).
        prop_assert!(rendered.chars().all(|c| u32::from(c) >= 0x20), "{:?}", rendered);
        let expected = Json::Object(vec![
            ("code".to_string(), Json::Text("LIM-TEST-0007".to_string())),
            ("message".to_string(), Json::Text(message.clone())),
            ("help".to_string(), help.clone().map_or(Json::Null, Json::Text)),
            (
                "context".to_string(),
                Json::Object(
                    context.iter().map(|(key, value)| (key.to_string(), expected_json(value))).collect(),
                ),
            ),
        ]);
        // The strict decoder rejects whitespace outside strings and raw control characters.
        prop_assert_eq!(json::parse(&rendered), Ok(expected));
    }

    /// Verifies: CORE-R009
    #[test]
    fn json_strings_escape_only_what_must_be_escaped(text in any_text()) {
        let rendered = render_json(&TestError::new(&text));
        let prefix = r#"{"code":"LIM-TEST-0007","message":""#;
        let suffix = r#"","help":null,"context":{}}"#;
        prop_assert!(rendered.starts_with(prefix) && rendered.ends_with(suffix));
        let body = &rendered[prefix.len()..rendered.len() - suffix.len()];
        // Escapes exist only for `"`, `\` and U+0000..=U+001F, so the body grows by exactly:
        // 1 byte for `"`, `\`, \b, \f, \n, \r, \t; 5 bytes for the other control characters.
        let growth: usize = text
            .chars()
            .map(|c| match c {
                '"' | '\\' | '\u{08}' | '\u{0C}' | '\n' | '\r' | '\t' => 1,
                c if u32::from(c) < 0x20 => 5,
                _ => 0,
            })
            .sum();
        prop_assert_eq!(body.len(), text.len() + growth);
    }

    /// Verifies: CORE-R010
    #[test]
    fn json_numbers_parse_back_to_the_value(int in any::<i64>(), uint in any::<u64>(), bits in any::<u64>()) {
        let float = f64::from_bits(bits);
        let context = ErrorContext::new().with("int", int).with("uint", uint).with("float", float);
        let rendered = render_json(&TestError::new("m").context(context));
        let Ok(Json::Object(members)) = json::parse(&rendered) else {
            return Err(TestCaseError::fail(format!("not an object: {rendered}")));
        };
        let Json::Object(fields) = &members[3].1 else {
            return Err(TestCaseError::fail(format!("no context object: {rendered}")));
        };
        prop_assert_eq!(&fields[0].1, &Json::Number(int.to_string()));
        prop_assert_eq!(&fields[1].1, &Json::Number(uint.to_string()));
        match &fields[2].1 {
            Json::Number(text) => {
                prop_assert!(float.is_finite());
                prop_assert!(!text.contains(['e', 'E', '+']), "{}", text);
                prop_assert_eq!(text.parse::<f64>().unwrap().to_bits(), bits);
            }
            Json::Null => prop_assert!(!float.is_finite()),
            other => prop_assert!(false, "{:?}", other),
        }
    }

    /// Verifies: CORE-R011
    #[test]
    fn rendering_twice_gives_identical_bytes(
        message in any_text(),
        help in proptest::option::of(any_text()),
        context in any_context(),
    ) {
        let mut error = TestError::new(&message).context(context);
        error.help.clone_from(&help);
        let copy = error.clone();
        prop_assert_eq!(render_json(&error), render_json(&copy));
        prop_assert_eq!(render_json(&error), render_json(&error));
        prop_assert_eq!(error.to_string(), copy.to_string());
    }

    /// Verifies: CORE-R012
    #[test]
    fn valid_errors_pass_the_contract(
        message in any_text().prop_filter("non-empty", |m| !m.is_empty()),
        help in proptest::option::of(any_text().prop_filter("non-empty", |h| !h.is_empty())),
        keys in proptest::sample::subsequence(vec!["index", "name", "slot_limit", "x9", "a_", "z"], 0..=6),
        value in any_value(),
    ) {
        let context = keys
            .iter()
            .fold(ErrorContext::new(), |context, key| context.with(key, value.clone()));
        let mut error = TestError::new(&message).context(context);
        error.help.clone_from(&help);
        prop_assert_eq!(validate_error(&error, "TEST"), Ok(()));
    }
}

/// Displays a `&dyn EngineError` through `write_display`.
struct WriteDisplay<'a>(&'a dyn EngineError);

impl std::fmt::Display for WriteDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_display(self.0, f)
    }
}

/// Verifies: CORE-R005
#[test]
fn engine_error_is_dyn_compatible_with_std_error_as_supertrait() {
    let concrete = CoreError::StaleHandle {
        index: 3,
        generation: 2,
    };
    let error: &dyn EngineError = &concrete;

    assert_eq!(error.code(), ErrorCode::new("CORE", 10));
    assert_eq!(error.message(), "handle `3v2` is not live.");
    assert!(
        error
            .help()
            .unwrap()
            .starts_with("the handle was freed already")
    );
    assert_eq!(error.context().len(), 2);

    // Passed as `&dyn EngineError` to the three functions.
    assert_eq!(WriteDisplay(error).to_string(), concrete.to_string());
    assert!(render_json(error).starts_with(r#"{"code":"LIM-CORE-0010","#));
    assert_eq!(validate_error(error, "CORE"), Ok(()));

    // `std::error::Error` is a supertrait: a `dyn EngineError` is usable as a `dyn Error`.
    let as_std: &dyn std::error::Error = error;
    assert_eq!(as_std.to_string(), concrete.to_string());
    assert!(as_std.source().is_none());
    let boxed: Box<dyn EngineError> = Box::new(concrete.clone());
    assert_eq!(boxed.to_string(), concrete.to_string());
}

/// Verifies: CORE-R008
#[test]
fn json_has_the_four_members_in_order_without_whitespace() {
    let minimal = TestError::new("m");
    assert_eq!(
        render_json(&minimal),
        r#"{"code":"LIM-TEST-0007","message":"m","help":null,"context":{}}"#
    );

    let full = TestError::new("it broke.").help("fix it.").context(
        ErrorContext::new()
            .with("index", 3_u32)
            .with("name", "a b")
            .with("ok", false)
            .with("delta", -2_i32)
            .with("ratio", 0.5_f32)
            .with("index", f64::NAN),
    );
    assert_eq!(
        render_json(&full),
        concat!(
            r#"{"code":"LIM-TEST-0007","message":"it broke.","help":"fix it.","#,
            r#""context":{"index":3,"name":"a b","ok":false,"delta":-2,"ratio":0.5,"index":null}}"#
        )
    );
    // `Some("")` is rendered as an empty string, not as null.
    assert_eq!(
        render_json(&TestError::new("m").help("")),
        r#"{"code":"LIM-TEST-0007","message":"m","help":"","context":{}}"#
    );
}

/// Verifies: CORE-R009
#[test]
fn json_string_escapes_in_message_help_keys_and_text_values() {
    let tricky = "q\" b\\ /\u{08}\u{0C}\n\r\t\u{0}\u{1F}\u{7F}é😀";
    let escaped = r#"q\" b\\ /\b\f\n\r\t\u0000\u001f"#.to_string() + "\u{7F}é😀";
    let error = TestError::new(tricky)
        .help(tricky)
        .context(ErrorContext::new().with("quote\"back\\slash", tricky));
    assert_eq!(
        render_json(&error),
        format!(
            r#"{{"code":"LIM-TEST-0007","message":"{escaped}","help":"{escaped}","context":{{"quote\"back\\slash":"{escaped}"}}}}"#
        )
    );
}

/// Verifies: CORE-R012
#[test]
fn validate_error_reports_each_violation_with_its_code() {
    let code = ErrorCode::new("TEST", 7);
    let ok = TestError::new("it broke.")
        .help("fix it.")
        .context(ErrorContext::new().with("index", 1_u32).with("name_2", "x"));
    assert_eq!(validate_error(&ok, "TEST"), Ok(()));
    assert_eq!(
        validate_error(&TestError::new("no help, no context."), "TEST"),
        Ok(())
    );

    // Area: compared as a whole, case-sensitively.
    for area in ["CORE", "TES", "TESTS", "test", ""] {
        assert_eq!(
            validate_error(&ok, area),
            Err(CoreError::AreaMismatch {
                code,
                expected_area: area.to_string()
            })
        );
    }
    assert_eq!(
        validate_error(&TestError::new(""), "TEST"),
        Err(CoreError::EmptyMessage { code })
    );
    assert_eq!(
        validate_error(&TestError::new("m").help(""), "TEST"),
        Err(CoreError::EmptyHelp { code })
    );
    // The first bad key is named, not a later one.
    for (keys, bad) in [
        (vec!["Index"], "Index"),
        (vec!["ok", "9lives", "Also Bad"], "9lives"),
        (vec!["ok", "", "x"], ""),
        (vec!["_x"], "_x"),
        (vec!["a-b"], "a-b"),
        (vec!["naïve"], "naïve"),
    ] {
        let context = keys
            .iter()
            .fold(ErrorContext::new(), |context, key| context.with(key, 1));
        assert_eq!(
            validate_error(&TestError::new("m").context(context), "TEST"),
            Err(CoreError::InvalidContextKey {
                code,
                key: bad.to_string()
            }),
            "{keys:?}"
        );
    }
    // The first repeated key is the first field whose key occurred before.
    for (keys, repeated) in [
        (vec!["a", "a"], "a"),
        (vec!["a", "b", "c", "b", "a"], "b"),
        (vec!["a", "b", "b", "a", "a"], "b"),
        (vec!["x", "a", "b", "c", "a", "x"], "a"),
    ] {
        let context = keys
            .iter()
            .fold(ErrorContext::new(), |context, key| context.with(key, 1));
        assert_eq!(
            validate_error(&TestError::new("m").context(context), "TEST"),
            Err(CoreError::DuplicateContextKey {
                code,
                key: repeated.to_string()
            }),
            "{keys:?}"
        );
    }
    for wrong in [
        "it broke.",
        "LIM-TEST-0007: it broke.",
        "LIM-TEST-0007: it broke. help: fix it.\n",
        "LIM-TEST-0007: it broke.  help: fix it.",
        "",
    ] {
        assert_eq!(
            validate_error(&ok.clone().display(wrong), "TEST"),
            Err(CoreError::DisplayMismatch {
                code,
                expected: "LIM-TEST-0007: it broke. help: fix it.".to_string(),
                actual: wrong.to_string(),
            })
        );
    }
    // A hand-written `Display` that produces the same text passes.
    assert_eq!(
        validate_error(
            &ok.clone().display("LIM-TEST-0007: it broke. help: fix it."),
            "TEST"
        ),
        Ok(())
    );
}

/// Verifies: CORE-R012
#[test]
fn validate_error_checks_in_the_specified_order() {
    let code = ErrorCode::new("TEST", 7);
    // Violates every check. Each step below repairs the violation reported before it.
    let mut error = TestError::new("")
        .help("")
        .context(
            ErrorContext::new()
                .with("dup", 1)
                .with("dup", 2)
                .with("Bad Key", 3),
        )
        .display("wrong");

    assert_eq!(
        validate_error(&error, "OTHER"),
        Err(CoreError::AreaMismatch {
            code,
            expected_area: "OTHER".to_string()
        })
    );
    assert_eq!(
        validate_error(&error, "TEST"),
        Err(CoreError::EmptyMessage { code })
    );
    error.message = "m".to_string();
    assert_eq!(
        validate_error(&error, "TEST"),
        Err(CoreError::EmptyHelp { code })
    );
    error.help = None;
    // Key form is checked for all keys before duplicates, although the duplicate comes first.
    assert_eq!(
        validate_error(&error, "TEST"),
        Err(CoreError::InvalidContextKey {
            code,
            key: "Bad Key".to_string()
        })
    );
    error.context = ErrorContext::new().with("dup", 1).with("dup", 2);
    assert_eq!(
        validate_error(&error, "TEST"),
        Err(CoreError::DuplicateContextKey {
            code,
            key: "dup".to_string()
        })
    );
    error.context = ErrorContext::new().with("dup", 1);
    assert_eq!(
        validate_error(&error, "TEST"),
        Err(CoreError::DisplayMismatch {
            code,
            expected: "LIM-TEST-0007: m".to_string(),
            actual: "wrong".to_string()
        })
    );
    error.display = None;
    assert_eq!(validate_error(&error, "TEST"), Ok(()));
}

/// The approved text and JSON form of one value of every `CoreError` variant (spec §10). The
/// file was written by hand from the table in spec §8, not generated from the implementation.
/// It is an oracle (AGENTS.md rule 5): change it only with the spec.
///
/// Verifies: CORE-R006, CORE-R008, CORE-R011, CORE-R013
#[test]
fn core_errors_match_the_approved_snapshot() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/snapshots/core_errors.txt"
    );
    // `replace`: a Windows checkout may convert line endings.
    let approved = std::fs::read_to_string(path).unwrap().replace("\r\n", "\n");

    let mut actual = String::new();
    for error in core_errors() {
        actual += &error.to_string();
        actual.push('\n');
        actual += &render_json(&error);
        actual.push_str("\n\n");
    }
    assert_eq!(actual, approved);
}

/// Verifies: CORE-R013
#[test]
fn every_core_error_passes_the_contract_through_the_public_api() {
    let errors = core_errors();
    let codes: Vec<String> = errors.iter().map(|e| e.code().to_string()).collect();
    assert_eq!(
        codes,
        [
            "LIM-CORE-0001",
            "LIM-CORE-0002",
            "LIM-CORE-0003",
            "LIM-CORE-0004",
            "LIM-CORE-0005",
            "LIM-CORE-0006",
            "LIM-CORE-0007",
            "LIM-CORE-0010",
            "LIM-CORE-0011",
            "LIM-CORE-0020",
            "LIM-CORE-0030",
        ]
    );
    for error in &errors {
        assert_eq!(validate_error(error, "CORE"), Ok(()), "{error:?}");
        assert!(error.message().ends_with('.'), "{error:?}");
        assert!(json::parse(&render_json(error)).is_ok(), "{error:?}");
    }
}
