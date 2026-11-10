//! `Mat4` (spec §5.2, MATH-R040 to MATH-R049, MATH-R051).

use core::ops::Mul;

use liminal_core::hash::{StateHash, StateHasher};

use crate::{Affine3, Mat3, MathError, Quat, Vec3, Vec4, reference};

/// 4×4 `f32` matrix of column vectors `x_axis` to `w_axis`, column-major (spec §4). 64 bytes,
/// alignment 16. A matrix transforms a vector as `m * v`; in `a * b` the right-hand factor
/// applies first.
///
/// # Examples
///
/// A model-view-projection product applied to a point:
///
/// ```
/// use liminal_math::{Mat4, Quat, Vec3};
///
/// let model = Mat4::from_scale_rotation_translation(
///     Vec3::splat(2.0),
///     Quat::IDENTITY,
///     Vec3::new(0.0, 0.0, -5.0),
/// );
/// // Camera at the origin looking along −Z, +Y up.
/// let view = Mat4::look_to(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0), Vec3::Y)?;
/// let projection = Mat4::orthographic(-10.0, 10.0, -10.0, 10.0, 0.0, 10.0)?;
/// let model_view_projection = projection * view * model;
///
/// // The model-space origin lands at the world point (0, 0, −5): the centre of the view,
/// // with depth 5 / 10 = 0.5.
/// let clip = model_view_projection.project_point3(Vec3::ZERO);
/// assert_eq!(clip, Vec3::new(0.0, 0.0, 0.5));
/// # Ok::<(), liminal_math::MathError>(())
/// ```
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct Mat4 {
    /// First column.
    pub x_axis: Vec4,
    /// Second column.
    pub y_axis: Vec4,
    /// Third column.
    pub z_axis: Vec4,
    /// Fourth column (the translation of an affine matrix).
    pub w_axis: Vec4,
}

impl Mat4 {
    /// 1 on the diagonal, 0 elsewhere.
    pub const IDENTITY: Mat4 = Mat4::from_cols(Vec4::X, Vec4::Y, Vec4::Z, Vec4::W);

    /// The matrix with these columns, in order.
    #[must_use]
    pub const fn from_cols(x_axis: Vec4, y_axis: Vec4, z_axis: Vec4, w_axis: Vec4) -> Mat4 {
        Mat4 {
            x_axis,
            y_axis,
            z_axis,
            w_axis,
        }
    }

    /// Column-major: `elements[4 * c + r]` is row `r`, column `c`.
    #[must_use]
    pub const fn from_cols_array(elements: &[f32; 16]) -> Mat4 {
        let e = elements;
        Mat4::from_cols(
            Vec4::new(e[0], e[1], e[2], e[3]),
            Vec4::new(e[4], e[5], e[6], e[7]),
            Vec4::new(e[8], e[9], e[10], e[11]),
            Vec4::new(e[12], e[13], e[14], e[15]),
        )
    }

    /// Column-major: index `4 * c + r` is row `r`, column `c`.
    #[must_use]
    pub const fn to_cols_array(&self) -> [f32; 16] {
        let (x, y, z, w) = (self.x_axis, self.y_axis, self.z_axis, self.w_axis);
        [
            x.x, x.y, x.z, x.w, y.x, y.y, y.z, y.w, z.x, z.y, z.z, z.w, w.x, w.y, w.z, w.w,
        ]
    }

    /// The identity with `w_axis = (translation, 1)`.
    #[must_use]
    pub fn from_translation(translation: Vec3) -> Mat4 {
        Mat4::from_cols(Vec4::X, Vec4::Y, Vec4::Z, translation.extend(1.0))
    }

    /// Diagonal matrix with `scale.x`, `scale.y`, `scale.z`, 1 on the diagonal and `+0.0`
    /// elsewhere.
    #[must_use]
    pub fn from_scale(scale: Vec3) -> Mat4 {
        Mat4::from_mat3(Mat3::from_scale(scale), Vec3::ZERO)
    }

    /// Rotation matrix of a unit quaternion: [`Mat3::from_quat`] in the upper-left block.
    #[must_use]
    pub fn from_quat(rotation: Quat) -> Mat4 {
        Mat4::from_mat3(Mat3::from_quat(rotation), Vec3::ZERO)
    }

    /// Scales, then rotates, then translates: `T * R * S`, computed as the columns
    /// `R.x_axis × s.x`, `R.y_axis × s.y`, `R.z_axis × s.z` and `(t, 1)` (MATH-R047).
    #[must_use]
    pub fn from_scale_rotation_translation(scale: Vec3, rotation: Quat, translation: Vec3) -> Mat4 {
        Mat4::from(Affine3::from_scale_rotation_translation(
            scale,
            rotation,
            translation,
        ))
    }

    /// The matrix with `linear` in the upper-left block, `(translation, 1)` as `w_axis` and 0 in
    /// the fourth row of the other columns.
    pub(crate) fn from_mat3(linear: Mat3, translation: Vec3) -> Mat4 {
        Mat4::from_cols(
            linear.x_axis.extend(0.0),
            linear.y_axis.extend(0.0),
            linear.z_axis.extend(0.0),
            translation.extend(1.0),
        )
    }

    /// View matrix: maps `eye` to the origin, `direction` to −Z and `up` into the +Y half of the
    /// YZ plane (MATH-R049).
    ///
    /// # Errors
    ///
    /// - [`MathError::NonFiniteInput`] (LIM-MATH-0002) naming the first of `eye`, `direction`,
    ///   `up` with a NaN or infinite component;
    /// - the error of `direction.normalize()` with `parameter: "direction"` (LIM-MATH-0002,
    ///   LIM-MATH-0001);
    /// - [`MathError::InvalidViewBasis`] (LIM-MATH-0005) if `up` is zero or parallel to
    ///   `direction`.
    pub fn look_to(eye: Vec3, direction: Vec3, up: Vec3) -> Result<Mat4, MathError> {
        for (parameter, value) in [("eye", eye), ("direction", direction), ("up", up)] {
            if !value.is_finite() {
                return Err(MathError::NonFiniteInput { parameter });
            }
        }
        let f = direction
            .normalize()
            .map_err(|error| error.with_parameter("direction"))?;
        let s = f
            .cross(up)
            .normalize()
            .map_err(|_| MathError::InvalidViewBasis)?;
        let u = s.cross(f);
        Ok(Mat4::from_cols(
            Vec4::new(s.x, u.x, -f.x, 0.0),
            Vec4::new(s.y, u.y, -f.y, 0.0),
            Vec4::new(s.z, u.z, -f.z, 0.0),
            Vec4::new(-s.dot(eye), -u.dot(eye), f.dot(eye), 1.0),
        ))
    }

    /// Orthographic projection: `left`/`right` map to x −1/1, `bottom`/`top` to y −1/1 and the
    /// planes at distance `z_near`/`z_far` in front of the camera (z = −`z_near`, −`z_far`) to
    /// depth 0/1 (MATH-R051).
    ///
    /// # Errors
    ///
    /// [`MathError::InvalidProjection`] (LIM-MATH-0004) with the parameter's name and value, for
    /// the first of: a NaN or infinite parameter (first in signature order); `right` if
    /// `right == left`; `top` if `top == bottom`; `z_far` if `z_far == z_near`.
    #[expect(
        clippy::float_cmp,
        reason = "MATH-R051 rejects exactly equal bounds; nearly equal ones are valid input"
    )]
    pub fn orthographic(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        z_near: f32,
        z_far: f32,
    ) -> Result<Mat4, MathError> {
        let parameters = [
            ("left", left),
            ("right", right),
            ("bottom", bottom),
            ("top", top),
            ("z_near", z_near),
            ("z_far", z_far),
        ];
        let invalid = |parameter, value| Err(MathError::InvalidProjection { parameter, value });
        for (parameter, value) in parameters {
            if !value.is_finite() {
                return invalid(parameter, value);
            }
        }
        if right == left {
            return invalid("right", right);
        }
        if top == bottom {
            return invalid("top", top);
        }
        if z_far == z_near {
            return invalid("z_far", z_far);
        }
        let width = right - left;
        let height = top - bottom;
        let depth = z_near - z_far;
        Ok(Mat4::from_cols(
            Vec4::new(2.0 / width, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 2.0 / height, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0 / depth, 0.0),
            Vec4::new(
                -(right + left) / width,
                -(top + bottom) / height,
                z_near / depth,
                1.0,
            ),
        ))
    }

    /// The matrix with rows and columns swapped (bits unchanged).
    #[must_use]
    pub fn transpose(&self) -> Mat4 {
        let (x, y, z, w) = (self.x_axis, self.y_axis, self.z_axis, self.w_axis);
        Mat4::from_cols(
            Vec4::new(x.x, y.x, z.x, w.x),
            Vec4::new(x.y, y.y, z.y, w.y),
            Vec4::new(x.z, y.z, z.z, w.z),
            Vec4::new(x.w, y.w, z.w, w.w),
        )
    }

    /// The determinant, by Laplace expansion over 2×2 minors of the first two and last two rows.
    /// Exact for integer elements with `abs` at most 8.
    #[must_use]
    pub fn determinant(&self) -> f32 {
        let minors = Minors::of(self);
        minors.determinant()
    }

    /// The adjugate divided by the determinant.
    ///
    /// # Errors
    ///
    /// [`MathError::SingularMatrix`] (LIM-MATH-0003) if the determinant is zero, NaN or
    /// infinite.
    pub fn inverse(&self) -> Result<Mat4, MathError> {
        let minors = Minors::of(self);
        let determinant = minors.determinant();
        if determinant == 0.0 || !determinant.is_finite() {
            return Err(MathError::SingularMatrix { determinant });
        }
        let adjugate = minors.adjugate(self);
        Ok(Mat4::from_cols(
            adjugate.x_axis / determinant,
            adjugate.y_axis / determinant,
            adjugate.z_axis / determinant,
            adjugate.w_axis / determinant,
        ))
    }

    /// `(self * point.extend(1.0)).truncate()`; no perspective divide.
    #[must_use]
    pub fn transform_point3(&self, point: Vec3) -> Vec3 {
        (*self * point.extend(1.0)).truncate()
    }

    /// `(self * vector.extend(0.0)).truncate()`; ignores translation.
    #[must_use]
    pub fn transform_vector3(&self, vector: Vec3) -> Vec3 {
        (*self * vector.extend(0.0)).truncate()
    }

    /// `c.truncate() / c.w` with `c = self * point.extend(1.0)` (perspective divide). `c.w` is
    /// not checked: zero gives infinite or NaN components.
    #[must_use]
    pub fn project_point3(&self, point: Vec3) -> Vec3 {
        let clip = *self * point.extend(1.0);
        clip.truncate() / clip.w
    }
}

/// The twelve 2×2 minors of a `Mat4` that its determinant and adjugate are built from. `a(r, c)`
/// is row `r`, column `c`; `s*` use rows 0 and 1, `c*` rows 2 and 3.
struct Minors {
    s: [f32; 6],
    c: [f32; 6],
}

impl Minors {
    fn of(m: &Mat4) -> Minors {
        let elements = m.to_cols_array();
        let a = |r: usize, c: usize| elements[4 * c + r];
        Minors {
            s: [
                a(0, 0) * a(1, 1) - a(1, 0) * a(0, 1),
                a(0, 0) * a(1, 2) - a(1, 0) * a(0, 2),
                a(0, 0) * a(1, 3) - a(1, 0) * a(0, 3),
                a(0, 1) * a(1, 2) - a(1, 1) * a(0, 2),
                a(0, 1) * a(1, 3) - a(1, 1) * a(0, 3),
                a(0, 2) * a(1, 3) - a(1, 2) * a(0, 3),
            ],
            c: [
                a(2, 0) * a(3, 1) - a(3, 0) * a(2, 1),
                a(2, 0) * a(3, 2) - a(3, 0) * a(2, 2),
                a(2, 0) * a(3, 3) - a(3, 0) * a(2, 3),
                a(2, 1) * a(3, 2) - a(3, 1) * a(2, 2),
                a(2, 1) * a(3, 3) - a(3, 1) * a(2, 3),
                a(2, 2) * a(3, 3) - a(3, 2) * a(2, 3),
            ],
        }
    }

    fn determinant(&self) -> f32 {
        let (s, c) = (&self.s, &self.c);
        s[0] * c[5] - s[1] * c[4] + s[2] * c[3] + s[3] * c[2] - s[4] * c[1] + s[5] * c[0]
    }

    /// The adjugate (transposed cofactor matrix) of `m`.
    fn adjugate(&self, m: &Mat4) -> Mat4 {
        let (s, c) = (&self.s, &self.c);
        let elements = m.to_cols_array();
        let a = |r: usize, c: usize| elements[4 * c + r];
        // b(r, col) of the adjugate, written row by row.
        let rows = [
            [
                a(1, 1) * c[5] - a(1, 2) * c[4] + a(1, 3) * c[3],
                -a(0, 1) * c[5] + a(0, 2) * c[4] - a(0, 3) * c[3],
                a(3, 1) * s[5] - a(3, 2) * s[4] + a(3, 3) * s[3],
                -a(2, 1) * s[5] + a(2, 2) * s[4] - a(2, 3) * s[3],
            ],
            [
                -a(1, 0) * c[5] + a(1, 2) * c[2] - a(1, 3) * c[1],
                a(0, 0) * c[5] - a(0, 2) * c[2] + a(0, 3) * c[1],
                -a(3, 0) * s[5] + a(3, 2) * s[2] - a(3, 3) * s[1],
                a(2, 0) * s[5] - a(2, 2) * s[2] + a(2, 3) * s[1],
            ],
            [
                a(1, 0) * c[4] - a(1, 1) * c[2] + a(1, 3) * c[0],
                -a(0, 0) * c[4] + a(0, 1) * c[2] - a(0, 3) * c[0],
                a(3, 0) * s[4] - a(3, 1) * s[2] + a(3, 3) * s[0],
                -a(2, 0) * s[4] + a(2, 1) * s[2] - a(2, 3) * s[0],
            ],
            [
                -a(1, 0) * c[3] + a(1, 1) * c[1] - a(1, 2) * c[0],
                a(0, 0) * c[3] - a(0, 1) * c[1] + a(0, 2) * c[0],
                -a(3, 0) * s[3] + a(3, 1) * s[1] - a(3, 2) * s[0],
                a(2, 0) * s[3] - a(2, 1) * s[1] + a(2, 2) * s[0],
            ],
        ];
        Mat4::from_cols(
            Vec4::from_array(rows[0]),
            Vec4::from_array(rows[1]),
            Vec4::from_array(rows[2]),
            Vec4::from_array(rows[3]),
        )
        .transpose()
    }
}

/// `((x_axis × v.x + y_axis × v.y) + z_axis × v.z) + w_axis × v.w` (MATH-R041), computed by
/// [`reference::mat4_mul_vec4`] (a SIMD path arrives with task M1-T022).
impl Mul<Vec4> for Mat4 {
    type Output = Vec4;
    fn mul(self, vector: Vec4) -> Vec4 {
        reference::mat4_mul_vec4(self, vector)
    }
}

/// The matrix whose columns are `self * other.x_axis` to `self * other.w_axis` (MATH-R042),
/// computed by [`reference::mat4_mul`] (a SIMD path arrives with task M1-T022).
impl Mul for Mat4 {
    type Output = Mat4;
    fn mul(self, other: Mat4) -> Mat4 {
        reference::mat4_mul(self, other)
    }
}

/// Columns `(matrix3.x_axis, 0)`, `(matrix3.y_axis, 0)`, `(matrix3.z_axis, 0)`,
/// `(translation, 1)` (MATH-R090).
impl From<Affine3> for Mat4 {
    fn from(affine: Affine3) -> Mat4 {
        Mat4::from_mat3(affine.matrix3, affine.translation)
    }
}

/// The columns in order (MATH-R007).
impl StateHash for Mat4 {
    fn state_hash(&self, hasher: &mut StateHasher) {
        self.x_axis.state_hash(hasher);
        self.y_axis.state_hash(hasher);
        self.z_axis.state_hash(hasher);
        self.w_axis.state_hash(hasher);
    }
}
