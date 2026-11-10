//! Vector operations (spec §5.1): MATH-R021 to MATH-R026. Exact expressions are checked with
//! inputs whose result changes under a different association order or a fused multiply-add,
//! and with property tests against the expression written out with `f32` operations.

#![allow(
    clippy::float_cmp,
    clippy::many_single_char_names,
    reason = "exact results are the contract (spec §4 \"Exact expression\"); the requirement formulas use one-letter names"
)]

use liminal_math::{MathError, Vec2, Vec3, Vec4};
use proptest::prelude::*;

#[allow(dead_code, reason = "each test file uses a subset of the helpers")]
#[path = "common/math.rs"]
mod common;
use common::{
    any_f32, any_vec2, any_vec3, any_vec4, approx, bits, d_vec3, d_vec4, edgy_vec3, same_bits,
    same_bits_all,
};

/// `1 + 2^-12`: its square `1 + 2^-11 + 2^-24` rounds to `1 + 2^-11`, so `x × x − (1 + 2^-11)`
/// is 0 without and `2^-24` with a fused multiply-add.
const FMA_PROBE: f32 = 1.0 + 1.0 / 4096.0;
const FMA_PROBE_SQUARE: f32 = 1.0 + 1.0 / 2048.0;

proptest! {
    /// Verifies: MATH-R021
    #[test]
    fn operators_are_one_operation_per_component(a in any_vec4(), b in any_vec4(), s in any_f32()) {
        let (aa, ba) = (a.to_array(), b.to_array());
        let each = |f: &dyn Fn(f32, f32) -> f32| -> Vec<f32> {
            aa.iter().zip(ba).map(|(x, y)| f(*x, y)).collect()
        };
        prop_assert!(same_bits_all(&(a + b).to_array(), &each(&|x, y| x + y)));
        prop_assert!(same_bits_all(&(a - b).to_array(), &each(&|x, y| x - y)));
        prop_assert!(same_bits_all(&(a * b).to_array(), &each(&|x, y| x * y)));
        prop_assert!(same_bits_all(&(a * s).to_array(), &aa.map(|x| x * s)));
        prop_assert!(same_bits_all(&(s * a).to_array(), &aa.map(|x| s * x)));
        prop_assert!(same_bits_all(&(a / s).to_array(), &aa.map(|x| x / s)));
        // Negation flips the sign bit only, also of NaN.
        prop_assert_eq!(bits((-a).to_array()), aa.map(|x| x.to_bits() ^ 0x8000_0000));

        let mut c = a;
        c += b;
        prop_assert!(same_bits_all(&c.to_array(), &(a + b).to_array()));
        let mut c = a;
        c -= b;
        prop_assert!(same_bits_all(&c.to_array(), &(a - b).to_array()));
        let mut c = a;
        c *= s;
        prop_assert!(same_bits_all(&c.to_array(), &(a * s).to_array()));
        let mut c = a;
        c /= s;
        prop_assert!(same_bits_all(&c.to_array(), &(a / s).to_array()));
    }

    /// Verifies: MATH-R021
    #[test]
    fn vec2_and_vec3_operators_match_vec4(a in any_vec3(), b in any_vec3(), s in any_f32()) {
        let (a4, b4) = (a.extend(0.0), b.extend(0.0));
        prop_assert!(same_bits_all(&(a + b).to_array(), &(a4 + b4).truncate().to_array()));
        prop_assert!(same_bits_all(&(a - b).to_array(), &(a4 - b4).truncate().to_array()));
        prop_assert!(same_bits_all(&(a * b).to_array(), &(a4 * b4).truncate().to_array()));
        prop_assert!(same_bits_all(&(a / s).to_array(), &(a4 / s).truncate().to_array()));
        prop_assert!(same_bits_all(&(s * a).to_array(), &(s * a4).truncate().to_array()));
        prop_assert_eq!(bits((-a).to_array()), bits((-a4).truncate().to_array()));
        let (a2, b2) = (Vec2::new(a.x, a.y), Vec2::new(b.x, b.y));
        let mut c2 = a2;
        c2 += b2;
        c2 -= a2;
        c2 *= s;
        c2 /= s;
        let expected = [((a.x + b.x) - a.x) * s / s, ((a.y + b.y) - a.y) * s / s];
        prop_assert!(same_bits_all(&c2.to_array(), &expected));
        prop_assert!(same_bits_all(&(a2 * b2 * s).to_array(), &[a.x * b.x * s, a.y * b.y * s]));
        prop_assert!(same_bits_all(&(s * (a2 + b2)).to_array(), &[s * (a.x + b.x), s * (a.y + b.y)]));
        let mut c3 = a;
        c3 += b;
        c3 -= b;
        c3 *= s;
        prop_assert!(same_bits_all(&c3.to_array(), &(((a + b) - b) * s).to_array()));
        prop_assert!(same_bits_all(&(-a2).to_array(), &[-a.x, -a.y]));
    }

    /// Verifies: MATH-R022
    #[test]
    fn dot_and_length_are_the_left_to_right_sums(a in any_vec4(), b in any_vec4()) {
        let d4 = ((a.x * b.x + a.y * b.y) + a.z * b.z) + a.w * b.w;
        prop_assert!(same_bits(a.dot(b), d4));
        let (a3, b3) = (a.truncate(), b.truncate());
        prop_assert!(same_bits(a3.dot(b3), (a.x * b.x + a.y * b.y) + a.z * b.z));
        let (a2, b2) = (Vec2::new(a.x, a.y), Vec2::new(b.x, b.y));
        prop_assert!(same_bits(a2.dot(b2), a.x * b.x + a.y * b.y));
        prop_assert!(same_bits(a.length_squared(), a.dot(a)));
        prop_assert!(same_bits(a.length(), a.dot(a).sqrt()));
        prop_assert!(same_bits(a3.length(), a3.dot(a3).sqrt()));
        prop_assert!(same_bits(a2.length(), a2.dot(a2).sqrt()));
    }
}

/// Verifies: MATH-R022
#[test]
fn dot_association_order_and_no_fma() {
    // (1e8 − 1e8) + 1 = 1, while 1e8 + (−1e8 + 1) = 0.
    let a = Vec3::new(1e8, -1e8, 1.0);
    assert_eq!(a.dot(Vec3::ONE), 1.0);
    let a = Vec4::new(1e8, -1e8, 1.0, -1.0);
    assert_eq!(a.dot(Vec4::ONE), 0.0);
    assert_eq!(Vec4::new(1e8, -1e8, 0.0, 1.0).dot(Vec4::ONE), 1.0);
    // A fused multiply-add of the first product would give 2^-24.
    let a = Vec2::new(FMA_PROBE, FMA_PROBE_SQUARE);
    let b = Vec2::new(FMA_PROBE, -1.0);
    assert_eq!(a.dot(b).to_bits(), 0);
    // No rescaling: overflow and underflow as the requirement states.
    assert_eq!(Vec3::new(2e19, 0.0, 0.0).length(), f32::INFINITY);
    assert_eq!(Vec3::new(1e-24, 0.0, 0.0).length(), 0.0);
}

proptest! {
    /// Verifies: MATH-R023
    #[test]
    fn cross_is_the_exact_expression_and_antisymmetric(a in any_vec3(), b in any_vec3()) {
        let c = a.cross(b);
        let expected = [
            a.y * b.z - a.z * b.y,
            a.z * b.x - a.x * b.z,
            a.x * b.y - a.y * b.x,
        ];
        prop_assert!(same_bits_all(&c.to_array(), &expected));
        let minus_ba = -(b.cross(a));
        for (x, y) in c.to_array().into_iter().zip(minus_ba.to_array()) {
            prop_assert!((x.is_nan() && y.is_nan()) || x == y, "{} vs {}", x, y);
        }
    }

    /// Verifies: MATH-R023
    #[test]
    fn cross_is_antisymmetric_for_special_values(a in edgy_vec3(), b in edgy_vec3()) {
        let c = a.cross(b);
        let minus_ba = -(b.cross(a));
        for (x, y) in c.to_array().into_iter().zip(minus_ba.to_array()) {
            prop_assert!((x.is_nan() && y.is_nan()) || x == y, "{} vs {}", x, y);
        }
    }

    /// Verifies: MATH-R023
    #[test]
    fn cross_is_orthogonal_to_its_operands(a in d_vec3(), b in d_vec3()) {
        let c = a.cross(b);
        let (la, lb) = (f64::from(a.length()), f64::from(b.length()));
        prop_assert!(approx(c.dot(a), 0.0, la * la * lb), "{}", c.dot(a));
        prop_assert!(approx(c.dot(b), 0.0, la * lb * lb), "{}", c.dot(b));
    }
}

/// Verifies: MATH-R023
#[test]
fn cross_overflow_and_zero_sign_cases() {
    let a = Vec3::new(0.0, 3e19, 3e19);
    let (ab, ba) = (a.cross(a), -(a.cross(a)));
    assert!(ab.x.is_nan() && ba.x.is_nan());
    // p − p is +0, −(p − p) is −0: equal under `==`, different bits.
    let (p, q) = (Vec3::new(1.0, 2.0, 3.0), Vec3::new(2.0, 4.0, 6.0));
    let (pq, qp) = (p.cross(q), -(q.cross(p)));
    assert_eq!(pq, qp);
    assert_eq!(bits(pq.to_array()), [0; 3]);
    assert_eq!(bits(qp.to_array()), [0x8000_0000; 3]);
    // Exact expression with an FMA probe: x = a.y × b.z − a.z × b.y.
    let c = Vec3::new(0.0, FMA_PROBE, 1.0).cross(Vec3::new(0.0, FMA_PROBE_SQUARE, FMA_PROBE));
    assert_eq!(c.x.to_bits(), 0);
}

/// Verifies: MATH-R024
#[test]
fn normalize_errors_and_fallback() {
    let self_zero = Err(MathError::ZeroLengthVector { parameter: "self" });
    let self_non_finite = Err(MathError::NonFiniteInput { parameter: "self" });
    assert_eq!(Vec3::ZERO.normalize(), self_zero);
    assert_eq!(Vec2::ZERO.normalize(), self_zero.map(|_: Vec3| Vec2::ZERO));
    assert_eq!(
        Vec4::new(-0.0, 0.0, -0.0, 0.0).normalize(),
        self_zero.map(|_: Vec3| Vec4::ZERO)
    );
    // Underflow to a zero length.
    assert_eq!(Vec3::new(1e-30, 0.0, 0.0).normalize(), self_zero);
    assert_eq!(Vec3::new(f32::NAN, 0.0, 0.0).normalize(), self_non_finite);
    assert_eq!(
        Vec3::new(0.0, f32::INFINITY, 0.0).normalize(),
        self_non_finite
    );
    // Overflow of the squared length.
    assert_eq!(
        Vec2::new(2e19, 0.0).normalize(),
        self_non_finite.map(|_: Vec3| Vec2::ZERO)
    );
    assert_eq!(
        Vec4::new(1.0, 2.0, 2.0, 4.0).normalize(),
        Ok(Vec4::new(0.2, 0.4, 0.4, 0.8))
    );

    let fallback = Vec3::new(7.0, -0.0, 9.0);
    assert_eq!(
        bits(Vec3::ZERO.normalize_or(fallback).to_array()),
        bits(fallback.to_array())
    );
    assert_eq!(Vec3::new(0.0, 0.0, -2.0).normalize_or(fallback), -Vec3::Z);
    assert_eq!(Vec2::new(f32::NAN, 1.0).normalize_or(Vec2::X), Vec2::X);
    assert_eq!(Vec4::ZERO.normalize_or(Vec4::W), Vec4::W);
}

proptest! {
    /// Verifies: MATH-R024
    #[test]
    fn normalize_divides_by_the_length(v in any_vec4()) {
        let length = v.length();
        match v.normalize() {
            Ok(n) => {
                prop_assert!(length.is_finite() && length != 0.0);
                prop_assert!(same_bits_all(&n.to_array(), &(v / length).to_array()));
            }
            Err(MathError::NonFiniteInput { parameter: "self" }) => prop_assert!(!length.is_finite()),
            Err(MathError::ZeroLengthVector { parameter: "self" }) => prop_assert!(length == 0.0),
            Err(other) => prop_assert!(false, "unexpected {:?}", other),
        }
    }
}

/// A component with `abs` in 2^-33 (1.2e-10) to 2^34 (1.7e10, excluded), log-uniform by
/// exponent, with a random sign; or zero.
fn scaled_component() -> impl Strategy<Value = f32> {
    prop_oneof![
        1 => Just(0.0_f32),
        4 => (-33_i32..=33, 0_u32..(1 << 23), any::<bool>()).prop_map(|(exponent, mantissa, negative)| {
            let biased = u32::try_from(exponent + 127).expect("test setup: exponent above -127");
            let magnitude = f32::from_bits((biased << 23) | mantissa);
            if negative { -magnitude } else { magnitude }
        }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]
    /// Verifies: MATH-R025
    #[test]
    fn normalize_gives_unit_length_and_keeps_signs(
        components in [scaled_component(), scaled_component(), scaled_component(), scaled_component()]
    ) {
        prop_assume!(components.iter().any(|c| *c != 0.0));
        let tolerance = 4.0 * f32::EPSILON;
        let v4 = Vec4::from_array(components);
        let n4 = v4.normalize().unwrap();
        prop_assert!((n4.length() - 1.0).abs() <= tolerance, "{}", n4.length());
        for (n, c) in n4.to_array().into_iter().zip(components) {
            prop_assert_eq!(n.is_sign_negative(), c.is_sign_negative());
        }
        let [x, y, z, _] = components;
        if [x, y, z].iter().any(|c| *c != 0.0) {
            let n3 = Vec3::new(x, y, z).normalize().unwrap();
            prop_assert!((n3.length() - 1.0).abs() <= tolerance, "{}", n3.length());
            for (n, c) in n3.to_array().into_iter().zip([x, y, z]) {
                prop_assert_eq!(n.is_sign_negative(), c.is_sign_negative());
            }
        }
        if [x, y].iter().any(|c| *c != 0.0) {
            let n2 = Vec2::new(x, y).normalize().unwrap();
            prop_assert!((n2.length() - 1.0).abs() <= tolerance, "{}", n2.length());
        }
    }
}

proptest! {
    /// Verifies: MATH-R026
    #[test]
    fn min_max_abs_lerp_is_finite(a in any_vec4(), b in any_vec4(), t in any_f32()) {
        let (aa, ba) = (a.to_array(), b.to_array());
        let min: Vec<f32> = aa.iter().zip(ba).map(|(x, y)| if *x < y { *x } else { y }).collect();
        let max: Vec<f32> = aa.iter().zip(ba).map(|(x, y)| if *x > y { *x } else { y }).collect();
        prop_assert_eq!(bits(a.min(b).to_array()).to_vec(), min.iter().map(|c| c.to_bits()).collect::<Vec<_>>());
        prop_assert_eq!(bits(a.max(b).to_array()).to_vec(), max.iter().map(|c| c.to_bits()).collect::<Vec<_>>());
        prop_assert_eq!(bits(a.abs().to_array()), aa.map(|c| c.to_bits() & 0x7FFF_FFFF));
        prop_assert!(same_bits_all(&a.lerp(b, t).to_array(), &(a + (b - a) * t).to_array()));
        prop_assert_eq!(a.is_finite(), aa.iter().all(|c| c.is_finite()));

        let (a3, b3) = (a.truncate(), b.truncate());
        prop_assert_eq!(bits(a3.min(b3).to_array()), bits(a.min(b).truncate().to_array()));
        prop_assert_eq!(bits(a3.max(b3).to_array()), bits(a.max(b).truncate().to_array()));
        prop_assert_eq!(bits(a3.abs().to_array()), bits(a.abs().truncate().to_array()));
        prop_assert!(same_bits_all(&a3.lerp(b3, t).to_array(), &a.lerp(b, t).truncate().to_array()));
        prop_assert_eq!(a3.is_finite(), aa[..3].iter().all(|c| c.is_finite()));
        let (a2, b2) = (Vec2::new(a.x, a.y), Vec2::new(b.x, b.y));
        prop_assert_eq!(bits(a2.min(b2).to_array()), [min[0].to_bits(), min[1].to_bits()]);
        prop_assert_eq!(bits(a2.max(b2).to_array()), [max[0].to_bits(), max[1].to_bits()]);
        prop_assert_eq!(bits(a2.abs().to_array()), [aa[0].to_bits() & 0x7FFF_FFFF, aa[1].to_bits() & 0x7FFF_FFFF]);
        prop_assert!(same_bits_all(&a2.lerp(b2, t).to_array(), &[a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t]));
        prop_assert_eq!(a2.is_finite(), a.x.is_finite() && a.y.is_finite());
    }

    /// Verifies: MATH-R026
    #[test]
    fn lerp_at_zero_is_self(a in d_vec4(), b in d_vec4()) {
        prop_assert_eq!(a.lerp(b, 0.0), a);
        prop_assert_eq!(a.truncate().lerp(b.truncate(), 0.0), a.truncate());
    }

    /// Verifies: MATH-R026
    #[test]
    fn vec2_lerp_at_zero_is_self(a in any_vec2()) {
        prop_assume!(a.is_finite());
        let a = Vec2::new(a.x % 100.0, a.y % 100.0);
        prop_assert_eq!(a.lerp(Vec2::ONE, 0.0), a);
    }
}

/// Verifies: MATH-R026
#[test]
fn min_max_with_nan_and_signed_zero() {
    let nan = Vec3::new(f32::NAN, 1.0, -0.0);
    let other = Vec3::new(2.0, f32::NAN, 0.0);
    // A NaN in `self` gives `other`'s component; a NaN in `other` loses only if `self`'s
    // comparison is true, which it is never against NaN.
    assert_eq!(bits(nan.min(other).to_array()), bits([2.0, f32::NAN, 0.0]));
    assert_eq!(bits(nan.max(other).to_array()), bits([2.0, f32::NAN, 0.0]));
    // Lerp: exact expression, not `self × (1 − t) + end × t`.
    let a = Vec3::new(1e8, 0.0, 0.0);
    let b = Vec3::new(-1e8, 1.0, 0.0);
    assert_eq!(a.lerp(b, 0.5), Vec3::new(0.0, 0.5, 0.0));
    assert!(!Vec4::new(0.0, 0.0, 0.0, f32::INFINITY).is_finite());
    assert!(Vec4::new(f32::MAX, f32::MIN, f32::MIN_POSITIVE, -0.0).is_finite());
}
