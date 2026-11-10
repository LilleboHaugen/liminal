//! The derive's generated code is not broken by local items that shadow the names it could use
//! (REFLD-R003 test 2).
//!
//! Rule for this file: it names no crate other than `liminal_reflect`, `core` and `std` (no path,
//! `use` or `extern crate` naming `liminal_reflect_derive`, `liminal_core` or `proptest`). It
//! cannot show that a dependency is absent: an integration test is compiled with every
//! dependency and dev-dependency of `liminal_reflect` available. REFLD-R003 test 1 (unit test of
//! `expand` in `liminal_reflect_derive`) checks the expansion itself.

#![allow(dead_code, non_camel_case_types)]

struct Reflect;
struct Typed;
struct TypeInfo;
struct Shape;
struct Option;
struct Some;
struct None;
struct Result;
struct Ok;
struct Err;
struct Vec;
struct String;

#[derive(liminal_reflect::value::Reflect)]
struct Named {
    a: u8,
    list: ::std::vec::Vec<u16>,
    text: ::std::string::String,
    maybe: ::core::option::Option<i32>,
}

#[derive(liminal_reflect::value::Reflect)]
struct Tuple(u8, #[reflect(ignore)] Shape, f64);

#[derive(liminal_reflect::value::Reflect)]
struct Unit;

#[derive(liminal_reflect::value::Reflect)]
enum Mixed<T> {
    A,
    B(T, u32),
    C { x: bool },
}

/// Verifies: REFLD-R003
#[test]
fn derives_compile_with_shadowed_names() {
    use liminal_reflect::value::Typed as _;

    let mut registry = liminal_reflect::registry::TypeRegistry::new();
    assert!(registry.register::<Named>().is_ok());
    assert!(registry.register::<Tuple>().is_ok());
    assert!(registry.register::<Unit>().is_ok());
    assert!(registry.register::<Mixed<u8>>().is_ok());
    assert_eq!(Named::type_path(), "derive_shadowing::Named");
    let value = Mixed::<u8>::B(1, 2);
    assert_eq!(
        liminal_reflect::value::validate_reflect(&value),
        ::core::result::Result::Ok(())
    );
}
