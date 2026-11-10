//! Layout, constructors and constants (spec §10 "Layout, conventions, constructors"): MATH-R001,
//! MATH-R003, MATH-R020, MATH-R040. Oracles: `size_of`, `align_of`, `offset_of` and
//! hand-written values.

use core::mem::{align_of, offset_of, size_of};

use liminal_math::{Affine3, Mat3, Mat4, Quat, Vec2, Vec3, Vec4};
use proptest::prelude::*;

#[allow(dead_code, reason = "each test file uses a subset of the helpers")]
#[path = "common/math.rs"]
mod common;
use common::{any_f32, bits};

/// Verifies: MATH-R001
#[test]
fn sizes_and_alignments() {
    assert_eq!((size_of::<Vec2>(), align_of::<Vec2>()), (8, 4));
    assert_eq!((size_of::<Vec3>(), align_of::<Vec3>()), (12, 4));
    assert_eq!((size_of::<Vec4>(), align_of::<Vec4>()), (16, 16));
    assert_eq!((size_of::<Quat>(), align_of::<Quat>()), (16, 16));
    assert_eq!((size_of::<Mat3>(), align_of::<Mat3>()), (36, 4));
    assert_eq!((size_of::<Mat4>(), align_of::<Mat4>()), (64, 16));
    assert_eq!((size_of::<Affine3>(), align_of::<Affine3>()), (48, 4));
}

/// Verifies: MATH-R001
#[test]
fn fields_are_at_offsets_four_times_their_index() {
    assert_eq!([offset_of!(Vec2, x), offset_of!(Vec2, y)], [0, 4]);
    assert_eq!(
        [offset_of!(Vec3, x), offset_of!(Vec3, y), offset_of!(Vec3, z)],
        [0, 4, 8]
    );
    assert_eq!(
        [
            offset_of!(Vec4, x),
            offset_of!(Vec4, y),
            offset_of!(Vec4, z),
            offset_of!(Vec4, w)
        ],
        [0, 4, 8, 12]
    );
    assert_eq!(
        [
            offset_of!(Quat, x),
            offset_of!(Quat, y),
            offset_of!(Quat, z),
            offset_of!(Quat, w)
        ],
        [0, 4, 8, 12]
    );
    assert_eq!(
        [
            offset_of!(Mat3, x_axis),
            offset_of!(Mat3, y_axis),
            offset_of!(Mat3, z_axis)
        ],
        [0, 12, 24]
    );
    assert_eq!(
        [
            offset_of!(Mat4, x_axis),
            offset_of!(Mat4, y_axis),
            offset_of!(Mat4, z_axis),
            offset_of!(Mat4, w_axis)
        ],
        [0, 16, 32, 48]
    );
    assert_eq!(
        [
            offset_of!(Affine3, matrix3),
            offset_of!(Affine3, translation)
        ],
        [0, 36]
    );
}

/// Verifies: MATH-R001
#[test]
fn vector_defaults_are_zero() {
    assert_eq!(bits(Vec2::default().to_array()), bits(Vec2::ZERO.to_array()));
    assert_eq!(bits(Vec3::default().to_array()), bits(Vec3::ZERO.to_array()));
    assert_eq!(bits(Vec4::default().to_array()), bits(Vec4::ZERO.to_array()));
    assert_eq!(bits(Vec3::ZERO.to_array()), [0; 3]);
}

/// Verifies: MATH-R020
#[test]
fn constants_have_the_specified_values() {
    assert_eq!(Vec2::ONE.to_array(), [1.0, 1.0]);
    assert_eq!(Vec2::X.to_array(), [1.0, 0.0]);
    assert_eq!(Vec2::Y.to_array(), [0.0, 1.0]);
    assert_eq!(bits(Vec3::ZERO.to_array()), [0; 3]);
    assert_eq!(Vec3::ONE.to_array(), [1.0; 3]);
    assert_eq!(Vec3::X.to_array(), [1.0, 0.0, 0.0]);
    assert_eq!(Vec3::Y.to_array(), [0.0, 1.0, 0.0]);
    assert_eq!(Vec3::Z.to_array(), [0.0, 0.0, 1.0]);
    assert_eq!(bits(Vec4::ZERO.to_array()), [0; 4]);
    assert_eq!(Vec4::ONE.to_array(), [1.0; 4]);
    assert_eq!(Vec4::X.to_array(), [1.0, 0.0, 0.0, 0.0]);
    assert_eq!(Vec4::Y.to_array(), [0.0, 1.0, 0.0, 0.0]);
    assert_eq!(Vec4::Z.to_array(), [0.0, 0.0, 1.0, 0.0]);
    assert_eq!(Vec4::W.to_array(), [0.0, 0.0, 0.0, 1.0]);
    // Zeros in the constants are +0.0.
    for array in [Vec3::X.to_array(), Vec3::Y.to_array(), Vec3::Z.to_array()] {
        assert!(array.iter().all(|c| c.to_bits() == 0 || *c == 1.0));
    }
}

proptest! {
    /// Verifies: MATH-R020
    #[test]
    fn new_splat_and_arrays_keep_bits(a in any_f32(), b in any_f32(), c in any_f32(), d in any_f32()) {
        let v2 = Vec2::new(a, b);
        prop_assert_eq!([v2.x.to_bits(), v2.y.to_bits()], bits([a, b]));
        let v3 = Vec3::new(a, b, c);
        prop_assert_eq!(bits([v3.x, v3.y, v3.z]), bits([a, b, c]));
        let v4 = Vec4::new(a, b, c, d);
        prop_assert_eq!(bits([v4.x, v4.y, v4.z, v4.w]), bits([a, b, c, d]));

        prop_assert_eq!(bits(Vec2::splat(a).to_array()), [a.to_bits(); 2]);
        prop_assert_eq!(bits(Vec3::splat(a).to_array()), [a.to_bits(); 3]);
        prop_assert_eq!(bits(Vec4::splat(a).to_array()), [a.to_bits(); 4]);

        prop_assert_eq!(bits(Vec2::from_array([a, b]).to_array()), bits([a, b]));
        prop_assert_eq!(bits(Vec3::from_array([a, b, c]).to_array()), bits([a, b, c]));
        prop_assert_eq!(bits(Vec4::from_array([a, b, c, d]).to_array()), bits([a, b, c, d]));
        prop_assert_eq!(Vec3::from_array([a, b, c]).x.to_bits(), a.to_bits());

        prop_assert_eq!(bits(v3.extend(d).to_array()), bits([a, b, c, d]));
        prop_assert_eq!(bits(v4.truncate().to_array()), bits([a, b, c]));
    }

    /// Verifies: MATH-R003
    #[test]
    fn cols_arrays_round_trip_with_bits(raw in any::<[u32; 16]>()) {
        let e16 = raw.map(f32::from_bits);
        let m4 = Mat4::from_cols_array(&e16);
        prop_assert_eq!(bits(m4.to_cols_array()), raw);
        prop_assert_eq!(bits(Mat4::from_cols_array(&m4.to_cols_array()).to_cols_array()), raw);

        let mut e9 = [0.0_f32; 9];
        e9.copy_from_slice(&e16[..9]);
        let m3 = Mat3::from_cols_array(&e9);
        prop_assert_eq!(bits(m3.to_cols_array()), bits(e9));
    }
}

/// Verifies: MATH-R003
#[test]
fn cols_array_index_is_column_times_rows_plus_row() {
    let e: [f32; 16] = core::array::from_fn(|i| i as f32);
    let m = Mat4::from_cols_array(&e);
    // Row r, column c is e[4c + r]: column 2 holds 8, 9, 10, 11.
    assert_eq!(m.z_axis, Vec4::new(8.0, 9.0, 10.0, 11.0));
    assert_eq!(m.x_axis.y, 1.0); // row 1, column 0
    assert_eq!(m.y_axis.x, 4.0); // row 0, column 1
    let m = Mat4::from_cols_array(&e);
    assert_eq!(m, Mat4::from_cols_array(&m.to_cols_array()));

    let e: [f32; 9] = core::array::from_fn(|i| i as f32);
    let m = Mat3::from_cols_array(&e);
    assert_eq!(m.x_axis, Vec3::new(0.0, 1.0, 2.0));
    assert_eq!(m.y_axis, Vec3::new(3.0, 4.0, 5.0));
    assert_eq!(m.z_axis, Vec3::new(6.0, 7.0, 8.0));
}

/// Verifies: MATH-R003
#[test]
fn from_cols_stores_columns_in_order() {
    let (a, b, c, d) = (
        Vec4::new(1.0, 2.0, 3.0, 4.0),
        Vec4::new(5.0, 6.0, 7.0, 8.0),
        Vec4::new(9.0, 10.0, 11.0, 12.0),
        Vec4::new(13.0, 14.0, 15.0, 16.0),
    );
    let m = Mat4::from_cols(a, b, c, d);
    assert_eq!((m.x_axis, m.y_axis, m.z_axis, m.w_axis), (a, b, c, d));
    let m = Mat3::from_cols(a.truncate(), b.truncate(), c.truncate());
    assert_eq!(
        (m.x_axis, m.y_axis, m.z_axis),
        (a.truncate(), b.truncate(), c.truncate())
    );
}

/// The matrix with these rows, for writing expected values readably.
fn mat4_rows(rows: [[f32; 4]; 4]) -> Mat4 {
    Mat4::from_cols_array(&core::array::from_fn(|i| rows[i % 4][i / 4]))
}

/// Verifies: MATH-R040
#[test]
fn identity_scale_and_translation_matrices() {
    let identity = mat4_rows([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    assert_eq!(bits(Mat4::IDENTITY.to_cols_array()), bits(identity.to_cols_array()));
    assert_eq!(
        bits(Mat3::IDENTITY.to_cols_array()),
        bits([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0])
    );
    let s = Vec3::new(2.0, -3.0, 0.5);
    assert_eq!(
        bits(Mat3::from_scale(s).to_cols_array()),
        bits([2.0, 0.0, 0.0, 0.0, -3.0, 0.0, 0.0, 0.0, 0.5])
    );
    let scaled = mat4_rows([
        [2.0, 0.0, 0.0, 0.0],
        [0.0, -3.0, 0.0, 0.0],
        [0.0, 0.0, 0.5, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    assert_eq!(bits(Mat4::from_scale(s).to_cols_array()), bits(scaled.to_cols_array()));
    let t = Vec3::new(-1.0, 2.0, 7.0);
    let translated = mat4_rows([
        [1.0, 0.0, 0.0, -1.0],
        [0.0, 1.0, 0.0, 2.0],
        [0.0, 0.0, 1.0, 7.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    // `bits` comparison: off-diagonal and unset elements are +0.0, not −0.0.
    assert_eq!(
        bits(Mat4::from_translation(t).to_cols_array()),
        bits(translated.to_cols_array())
    );
    // A negative zero scale stays on the diagonal; the rest stays +0.0.
    let m = Mat3::from_scale(Vec3::splat(-0.0)).to_cols_array();
    assert_eq!(
        bits(m),
        bits([-0.0, 0.0, 0.0, 0.0, -0.0, 0.0, 0.0, 0.0, -0.0])
    );
}
