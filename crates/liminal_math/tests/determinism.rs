//! Determinism tests of `liminal_math` (ADR-0024, spec §11): workload `linear_algebra`
//! (MATH-R190). Workloads `det_functions` (MATH-R110), `simd_paths` (MATH-R191) and
//! `geometry_queries` (MATH-R192) are added by tasks M1-T021, M1-T022 and M1-T023.
//!
//! Each workload runs a seeded sequence of operations, feeds every result into a
//! `StateHasher` and returns the final hash; the test runs it twice in this process and
//! compares both hashes with each other and with a recorded literal. The spec claims
//! cross-platform bit-identity, so CI checks the same literal on macOS (arm64), Windows
//! (`x86_64`) and Linux (`x86_64`).
//!
//! The literal was recorded from the first run of this test (macOS arm64, 2026-11-10, task
//! M1-T020). It is an oracle (AGENTS.md rule 5): changing the workload's parameters, the hasher
//! or an encoding changes it, and needs a journal entry saying why.

#![allow(
    clippy::float_cmp,
    clippy::many_single_char_names,
    reason = "exact results are the contract (spec §4 \"Exact expression\"); the requirement formulas use one-letter names"
)]

use liminal_core::error::EngineError;
use liminal_core::hash::{StateHash, StateHasher};
use liminal_core::rng::{Rng, StreamKey};
use liminal_math::{Affine3, Mat3, Mat4, MathError, Quat, Vec2, Vec3, Vec4};

/// Runs `workload` twice and checks both hashes against each other and against `recorded`.
fn assert_deterministic(name: &str, workload: fn() -> u64, recorded: u64) {
    let first = workload();
    let second = workload();
    assert_eq!(first, second, "{name}: two runs in one process differ");
    assert_eq!(
        first, recorded,
        "{name}: hash {first:#018X} differs from the recorded {recorded:#018X}"
    );
}

/// Seeded inputs from the domains of spec §4.
struct Inputs {
    rng: Rng,
}

impl Inputs {
    /// A value in `low` to `high`: `low + (high − low) × next_f32()`.
    fn range(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.rng.next_f32()
    }

    /// D-vec component.
    fn component(&mut self) -> f32 {
        self.range(-100.0, 100.0)
    }

    fn vec2(&mut self) -> Vec2 {
        Vec2::new(self.component(), self.component())
    }

    fn vec3(&mut self) -> Vec3 {
        Vec3::new(self.component(), self.component(), self.component())
    }

    fn vec4(&mut self) -> Vec4 {
        Vec4::new(
            self.component(),
            self.component(),
            self.component(),
            self.component(),
        )
    }

    /// D-unit: rejection sampling until the length is at least 0.1.
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

    /// D-trs.
    fn trs(&mut self) -> (Vec3, Quat, Vec3) {
        let scale = Vec3::new(
            self.range(0.5, 2.0),
            self.range(0.5, 2.0),
            self.range(0.5, 2.0),
        );
        let rotation = self.unit_quat();
        let translation = Vec3::new(
            self.range(-10.0, 10.0),
            self.range(-10.0, 10.0),
            self.range(-10.0, 10.0),
        );
        (scale, rotation, translation)
    }
}

/// Hashes `Ok` values, or the error code number of an `Err`.
fn hash_result<T: StateHash>(hasher: &mut StateHasher, result: &Result<T, MathError>) {
    match result {
        Ok(value) => {
            hasher.write_u8(0);
            value.state_hash(hasher);
        }
        Err(error) => {
            hasher.write_u8(1);
            hasher.write_u16(error.code().number());
        }
    }
}

/// One round over the operations of spec §5.1 (vectors).
fn vector_round(inputs: &mut Inputs, hasher: &mut StateHasher) {
    let (a, b, s) = (inputs.vec3(), inputs.vec3(), inputs.range(-2.0, 2.0));
    let mut c = a;
    c += b;
    c -= a * 0.5;
    c *= s;
    c /= 3.0;
    for value in [
        a + b,
        a - b,
        -a,
        a * b,
        a * s,
        s * b,
        a / 7.0,
        c,
        a.cross(b),
    ] {
        value.state_hash(hasher);
    }
    for value in [
        a.min(b),
        a.max(b),
        a.abs(),
        a.lerp(b, s),
        a.normalize_or(Vec3::Y),
    ] {
        value.state_hash(hasher);
    }
    for value in [a.dot(b), a.length_squared(), a.length()] {
        hasher.write_f32(value);
    }
    hasher.write_bool(a.is_finite());
    hash_result(hasher, &a.normalize());
    hash_result(hasher, &(a * 0.0).normalize());
    a.extend(s).state_hash(hasher);

    let (a2, b2) = (inputs.vec2(), inputs.vec2());
    for value in [
        a2 + b2,
        a2 - b2,
        -a2,
        a2 * b2,
        a2 * s,
        a2 / (s + 3.0),
        a2.min(b2),
        a2.max(b2),
        a2.abs(),
        a2.lerp(b2, s),
    ] {
        value.state_hash(hasher);
    }
    hasher.write_f32(a2.dot(b2));
    hasher.write_f32(a2.length());
    hash_result(hasher, &a2.normalize());

    let (a4, b4) = (inputs.vec4(), inputs.vec4());
    for value in [
        a4 + b4,
        a4 - b4,
        -a4,
        a4 * b4,
        a4 * s,
        s * a4,
        a4 / (s + 3.0),
        a4.min(b4),
        a4.max(b4),
        a4.abs(),
        a4.lerp(b4, s),
    ] {
        value.state_hash(hasher);
    }
    hasher.write_f32(a4.dot(b4));
    hasher.write_f32(a4.length());
    hash_result(hasher, &a4.normalize());
    a4.truncate().state_hash(hasher);
}

/// One round over the operations of spec §5.2 to §5.4 (matrices, quaternions, affine).
fn transform_round(inputs: &mut Inputs, hasher: &mut StateHasher) {
    let (s, r, t) = inputs.trs();
    let (s2, r2, t2) = inputs.trs();
    let (p, v) = (inputs.vec3(), inputs.vec3());

    let m = Mat4::from_scale_rotation_translation(s, r, t);
    for matrix in [
        m,
        m.transpose(),
        Mat4::from_translation(t),
        Mat4::from_scale(s),
        Mat4::from_quat(r),
    ] {
        matrix.state_hash(hasher);
    }
    hasher.write_f32(m.determinant());
    hash_result(hasher, &m.inverse());
    for value in [
        m.transform_point3(p),
        m.transform_vector3(v),
        m.project_point3(p),
    ] {
        value.state_hash(hasher);
    }
    hash_result(hasher, &Mat4::look_to(p, v, Vec3::Y));
    let (left, bottom) = (inputs.range(-10.0, -1.0), inputs.range(-10.0, -1.0));
    let (z_near, z_far) = (inputs.range(0.05, 1.0), inputs.range(10.0, 500.0));
    hash_result(
        hasher,
        &Mat4::orthographic(left, -left, bottom, -bottom, z_near, z_far),
    );

    let m3 = Mat3::from_quat(r) * Mat3::from_scale(s);
    for matrix in [
        m3,
        m3.transpose(),
        Mat3::from_scale(s2),
        Mat3::from_quat(r2),
    ] {
        matrix.state_hash(hasher);
    }
    (m3 * p).state_hash(hasher);
    hasher.write_f32(m3.determinant());
    hash_result(hasher, &m3.inverse());

    for value in [r.dot(r2), r.length()] {
        hasher.write_f32(value);
    }
    r.conjugate().state_hash(hasher);
    (r * v).state_hash(hasher);
    hash_result(hasher, &Quat::from_xyzw(p.x, p.y, p.z, v.x).normalize());

    let a = Affine3::from_scale_rotation_translation(s, r, t);
    let b = Affine3::from_scale_rotation_translation(s2, r2, t2);
    (a * b).state_hash(hasher);
    a.transform_point3(p).state_hash(hasher);
    a.transform_vector3(v).state_hash(hasher);
    hash_result(hasher, &a.inverse());
    Mat4::from(b).state_hash(hasher);
}

/// 10,000 rounds over seeded values from D-vec, D-unit and D-trs: every operation of spec §5.1
/// to §5.4 except the four of §5.6 (workload `simd_paths`) and the three that call `det`;
/// every result hashed.
fn linear_algebra() -> u64 {
    let mut inputs = Inputs {
        rng: Rng::new(2026, StreamKey::from_name("determinism.linear_algebra")),
    };
    let mut hasher = StateHasher::new();
    for _ in 0..10_000 {
        vector_round(&mut inputs, &mut hasher);
        transform_round(&mut inputs, &mut hasher);
    }
    hasher.finish()
}

/// Verifies: MATH-R190
#[test]
fn linear_algebra_is_deterministic() {
    assert_deterministic("linear_algebra", linear_algebra, 0xDDA3_807E_8CE9_3C5F);
}
