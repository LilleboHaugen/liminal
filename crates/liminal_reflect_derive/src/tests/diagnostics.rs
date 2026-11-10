//! Diagnostics of rejected inputs (REFLD-R050, REFLD-R051, REFLD-R052, spec §8, INV-2).
//!
//! Each expected span is the hand-written line (1-based) and column (0-based) at which the
//! token named in §8's "Span" column starts in the input string.

use proc_macro2::TokenStream;

use super::expand;

/// `(message, line, column)` of every `::core::compile_error!` in `output`, in output order.
fn diagnostics(output: TokenStream) -> Vec<(String, usize, usize)> {
    let file: syn::File = syn::parse2(output).expect("the diagnostics parse as items");
    file.items
        .iter()
        .map(|item| {
            let syn::Item::Macro(item) = item else {
                panic!("only compile_error! items are emitted for rejected input (INV-2)");
            };
            let segments: Vec<_> = item.mac.path.segments.iter().collect();
            assert!(item.mac.path.leading_colon.is_some());
            assert_eq!(segments.len(), 2);
            assert_eq!(segments[0].ident, "core");
            assert_eq!(segments[1].ident, "compile_error");
            let start = segments[1].ident.span().start();
            let message: syn::LitStr = syn::parse2(item.mac.tokens.clone()).expect("a string");
            (message.value(), start.line, start.column)
        })
        .collect()
}

fn single(source: &str) -> (String, usize, usize) {
    let found = diagnostics(expand(source));
    assert_eq!(found.len(), 1, "{source}: {found:?}");
    found.into_iter().next().expect("one diagnostic")
}

const HELP_0005: &str = "help: the accepted forms are `#[reflect(type_path = \"a::B\")]` on the \
                         type and `#[reflect(ignore)]` on a field.";
const HELP_0008: &str = "help: put `type_path` on the type and `ignore` on fields.";

/// Verifies: REFLD-R050
#[test]
fn every_code_has_its_message_and_span() {
    let cases: [(&str, String, usize, usize); 9] = [
        (
            "union U { a: u8 }",
            "LIM-REFLD-0001: `#[derive(Reflect)]` does not support unions. help: use a struct or \
             an enum."
                .to_string(),
            1,
            0,
        ),
        (
            "enum Never {}",
            "LIM-REFLD-0002: `#[derive(Reflect)]` needs at least one variant in enum `Never`. \
             help: add a variant, or do not derive `Reflect` for an uninhabited type."
                .to_string(),
            1,
            5,
        ),
        (
            "struct B<'a> { a: &'a u8 }",
            "LIM-REFLD-0003: `#[derive(Reflect)]` does not support lifetime parameter `'a`: \
             reflected types are `'static`. help: store owned data (`String`, `Vec<T>`) instead \
             of references."
                .to_string(),
            1,
            9,
        ),
        (
            "struct G<const W: usize> { a: u8 }",
            "LIM-REFLD-0004: `#[derive(Reflect)]` does not support const parameter `W`. help: \
             use a fixed-size array field `[T; N]` with a literal `N`, or a `Vec<T>`."
                .to_string(),
            1,
            9,
        ),
        (
            "#[reflect(rename = \"x\")] struct S;",
            format!(
                "LIM-REFLD-0005: unknown or malformed attribute `#[reflect(rename = \"x\")]`. {HELP_0005}"
            ),
            1,
            10,
        ),
        (
            "#[reflect(type_path = \"Health\")] struct S;",
            "LIM-REFLD-0006: type path `Health` is not a Rust path with at least two segments. \
             help: write segments of letters, digits and `_` separated by `::`, for example \
             `my_game::Health`."
                .to_string(),
            1,
            22,
        ),
        (
            "#[reflect(type_path = \"a::S\")]\n#[reflect(type_path = \"b::S\")] struct S;",
            "LIM-REFLD-0007: `type_path` is given more than once. help: keep one \
             `#[reflect(type_path = \"…\")]`."
                .to_string(),
            2,
            10,
        ),
        (
            "#[reflect(ignore)] struct S;",
            format!("LIM-REFLD-0008: `#[reflect(ignore)]` is not allowed on a type. {HELP_0008}"),
            1,
            0,
        ),
        (
            "struct S { #[reflect(ignore)] #[reflect(ignore)] a: u8 }",
            "LIM-REFLD-0009: field `a` is marked `#[reflect(ignore)]` more than once. help: keep \
             one `#[reflect(ignore)]`."
                .to_string(),
            1,
            30,
        ),
    ];
    for (source, message, line, column) in cases {
        assert_eq!(single(source), (message, line, column), "{source}");
    }
}

/// Verifies: REFLD-R050
#[test]
fn malformed_attribute_forms_are_lim_refld_0005() {
    let malformed = |tokens: &str| {
        format!(
            "LIM-REFLD-0005: unknown or malformed attribute `#[reflect({tokens})]`. {HELP_0005}"
        )
    };
    let cases = [
        ("#[reflect] struct S;", malformed(""), 1, 2),
        ("#[reflect = \"x\"] struct S;", malformed("= \"x\""), 1, 10),
        ("#[reflect()] struct S;", malformed(""), 1, 9),
        (
            "#[reflect(type_path)] struct S;",
            malformed("type_path"),
            1,
            10,
        ),
        (
            "#[reflect(type_path = 3)] struct S;",
            malformed("type_path = 3"),
            1,
            10,
        ),
        (
            "#[reflect(type_path = a::b)] struct S;",
            malformed("type_path = a :: b"),
            1,
            10,
        ),
        (
            "#[reflect(ignore = true)] struct S;",
            malformed("ignore = true"),
            1,
            10,
        ),
        ("#[reflect(, ,)] struct S;", malformed(", ,"), 1, 10),
        (
            "struct S { #[reflect(skip)] a: u8 }",
            malformed("skip"),
            1,
            21,
        ),
    ];
    for (source, message, line, column) in cases {
        assert_eq!(single(source), (message, line, column), "{source}");
    }
}

/// Verifies: REFLD-R050
#[test]
fn misplaced_attributes_are_lim_refld_0008() {
    let misplaced = |key: &str, place: &str| {
        format!("LIM-REFLD-0008: `#[reflect({key})]` is not allowed on {place}. {HELP_0008}")
    };
    let cases = [
        (
            "enum E { #[reflect(ignore)] A }",
            misplaced("ignore", "a variant"),
            1,
            9,
        ),
        (
            "enum E { #[reflect(type_path = \"a::B\")] A }",
            misplaced("type_path", "a variant"),
            1,
            9,
        ),
        (
            "struct S { #[reflect(type_path = \"a::B\")] a: u8 }",
            misplaced("type_path", "a field"),
            1,
            11,
        ),
        (
            "struct S<#[reflect(ignore)] T> { a: T }",
            misplaced("ignore", "a generic parameter"),
            1,
            9,
        ),
        (
            "struct S { a: u8, #[reflect(ignore, ignore)] b: u8 }",
            String::new(),
            0,
            0,
        ),
    ];
    for (source, message, line, column) in &cases[..4] {
        assert_eq!(
            single(source),
            (message.clone(), *line, *column),
            "{source}"
        );
    }
    // Two `ignore` items in one attribute: LIM-REFLD-0009, spanned at that attribute.
    let (message, line, column) = single(cases[4].0);
    assert!(
        message.starts_with("LIM-REFLD-0009: field `b` "),
        "{message}"
    );
    assert_eq!((line, column), (1, 18));
}

/// Verifies: REFLD-R051
#[test]
fn all_diagnostics_are_reported_in_source_order() {
    let source = "#[reflect(ignore)]\n#[reflect(type_path = \"x\")]\n\
                  struct S<'a, const N: usize> {\n\
                  #[reflect(type_path = \"a::b\")] a: u8,\n\
                  #[reflect(ignore)] #[reflect(ignore)] b: u8,\n\
                  }";
    let found = diagnostics(expand(source));
    let codes: Vec<_> = found
        .iter()
        .map(|(message, line, column)| (message[..14].to_string(), *line, *column))
        .collect();
    let expected = [
        ("LIM-REFLD-0008", 1, 0),
        ("LIM-REFLD-0006", 2, 22),
        ("LIM-REFLD-0003", 3, 9),
        ("LIM-REFLD-0004", 3, 13),
        ("LIM-REFLD-0008", 4, 0),
        ("LIM-REFLD-0009", 5, 19),
    ];
    let expected: Vec<_> = expected
        .iter()
        .map(|(code, line, column)| ((*code).to_string(), *line, *column))
        .collect();
    assert_eq!(codes, expected);

    let union = "#[reflect(bad)] union U<'a> { a: &'a u8 }";
    let found = diagnostics(expand(union));
    let codes: Vec<_> = found.iter().map(|(message, ..)| &message[..14]).collect();
    assert_eq!(
        codes,
        ["LIM-REFLD-0005", "LIM-REFLD-0001", "LIM-REFLD-0003"]
    );
}

/// Verifies: REFLD-R052
#[test]
fn non_typed_field_types_get_no_derive_error() {
    let output = expand("struct S { a: NotReflected, b: ::std::cell::Cell<u8> }");
    assert!(!output.to_string().contains("compile_error"));
    let text = output.to_string();
    assert!(
        text.contains("TypeRef :: of :: < NotReflected > ()"),
        "{text}"
    );
}

/// Input that is not an item at all: the parse error of `syn` is reported as is.
#[test]
fn non_item_input_reports_the_parse_error() {
    let found = diagnostics(expand("fn f() {}"));
    assert_eq!(found.len(), 1);
    assert!(!found[0].0.starts_with("LIM-REFLD"), "{found:?}");
}
