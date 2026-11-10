//! Engine-wide conventions of spec §4: MATH-R002.

use core::f32::consts::FRAC_1_SQRT_2;

use liminal_math::{Mat3, Mat4, Quat, Vec3, Vec4};

#[allow(dead_code, reason = "each test file uses a subset of the helpers")]
#[path = "common/math.rs"]
mod common;
use common::{approx_vec3, bits};

/// Verifies: MATH-R002
#[test]
fn right_handed_cross_products_are_exact() {
    assert_eq!(bits(Vec3::X.cross(Vec3::Y).to_array()), bits(Vec3::Z.to_array()));
    assert_eq!(bits(Vec3::Y.cross(Vec3::Z).to_array()), bits(Vec3::X.to_array()));
    assert_eq!(bits(Vec3::Z.cross(Vec3::X).to_array()), bits(Vec3::Y.to_array()));
}

/// Verifies: MATH-R002
#[test]
fn positive_quarter_turns_follow_the_right_hand_rule() {
    let h = FRAC_1_SQRT_2;
    assert!(approx_vec3(Quat::from_xyzw(0.0, 0.0, h, h) * Vec3::X, Vec3::Y, 1.0));
    assert!(approx_vec3(Quat::from_xyzw(h, 0.0, 0.0, h) * Vec3::Y, Vec3::Z, 1.0));
    assert!(approx_vec3(Quat::from_xyzw(0.0, h, 0.0, h) * Vec3::Z, Vec3::X, 1.0));
    // The matrix form agrees.
    let about_z = Mat3::from_quat(Quat::from_xyzw(0.0, 0.0, h, h));
    assert!(approx_vec3(about_z * Vec3::X, Vec3::Y, 1.0));
}

/// Verifies: MATH-R002
#[test]
fn translation_moves_points_and_column_vectors_compose_right_to_left() {
    let t = Vec3::new(1.5, -2.0, 4.0);
    let p = Vec3::new(3.0, 0.25, -8.0);
    assert_eq!(
        Mat4::from_translation(t) * p.extend(1.0),
        (p + t).extend(1.0)
    );

    // Small integer matrices: no rounding, so `(a * b) * v == a * (b * v)` exactly, and the
    // right-hand factor applies first.
    let a = Mat4::from_cols_array(&[
        1.0, 2.0, 0.0, 0.0, 0.0, 1.0, 3.0, 0.0, -1.0, 0.0, 1.0, 0.0, 2.0, 0.0, 0.0, 1.0,
    ]);
    let b = Mat4::from_cols_array(&[
        0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 3.0, 0.0, 1.0,
    ]);
    let v = Vec4::new(1.0, -2.0, 3.0, 1.0);
    assert_eq!((a * b) * v, a * (b * v));
    let scale_then_move = Mat4::from_translation(Vec3::X) * Mat4::from_scale(Vec3::splat(2.0));
    assert_eq!(
        scale_then_move.transform_point3(Vec3::ONE),
        Vec3::new(3.0, 2.0, 2.0)
    );
}
