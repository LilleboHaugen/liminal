//! `liminal_math::reference` (spec §5.6): MATH-R130. Oracles: hand-computed exact results and
//! a scan of `src/reference.rs`.

#![allow(
    clippy::float_cmp,
    clippy::many_single_char_names,
    reason = "exact results are the contract (spec §4 \"Exact expression\"); the requirement formulas use one-letter names"
)]

use liminal_math::{Affine3, Mat3, Mat4, Quat, Vec3, Vec4, reference};
use proptest::prelude::*;

#[allow(dead_code, reason = "each test file uses a subset of the helpers")]
#[path = "common/math.rs"]
mod common;
use common::{any_mat3, any_mat4, any_quat, any_vec3, any_vec4, same_bits_all};

/// Verifies: MATH-R130
#[test]
fn module_has_exactly_the_four_functions_and_no_unsafe_or_arch() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/reference.rs");
    let source = std::fs::read_to_string(path).unwrap();
    let code: Vec<&str> = source
        .lines()
        .map(str::trim_start)
        .filter(|line| !line.starts_with("//"))
        .collect();
    let functions: Vec<&str> = code
        .iter()
        .filter_map(|line| line.strip_prefix("pub fn "))
        .map(|rest| &rest[..rest.find('(').unwrap()])
        .collect();
    assert_eq!(
        functions,
        [
            "mat4_mul",
            "mat4_mul_vec4",
            "quat_mul",
            "affine3_transform_points"
        ]
    );
    assert!(
        !code
            .iter()
            .any(|line| line.contains("pub ") && !line.starts_with("pub fn "))
    );
    for forbidden in ["unsafe", "core::arch", "std::arch", "arch::"] {
        assert!(
            !code.iter().any(|line| line.contains(forbidden)),
            "{forbidden}"
        );
    }
}

/// Verifies: MATH-R130
#[test]
fn hand_computed_results() {
    // Rows (1 2 0 0), (0 1 0 3), (0 0 2 0), (0 0 0 1) times (1, 1, 1, 1).
    let a = Mat4::from_cols_array(&[
        1.0, 0.0, 0.0, 0.0, 2.0, 1.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 3.0, 0.0, 1.0,
    ]);
    assert_eq!(
        reference::mat4_mul_vec4(a, Vec4::ONE),
        Vec4::new(3.0, 4.0, 2.0, 1.0)
    );
    // a × a: rows (1 4 0 6), (0 1 0 6), (0 0 4 0), (0 0 0 1).
    let expected = Mat4::from_cols_array(&[
        1.0, 0.0, 0.0, 0.0, 4.0, 1.0, 0.0, 0.0, 0.0, 0.0, 4.0, 0.0, 6.0, 6.0, 0.0, 1.0,
    ]);
    assert_eq!(reference::mat4_mul(a, a), expected);
    // i × j = k, j × i = −k.
    let (i, j) = (
        Quat::from_xyzw(1.0, 0.0, 0.0, 0.0),
        Quat::from_xyzw(0.0, 1.0, 0.0, 0.0),
    );
    assert_eq!(
        reference::quat_mul(i, j),
        Quat::from_xyzw(0.0, 0.0, 1.0, 0.0)
    );
    assert_eq!(
        reference::quat_mul(j, i),
        Quat::from_xyzw(0.0, 0.0, -1.0, 0.0)
    );
    assert_eq!(
        reference::quat_mul(i, i),
        Quat::from_xyzw(0.0, 0.0, 0.0, -1.0)
    );
    // Association order of the product: x = ((w b.x + x b.w) + y b.z) − z b.y with
    // w b.x = 1e8, x b.w = −1e8, y b.z = 1, z b.y = 1 gives 0.
    let p = Quat::from_xyzw(-1e8, 1.0, 1.0, 1e8);
    let q = Quat::from_xyzw(1.0, 1.0, 1.0, 1.0);
    assert_eq!(reference::quat_mul(p, q).x, 0.0);

    let affine = Affine3 {
        matrix3: Mat3::from_scale(Vec3::new(2.0, 2.0, 2.0)),
        translation: Vec3::new(0.0, 0.0, 1.0),
    };
    let mut points = [Vec3::X, Vec3::ZERO, Vec3::new(-1.0, 0.5, 0.0)];
    reference::affine3_transform_points(&affine, &mut points);
    assert_eq!(
        points,
        [
            Vec3::new(2.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(-2.0, 1.0, 1.0)
        ]
    );
}

proptest! {
    /// Verifies: MATH-R130
    #[test]
    fn reference_items_are_the_exact_products(
        a in any_mat4(), b in any_mat4(), v in any_vec4(), p in any_quat(), q in any_quat(),
        m3 in any_mat3(), t in any_vec3(), points in prop::collection::vec(any_vec3(), 0..=8)
    ) {
        let mv = ((a.x_axis * v.x + a.y_axis * v.y) + a.z_axis * v.z) + a.w_axis * v.w;
        prop_assert!(same_bits_all(&reference::mat4_mul_vec4(a, v).to_array(), &mv.to_array()));
        let ab = reference::mat4_mul(a, b);
        for (column, of_b) in [(ab.x_axis, b.x_axis), (ab.y_axis, b.y_axis), (ab.z_axis, b.z_axis), (ab.w_axis, b.w_axis)] {
            let expected = ((a.x_axis * of_b.x + a.y_axis * of_b.y) + a.z_axis * of_b.z) + a.w_axis * of_b.w;
            prop_assert!(same_bits_all(&column.to_array(), &expected.to_array()));
        }
        let pq = reference::quat_mul(p, q);
        let expected = [
            ((p.w * q.x + p.x * q.w) + p.y * q.z) - p.z * q.y,
            ((p.w * q.y - p.x * q.z) + p.y * q.w) + p.z * q.x,
            ((p.w * q.z + p.x * q.y) - p.y * q.x) + p.z * q.w,
            ((p.w * q.w - p.x * q.x) - p.y * q.y) - p.z * q.z,
        ];
        prop_assert!(same_bits_all(&[pq.x, pq.y, pq.z, pq.w], &expected));
        let affine = Affine3 { matrix3: m3, translation: t };
        let mut transformed = points.clone();
        reference::affine3_transform_points(&affine, &mut transformed);
        for (got, point) in transformed.iter().zip(&points) {
            let expected = ((m3.x_axis * point.x + m3.y_axis * point.y) + m3.z_axis * point.z) + t;
            prop_assert!(same_bits_all(&got.to_array(), &expected.to_array()));
        }
        // The public items use them (no SIMD path yet, task M1-T022).
        prop_assert!(same_bits_all(&(a * v).to_array(), &mv.to_array()));
    }
}
