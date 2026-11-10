//! `liminal_reflect::value::Reflect` names the trait and the derive macro (REFL-R120).
//!
//! Rule for this file: it names no crate other than `liminal_reflect` (and `core` / `std`): no
//! path, `use` or `extern crate` naming `liminal_reflect_derive`, `liminal_core`, `proptest` or
//! any other crate. This shows that a user's source needs to name `liminal_reflect` only; it does
//! not show that a dependency is absent (every dev-dependency is available to this file).

use liminal_reflect::hash::reflect_state_hash;
use liminal_reflect::registry::TypeRegistry;
use liminal_reflect::value::{Reflect, Typed};

#[derive(Reflect)]
struct S {
    a: u8,
}

/// Verifies: REFL-R120
#[test]
fn one_import_gives_trait_and_derive() {
    let value = S { a: 7 };
    // The encoding of a one-field struct is the field's encoding (REFL-R100).
    assert_eq!(reflect_state_hash(&value), reflect_state_hash(&7_u8));
    let mut registry = TypeRegistry::new();
    assert_eq!(registry.register::<S>(), Ok(S::stable_type_id()));
    let r: &dyn Reflect = &value;
    assert!(matches!(
        r.field("a").map(|f| f.downcast_ref::<u8>().copied()),
        Ok(Ok(7))
    ));
}
