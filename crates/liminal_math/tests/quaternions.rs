//! Quaternion operations (spec §5.3): MATH-R070 to MATH-R072.

#![allow(
    clippy::float_cmp,
    clippy::many_single_char_names,
    reason = "exact results are the contract (spec §4 \"Exact expression\"); the requirement formulas use one-letter names"
)]

use liminal_math::{MathError, Quat, Vec3, Vec4};
use proptest::prelude::*;

#[allow(dead_code, reason = "each test file uses a subset of the helpers")]
#[path = "common/math.rs"]
mod common;
use common::{
    any_f32, any_quat, any_vec3, approx, approx_vec3, bits, d_unit, d_vec3, same_bits,
    same_bits_all,
};

fn components(q: Quat) -> [f32; 4] {
    [q.x, q.y, q.z, q.w]
}

/// Verifies: MATH-R070
#[test]
fn identity_and_normalize_errors() {
    assert_eq!(bits(components(Quat::IDENTITY)), bits([0.0, 0.0, 0.0, 1.0]));
    assert_eq!(
        Quat::from_xyzw(0.0, -0.0, 0.0, 0.0).normalize(),
        Err(MathError::ZeroLengthVector { parameter: "self" })
    );
    assert_eq!(
        Quat::from_xyzw(0.0, 0.0, f32::NAN, 1.0).normalize(),
        Err(MathError::NonFiniteInput { parameter: "self" })
    );
    assert_eq!(
        Quat::from_xyzw(0.0, 0.0, 0.0, 3e19).normalize(),
        Err(MathError::NonFiniteInput { parameter: "self" })
    );
    assert_eq!(
        Quat::from_xyzw(0.0, 3.0, 0.0, 4.0).normalize(),
        Ok(Quat::from_xyzw(0.0, 0.6, 0.0, 0.8))
    );
    // Association order of `dot`: ((1e8 − 1e8) + 1) + 0.
    let q = Quat::from_xyzw(1e8, -1e8, 1.0, 0.0);
    assert_eq!(q.dot(Quat::from_xyzw(1.0, 1.0, 1.0, 1.0)), 1.0);
}

proptest! {
    /// Verifies: MATH-R070
    #[test]
    fn components_dot_length_conjugate_normalize(a in any_quat(), b in any_quat(), x in any_f32()) {
        let q = Quat::from_xyzw(x, a.y, a.z, a.w);
        prop_assert_eq!(bits(components(q)), bits([x, a.y, a.z, a.w]));
        let d = ((a.x * b.x + a.y * b.y) + a.z * b.z) + a.w * b.w;
        prop_assert!(same_bits(a.dot(b), d));
        prop_assert!(same_bits(a.length(), a.dot(a).sqrt()));
        let c = a.conjugate();
        prop_assert_eq!(
            bits(components(c)),
            [a.x.to_bits() ^ 0x8000_0000, a.y.to_bits() ^ 0x8000_0000, a.z.to_bits() ^ 0x8000_0000, a.w.to_bits()]
        );
        let as_vec4 = Vec4::new(a.x, a.y, a.z, a.w).normalize();
        match (a.normalize(), as_vec4) {
            (Ok(n), Ok(v)) => prop_assert!(same_bits_all(&components(n), &v.to_array())),
            (Err(e), Err(f)) => prop_assert_eq!(e, f),
            (left, right) => prop_assert!(false, "{:?} vs {:?}", left, right),
        }
    }

    /// Verifies: MATH-R070
    #[test]
    fn normalize_gives_unit_length(q in d_unit()) {
        prop_assert!((q.length() - 1.0).abs() <= 4.0 * f32::EPSILON);
    }

    /// Verifies: MATH-R071
    #[test]
    fn hamilton_product_is_the_exact_expression(a in any_quat(), b in any_quat()) {
        let expected = [
            ((a.w * b.x + a.x * b.w) + a.y * b.z) - a.z * b.y,
            ((a.w * b.y - a.x * b.z) + a.y * b.w) + a.z * b.x,
            ((a.w * b.z + a.x * b.y) - a.y * b.x) + a.z * b.w,
            ((a.w * b.w - a.x * b.x) - a.y * b.y) - a.z * b.z,
        ];
        prop_assert!(same_bits_all(&components(a * b), &expected));
    }

    /// Verifies: MATH-R071
    #[test]
    fn identity_is_neutral_for_finite_quaternions(
        q in prop::array::uniform4(-100.0_f32..=100.0).prop_map(|[x, y, z, w]| Quat::from_xyzw(x, y, z, w))
    ) {
        prop_assert_eq!(q * Quat::IDENTITY, q);
        prop_assert_eq!(Quat::IDENTITY * q, q);
    }

    /// Verifies: MATH-R072
    #[test]
    fn rotation_is_the_exact_expression(q in any_quat(), v in any_vec3()) {
        let u = Vec3::new(q.x, q.y, q.z);
        let t = u.cross(v) * 2.0;
        let expected = (v + t * q.w) + u.cross(t);
        prop_assert!(same_bits_all(&(q * v).to_array(), &expected.to_array()));
    }

    /// Verifies: MATH-R072
    #[test]
    fn rotation_properties(a in d_unit(), b in d_unit(), v in d_vec3()) {
        let scale = f64::from(v.length());
        prop_assert!(approx((a * v).length(), scale, scale));
        prop_assert!(approx_vec3((a * b) * v, a * (b * v), scale));
        prop_assert!(approx_vec3(a.conjugate() * (a * v), v, scale));
    }

    /// Verifies: MATH-R072
    #[test]
    fn identity_rotation_keeps_finite_vectors(v in any_vec3()) {
        prop_assume!(v.is_finite());
        prop_assert_eq!(Quat::IDENTITY * v, v);
    }
}

/// Verifies: MATH-R072
#[test]
fn identity_rotation_of_an_infinite_vector_gives_nan() {
    let rotated = Quat::IDENTITY * Vec3::new(f32::INFINITY, 0.0, 0.0);
    assert!(rotated.to_array().iter().any(|c| c.is_nan()));
}
