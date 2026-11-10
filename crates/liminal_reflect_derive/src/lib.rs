//! `#[derive(Reflect)]` for Liminal (`docs/specs/liminal_reflect_derive.md`).
//!
//! Users do not depend on this crate: `liminal_reflect::value::Reflect` re-exports the macro next
//! to the trait of the same name, and its documentation has the examples (REFL-R120).
//!
//! The entry point only converts token streams. Parsing and validation (`parse`) and generation
//! (`generate`) operate on `proc_macro2` types, so the unit tests run them outside the compiler.

#![forbid(unsafe_code)]

mod generate;
mod parse;

#[cfg(test)]
mod tests;

/// Derives `Reflect`, `Typed` and `ReflectFields` (and `ReflectEnum` for enums) of
/// `liminal_reflect`.
///
/// Accepted input, attributes (`#[reflect(type_path = "a::B")]` on the type,
/// `#[reflect(ignore)]` on a field) and the compile errors `LIM-REFLD-0001` to `LIM-REFLD-0009`
/// are specified in `docs/specs/liminal_reflect_derive.md`. Examples: the re-export
/// `liminal_reflect::value::Reflect`.
#[proc_macro_derive(Reflect, attributes(reflect))]
pub fn derive_reflect(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand(input.into()).into()
}

/// The whole derive on `proc_macro2` tokens: the impls for an accepted input, else every
/// diagnostic as `::core::compile_error!` invocations and nothing else (spec INV-2).
fn expand(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    match parse::parse(input) {
        Ok(model) => generate::generate(&model),
        Err(errors) => errors.iter().map(syn::Error::to_compile_error).collect(),
    }
}
