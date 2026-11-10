//! Shared test helpers of `liminal_math`: the input domains of spec §4 as proptest strategies,
//! "approximately equal at scale" (spec §4) and bit-pattern helpers.

use liminal_math::{Mat3, Mat4, Quat, Vec2, Vec3, Vec4};
use proptest::prelude::*;

/// Spec §4 "Approximately equal at scale `s`": `abs(a − b) ≤ 1e-5 × max(1, s)` in `f64`.
pub fn approx(a: f32, b: f64, scale: f64) -> bool {
    (f64::from(a) - b).abs() <= 1e-5 * scale.max(1.0)
}

/// `approx` for every component pair of two `Vec3`.
pub fn approx_vec3(a: Vec3, b: Vec3, scale: f64) -> bool {
    a.to_array()
        .iter()
        .zip(b.to_array())
        .all(|(x, y)| approx(*x, f64::from(y), scale))
}

/// An `f32` of any bit pattern (NaN, infinities, subnormal numbers, `−0.0` included).
pub fn any_f32() -> impl Strategy<Value = f32> {
    any::<u32>().prop_map(f32::from_bits)
}

pub fn any_vec2() -> impl Strategy<Value = Vec2> {
    any::<[u32; 2]>().prop_map(|b| Vec2::from_array(b.map(f32::from_bits)))
}

pub fn any_vec3() -> impl Strategy<Value = Vec3> {
    any::<[u32; 3]>().prop_map(|b| Vec3::from_array(b.map(f32::from_bits)))
}

pub fn any_vec4() -> impl Strategy<Value = Vec4> {
    any::<[u32; 4]>().prop_map(|b| Vec4::from_array(b.map(f32::from_bits)))
}

pub fn any_quat() -> impl Strategy<Value = Quat> {
    any_vec4().prop_map(|v| Quat::from_xyzw(v.x, v.y, v.z, v.w))
}

pub fn any_mat3() -> impl Strategy<Value = Mat3> {
    any::<[u32; 9]>().prop_map(|b| Mat3::from_cols_array(&b.map(f32::from_bits)))
}

pub fn any_mat4() -> impl Strategy<Value = Mat4> {
    any::<[u32; 16]>().prop_map(|b| Mat4::from_cols_array(&b.map(f32::from_bits)))
}

/// Mostly arbitrary bit patterns, with a share of special values so that NaN, infinities,
/// zeros and subnormal numbers occur often.
pub fn edgy_f32() -> impl Strategy<Value = f32> {
    prop_oneof![
        3 => any_f32(),
        1 => prop::sample::select(vec![
            0.0_f32,
            -0.0,
            1.0,
            -1.0,
            f32::NAN,
            -f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::MIN_POSITIVE,
            f32::from_bits(1),
            f32::MAX,
            f32::MIN,
        ]),
    ]
}

pub fn edgy_vec3() -> impl Strategy<Value = Vec3> {
    [edgy_f32(), edgy_f32(), edgy_f32()].prop_map(Vec3::from_array)
}

pub fn edgy_vec4() -> impl Strategy<Value = Vec4> {
    [edgy_f32(), edgy_f32(), edgy_f32(), edgy_f32()].prop_map(Vec4::from_array)
}

/// D-vec component: −100 to 100.
pub fn d_component() -> impl Strategy<Value = f32> {
    -100.0_f32..=100.0
}

/// D-vec for `Vec2`.
pub fn d_vec2() -> impl Strategy<Value = Vec2> {
    [d_component(), d_component()].prop_map(Vec2::from_array)
}

/// D-vec for `Vec3`.
pub fn d_vec3() -> impl Strategy<Value = Vec3> {
    [d_component(), d_component(), d_component()].prop_map(Vec3::from_array)
}

/// D-vec for `Vec4`.
pub fn d_vec4() -> impl Strategy<Value = Vec4> {
    [d_component(), d_component(), d_component(), d_component()].prop_map(Vec4::from_array)
}

/// D-unit: four components in −1 to 1 with length at least 0.1, normalized.
pub fn d_unit() -> impl Strategy<Value = Quat> {
    [
        -1.0_f32..=1.0,
        -1.0_f32..=1.0,
        -1.0_f32..=1.0,
        -1.0_f32..=1.0,
    ]
    .prop_map(|[x, y, z, w]| Quat::from_xyzw(x, y, z, w))
    .prop_filter("length at least 0.1", |q| q.length() >= 0.1)
    .prop_map(|q| q.normalize().expect("test setup: length at least 0.1"))
}

/// D-trs: (scale, rotation, translation) with scale components in 0.5 to 2, rotation in
/// D-unit and translation components in −10 to 10.
pub fn d_trs() -> impl Strategy<Value = (Vec3, Quat, Vec3)> {
    let scale = [0.5_f32..=2.0, 0.5_f32..=2.0, 0.5_f32..=2.0].prop_map(Vec3::from_array);
    let translation =
        [-10.0_f32..=10.0, -10.0_f32..=10.0, -10.0_f32..=10.0].prop_map(Vec3::from_array);
    (scale, d_unit(), translation)
}

/// True if `a` and `b` are the same bits, or both NaN (spec §4 "NaN bits").
pub fn same_bits(a: f32, b: f32) -> bool {
    (a.is_nan() && b.is_nan()) || a.to_bits() == b.to_bits()
}

/// `same_bits` for every component of two arrays.
pub fn same_bits_all(a: &[f32], b: &[f32]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| same_bits(*x, *y))
}

/// The bits of every component, for `assert_eq!` on exact results.
pub fn bits<const N: usize>(values: [f32; N]) -> [u32; N] {
    values.map(f32::to_bits)
}
