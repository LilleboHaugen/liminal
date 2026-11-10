//! `ReflectError` against the error contract (spec §8, §10, REFL-R140). Oracles:
//! `liminal_core::error::validate_error` and the approved snapshot
//! `tests/snapshots/reflect_errors.txt`.

use liminal_core::error::{EngineError, render_json, validate_error};
use liminal_reflect::error::ReflectError;
use liminal_reflect::info::StableTypeId;

/// One value of every variant, in code order.
fn reflect_errors() -> Vec<ReflectError> {
    let text = str::to_string;
    vec![
        ReflectError::TypeIdCollision {
            id: StableTypeId::new(0x00ab_cdef_0123_4567),
            existing_path: text("refl_test::TwinA"),
            new_path: text("refl_test::TwinB"),
        },
        ReflectError::TypeIdMismatch {
            type_path: text("refl_test::WrongId"),
            id: StableTypeId::new(1),
            expected_id: StableTypeId::new(0xffff_0000_ffff_0000),
        },
        ReflectError::DuplicateName {
            type_path: text("refl_test::E"),
            name: text("x"),
        },
        ReflectError::EmptyTypePath,
        ReflectError::ShapeMismatch {
            type_path: text("refl_test::Fake"),
            at: text(".items[2].0"),
            expected: text("struct with 2 fields"),
            actual: text("list"),
        },
        ReflectError::NoSuchField {
            type_path: text("refl_test::Sample"),
            field: text("z"),
        },
        ReflectError::TypeMismatch {
            expected: text("i32"),
            actual: text("u32"),
        },
        ReflectError::NoFields {
            type_path: text("Vec<u8>"),
            shape: text("list"),
        },
        ReflectError::FieldNotInVariant {
            type_path: text("refl_test::Mode"),
            variant: text("Fleeing"),
            field: text("target"),
        },
    ]
}

/// Verifies: REFL-R140
#[test]
fn every_variant_passes_the_error_contract() {
    let errors = reflect_errors();
    let codes: Vec<String> = errors.iter().map(|e| e.code().to_string()).collect();
    assert_eq!(
        codes,
        [
            "LIM-REFL-0001",
            "LIM-REFL-0002",
            "LIM-REFL-0003",
            "LIM-REFL-0004",
            "LIM-REFL-0005",
            "LIM-REFL-0010",
            "LIM-REFL-0011",
            "LIM-REFL-0012",
            "LIM-REFL-0013",
        ]
    );
    for error in &errors {
        assert_eq!(validate_error(error, "REFL"), Ok(()), "{error}");
        // `std::error::Error` is implemented; no source.
        assert!(std::error::Error::source(error).is_none());
    }
}

/// The approved text and JSON form of one value of every variant. The file was written by hand
/// from the table in spec §8, not generated from the implementation. It is an oracle (AGENTS.md
/// rule 5): change it only with the spec.
///
/// Verifies: REFL-R140
#[test]
fn reflect_errors_match_the_approved_snapshot() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/snapshots/reflect_errors.txt"
    );
    // `replace`: a Windows checkout may convert line endings.
    let approved = std::fs::read_to_string(path).unwrap().replace("\r\n", "\n");
    let mut actual = String::new();
    for error in reflect_errors() {
        actual += &error.to_string();
        actual.push('\n');
        actual += &render_json(&error);
        actual.push_str("\n\n");
    }
    assert_eq!(actual, approved);
}
