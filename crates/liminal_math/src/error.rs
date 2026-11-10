//! [`MathError`]: the errors of `liminal_math` (spec §5.8, §8).

use liminal_core::error::{EngineError, ErrorCode, ErrorContext, write_display};

/// The errors of `liminal_math` (codes `LIM-MATH-NNNN`, spec §8).
///
/// Match on the variant or compare [`EngineError::code`]; message wording is not a stable
/// interface for programs. `parameter` is the name of the function parameter as written in the
/// spec (`"axis"`, `"direction"`), or `"self"` for a method receiver.
///
/// Some variants have no producer yet: `InvalidProjection` from `Mat4::perspective` and
/// `NonFiniteInput` from `Quat::from_axis_angle` arrive with task M1-T021; `InvalidAabb`,
/// `NegativeExtent`, `EmptyPointSet`, `CollinearPoints`, `DegenerateFrustum` and
/// `VectorTooShort` with the geometry types of task M1-T023.
///
/// # Examples
///
/// ```
/// use liminal_core::error::EngineError;
/// use liminal_math::{MathError, Vec3};
///
/// let error = Vec3::ZERO.normalize().unwrap_err();
/// match error {
///     MathError::ZeroLengthVector { parameter } => assert_eq!(parameter, "self"),
///     other => panic!("unexpected error {other}"),
/// }
/// assert_eq!(error.code().to_string(), "LIM-MATH-0001");
/// assert!(error.to_string().starts_with("LIM-MATH-0001: `self` has length zero"));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum MathError {
    /// LIM-MATH-0001: a vector to normalize has length zero.
    ZeroLengthVector {
        /// The parameter holding the vector.
        parameter: &'static str,
    },
    /// LIM-MATH-0002: an input contains NaN or an infinity, or a result computed from it
    /// overflows.
    NonFiniteInput {
        /// The parameter holding the input.
        parameter: &'static str,
    },
    /// LIM-MATH-0003: a matrix has a zero or non-finite determinant.
    SingularMatrix {
        /// The determinant.
        determinant: f32,
    },
    /// LIM-MATH-0004: a projection parameter is out of range.
    InvalidProjection {
        /// The parameter that failed its check.
        parameter: &'static str,
        /// Its value.
        value: f32,
    },
    /// LIM-MATH-0005: `up` is zero or parallel to `direction` in `Mat4::look_to`.
    InvalidViewBasis,
    /// LIM-MATH-0010: a box minimum is greater than its maximum on an axis.
    InvalidAabb {
        /// `"x"`, `"y"` or `"z"`.
        axis: &'static str,
        /// The minimum on that axis.
        min: f32,
        /// The maximum on that axis.
        max: f32,
    },
    /// LIM-MATH-0011: a radius or half extent is negative.
    NegativeExtent {
        /// The parameter holding the value.
        parameter: &'static str,
        /// The negative value.
        value: f32,
    },
    /// LIM-MATH-0012: a bounding box was requested for zero points.
    EmptyPointSet,
    /// LIM-MATH-0013: three points define no plane.
    CollinearPoints,
    /// LIM-MATH-0014: a plane of a view-projection matrix is degenerate.
    DegenerateFrustum {
        /// `"left"`, `"right"`, `"bottom"`, `"top"`, `"near"` or `"far"`.
        plane: &'static str,
    },
    /// LIM-MATH-0015: a vector's squared length is below `f32::MIN_POSITIVE`.
    VectorTooShort {
        /// The parameter holding the vector.
        parameter: &'static str,
        /// Its squared length.
        length_squared: f32,
    },
}

impl EngineError for MathError {
    fn code(&self) -> ErrorCode {
        const fn math(number: u16) -> ErrorCode {
            ErrorCode::new("MATH", number)
        }
        match self {
            MathError::ZeroLengthVector { .. } => const { math(1) },
            MathError::NonFiniteInput { .. } => const { math(2) },
            MathError::SingularMatrix { .. } => const { math(3) },
            MathError::InvalidProjection { .. } => const { math(4) },
            MathError::InvalidViewBasis => const { math(5) },
            MathError::InvalidAabb { .. } => const { math(10) },
            MathError::NegativeExtent { .. } => const { math(11) },
            MathError::EmptyPointSet => const { math(12) },
            MathError::CollinearPoints => const { math(13) },
            MathError::DegenerateFrustum { .. } => const { math(14) },
            MathError::VectorTooShort { .. } => const { math(15) },
        }
    }

    fn message(&self) -> String {
        match self {
            MathError::ZeroLengthVector { parameter } => {
                format!("`{parameter}` has length zero and cannot be normalized.")
            }
            MathError::NonFiniteInput { parameter } => format!(
                "`{parameter}` is not finite: it contains NaN or an infinity, or the result \
                 computed from it overflows."
            ),
            MathError::SingularMatrix { determinant } => {
                format!("the matrix has determinant `{determinant}` and cannot be inverted.")
            }
            MathError::InvalidProjection { parameter, value } => {
                format!("projection parameter `{parameter}` has the invalid value `{value}`.")
            }
            MathError::InvalidViewBasis => {
                "`up` is zero or parallel to `direction`, so no view basis exists.".to_string()
            }
            MathError::InvalidAabb { axis, min, max } => {
                format!("box minimum `{min}` is greater than maximum `{max}` on axis `{axis}`.")
            }
            MathError::NegativeExtent { parameter, value } => {
                format!("`{parameter}` has the negative value `{value}`.")
            }
            MathError::EmptyPointSet => "cannot build a bounding box from zero points.".to_string(),
            MathError::CollinearPoints => "the three points lie on one line, coincide or span \
                                           too small a triangle, so they define no plane."
                .to_string(),
            MathError::DegenerateFrustum { plane } => format!(
                "the `{plane}` plane of the view-projection matrix has a normal that is zero, \
                 shorter than 1.1e-19 or not finite, or a distance that is not finite."
            ),
            MathError::VectorTooShort {
                parameter,
                length_squared,
            } => format!(
                "`{parameter}` has the squared length `{length_squared}`, which is below \
                 1.1754944e-38, and cannot be normalized to unit length."
            ),
        }
    }

    fn help(&self) -> Option<String> {
        let help = match self {
            MathError::ZeroLengthVector { .. } => {
                "pass a non-zero vector; components with absolute value below 1e-19 can \
                 underflow to a zero length. Use `normalize_or` to supply a fallback direction."
            }
            MathError::NonFiniteInput { .. } => {
                "check the value for NaN and infinities before the call; components with \
                 absolute value above 1.8e19 overflow when squared."
            }
            MathError::SingularMatrix { .. } => {
                "a matrix with a zero scale component or with linearly dependent columns has \
                 no inverse; check the scale passed to `from_scale_rotation_translation`."
            }
            MathError::InvalidProjection { .. } => {
                "`perspective` needs 0 < fov_y_radians < π, aspect_ratio > 0 and \
                 0 < z_near < z_far; `orthographic` needs left != right, bottom != top and \
                 z_near != z_far; all values finite."
            }
            MathError::InvalidViewBasis => {
                "pass an `up` vector that is not parallel to `direction`, for example \
                 `Vec3::Z` when looking straight up or down."
            }
            MathError::InvalidAabb { .. } => {
                "pass `min <= max` on every axis, or build the box with `Aabb::from_points`."
            }
            MathError::NegativeExtent { .. } => "pass a radius or half extent of zero or more.",
            MathError::EmptyPointSet => "pass at least one point.",
            MathError::CollinearPoints => {
                "pass three points that form a triangle with non-zero area; the cross product \
                 of two of its edges needs a length of at least 1.1e-19 (two perpendicular \
                 edges of length 1e-9 are enough)."
            }
            MathError::DegenerateFrustum { .. } => {
                "pass the product `projection * view` of matrices from `Mat4::perspective` or \
                 `Mat4::orthographic` and `Mat4::look_to`."
            }
            MathError::VectorTooShort { .. } => {
                "pass a longer vector; a length of at least 1.1e-19 is enough. Below that the \
                 squared length is a subnormal `f32`, too coarse to give a unit vector."
            }
        };
        Some(help.to_string())
    }

    fn context(&self) -> ErrorContext {
        let context = ErrorContext::new();
        match *self {
            MathError::ZeroLengthVector { parameter } | MathError::NonFiniteInput { parameter } => {
                context.with("parameter", parameter)
            }
            MathError::SingularMatrix { determinant } => context.with("determinant", determinant),
            MathError::InvalidProjection { parameter, value }
            | MathError::NegativeExtent { parameter, value } => {
                context.with("parameter", parameter).with("value", value)
            }
            MathError::InvalidAabb { axis, min, max } => {
                context.with("axis", axis).with("min", min).with("max", max)
            }
            MathError::DegenerateFrustum { plane } => context.with("plane", plane),
            MathError::VectorTooShort {
                parameter,
                length_squared,
            } => context
                .with("parameter", parameter)
                .with("length_squared", length_squared),
            MathError::InvalidViewBasis | MathError::EmptyPointSet | MathError::CollinearPoints => {
                context
            }
        }
    }
}

impl core::fmt::Display for MathError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write_display(self, f)
    }
}

impl std::error::Error for MathError {}

impl MathError {
    /// The same error with `parameter` renamed, for errors of a nested call reported under the
    /// caller's parameter name (spec: "with its error returned with `parameter: …`").
    pub(crate) fn with_parameter(self, name: &'static str) -> MathError {
        match self {
            MathError::ZeroLengthVector { .. } => MathError::ZeroLengthVector { parameter: name },
            MathError::NonFiniteInput { .. } => MathError::NonFiniteInput { parameter: name },
            MathError::InvalidProjection { value, .. } => MathError::InvalidProjection {
                parameter: name,
                value,
            },
            MathError::NegativeExtent { value, .. } => MathError::NegativeExtent {
                parameter: name,
                value,
            },
            MathError::VectorTooShort { length_squared, .. } => MathError::VectorTooShort {
                parameter: name,
                length_squared,
            },
            other => other,
        }
    }
}
