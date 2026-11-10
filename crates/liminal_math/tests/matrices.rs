//! Matrix operations (spec §5.2): MATH-R041 to MATH-R049, MATH-R051.

use liminal_math::{Mat3, Mat4, MathError, Quat, Vec3, Vec4};
use proptest::prelude::*;

#[allow(dead_code, reason = "each test file uses a subset of the helpers")]
#[path = "common/math.rs"]
mod common;
use common::{
    any_f32, any_mat3, any_mat4, any_quat, any_vec3, any_vec4, approx, approx_vec3, bits, d_trs,
    d_unit, d_vec3, same_bits_all,
};

/// A finite matrix with elements in −100 to 100.
fn finite_mat4() -> impl Strategy<Value = Mat4> {
    prop::array::uniform16(-100.0_f32..=100.0).prop_map(|e| Mat4::from_cols_array(&e))
}

fn finite_mat3() -> impl Strategy<Value = Mat3> {
    prop::array::uniform9(-100.0_f32..=100.0).prop_map(|e| Mat3::from_cols_array(&e))
}

/// A matrix with integer elements in −8 to 8.
fn small_integer_mat4() -> impl Strategy<Value = [i64; 16]> {
    prop::array::uniform16(-8_i64..=8)
}

/// Row `r`, column `c` of a column-major 4×4 integer matrix.
fn int_at(e: &[i64; 16], r: usize, c: usize) -> i64 {
    e[4 * c + r]
}

/// Determinant by cofactor expansion along the first row, in `i64` (test oracle).
fn int_det(e: &[i64], n: usize) -> i64 {
    if n == 1 {
        return e[0];
    }
    let mut total = 0;
    for column in 0..n {
        // Minor without row 0 and `column`, column-major.
        let mut minor = Vec::with_capacity((n - 1) * (n - 1));
        for c in (0..n).filter(|c| *c != column) {
            for r in 1..n {
                minor.push(e[n * c + r]);
            }
        }
        let sign = if column % 2 == 0 { 1 } else { -1 };
        total += sign * e[n * column] * int_det(&minor, n - 1);
    }
    total
}

fn to_f32(value: i64) -> f32 {
    i16::try_from(value).map(f32::from).unwrap()
}

proptest! {
    /// Verifies: MATH-R041
    #[test]
    fn matrix_vector_products_are_the_exact_expression(m in any_mat4(), v in any_vec4(), m3 in any_mat3()) {
        let expected = ((m.x_axis * v.x + m.y_axis * v.y) + m.z_axis * v.z) + m.w_axis * v.w;
        prop_assert!(same_bits_all(&(m * v).to_array(), &expected.to_array()));
        let v3 = v.truncate();
        let expected3 = (m3.x_axis * v3.x + m3.y_axis * v3.y) + m3.z_axis * v3.z;
        prop_assert!(same_bits_all(&(m3 * v3).to_array(), &expected3.to_array()));
    }

    /// Verifies: MATH-R042
    #[test]
    fn matrix_products_are_column_by_column(a in any_mat4(), b in any_mat4(), a3 in any_mat3(), b3 in any_mat3()) {
        let p = a * b;
        for (column, of_b) in [(p.x_axis, b.x_axis), (p.y_axis, b.y_axis), (p.z_axis, b.z_axis), (p.w_axis, b.w_axis)] {
            prop_assert!(same_bits_all(&column.to_array(), &(a * of_b).to_array()));
        }
        let p3 = a3 * b3;
        for (column, of_b) in [(p3.x_axis, b3.x_axis), (p3.y_axis, b3.y_axis), (p3.z_axis, b3.z_axis)] {
            prop_assert!(same_bits_all(&column.to_array(), &(a3 * of_b).to_array()));
        }
    }

    /// Verifies: MATH-R042
    #[test]
    fn identity_is_neutral_for_finite_matrices(m in finite_mat4(), m3 in finite_mat3()) {
        prop_assert_eq!(m * Mat4::IDENTITY, m);
        prop_assert_eq!(Mat4::IDENTITY * m, m);
        prop_assert_eq!(m3 * Mat3::IDENTITY, m3);
        prop_assert_eq!(Mat3::IDENTITY * m3, m3);
    }

    /// Verifies: MATH-R043
    #[test]
    fn transpose_swaps_rows_and_columns_with_bits(m in any_mat4(), m3 in any_mat3()) {
        let (e, t) = (m.to_cols_array(), m.transpose().to_cols_array());
        for r in 0..4 {
            for c in 0..4 {
                prop_assert_eq!(t[4 * c + r].to_bits(), e[4 * r + c].to_bits());
            }
        }
        let (e, t) = (m3.to_cols_array(), m3.transpose().to_cols_array());
        for r in 0..3 {
            for c in 0..3 {
                prop_assert_eq!(t[3 * c + r].to_bits(), e[3 * r + c].to_bits());
            }
        }
    }

    /// Verifies: MATH-R044
    #[test]
    fn determinant_is_exact_for_small_integers(e in small_integer_mat4()) {
        let m = Mat4::from_cols_array(&e.map(to_f32));
        prop_assert_eq!(m.determinant(), to_f32(int_det(&e, 4)));
        let e3 = [
            int_at(&e, 0, 0), int_at(&e, 1, 0), int_at(&e, 2, 0),
            int_at(&e, 0, 1), int_at(&e, 1, 1), int_at(&e, 2, 1),
            int_at(&e, 0, 2), int_at(&e, 1, 2), int_at(&e, 2, 2),
        ];
        let m3 = Mat3::from_cols_array(&e3.map(to_f32));
        prop_assert_eq!(m3.determinant(), to_f32(int_det(&e3, 3)));
    }

    /// Verifies: MATH-R044
    #[test]
    fn determinant_of_a_trs_matrix_is_the_scale_product((s, r, t) in d_trs()) {
        let expected = f64::from(s.x) * f64::from(s.y) * f64::from(s.z);
        let m = Mat4::from_scale_rotation_translation(s, r, t);
        prop_assert!(approx(m.determinant(), expected, 8.0), "{} vs {}", m.determinant(), expected);
        let m3 = Mat3::from_quat(r) * Mat3::from_scale(s);
        prop_assert!(approx(m3.determinant(), expected, 8.0), "{} vs {}", m3.determinant(), expected);
    }

    /// Verifies: MATH-R045
    #[test]
    fn inverse_of_a_trs_matrix((s, r, t) in d_trs()) {
        let m = Mat4::from_scale_rotation_translation(s, r, t);
        let inverse = m.inverse().unwrap();
        let identity = Mat4::IDENTITY.to_cols_array();
        for product in [m * inverse, inverse * m] {
            for (x, i) in product.to_cols_array().into_iter().zip(identity) {
                prop_assert!((x - i).abs() <= 1e-4, "{:?}", product);
            }
        }
        let m3 = Mat3::from_quat(r) * Mat3::from_scale(s);
        let inverse3 = m3.inverse().unwrap();
        let identity3 = Mat3::IDENTITY.to_cols_array();
        for product in [m3 * inverse3, inverse3 * m3] {
            for (x, i) in product.to_cols_array().into_iter().zip(identity3) {
                prop_assert!((x - i).abs() <= 1e-4, "{:?}", product);
            }
        }
    }

    /// Verifies: MATH-R045
    #[test]
    fn inverse_fails_exactly_when_the_determinant_is_zero_or_not_finite(m in any_mat4(), m3 in any_mat3()) {
        let d = m.determinant();
        match m.inverse() {
            Ok(_) => prop_assert!(d != 0.0 && d.is_finite()),
            Err(MathError::SingularMatrix { determinant }) => {
                prop_assert!(d == 0.0 || !d.is_finite());
                prop_assert!(common::same_bits(determinant, d));
            }
            Err(other) => prop_assert!(false, "unexpected {:?}", other),
        }
        let d3 = m3.determinant();
        match m3.inverse() {
            Ok(_) => prop_assert!(d3 != 0.0 && d3.is_finite()),
            Err(MathError::SingularMatrix { determinant }) => {
                prop_assert!(d3 == 0.0 || !d3.is_finite());
                prop_assert!(common::same_bits(determinant, d3));
            }
            Err(other) => prop_assert!(false, "unexpected {:?}", other),
        }
    }
}

/// Verifies: MATH-R044, MATH-R045
#[test]
fn identity_determinant_inverse_and_singular_cases() {
    assert_eq!(Mat4::IDENTITY.determinant(), 1.0);
    assert_eq!(Mat3::IDENTITY.determinant(), 1.0);
    assert_eq!(Mat4::IDENTITY.inverse(), Ok(Mat4::IDENTITY));
    assert_eq!(Mat3::IDENTITY.inverse(), Ok(Mat3::IDENTITY));

    let flat = Mat4::from_scale(Vec3::new(1.0, 0.0, 1.0));
    assert_eq!(flat.inverse(), Err(MathError::SingularMatrix { determinant: 0.0 }));
    let flat3 = Mat3::from_scale(Vec3::new(1.0, 1.0, 0.0));
    assert_eq!(flat3.inverse(), Err(MathError::SingularMatrix { determinant: 0.0 }));
    let huge = Mat4::from_scale(Vec3::splat(1e30));
    assert_eq!(
        huge.inverse(),
        Err(MathError::SingularMatrix { determinant: f32::INFINITY })
    );
    let nan = Mat3::from_scale(Vec3::new(f32::NAN, 1.0, 1.0));
    assert!(matches!(nan.inverse(), Err(MathError::SingularMatrix { determinant }) if determinant.is_nan()));

    // A hand-computed integer case: rows (2 0 0 1), (0 1 0 0), (0 0 4 0), (0 0 0 1).
    let m = Mat4::from_cols_array(&[
        2.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 4.0, 0.0, 1.0, 0.0, 0.0, 1.0,
    ]);
    assert_eq!(m.determinant(), 8.0);
    let expected = Mat4::from_cols_array(&[
        0.5, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.25, 0.0, -0.5, 0.0, 0.0, 1.0,
    ]);
    assert_eq!(m.inverse(), Ok(expected));
    // Mat3: rows (1 2 0), (0 1 0), (0 0 2); inverse rows (1 −2 0), (0 1 0), (0 0 0.5).
    let m3 = Mat3::from_cols_array(&[1.0, 0.0, 0.0, 2.0, 1.0, 0.0, 0.0, 0.0, 2.0]);
    assert_eq!(m3.determinant(), 2.0);
    let expected3 = Mat3::from_cols_array(&[1.0, 0.0, 0.0, -2.0, 1.0, 0.0, 0.0, 0.0, 0.5]);
    assert_eq!(m3.inverse(), Ok(expected3));
}

/// Verifies: MATH-R041, MATH-R042
#[test]
fn products_use_the_specified_association_order() {
    // Row 0 of the columns: 1e8, −1e8, 1, −1. Left to right: ((1e8 − 1e8) + 1) − 1 = 0; any
    // order that adds 1 to ±1e8 first gives a different result.
    let m = Mat4::from_cols(
        Vec4::new(1e8, 0.0, 0.0, 0.0),
        Vec4::new(-1e8, 0.0, 0.0, 0.0),
        Vec4::new(1.0, 0.0, 0.0, 0.0),
        Vec4::new(1.0, 0.0, 0.0, 0.0),
    );
    assert_eq!((m * Vec4::ONE).x, 2.0);
    assert_eq!((m * Mat4::from_cols(Vec4::ONE, Vec4::ZERO, Vec4::ZERO, Vec4::ZERO)).x_axis.x, 2.0);
    let m3 = Mat3::from_cols(Vec3::new(1e8, 0.0, 0.0), Vec3::new(-1e8, 0.0, 0.0), Vec3::X);
    assert_eq!((m3 * Vec3::ONE).x, 1.0);
}

proptest! {
    /// Verifies: MATH-R046
    #[test]
    fn rotation_matrix_agrees_with_the_quaternion(q in d_unit(), v in d_vec3()) {
        let scale = f64::from(v.length());
        prop_assert!(approx_vec3(Mat3::from_quat(q) * v, q * v, scale));
    }

    /// Verifies: MATH-R046
    #[test]
    fn mat4_from_quat_embeds_mat3_from_quat(q in any_quat()) {
        let m3 = Mat3::from_quat(q);
        let m4 = Mat4::from_quat(q);
        prop_assert_eq!(bits(m4.x_axis.truncate().to_array()), bits(m3.x_axis.to_array()));
        prop_assert_eq!(bits(m4.y_axis.truncate().to_array()), bits(m3.y_axis.to_array()));
        prop_assert_eq!(bits(m4.z_axis.truncate().to_array()), bits(m3.z_axis.to_array()));
        prop_assert_eq!([m4.x_axis.w, m4.y_axis.w, m4.z_axis.w], [0.0; 3]);
        prop_assert_eq!(bits(m4.w_axis.to_array()), bits([0.0, 0.0, 0.0, 1.0]));
    }

    /// Verifies: MATH-R047
    #[test]
    fn srt_matrix_has_the_specified_columns(s in any_vec3(), q in any_quat(), t in any_vec3()) {
        let r = Mat3::from_quat(q);
        let m = Mat4::from_scale_rotation_translation(s, q, t);
        prop_assert!(same_bits_all(&m.x_axis.to_array(), &(r.x_axis * s.x).extend(0.0).to_array()));
        prop_assert!(same_bits_all(&m.y_axis.to_array(), &(r.y_axis * s.y).extend(0.0).to_array()));
        prop_assert!(same_bits_all(&m.z_axis.to_array(), &(r.z_axis * s.z).extend(0.0).to_array()));
        prop_assert!(same_bits_all(&m.w_axis.to_array(), &t.extend(1.0).to_array()));
    }

    /// Verifies: MATH-R048
    #[test]
    fn point_vector_and_projection_transforms(m in any_mat4(), p in any_vec3()) {
        prop_assert!(same_bits_all(&m.transform_point3(p).to_array(), &(m * p.extend(1.0)).truncate().to_array()));
        prop_assert!(same_bits_all(&m.transform_vector3(p).to_array(), &(m * p.extend(0.0)).truncate().to_array()));
        let c = m * p.extend(1.0);
        prop_assert!(same_bits_all(&m.project_point3(p).to_array(), &(c.truncate() / c.w).to_array()));
    }
}

/// Verifies: MATH-R046
#[test]
fn identity_quaternion_gives_the_identity_matrix() {
    assert_eq!(Mat3::from_quat(Quat::IDENTITY), Mat3::IDENTITY);
    assert_eq!(Mat4::from_quat(Quat::IDENTITY), Mat4::IDENTITY);
}

/// Verifies: MATH-R048
#[test]
fn projection_with_zero_w_is_not_checked() {
    let zero_w = Mat4::from_cols(Vec4::X, Vec4::Y, Vec4::Z, Vec4::ZERO);
    let projected = zero_w.project_point3(Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(projected.x, f32::INFINITY);
    assert!(projected.y.is_nan());
    let perspective_like = Mat4::from_cols_array(&[
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, -1.0, 0.0, 0.0, 0.0, 0.0,
    ]);
    assert_eq!(
        perspective_like.project_point3(Vec3::new(2.0, 4.0, -2.0)),
        Vec3::new(1.0, 2.0, -1.0)
    );
}

/// Verifies: MATH-R049
#[test]
fn look_to_errors_in_signature_order() {
    let nan = Vec3::new(0.0, f32::NAN, 0.0);
    let inf = Vec3::new(f32::INFINITY, 0.0, 0.0);
    let forward = Vec3::new(0.0, 0.0, -1.0);
    let non_finite = |parameter| Err(MathError::NonFiniteInput { parameter });
    assert_eq!(Mat4::look_to(nan, inf, inf), non_finite("eye"));
    assert_eq!(Mat4::look_to(Vec3::ZERO, inf, nan), non_finite("direction"));
    assert_eq!(Mat4::look_to(Vec3::ZERO, forward, nan), non_finite("up"));
    assert_eq!(
        Mat4::look_to(Vec3::ZERO, Vec3::ZERO, Vec3::Y),
        Err(MathError::ZeroLengthVector { parameter: "direction" })
    );
    // A finite direction whose squared length overflows.
    assert_eq!(
        Mat4::look_to(Vec3::ZERO, Vec3::new(0.0, 0.0, -3e19), Vec3::Y),
        non_finite("direction")
    );
    assert_eq!(
        Mat4::look_to(Vec3::ZERO, forward, Vec3::new(0.0, 0.0, 2.0)),
        Err(MathError::InvalidViewBasis)
    );
    assert_eq!(
        Mat4::look_to(Vec3::ZERO, forward, Vec3::ZERO),
        Err(MathError::InvalidViewBasis)
    );
}

/// Verifies: MATH-R049
#[test]
fn look_to_has_the_specified_columns() {
    // Looking along +X from (1, 2, 3) with +Y up: f = X, s = f × up = Z, u = s × f = Y.
    let eye = Vec3::new(1.0, 2.0, 3.0);
    let m = Mat4::look_to(eye, Vec3::new(2.0, 0.0, 0.0), Vec3::Y).unwrap();
    let expected = Mat4::from_cols(
        Vec4::new(0.0, 0.0, -1.0, 0.0),
        Vec4::new(0.0, 1.0, 0.0, 0.0),
        Vec4::new(1.0, 0.0, 0.0, 0.0),
        Vec4::new(-3.0, -2.0, 1.0, 1.0),
    );
    assert_eq!(m, expected);
    assert_eq!(m.transform_point3(eye), Vec3::ZERO);
    assert_eq!(m.transform_vector3(Vec3::X), Vec3::new(0.0, 0.0, -1.0));
}

/// A unit `up` whose angle to the unit `direction` is between 0.1 and π − 0.1 radians.
fn view_inputs() -> impl Strategy<Value = (Vec3, Vec3, Vec3)> {
    let unit = [-1.0_f32..=1.0, -1.0_f32..=1.0, -1.0_f32..=1.0]
        .prop_map(Vec3::from_array)
        .prop_filter("length at least 0.1", |v| v.length() >= 0.1)
        .prop_map(|v| v.normalize().unwrap());
    (d_vec3(), unit.clone(), unit).prop_filter("angle in 0.1 to π − 0.1", |(_, d, u)| {
        // cos(0.1) = 0.995004
        f64::from(d.dot(*u)).abs() <= 0.995_004
    })
}

proptest! {
    /// Verifies: MATH-R049
    #[test]
    fn look_to_maps_eye_direction_and_up((eye, direction, up) in view_inputs()) {
        let m = Mat4::look_to(eye, direction, up).unwrap();
        prop_assert!(approx_vec3(m.transform_point3(eye), Vec3::ZERO, f64::from(eye.length())));
        prop_assert!(approx_vec3(m.transform_vector3(direction), Vec3::new(0.0, 0.0, -1.0), 1.0));
        let mapped_up = m.transform_vector3(up);
        prop_assert!(approx(mapped_up.x, 0.0, 1.0), "{:?}", mapped_up);
        prop_assert!(mapped_up.y > 0.0, "{:?}", mapped_up);
    }
}

/// Verifies: MATH-R051
#[test]
fn orthographic_errors_in_the_specified_order() {
    let invalid = |parameter, value| Err(MathError::InvalidProjection { parameter, value });
    let nan = f32::NAN;
    let inf = f32::INFINITY;
    assert_eq!(Mat4::orthographic(-inf, 1.0, -1.0, 1.0, 0.1, 10.0), invalid("left", -inf));
    assert_eq!(Mat4::orthographic(-1.0, inf, -1.0, 1.0, 0.1, 10.0), invalid("right", inf));
    assert_eq!(Mat4::orthographic(-1.0, 1.0, -inf, inf, 0.1, 10.0), invalid("bottom", -inf));
    assert_eq!(Mat4::orthographic(-1.0, 1.0, -1.0, inf, 0.1, 10.0), invalid("top", inf));
    assert_eq!(Mat4::orthographic(-1.0, 1.0, -1.0, 1.0, inf, 10.0), invalid("z_near", inf));
    assert_eq!(Mat4::orthographic(-1.0, 1.0, -1.0, 1.0, 0.1, -inf), invalid("z_far", -inf));
    let result = Mat4::orthographic(-1.0, 1.0, -1.0, 1.0, 0.1, nan);
    assert!(matches!(result, Err(MathError::InvalidProjection { parameter: "z_far", value }) if value.is_nan()));
    // Non-finite values are reported before equal bounds.
    assert_eq!(Mat4::orthographic(1.0, 1.0, -1.0, 1.0, 0.1, inf), invalid("z_far", inf));
    assert_eq!(Mat4::orthographic(2.0, 2.0, 3.0, 3.0, 1.0, 1.0), invalid("right", 2.0));
    assert_eq!(Mat4::orthographic(-2.0, 2.0, 3.0, 3.0, 1.0, 1.0), invalid("top", 3.0));
    assert_eq!(Mat4::orthographic(-2.0, 2.0, -3.0, 3.0, 1.0, 1.0), invalid("z_far", 1.0));
    // −0.0 == 0.0: equal bounds.
    assert_eq!(Mat4::orthographic(-0.0, 0.0, -1.0, 1.0, 0.1, 10.0), invalid("right", 0.0));
    // Inverted bounds are valid (mirroring).
    assert!(Mat4::orthographic(1.0, -1.0, 1.0, -1.0, 10.0, 0.1).is_ok());
}

proptest! {
    /// Verifies: MATH-R051
    #[test]
    fn orthographic_is_the_exact_matrix(
        left in any_f32(), right in any_f32(), bottom in any_f32(), top in any_f32(),
        z_near in any_f32(), z_far in any_f32()
    ) {
        let all = [left, right, bottom, top, z_near, z_far];
        prop_assume!(all.iter().all(|v| v.is_finite()));
        prop_assume!(right != left && top != bottom && z_far != z_near);
        let m = Mat4::orthographic(left, right, bottom, top, z_near, z_far).unwrap();
        let expected = Mat4::from_cols(
            Vec4::new(2.0 / (right - left), 0.0, 0.0, 0.0),
            Vec4::new(0.0, 2.0 / (top - bottom), 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0 / (z_near - z_far), 0.0),
            Vec4::new(
                -(right + left) / (right - left),
                -(top + bottom) / (top - bottom),
                z_near / (z_near - z_far),
                1.0,
            ),
        );
        prop_assert!(same_bits_all(&m.to_cols_array(), &expected.to_cols_array()));
    }

    /// Verifies: MATH-R051
    #[test]
    fn orthographic_maps_the_box_corners(
        left in -10.0_f32..=10.0, width in 1.0_f32..=20.0,
        bottom in -10.0_f32..=10.0, height in 1.0_f32..=20.0,
        z_near in 0.05_f32..=1.0, z_far in 10.0_f32..=500.0
    ) {
        let right = left + width;
        let top = bottom + height;
        prop_assume!(right <= 10.0 && top <= 10.0 && right - left >= 1.0 && top - bottom >= 1.0);
        let m = Mat4::orthographic(left, right, bottom, top, z_near, z_far).unwrap();
        let near_corner = m.transform_point3(Vec3::new(left, bottom, -z_near));
        let far_corner = m.transform_point3(Vec3::new(right, top, -z_far));
        for (got, want) in near_corner.to_array().into_iter().zip([-1.0, -1.0, 0.0]) {
            prop_assert!((got - want).abs() <= 1e-4, "{:?}", near_corner);
        }
        for (got, want) in far_corner.to_array().into_iter().zip([1.0, 1.0, 1.0]) {
            prop_assert!((got - want).abs() <= 1e-4, "{:?}", far_corner);
        }
    }
}
