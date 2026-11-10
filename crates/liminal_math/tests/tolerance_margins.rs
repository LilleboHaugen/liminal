//! Measured error margins of the tolerance properties of M1-T020's requirements (spec §12 open
//! question 1: "M1-T020 records the maximum observed error of each property in its journal").
//!
//! Each property runs 100,000 seeded cases from the domains of spec §4, asserts the spec's
//! tolerance and prints the largest observed error as a fraction of it (run with
//! `cargo test -p liminal_math --test tolerance_margins -- --nocapture`). The proptest tests in
//! the other files check the same properties with random inputs; this file adds a fixed,
//! larger sample and the measurement.

#![allow(
    clippy::many_single_char_names,
    clippy::print_stdout,
    reason = "the requirement formulas use one-letter names; the margins are printed for the journal"
)]

use core::num::NonZeroU32;

use liminal_core::rng::{Rng, StreamKey};
use liminal_math::{Affine3, Mat3, Mat4, Quat, Vec3};

const CASES: usize = 100_000;

/// Seeded inputs from the domains of spec §4.
struct Inputs(Rng);

impl Inputs {
    fn new(name: &str) -> Inputs {
        Inputs(Rng::new(20_261_110, StreamKey::from_name(name)))
    }

    fn range(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.0.next_f32()
    }

    fn vec(&mut self, low: f32, high: f32) -> Vec3 {
        Vec3::new(
            self.range(low, high),
            self.range(low, high),
            self.range(low, high),
        )
    }

    fn unit_vec(&mut self) -> Vec3 {
        loop {
            let v = self.vec(-1.0, 1.0);
            if v.length() >= 0.1 {
                return v.normalize().expect("invariant: length at least 0.1");
            }
        }
    }

    fn unit_quat(&mut self) -> Quat {
        loop {
            let q = Quat::from_xyzw(
                self.range(-1.0, 1.0),
                self.range(-1.0, 1.0),
                self.range(-1.0, 1.0),
                self.range(-1.0, 1.0),
            );
            if q.length() >= 0.1 {
                return q.normalize().expect("invariant: length at least 0.1");
            }
        }
    }

    fn trs(&mut self) -> (Vec3, Quat, Vec3) {
        let scale = self.vec(0.5, 2.0);
        let rotation = self.unit_quat();
        (scale, rotation, self.vec(-10.0, 10.0))
    }
}

/// The largest observed error as a fraction of its tolerance.
struct Margin {
    name: &'static str,
    worst: f64,
}

impl Margin {
    fn new(name: &'static str) -> Margin {
        Margin { name, worst: 0.0 }
    }

    /// Records `abs(got − want)` against `tolerance`.
    fn check(&mut self, got: f32, want: f64, tolerance: f64) {
        let fraction = (f64::from(got) - want).abs() / tolerance;
        assert!(
            fraction <= 1.0,
            "{}: {got} vs {want}, tolerance {tolerance}",
            self.name
        );
        self.worst = self.worst.max(fraction);
    }

    fn check_vec(&mut self, got: Vec3, want: Vec3, tolerance: f64) {
        for (g, w) in got.to_array().into_iter().zip(want.to_array()) {
            self.check(g, f64::from(w), tolerance);
        }
    }
}

impl Drop for Margin {
    fn drop(&mut self) {
        println!(
            "{:<48} worst error / tolerance = {:.4}",
            self.name, self.worst
        );
    }
}

/// Spec §4: `1e-5 × max(1, s)`.
fn at_scale(scale: f64) -> f64 {
    1e-5 * scale.max(1.0)
}

/// Verifies: MATH-R023, MATH-R025, MATH-R070
#[test]
fn vector_and_normalize_margins() {
    let mut inputs = Inputs::new("margins.vector");
    let mut cross = Margin::new("R023 cross orthogonal to operands");
    let mut normalize = Margin::new("R025 Vec3 normalize length (4 eps)");
    let mut quat = Margin::new("R070 Quat normalize length (4 eps)");
    for _ in 0..CASES {
        let (a, b) = (inputs.vec(-100.0, 100.0), inputs.vec(-100.0, 100.0));
        let (la, lb) = (f64::from(a.length()), f64::from(b.length()));
        let c = a.cross(b);
        cross.check(c.dot(a), 0.0, at_scale(la * la * lb));
        cross.check(c.dot(b), 0.0, at_scale(la * lb * lb));
        // Components with abs from 2^-33 to 2^34 by exponent.
        // Biased exponent 94 to 160: powers of two from 2^-33 to 2^33.
        let biased = 94
            + inputs
                .0
                .below_u32(NonZeroU32::new(67).expect("invariant: 67 is not 0"));
        let scaled = a * f32::from_bits(biased << 23);
        if let Ok(n) = scaled.normalize() {
            normalize.check(n.length(), 1.0, 4.0 * f64::from(f32::EPSILON));
        }
        quat.check(
            inputs.unit_quat().length(),
            1.0,
            4.0 * f64::from(f32::EPSILON),
        );
    }
}

/// Verifies: MATH-R044, MATH-R045
#[test]
fn determinant_and_inverse_margins() {
    let mut inputs = Inputs::new("margins.matrix");
    let mut det4 = Margin::new("R044 Mat4 determinant of TRS");
    let mut det3 = Margin::new("R044 Mat3 determinant of R * S");
    let mut inv4 = Margin::new("R045 Mat4 m * inverse (1e-4)");
    let mut inv3 = Margin::new("R045 Mat3 m * inverse (1e-4)");
    for _ in 0..CASES {
        let (s, r, t) = inputs.trs();
        let expected = f64::from(s.x) * f64::from(s.y) * f64::from(s.z);
        let m = Mat4::from_scale_rotation_translation(s, r, t);
        det4.check(m.determinant(), expected, at_scale(8.0));
        let m3 = Mat3::from_quat(r) * Mat3::from_scale(s);
        det3.check(m3.determinant(), expected, at_scale(8.0));
        let inverse = m.inverse().expect("TRS matrices are invertible");
        let identity = Mat4::IDENTITY.to_cols_array();
        for product in [m * inverse, inverse * m] {
            for (x, i) in product.to_cols_array().into_iter().zip(identity) {
                inv4.check(x, f64::from(i), 1e-4);
            }
        }
        let inverse3 = m3.inverse().expect("R * S is invertible");
        let identity3 = Mat3::IDENTITY.to_cols_array();
        for product in [m3 * inverse3, inverse3 * m3] {
            for (x, i) in product.to_cols_array().into_iter().zip(identity3) {
                inv3.check(x, f64::from(i), 1e-4);
            }
        }
    }
}

/// Verifies: MATH-R046, MATH-R072
#[test]
fn rotation_margins() {
    let mut inputs = Inputs::new("margins.rotation");
    let mut matrix = Margin::new("R046 Mat3::from_quat(q) * v vs q * v");
    let mut length = Margin::new("R072 length(a * v)");
    let mut compose = Margin::new("R072 (a * b) * v vs a * (b * v)");
    let mut undo = Margin::new("R072 conjugate undoes rotation");
    for _ in 0..CASES {
        let (a, b) = (inputs.unit_quat(), inputs.unit_quat());
        let v = inputs.vec(-100.0, 100.0);
        let tolerance = at_scale(f64::from(v.length()));
        matrix.check_vec(Mat3::from_quat(a) * v, a * v, tolerance);
        length.check((a * v).length(), f64::from(v.length()), tolerance);
        compose.check_vec((a * b) * v, a * (b * v), tolerance);
        undo.check_vec(a.conjugate() * (a * v), v, tolerance);
    }
}

/// Verifies: MATH-R049, MATH-R051
#[test]
fn view_and_projection_margins() {
    let mut inputs = Inputs::new("margins.view");
    let mut eye_margin = Margin::new("R049 look_to maps eye to origin");
    let mut forward = Margin::new("R049 look_to maps direction to -Z");
    let mut up_x = Margin::new("R049 look_to up has x = 0");
    let mut corners = Margin::new("R051 orthographic corners (1e-4)");
    let mut done = 0;
    while done < CASES {
        let eye = inputs.vec(-100.0, 100.0);
        let (direction, up) = (inputs.unit_vec(), inputs.unit_vec());
        if f64::from(direction.dot(up)).abs() > 0.995_004 {
            continue;
        }
        done += 1;
        let m = Mat4::look_to(eye, direction, up).expect("a valid view basis");
        eye_margin.check_vec(
            m.transform_point3(eye),
            Vec3::ZERO,
            at_scale(f64::from(eye.length())),
        );
        forward.check_vec(
            m.transform_vector3(direction),
            Vec3::new(0.0, 0.0, -1.0),
            at_scale(1.0),
        );
        let mapped_up = m.transform_vector3(up);
        up_x.check(mapped_up.x, 0.0, at_scale(1.0));
        assert!(mapped_up.y > 0.0);

        let (left, bottom) = (inputs.range(-10.0, 9.0), inputs.range(-10.0, 9.0));
        let right = inputs.range(left + 1.0, 10.0);
        let top = inputs.range(bottom + 1.0, 10.0);
        let (z_near, z_far) = (inputs.range(0.05, 1.0), inputs.range(10.0, 500.0));
        if right - left < 1.0 || top - bottom < 1.0 {
            continue;
        }
        let p = Mat4::orthographic(left, right, bottom, top, z_near, z_far).expect("valid");
        corners.check_vec(
            p.transform_point3(Vec3::new(left, bottom, -z_near)),
            Vec3::new(-1.0, -1.0, 0.0),
            1e-4,
        );
        corners.check_vec(
            p.transform_point3(Vec3::new(right, top, -z_far)),
            Vec3::ONE,
            1e-4,
        );
    }
}

/// Verifies: MATH-R092, MATH-R093
#[test]
fn affine_margins() {
    let mut inputs = Inputs::new("margins.affine");
    let mut compose = Margin::new("R092 (a * b) p vs a (b p), scale 100");
    let mut inverse = Margin::new("R093 inverse undoes transform");
    for _ in 0..CASES {
        let (s, r, t) = inputs.trs();
        let a = Affine3::from_scale_rotation_translation(s, r, t);
        let (s, r, t) = inputs.trs();
        let b = Affine3::from_scale_rotation_translation(s, r, t);
        let p = inputs.vec(-10.0, 10.0);
        compose.check_vec(
            (a * b).transform_point3(p),
            a.transform_point3(b.transform_point3(p)),
            at_scale(100.0),
        );
        let back = a
            .inverse()
            .expect("TRS is invertible")
            .transform_point3(a.transform_point3(p));
        inverse.check_vec(back, p, 1e-4 * f64::from(p.length()).max(1.0));
    }
}
