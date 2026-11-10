//! `Quat` (spec §5.3, MATH-R070 to MATH-R072).

use core::ops::Mul;

use liminal_core::hash::{StateHash, StateHasher};

use crate::{MathError, Vec3, Vec4, reference};

/// Quaternion `x i + y j + z k + w`; a rotation when its length is 1 (unit quaternion).
/// 16 bytes, alignment 16.
///
/// Operations on a non-unit quaternion return what their formulas give; nothing panics. In
/// `a * b` the right-hand rotation applies first. A positive angle rotates counter-clockwise
/// when the axis points at the viewer (spec §4).
///
/// # Examples
///
/// ```
/// use core::f32::consts::FRAC_1_SQRT_2;
/// use liminal_math::{Quat, Vec3};
///
/// // A rotation by +π/2 about +Z takes +X to +Y.
/// let quarter_turn = Quat::from_xyzw(0.0, 0.0, FRAC_1_SQRT_2, FRAC_1_SQRT_2);
/// let rotated = quarter_turn * Vec3::X;
/// assert!((rotated - Vec3::Y).length() < 1e-6);
///
/// // The conjugate of a unit quaternion is the inverse rotation.
/// let back = quarter_turn.conjugate() * rotated;
/// assert!((back - Vec3::X).length() < 1e-6);
/// ```
///
/// `Quat::from_axis_angle` arrives with `det` (task M1-T021).
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C, align(16))]
pub struct Quat {
    /// Coefficient of `i`.
    pub x: f32,
    /// Coefficient of `j`.
    pub y: f32,
    /// Coefficient of `k`.
    pub z: f32,
    /// Real part.
    pub w: f32,
}

impl Quat {
    /// `(0, 0, 0, 1)`: no rotation.
    pub const IDENTITY: Quat = Quat::from_xyzw(0.0, 0.0, 0.0, 1.0);

    /// The quaternion `x i + y j + z k + w`, stored unchanged (no normalization).
    #[must_use]
    pub const fn from_xyzw(x: f32, y: f32, z: f32, w: f32) -> Quat {
        Quat { x, y, z, w }
    }

    /// The four components as a `Vec4`, for the operations both types share.
    const fn to_vec4(self) -> Vec4 {
        Vec4::new(self.x, self.y, self.z, self.w)
    }

    const fn from_vec4(v: Vec4) -> Quat {
        Quat::from_xyzw(v.x, v.y, v.z, v.w)
    }

    /// `((x × x' + y × y') + z × z') + w × w'`.
    #[must_use]
    pub fn dot(self, other: Quat) -> f32 {
        self.to_vec4().dot(other.to_vec4())
    }

    /// `sqrt(self.dot(self))`.
    #[must_use]
    pub fn length(self) -> f32 {
        self.to_vec4().length()
    }

    /// `self / length()`, component-wise.
    ///
    /// # Errors
    ///
    /// [`MathError::NonFiniteInput`] (LIM-MATH-0002) if the length is NaN or infinite,
    /// [`MathError::ZeroLengthVector`] (LIM-MATH-0001) if it is zero; both with
    /// `parameter: "self"`.
    pub fn normalize(self) -> Result<Quat, MathError> {
        self.to_vec4().normalize().map(Quat::from_vec4)
    }

    /// `(-x, -y, -z, w)`: the inverse rotation of a unit quaternion.
    #[must_use]
    pub fn conjugate(self) -> Quat {
        Quat::from_xyzw(-self.x, -self.y, -self.z, self.w)
    }
}

/// Hamilton product (MATH-R071), computed by [`reference::quat_mul`] (a SIMD path arrives with
/// task M1-T022).
impl Mul for Quat {
    type Output = Quat;
    fn mul(self, other: Quat) -> Quat {
        reference::quat_mul(self, other)
    }
}

/// Rotates `v`: `(v + t × w) + u.cross(t)` with `u = (x, y, z)` and `t = u.cross(v) × 2`
/// (MATH-R072). Meaningful for unit quaternions.
impl Mul<Vec3> for Quat {
    type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 {
        let u = Vec3::new(self.x, self.y, self.z);
        let t = u.cross(v) * 2.0;
        (v + t * self.w) + u.cross(t)
    }
}

/// `x, y, z, w` with `write_f32` (MATH-R007).
impl StateHash for Quat {
    fn state_hash(&self, hasher: &mut StateHasher) {
        self.to_vec4().state_hash(hasher);
    }
}
