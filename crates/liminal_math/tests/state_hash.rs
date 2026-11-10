//! `StateHash` encodings (spec §5.8): MATH-R007. Oracle: the component list written with
//! `write_f32` by hand.

use liminal_core::hash::{StateHasher, state_hash_of};
use liminal_math::{Affine3, Mat3, Mat4, Quat, Vec2, Vec3, Vec4};
use proptest::prelude::*;

/// The hash of `values` written one by one with `write_f32`, no length prefix.
fn hash_f32s(values: &[f32]) -> u64 {
    let mut hasher = StateHasher::new();
    for value in values {
        hasher.write_f32(*value);
    }
    hasher.finish()
}

proptest! {
    /// Verifies: MATH-R007
    #[test]
    fn encodings_are_the_components_in_order(raw in any::<[u32; 16]>()) {
        let e = raw.map(f32::from_bits);
        let v2 = Vec2::new(e[0], e[1]);
        prop_assert_eq!(state_hash_of(&v2), hash_f32s(&v2.to_array()));
        let v3 = Vec3::new(e[0], e[1], e[2]);
        prop_assert_eq!(state_hash_of(&v3), hash_f32s(&v3.to_array()));
        let v4 = Vec4::new(e[0], e[1], e[2], e[3]);
        prop_assert_eq!(state_hash_of(&v4), hash_f32s(&v4.to_array()));
        let q = Quat::from_xyzw(e[0], e[1], e[2], e[3]);
        prop_assert_eq!(state_hash_of(&q), hash_f32s(&[e[0], e[1], e[2], e[3]]));

        let m4 = Mat4::from_cols_array(&e);
        prop_assert_eq!(state_hash_of(&m4), hash_f32s(&m4.to_cols_array()));
        let mut e9 = [0.0_f32; 9];
        e9.copy_from_slice(&e[..9]);
        let m3 = Mat3::from_cols_array(&e9);
        prop_assert_eq!(state_hash_of(&m3), hash_f32s(&m3.to_cols_array()));

        let affine = Affine3 { matrix3: m3, translation: Vec3::new(e[9], e[10], e[11]) };
        prop_assert_eq!(state_hash_of(&affine), hash_f32s(&e[..12]));
    }
}

/// Verifies: MATH-R007
#[test]
fn order_and_negative_zero_are_part_of_the_state() {
    assert_ne!(
        state_hash_of(&Vec3::new(1.0, 2.0, 3.0)),
        state_hash_of(&Vec3::new(3.0, 2.0, 1.0))
    );
    assert_ne!(
        state_hash_of(&Vec2::new(0.0, 0.0)),
        state_hash_of(&Vec2::new(-0.0, 0.0))
    );
    // No length prefix: a Vec3 hashes like three loose floats.
    let mut hasher = StateHasher::new();
    hasher.write_f32(1.0);
    hasher.write_f32(2.0);
    hasher.write_f32(3.0);
    assert_eq!(state_hash_of(&Vec3::new(1.0, 2.0, 3.0)), hasher.finish());
}
