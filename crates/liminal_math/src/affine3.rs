//! `Affine3` (spec §5.4, MATH-R090 to MATH-R094).

use core::ops::Mul;

use liminal_core::hash::{StateHash, StateHasher};

use crate::{Mat3, MathError, Quat, Vec3, reference};

/// Affine transform `point -> matrix3 * point + translation`. 48 bytes, alignment 4.
///
/// # Examples
///
/// ```
/// use liminal_math::{Affine3, Quat, Vec3};
///
/// let to_world = Affine3::from_scale_rotation_translation(
///     Vec3::splat(2.0),
///     Quat::IDENTITY,
///     Vec3::new(1.0, 0.0, 0.0),
/// );
/// assert_eq!(to_world.transform_point3(Vec3::Y), Vec3::new(1.0, 2.0, 0.0));
/// // Directions ignore the translation.
/// assert_eq!(to_world.transform_vector3(Vec3::Y), Vec3::new(0.0, 2.0, 0.0));
///
/// let to_local = to_world.inverse()?;
/// assert_eq!(to_local.transform_point3(Vec3::new(1.0, 2.0, 0.0)), Vec3::Y);
///
/// let mut points = [Vec3::ZERO, Vec3::X];
/// to_world.transform_points(&mut points);
/// assert_eq!(points, [Vec3::new(1.0, 0.0, 0.0), Vec3::new(3.0, 0.0, 0.0)]);
/// # Ok::<(), liminal_math::MathError>(())
/// ```
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct Affine3 {
    /// The linear part (rotation and scale).
    pub matrix3: Mat3,
    /// Added after the linear part.
    pub translation: Vec3,
}

impl Affine3 {
    /// `Mat3::IDENTITY` and zero translation.
    pub const IDENTITY: Affine3 = Affine3 {
        matrix3: Mat3::IDENTITY,
        translation: Vec3::ZERO,
    };

    /// Scales, then rotates, then translates: `matrix3` has the columns `R.x_axis × s.x`,
    /// `R.y_axis × s.y`, `R.z_axis × s.z` with `R = Mat3::from_quat(rotation)` (MATH-R090).
    #[must_use]
    pub fn from_scale_rotation_translation(
        scale: Vec3,
        rotation: Quat,
        translation: Vec3,
    ) -> Affine3 {
        let r = Mat3::from_quat(rotation);
        Affine3 {
            matrix3: Mat3::from_cols(r.x_axis * scale.x, r.y_axis * scale.y, r.z_axis * scale.z),
            translation,
        }
    }

    /// `((x_axis × p.x + y_axis × p.y) + z_axis × p.z) + translation` (MATH-R091).
    #[must_use]
    pub fn transform_point3(&self, point: Vec3) -> Vec3 {
        let m = &self.matrix3;
        m.x_axis * point.x + m.y_axis * point.y + m.z_axis * point.z + self.translation
    }

    /// `matrix3 * vector`; ignores translation.
    #[must_use]
    pub fn transform_vector3(&self, vector: Vec3) -> Vec3 {
        self.matrix3 * vector
    }

    /// Replaces every element `p` of `points` with `transform_point3(p)`, for slices of every
    /// length (MATH-R094). Computed by [`reference::affine3_transform_points`] (a SIMD path
    /// arrives with task M1-T022).
    pub fn transform_points(&self, points: &mut [Vec3]) {
        reference::affine3_transform_points(self, points);
    }

    /// The inverse transform: `matrix3` inverted, translation `−(m * translation)`.
    ///
    /// # Errors
    ///
    /// [`MathError::SingularMatrix`] (LIM-MATH-0003) if `matrix3` has a zero or non-finite
    /// determinant.
    pub fn inverse(&self) -> Result<Affine3, MathError> {
        let m = self.matrix3.inverse()?;
        Ok(Affine3 {
            matrix3: m,
            translation: -(m * self.translation),
        })
    }
}

/// `self` after `other`: `matrix3: a.matrix3 * b.matrix3`, `translation:
/// a.transform_point3(b.translation)` (MATH-R092).
impl Mul for Affine3 {
    type Output = Affine3;
    fn mul(self, other: Affine3) -> Affine3 {
        Affine3 {
            matrix3: self.matrix3 * other.matrix3,
            translation: self.transform_point3(other.translation),
        }
    }
}

/// `matrix3` then `translation` (MATH-R007).
impl StateHash for Affine3 {
    fn state_hash(&self, hasher: &mut StateHasher) {
        self.matrix3.state_hash(hasher);
        self.translation.state_hash(hasher);
    }
}
