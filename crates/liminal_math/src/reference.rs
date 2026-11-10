//! Scalar reference implementations (oracles) of the operations that get SIMD paths (spec §5.6,
//! MATH-R130; PERFORMANCE.md "Reference implementations reachable from benchmarks").
//!
//! Each function computes the exact expression of its requirement with plain `f32` operations,
//! contains no `unsafe` and no `core::arch` item, and is always compiled. The public operators
//! and [`Affine3::transform_points`] return the same bits as these functions; on targets without
//! a SIMD path they call them.
//!
//! | Public item | Reference item |
//! |---|---|
//! | `Mat4 * Mat4` | [`mat4_mul`] |
//! | `Mat4 * Vec4` | [`mat4_mul_vec4`] |
//! | `Quat * Quat` | [`quat_mul`] |
//! | [`Affine3::transform_points`] | [`affine3_transform_points`] |
//!
//! # Examples
//!
//! ```
//! use liminal_math::{Mat4, Vec3, Vec4, reference};
//!
//! let m = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
//! let v = Vec4::new(1.0, 1.0, 1.0, 1.0);
//! assert_eq!(reference::mat4_mul_vec4(m, v), Vec4::new(2.0, 3.0, 4.0, 1.0));
//! assert_eq!(reference::mat4_mul_vec4(m, v), m * v);
//! ```

use crate::{Affine3, Mat4, Quat, Vec3, Vec4};

/// Oracle for `<Mat4 as Mul<Mat4>>::mul`: the matrix whose columns are `a * b.x_axis` to
/// `a * b.w_axis`, each by [`mat4_mul_vec4`] (MATH-R042).
#[must_use]
pub fn mat4_mul(a: Mat4, b: Mat4) -> Mat4 {
    Mat4::from_cols(
        mat4_mul_vec4(a, b.x_axis),
        mat4_mul_vec4(a, b.y_axis),
        mat4_mul_vec4(a, b.z_axis),
        mat4_mul_vec4(a, b.w_axis),
    )
}

/// Oracle for `<Mat4 as Mul<Vec4>>::mul`:
/// `((x_axis × v.x + y_axis × v.y) + z_axis × v.z) + w_axis × v.w` (MATH-R041).
#[must_use]
pub fn mat4_mul_vec4(matrix: Mat4, vector: Vec4) -> Vec4 {
    matrix.x_axis * vector.x
        + matrix.y_axis * vector.y
        + matrix.z_axis * vector.z
        + matrix.w_axis * vector.w
}

/// Oracle for `<Quat as Mul<Quat>>::mul`: the Hamilton product of MATH-R071.
#[must_use]
pub fn quat_mul(a: Quat, b: Quat) -> Quat {
    Quat::from_xyzw(
        a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
        a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
        a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
        a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
    )
}

/// Oracle for [`Affine3::transform_points`]: replaces each element `p` with
/// `affine.transform_point3(p)` (MATH-R091).
pub fn affine3_transform_points(affine: &Affine3, points: &mut [Vec3]) {
    for point in points {
        *point = affine.transform_point3(*point);
    }
}
