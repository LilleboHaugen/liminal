//! The items of an accepted expansion (REFLD-R001, INV-3), field names (REFLD-R004), type-path
//! expressions (REFLD-R020, REFLD-R021) and purity (REFLD-R070).

use quote::ToTokens;

use super::{ACCEPTED, expand};
use crate::parse::is_type_path;

/// The trait names of the impls inside the single `const _` block of `source`'s expansion.
fn impls(source: &str) -> Vec<String> {
    let file: syn::File = syn::parse2(expand(source)).expect("the expansion parses");
    assert_eq!(file.items.len(), 1, "one item: the const block (INV-3)");
    let syn::Item::Const(item) = &file.items[0] else {
        panic!("the item is `const _: () = {{ … }};`");
    };
    assert_eq!(item.ident, "_");
    assert_eq!(item.ty.to_token_stream().to_string(), "()");
    let syn::Expr::Block(block) = item.expr.as_ref() else {
        panic!("a block");
    };
    block
        .block
        .stmts
        .iter()
        .filter_map(|statement| match statement {
            syn::Stmt::Item(syn::Item::Impl(item)) => {
                let (path, _) = item.trait_.as_ref().expect("trait impls only");
                Some(path.to_token_stream().to_string().replace(' ', ""))
            }
            syn::Stmt::Item(syn::Item::Const(_)) => None,
            other => panic!("unexpected statement {}", other.to_token_stream()),
        })
        .collect()
}

/// Verifies: REFLD-R001
#[test]
fn expansion_is_one_block_of_trait_impls() {
    let structs = [
        "::liminal_reflect::value::Reflect",
        "::liminal_reflect::value::Typed",
        "::liminal_reflect::value::ReflectFields",
    ];
    let enums = [
        structs[0],
        structs[1],
        structs[2],
        "::liminal_reflect::value::ReflectEnum",
    ];
    for source in ACCEPTED {
        let expected: &[&str] = if source.contains("enum ") {
            &enums
        } else {
            &structs
        };
        assert_eq!(impls(source), expected, "{source}");
    }
}

/// Verifies: REFLD-R002
#[test]
fn every_type_parameter_gets_the_typed_bound() {
    let text = expand(ACCEPTED[4]).to_string();
    assert!(
        text.contains(
            "impl < T : Copy , U > :: liminal_reflect :: value :: Reflect for Pair < T , U > \
             where U : Clone , T : :: liminal_reflect :: value :: Typed , U : :: liminal_reflect \
             :: value :: Typed"
        ),
        "{text}"
    );
}

/// Verifies: REFLD-R004
#[test]
fn field_names_strip_raw_prefixes_and_count_ignored_positions() {
    let text = expand(ACCEPTED[0]).to_string();
    assert!(
        text.contains("& [\"current\" , \"max\" , \"type\"]"),
        "{text}"
    );
    let text = expand(ACCEPTED[1]).to_string();
    assert!(text.contains("& [\"0\" , \"2\"]"), "{text}");
    assert!(text.contains("& self . 2"), "{text}");
    let text = expand(ACCEPTED[3]).to_string();
    assert!(text.contains("\"Raw\""), "{text}");
}

/// Verifies: REFLD-R020
#[test]
fn default_type_path_expressions() {
    let text = expand(ACCEPTED[0]).to_string();
    assert!(
        text.contains(
            ":: std :: string :: String :: from (:: core :: concat ! (:: core :: module_path ! () , \"::Health\"))"
        ),
        "{text}"
    );
    let text = expand(ACCEPTED[5]).to_string();
    assert!(
        text.contains(
            ":: std :: format ! (\"{}::Toggle<{}>\" , :: core :: module_path ! () , < T as :: liminal_reflect :: value :: Typed > :: type_path ())"
        ),
        "{text}"
    );
}

/// Verifies: REFLD-R021
#[test]
fn type_path_override_and_its_pattern() {
    let text = expand(ACCEPTED[4]).to_string();
    assert!(
        text.contains(":: std :: format ! (\"my::Pair<{}, {}>\""),
        "{text}"
    );
    let text = expand("#[reflect(type_path = \"a::B\")] struct B;").to_string();
    assert!(
        text.contains(":: std :: string :: String :: from (\"a::B\")"),
        "{text}"
    );
    for accepted in ["a::B", "_x::y_1::Z9", "_::_"] {
        assert!(is_type_path(accepted), "{accepted}");
    }
    for rejected in [
        "", "B", "a::", "::a::B", "a:::B", "a::1B", "a::B<u8>", "a ::B", "ä::B", "a::b-c",
    ] {
        assert!(!is_type_path(rejected), "{rejected}");
    }
}

/// Verifies: REFLD-R070
#[test]
fn expansion_is_a_pure_function_of_the_input() {
    let rejected = [
        "union U { a: u8 }",
        "#[reflect(ignore)] #[reflect(type_path = \"x\")] struct S<'a> { a: &'a u8 }",
    ];
    for source in ACCEPTED.iter().chain(&rejected) {
        assert_eq!(
            expand(source).to_string(),
            expand(source).to_string(),
            "{source}"
        );
    }
}
