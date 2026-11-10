//! `Vec2`, `Vec3` and `Vec4` (spec §5.1, MATH-R020 to MATH-R026).
//!
//! The three types share their methods and operators through the private macro
//! `vector_common!`; each component-wise operation is one IEEE 754 operation per component
//! (MATH-R021), and sums are evaluated left to right without fused multiply-add (spec §4 "Exact
//! expression").

use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use liminal_core::hash::{StateHash, StateHasher};

use crate::MathError;

/// Implements the methods, operators and `StateHash` that `Vec2`, `Vec3` and `Vec4` share.
///
/// `$first` and `$rest` are the field names in order; `$len` is the component count.
macro_rules! vector_common {
    ($Vec:ident, $len:literal, $first:ident $(, $rest:ident)*) => {
        impl $Vec {
            /// Every component 0.
            pub const ZERO: $Vec = $Vec::splat(0.0);
            /// Every component 1.
            pub const ONE: $Vec = $Vec::splat(1.0);

            /// Every component set to `value`.
            #[must_use]
            pub const fn splat(value: f32) -> $Vec {
                $Vec { $first: value $(, $rest: value)* }
            }

            /// The vector with `components[i]` as component `i` (`x` at index 0).
            #[must_use]
            pub const fn from_array(components: [f32; $len]) -> $Vec {
                let [$first $(, $rest)*] = components;
                $Vec { $first $(, $rest)* }
            }

            /// The components in order, `x` at index 0.
            #[must_use]
            pub const fn to_array(self) -> [f32; $len] {
                [self.$first $(, self.$rest)*]
            }

            /// Dot product, summed left to right: `(x × x' + y × y') + …` (MATH-R022).
            #[must_use]
            pub fn dot(self, other: $Vec) -> f32 {
                self.$first * other.$first $(+ self.$rest * other.$rest)*
            }

            /// `self.dot(self)`. Not rescaled: a component with `abs` above 1.8e19 can overflow
            /// to infinity, one below 1e-19 can underflow to zero.
            #[must_use]
            pub fn length_squared(self) -> f32 {
                self.dot(self)
            }

            /// `sqrt(length_squared())`.
            #[must_use]
            pub fn length(self) -> f32 {
                self.length_squared().sqrt()
            }

            /// `self / length()`.
            ///
            /// # Errors
            ///
            /// [`MathError::NonFiniteInput`] (LIM-MATH-0002) if the length is NaN or infinite,
            /// [`MathError::ZeroLengthVector`] (LIM-MATH-0001) if it is zero; both with
            /// `parameter: "self"`.
            pub fn normalize(self) -> Result<$Vec, MathError> {
                let length = self.length();
                if !length.is_finite() {
                    return Err(MathError::NonFiniteInput { parameter: "self" });
                }
                if length == 0.0 {
                    return Err(MathError::ZeroLengthVector { parameter: "self" });
                }
                Ok(self / length)
            }

            /// `normalize()`, or `fallback` unchanged if it fails.
            #[must_use]
            pub fn normalize_or(self, fallback: $Vec) -> $Vec {
                self.normalize().unwrap_or(fallback)
            }

            /// Component-wise: `self`'s component if it is less than `other`'s, else `other`'s
            /// (a NaN in `self` gives `other`'s component).
            #[must_use]
            pub fn min(self, other: $Vec) -> $Vec {
                $Vec {
                    $first: if self.$first < other.$first { self.$first } else { other.$first },
                    $($rest: if self.$rest < other.$rest { self.$rest } else { other.$rest },)*
                }
            }

            /// Component-wise: `self`'s component if it is greater than `other`'s, else
            /// `other`'s (a NaN in `self` gives `other`'s component).
            #[must_use]
            pub fn max(self, other: $Vec) -> $Vec {
                $Vec {
                    $first: if self.$first > other.$first { self.$first } else { other.$first },
                    $($rest: if self.$rest > other.$rest { self.$rest } else { other.$rest },)*
                }
            }

            /// Every component with its sign bit cleared.
            #[must_use]
            pub fn abs(self) -> $Vec {
                $Vec { $first: self.$first.abs() $(, $rest: self.$rest.abs())* }
            }

            /// `self + (end - self) * t`; `t` is not clamped.
            #[must_use]
            pub fn lerp(self, end: $Vec, t: f32) -> $Vec {
                self + (end - self) * t
            }

            /// True if no component is NaN or infinite.
            #[must_use]
            pub fn is_finite(self) -> bool {
                self.$first.is_finite() $(&& self.$rest.is_finite())*
            }
        }

        impl Add for $Vec {
            type Output = $Vec;
            fn add(self, other: $Vec) -> $Vec {
                $Vec { $first: self.$first + other.$first $(, $rest: self.$rest + other.$rest)* }
            }
        }

        impl Sub for $Vec {
            type Output = $Vec;
            fn sub(self, other: $Vec) -> $Vec {
                $Vec { $first: self.$first - other.$first $(, $rest: self.$rest - other.$rest)* }
            }
        }

        impl Neg for $Vec {
            type Output = $Vec;
            fn neg(self) -> $Vec {
                $Vec { $first: -self.$first $(, $rest: -self.$rest)* }
            }
        }

        /// Component-wise product.
        impl Mul for $Vec {
            type Output = $Vec;
            fn mul(self, other: $Vec) -> $Vec {
                $Vec { $first: self.$first * other.$first $(, $rest: self.$rest * other.$rest)* }
            }
        }

        impl Mul<f32> for $Vec {
            type Output = $Vec;
            fn mul(self, scalar: f32) -> $Vec {
                $Vec { $first: self.$first * scalar $(, $rest: self.$rest * scalar)* }
            }
        }

        impl Mul<$Vec> for f32 {
            type Output = $Vec;
            fn mul(self, vector: $Vec) -> $Vec {
                $Vec { $first: self * vector.$first $(, $rest: self * vector.$rest)* }
            }
        }

        /// Divides each component by the scalar (no multiplication by a reciprocal).
        impl Div<f32> for $Vec {
            type Output = $Vec;
            fn div(self, scalar: f32) -> $Vec {
                $Vec { $first: self.$first / scalar $(, $rest: self.$rest / scalar)* }
            }
        }

        impl AddAssign for $Vec {
            fn add_assign(&mut self, other: $Vec) {
                *self = *self + other;
            }
        }

        impl SubAssign for $Vec {
            fn sub_assign(&mut self, other: $Vec) {
                *self = *self - other;
            }
        }

        impl MulAssign<f32> for $Vec {
            fn mul_assign(&mut self, scalar: f32) {
                *self = *self * scalar;
            }
        }

        impl DivAssign<f32> for $Vec {
            fn div_assign(&mut self, scalar: f32) {
                *self = *self / scalar;
            }
        }

        /// The components in order, each with `write_f32`; no length prefix (MATH-R007).
        impl StateHash for $Vec {
            fn state_hash(&self, hasher: &mut StateHasher) {
                hasher.write_f32(self.$first);
                $(hasher.write_f32(self.$rest);)*
            }
        }
    };
}

/// A 2-component `f32` vector. 8 bytes, alignment 4.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[repr(C)]
pub struct Vec2 {
    /// First component.
    pub x: f32,
    /// Second component.
    pub y: f32,
}

/// A 3-component `f32` vector: a point or direction in the right-handed, +Y up, −Z forward
/// space of spec §4. 12 bytes, alignment 4.
///
/// # Examples
///
/// ```
/// use liminal_math::Vec3;
///
/// // Right-handed: X × Y = Z.
/// assert_eq!(Vec3::X.cross(Vec3::Y), Vec3::Z);
///
/// let v = Vec3::new(3.0, 0.0, 4.0);
/// assert_eq!(v.length(), 5.0);
/// assert_eq!(v.normalize()?, Vec3::new(0.6, 0.0, 0.8));
/// assert_eq!(v + Vec3::ONE * 2.0, Vec3::new(5.0, 2.0, 6.0));
/// // A zero vector has no direction: use a fallback.
/// assert_eq!(Vec3::ZERO.normalize_or(Vec3::Y), Vec3::Y);
/// # Ok::<(), liminal_math::MathError>(())
/// ```
#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[repr(C)]
pub struct Vec3 {
    /// First component (+X is right).
    pub x: f32,
    /// Second component (+Y is up).
    pub y: f32,
    /// Third component (−Z is forward).
    pub z: f32,
}

/// A 4-component `f32` vector, for example a homogeneous point. 16 bytes, alignment 16.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[repr(C, align(16))]
pub struct Vec4 {
    /// First component.
    pub x: f32,
    /// Second component.
    pub y: f32,
    /// Third component.
    pub z: f32,
    /// Fourth component.
    pub w: f32,
}

vector_common!(Vec2, 2, x, y);
vector_common!(Vec3, 3, x, y, z);
vector_common!(Vec4, 4, x, y, z, w);

impl Vec2 {
    /// `(1, 0)`.
    pub const X: Vec2 = Vec2::new(1.0, 0.0);
    /// `(0, 1)`.
    pub const Y: Vec2 = Vec2::new(0.0, 1.0);

    /// The vector `(x, y)`.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Vec2 {
        Vec2 { x, y }
    }
}

impl Vec3 {
    /// `(1, 0, 0)`: right.
    pub const X: Vec3 = Vec3::new(1.0, 0.0, 0.0);
    /// `(0, 1, 0)`: up.
    pub const Y: Vec3 = Vec3::new(0.0, 1.0, 0.0);
    /// `(0, 0, 1)`: backward (forward is −Z).
    pub const Z: Vec3 = Vec3::new(0.0, 0.0, 1.0);

    /// The vector `(x, y, z)`.
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x, y, z }
    }

    /// Cross product `(y × z' − z × y', z × x' − x × z', x × y' − y × x')` (MATH-R023).
    #[must_use]
    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// `(x, y, z, w)`.
    #[must_use]
    pub const fn extend(self, w: f32) -> Vec4 {
        Vec4::new(self.x, self.y, self.z, w)
    }
}

impl Vec4 {
    /// `(1, 0, 0, 0)`.
    pub const X: Vec4 = Vec4::new(1.0, 0.0, 0.0, 0.0);
    /// `(0, 1, 0, 0)`.
    pub const Y: Vec4 = Vec4::new(0.0, 1.0, 0.0, 0.0);
    /// `(0, 0, 1, 0)`.
    pub const Z: Vec4 = Vec4::new(0.0, 0.0, 1.0, 0.0);
    /// `(0, 0, 0, 1)`.
    pub const W: Vec4 = Vec4::new(0.0, 0.0, 0.0, 1.0);

    /// The vector `(x, y, z, w)`.
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Vec4 {
        Vec4 { x, y, z, w }
    }

    /// `(x, y, z)`: drops `w`.
    #[must_use]
    pub const fn truncate(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}
