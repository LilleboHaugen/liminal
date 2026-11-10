//! No public function or operator panics (spec §10 "No panics"): MATH-R005. Every public item
//! of the crate so far is called with arguments of arbitrary bit patterns and, for
//! `transform_points`, slices of length 0 to 67. Functions returning `Result` give only the
//! `Err` variants their requirements name. Later tasks extend this file with their functions.

use liminal_core::hash::state_hash_of;
use liminal_math::{Affine3, Mat3, Mat4, MathError, Quat, Vec2, Vec3, Vec4, reference};
use proptest::prelude::*;

#[allow(dead_code, reason = "each test file uses a subset of the helpers")]
#[path = "common/math.rs"]
mod common;
use common::{edgy_f32, edgy_vec3, edgy_vec4};

fn edgy_quat() -> impl Strategy<Value = Quat> {
    edgy_vec4().prop_map(|v| Quat::from_xyzw(v.x, v.y, v.z, v.w))
}

fn edgy_mat3() -> impl Strategy<Value = Mat3> {
    prop::array::uniform9(edgy_f32()).prop_map(|e| Mat3::from_cols_array(&e))
}

fn edgy_mat4() -> impl Strategy<Value = Mat4> {
    prop::array::uniform16(edgy_f32()).prop_map(|e| Mat4::from_cols_array(&e))
}

fn is_normalize_error(error: &MathError) -> bool {
    matches!(
        error,
        MathError::NonFiniteInput { parameter: "self" }
            | MathError::ZeroLengthVector { parameter: "self" }
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    /// Verifies: MATH-R005
    #[test]
    fn vector_functions_never_panic(a in edgy_vec4(), b in edgy_vec4(), s in edgy_f32()) {
        let (a3, b3) = (a.truncate(), b.truncate());
        let (a2, b2) = (Vec2::new(a.x, a.y), Vec2::new(b.x, b.y));
        macro_rules! exercise {
            ($a:expr, $b:expr) => {{
                let (mut x, y) = ($a, $b);
                let _ = (x + y, x - y, -x, x * y, x * s, s * x, x / s);
                x += y;
                x -= y;
                x *= s;
                x /= s;
                let _ = (x.dot(y), x.length_squared(), x.length(), x.min(y), x.max(y), x.abs());
                let _ = (x.lerp(y, s), x.is_finite(), x.normalize_or(y), x.to_array(), state_hash_of(&x));
                if let Err(error) = x.normalize() {
                    prop_assert!(is_normalize_error(&error), "{:?}", error);
                }
            }};
        }
        exercise!(a, b);
        exercise!(a3, b3);
        exercise!(a2, b2);
        let _ = (a3.cross(b3), a3.extend(s), a.truncate(), Vec3::splat(s), Vec2::splat(s), Vec4::splat(s));
        let _ = (Vec2::from_array([s, s]), Vec3::from_array([s; 3]), Vec4::from_array([s; 4]));
    }

    /// Verifies: MATH-R005
    #[test]
    fn matrix_functions_never_panic(
        m in edgy_mat4(), n in edgy_mat4(), m3 in edgy_mat3(), n3 in edgy_mat3(),
        v in edgy_vec4(), q in edgy_quat(), p in edgy_vec3(), r in edgy_vec3(), s in edgy_vec3()
    ) {
        let _ = (m * n, m * v, m.transpose(), m.determinant(), m.to_cols_array(), state_hash_of(&m));
        let _ = (m.transform_point3(p), m.transform_vector3(p), m.project_point3(p));
        let _ = (Mat4::from_translation(p), Mat4::from_scale(p), Mat4::from_quat(q));
        let _ = (Mat4::from_scale_rotation_translation(p, q, r), Mat4::from_cols_array(&m.to_cols_array()));
        let _ = (m3 * n3, m3 * p, m3.transpose(), m3.determinant(), m3.to_cols_array(), state_hash_of(&m3));
        let _ = (Mat3::from_scale(p), Mat3::from_quat(q), Mat3::from_cols(p, r, s));
        for inverse in [m.inverse().map(|_| ()), m3.inverse().map(|_| ())] {
            if let Err(error) = inverse {
                prop_assert!(matches!(error, MathError::SingularMatrix { .. }), "{:?}", error);
            }
        }
        if let Err(error) = Mat4::look_to(p, r, s) {
            let expected = matches!(
                error,
                MathError::NonFiniteInput { parameter: "eye" | "direction" | "up" }
                    | MathError::ZeroLengthVector { parameter: "direction" }
                    | MathError::InvalidViewBasis
            );
            prop_assert!(expected, "{:?}", error);
        }
        if let Err(error) = Mat4::orthographic(p.x, p.y, p.z, r.x, r.y, r.z) {
            prop_assert!(matches!(error, MathError::InvalidProjection { .. }), "{:?}", error);
        }
    }

    /// Verifies: MATH-R005
    #[test]
    fn quaternion_and_affine_functions_never_panic(
        q in edgy_quat(), o in edgy_quat(), p in edgy_vec3(), r in edgy_vec3(), m3 in edgy_mat3(),
        points in prop::collection::vec(edgy_vec3(), 0..=67)
    ) {
        let _ = (q * o, q * p, q.dot(o), q.length(), q.conjugate(), state_hash_of(&q));
        if let Err(error) = q.normalize() {
            prop_assert!(is_normalize_error(&error), "{:?}", error);
        }
        let a = Affine3 { matrix3: m3, translation: r };
        let b = Affine3::from_scale_rotation_translation(p, q, r);
        let _ = (a * b, a.transform_point3(p), a.transform_vector3(p), Mat4::from(a), state_hash_of(&a));
        if let Err(error) = a.inverse() {
            prop_assert!(matches!(error, MathError::SingularMatrix { .. }), "{:?}", error);
        }
        let mut buffer = points.clone();
        a.transform_points(&mut buffer);
        let mut buffer = points;
        reference::affine3_transform_points(&a, &mut buffer);
        let _ = (reference::quat_mul(q, o), reference::mat4_mul(Mat4::from(a), Mat4::from(b)));
        let _ = reference::mat4_mul_vec4(Mat4::from(a), p.extend(1.0));
    }
}
