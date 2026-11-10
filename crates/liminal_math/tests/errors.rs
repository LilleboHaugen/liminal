//! `MathError` (spec §5.8, §8): MATH-R006. Oracles: the approved snapshot
//! `tests/snapshots/math_errors.txt` and `liminal_core::error::validate_error`.

use liminal_core::error::{ContextValue, EngineError, render_json, validate_error};
use liminal_math::MathError;

/// One value of every variant, in code order. The snapshot holds their text and JSON forms.
fn math_errors() -> Vec<MathError> {
    vec![
        MathError::ZeroLengthVector { parameter: "self" },
        MathError::NonFiniteInput {
            parameter: "direction",
        },
        MathError::SingularMatrix { determinant: 0.0 },
        MathError::InvalidProjection {
            parameter: "right",
            value: -2.5,
        },
        MathError::InvalidViewBasis,
        MathError::InvalidAabb {
            axis: "y",
            min: 3.0,
            max: 1.5,
        },
        MathError::NegativeExtent {
            parameter: "radius",
            value: -0.25,
        },
        MathError::EmptyPointSet,
        MathError::CollinearPoints,
        MathError::DegenerateFrustum { plane: "near" },
        MathError::VectorTooShort {
            parameter: "normal",
            length_squared: f32::from_bits(0x0008_0000), // 2^-130 = 2^19 × 2^-149, subnormal
        },
    ]
}

/// The approved text and JSON form of one value of every variant (spec §10). The file was
/// generated from the message and help texts of spec §8 typed into a script, and from the
/// value formatting of `liminal_core::error` (Rust `Display` for `f32` in messages, for the
/// widened `f64` in JSON), not from this crate's implementation. It is an oracle (AGENTS.md
/// rule 5): change it only with the spec.
///
/// Verifies: MATH-R006
#[test]
fn math_errors_match_the_approved_snapshot() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/snapshots/math_errors.txt"
    );
    // `replace`: a Windows checkout may convert line endings.
    let approved = std::fs::read_to_string(path)
        .unwrap()
        .replace("\r\n", "\n");
    let mut actual = String::new();
    for error in math_errors() {
        actual += &error.to_string();
        actual.push('\n');
        actual += &render_json(&error);
        actual.push_str("\n\n");
    }
    assert_eq!(actual, approved);
}

/// Verifies: MATH-R006
#[test]
fn every_variant_passes_the_error_contract_with_its_code() {
    let errors = math_errors();
    let codes: Vec<String> = errors.iter().map(|e| e.code().to_string()).collect();
    assert_eq!(
        codes,
        [
            "LIM-MATH-0001",
            "LIM-MATH-0002",
            "LIM-MATH-0003",
            "LIM-MATH-0004",
            "LIM-MATH-0005",
            "LIM-MATH-0010",
            "LIM-MATH-0011",
            "LIM-MATH-0012",
            "LIM-MATH-0013",
            "LIM-MATH-0014",
            "LIM-MATH-0015",
        ]
    );
    for error in &errors {
        assert_eq!(validate_error(error, "MATH"), Ok(()), "{error}");
        assert!(error.message().ends_with('.'), "{error}");
        let as_std: &dyn std::error::Error = error;
        assert_eq!(as_std.to_string(), error.to_string());
    }
}

/// Verifies: MATH-R006
#[test]
fn context_fields_have_the_specified_kinds() {
    let error = MathError::InvalidAabb {
        axis: "x",
        min: 0.5,
        max: f32::NAN,
    };
    let context = error.context();
    assert_eq!(context.get("axis"), Some(&ContextValue::Text("x".to_string())));
    assert_eq!(context.get("min"), Some(&ContextValue::Float(0.5)));
    assert!(matches!(context.get("max"), Some(ContextValue::Float(v)) if v.is_nan()));
    // NaN and infinities render as `null` in JSON.
    assert!(render_json(&error).ends_with(r#""context":{"axis":"x","min":0.5,"max":null}}"#));
    assert!(MathError::EmptyPointSet.context().is_empty());
}
