//! Affine transforms (spec §5.4): MATH-R090 to MATH-R094.

#![allow(
    clippy::float_cmp,
    clippy::many_single_char_names,
    reason = "exact results are the contract (spec §4 \"Exact expression\"); the requirement formulas use one-letter names"
)]

use liminal_math::{Affine3, Mat3, Mat4, MathError, Quat, Vec3};
use proptest::prelude::*;

#[allow(dead_code, reason = "each test file uses a subset of the helpers")]
#[path = "common/math.rs"]
mod common;
use common::{any_mat3, any_quat, any_vec3, approx_vec3, bits, d_trs, same_bits_all};

fn any_affine() -> impl Strategy<Value = Affine3> {
    (any_mat3(), any_vec3()).prop_map(|(matrix3, translation)| Affine3 {
        matrix3,
        translation,
    })
}

fn trs_affine() -> impl Strategy<Value = Affine3> {
    d_trs().prop_map(|(s, r, t)| Affine3::from_scale_rotation_translation(s, r, t))
}

fn small_point() -> impl Strategy<Value = Vec3> {
    [-10.0_f32..=10.0, -10.0_f32..=10.0, -10.0_f32..=10.0].prop_map(Vec3::from_array)
}

/// Verifies: MATH-R090
#[test]
fn identity_is_identity_matrix_and_zero_translation() {
    assert_eq!(Affine3::IDENTITY.matrix3, Mat3::IDENTITY);
    assert_eq!(bits(Affine3::IDENTITY.translation.to_array()), [0; 3]);
    assert_eq!(Mat4::from(Affine3::IDENTITY), Mat4::IDENTITY);
}

proptest! {
    /// Verifies: MATH-R090
    #[test]
    fn srt_matches_the_mat4_function_and_conversion(s in any_vec3(), q in any_quat(), t in any_vec3()) {
        let affine = Affine3::from_scale_rotation_translation(s, q, t);
        let r = Mat3::from_quat(q);
        prop_assert!(same_bits_all(&affine.matrix3.x_axis.to_array(), &(r.x_axis * s.x).to_array()));
        prop_assert!(same_bits_all(&affine.matrix3.y_axis.to_array(), &(r.y_axis * s.y).to_array()));
        prop_assert!(same_bits_all(&affine.matrix3.z_axis.to_array(), &(r.z_axis * s.z).to_array()));
        prop_assert_eq!(bits(affine.translation.to_array()), bits(t.to_array()));
        let m = Mat4::from_scale_rotation_translation(s, q, t);
        prop_assert!(same_bits_all(&Mat4::from(affine).to_cols_array(), &m.to_cols_array()));
    }

    /// Verifies: MATH-R090
    #[test]
    fn conversion_to_mat4_has_the_specified_columns(a in any_affine()) {
        let m = Mat4::from(a);
        prop_assert_eq!(bits(m.x_axis.to_array()), bits(a.matrix3.x_axis.extend(0.0).to_array()));
        prop_assert_eq!(bits(m.y_axis.to_array()), bits(a.matrix3.y_axis.extend(0.0).to_array()));
        prop_assert_eq!(bits(m.z_axis.to_array()), bits(a.matrix3.z_axis.extend(0.0).to_array()));
        prop_assert_eq!(bits(m.w_axis.to_array()), bits(a.translation.extend(1.0).to_array()));
    }

    /// Verifies: MATH-R091
    #[test]
    fn point_and_vector_transforms_are_exact(a in any_affine(), p in any_vec3()) {
        let m = a.matrix3;
        let expected = ((m.x_axis * p.x + m.y_axis * p.y) + m.z_axis * p.z) + a.translation;
        prop_assert!(same_bits_all(&a.transform_point3(p).to_array(), &expected.to_array()));
        prop_assert!(same_bits_all(&a.transform_vector3(p).to_array(), &(m * p).to_array()));
    }

    /// Verifies: MATH-R092
    #[test]
    fn product_is_matrix_product_and_transformed_translation(a in any_affine(), b in any_affine()) {
        let p = a * b;
        prop_assert!(same_bits_all(&p.matrix3.to_cols_array(), &(a.matrix3 * b.matrix3).to_cols_array()));
        prop_assert!(same_bits_all(&p.translation.to_array(), &a.transform_point3(b.translation).to_array()));
    }

    /// Verifies: MATH-R092
    #[test]
    fn product_composes_right_to_left(a in trs_affine(), b in trs_affine(), p in small_point()) {
        prop_assert!(approx_vec3((a * b).transform_point3(p), a.transform_point3(b.transform_point3(p)), 100.0));
    }

    /// Verifies: MATH-R093
    #[test]
    fn inverse_undoes_the_transform(a in trs_affine(), p in small_point()) {
        let back = a.inverse().unwrap().transform_point3(a.transform_point3(p));
        let tolerance = 1e-4 * f64::from(p.length()).max(1.0);
        for (x, y) in back.to_array().into_iter().zip(p.to_array()) {
            prop_assert!((f64::from(x) - f64::from(y)).abs() <= tolerance, "{:?} vs {:?}", back, p);
        }
    }

    /// Verifies: MATH-R093
    #[test]
    fn inverse_is_matrix_inverse_and_negated_translation(a in any_affine()) {
        match (a.inverse(), a.matrix3.inverse()) {
            (Ok(inverse), Ok(m)) => {
                prop_assert!(same_bits_all(&inverse.matrix3.to_cols_array(), &m.to_cols_array()));
                prop_assert!(same_bits_all(&inverse.translation.to_array(), &(-(m * a.translation)).to_array()));
            }
            (
                Err(MathError::SingularMatrix { determinant: e }),
                Err(MathError::SingularMatrix { determinant: f }),
            ) => prop_assert!(common::same_bits(e, f)),
            (left, right) => prop_assert!(false, "{:?} vs {:?}", left, right),
        }
    }

    /// Verifies: MATH-R094
    #[test]
    fn transform_points_matches_transform_point3(
        a in any_affine(),
        points in prop::collection::vec(any_vec3(), 0..=67),
        start in 0_usize..4,
    ) {
        // The slice starts at an offset within a larger buffer; elements outside it stay.
        let mut buffer = vec![Vec3::splat(-7.0); start];
        buffer.extend_from_slice(&points);
        buffer.extend([Vec3::splat(9.0); 3]);
        let end = start + points.len();
        a.transform_points(&mut buffer[start..end]);
        for (got, original) in buffer[start..end].iter().zip(&points) {
            prop_assert!(same_bits_all(&got.to_array(), &a.transform_point3(*original).to_array()));
        }
        prop_assert!(buffer[..start].iter().all(|p| *p == Vec3::splat(-7.0)));
        prop_assert!(buffer[end..].iter().all(|p| *p == Vec3::splat(9.0)));
    }
}

/// Verifies: MATH-R091, MATH-R093
#[test]
fn hand_computed_transform_and_singular_inverse() {
    let a = Affine3::from_scale_rotation_translation(
        Vec3::new(2.0, 3.0, 4.0),
        Quat::IDENTITY,
        Vec3::new(1.0, -1.0, 0.5),
    );
    assert_eq!(a.transform_point3(Vec3::ONE), Vec3::new(3.0, 2.0, 4.5));
    assert_eq!(a.transform_vector3(Vec3::ONE), Vec3::new(2.0, 3.0, 4.0));
    // Association order: ((1e8 − 1e8) + 1) + 0 in the x component.
    let order = Affine3 {
        matrix3: Mat3::from_cols(Vec3::new(1e8, 0.0, 0.0), Vec3::new(-1e8, 0.0, 0.0), Vec3::X),
        translation: Vec3::new(-1.0, 0.0, 0.0),
    };
    assert_eq!(order.transform_point3(Vec3::ONE).x, 0.0);
    let flat = Affine3 {
        matrix3: Mat3::from_scale(Vec3::new(1.0, 0.0, 1.0)),
        translation: Vec3::ONE,
    };
    assert_eq!(
        flat.inverse(),
        Err(MathError::SingularMatrix { determinant: 0.0 })
    );
    let mut empty: [Vec3; 0] = [];
    a.transform_points(&mut empty);
}
