//! `Mat3` (spec §5.2, MATH-R040 to MATH-R046).

use core::ops::Mul;

use liminal_core::hash::{StateHash, StateHasher};

use crate::{MathError, Quat, Vec3};

/// 3×3 `f32` matrix of column vectors `x_axis`, `y_axis`, `z_axis`, column-major (spec §4).
/// 36 bytes, alignment 4. A matrix transforms a vector as `m * v`.
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct Mat3 {
    /// First column.
    pub x_axis: Vec3,
    /// Second column.
    pub y_axis: Vec3,
    /// Third column.
    pub z_axis: Vec3,
}

impl Mat3 {
    /// 1 on the diagonal, 0 elsewhere.
    pub const IDENTITY: Mat3 = Mat3::from_cols(Vec3::X, Vec3::Y, Vec3::Z);

    /// The matrix with these columns, in order.
    #[must_use]
    pub const fn from_cols(x_axis: Vec3, y_axis: Vec3, z_axis: Vec3) -> Mat3 {
        Mat3 {
            x_axis,
            y_axis,
            z_axis,
        }
    }

    /// Column-major: `elements[3 * c + r]` is row `r`, column `c`.
    #[must_use]
    pub const fn from_cols_array(elements: &[f32; 9]) -> Mat3 {
        let e = elements;
        Mat3::from_cols(
            Vec3::new(e[0], e[1], e[2]),
            Vec3::new(e[3], e[4], e[5]),
            Vec3::new(e[6], e[7], e[8]),
        )
    }

    /// Column-major: index `3 * c + r` is row `r`, column `c`.
    #[must_use]
    pub const fn to_cols_array(&self) -> [f32; 9] {
        let (x, y, z) = (self.x_axis, self.y_axis, self.z_axis);
        [x.x, x.y, x.z, y.x, y.y, y.z, z.x, z.y, z.z]
    }

    /// Diagonal matrix with `scale.x`, `scale.y`, `scale.z` on the diagonal and `+0.0` elsewhere.
    #[must_use]
    pub fn from_scale(scale: Vec3) -> Mat3 {
        Mat3::from_cols(
            Vec3::new(scale.x, 0.0, 0.0),
            Vec3::new(0.0, scale.y, 0.0),
            Vec3::new(0.0, 0.0, scale.z),
        )
    }

    /// Rotation matrix of a unit quaternion. For a non-unit quaternion the result is what the
    /// formula gives (not a rotation).
    #[must_use]
    pub fn from_quat(rotation: Quat) -> Mat3 {
        let Quat { x, y, z, w } = rotation;
        let (x2, y2, z2) = (x + x, y + y, z + z);
        let (xx, xy, xz) = (x * x2, x * y2, x * z2);
        let (yy, yz, zz) = (y * y2, y * z2, z * z2);
        let (wx, wy, wz) = (w * x2, w * y2, w * z2);
        Mat3::from_cols(
            Vec3::new(1.0 - (yy + zz), xy + wz, xz - wy),
            Vec3::new(xy - wz, 1.0 - (xx + zz), yz + wx),
            Vec3::new(xz + wy, yz - wx, 1.0 - (xx + yy)),
        )
    }

    /// The matrix with rows and columns swapped (bits unchanged).
    #[must_use]
    pub fn transpose(&self) -> Mat3 {
        let (x, y, z) = (self.x_axis, self.y_axis, self.z_axis);
        Mat3::from_cols(
            Vec3::new(x.x, y.x, z.x),
            Vec3::new(x.y, y.y, z.y),
            Vec3::new(x.z, y.z, z.z),
        )
    }

    /// `x_axis · (y_axis × z_axis)`. Exact for integer elements with `abs` at most 8.
    #[must_use]
    pub fn determinant(&self) -> f32 {
        self.x_axis.dot(self.y_axis.cross(self.z_axis))
    }

    /// The adjugate divided by the determinant.
    ///
    /// # Errors
    ///
    /// [`MathError::SingularMatrix`] (LIM-MATH-0003) if the determinant is zero, NaN or
    /// infinite.
    pub fn inverse(&self) -> Result<Mat3, MathError> {
        let determinant = self.determinant();
        if determinant == 0.0 || !determinant.is_finite() {
            return Err(MathError::SingularMatrix { determinant });
        }
        // The rows of the adjugate are the cross products of pairs of columns.
        let rows = Mat3::from_cols(
            self.y_axis.cross(self.z_axis),
            self.z_axis.cross(self.x_axis),
            self.x_axis.cross(self.y_axis),
        );
        let adjugate = rows.transpose();
        Ok(Mat3::from_cols(
            adjugate.x_axis / determinant,
            adjugate.y_axis / determinant,
            adjugate.z_axis / determinant,
        ))
    }
}

/// `(x_axis × v.x + y_axis × v.y) + z_axis × v.z` (MATH-R041).
impl Mul<Vec3> for Mat3 {
    type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 {
        self.x_axis * v.x + self.y_axis * v.y + self.z_axis * v.z
    }
}

/// The matrix whose columns are `self * other.x_axis`, `self * other.y_axis`,
/// `self * other.z_axis` (MATH-R042).
impl Mul for Mat3 {
    type Output = Mat3;
    fn mul(self, other: Mat3) -> Mat3 {
        Mat3::from_cols(
            self * other.x_axis,
            self * other.y_axis,
            self * other.z_axis,
        )
    }
}

/// The columns in order (MATH-R007).
impl StateHash for Mat3 {
    fn state_hash(&self, hasher: &mut StateHasher) {
        self.x_axis.state_hash(hasher);
        self.y_axis.state_hash(hasher);
        self.z_axis.state_hash(hasher);
    }
}
