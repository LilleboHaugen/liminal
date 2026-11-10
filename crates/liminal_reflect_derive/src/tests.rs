//! Unit tests of `expand` on `proc_macro2` tokens parsed from strings (spec §10).

mod diagnostics;
mod expansion;
mod paths;

use std::str::FromStr;

use proc_macro2::TokenStream;

/// The test inputs of REFLD-R003 test 1 and REFLD-R070: every accepted kind, field types that
/// are primitive types other than `String` or the input's own type parameters.
const ACCEPTED: [&str; 6] = [
    "struct Health { current: u32, max: u32, r#type: bool }",
    "struct Pos(f32, #[reflect(ignore)] u8, f64);",
    "struct Unit;",
    "enum Figure { Empty, Point(f32, f32), Rect { w: f32, h: f64 }, r#Raw { c: char } }",
    "#[reflect(type_path = \"my::Pair\")] struct Pair<T: Copy, U> where U: Clone { a: T, b: U, c: i128 }",
    "enum Toggle<T> { Off, On(T) }",
];

/// Parses `source` into tokens.
fn tokens(source: &str) -> TokenStream {
    TokenStream::from_str(source).expect("test input is valid Rust tokens")
}

/// `expand` on a source string.
fn expand(source: &str) -> TokenStream {
    crate::expand(tokens(source))
}
