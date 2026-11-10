//! The engine's `f32` math: vectors ([`Vec2`], [`Vec3`], [`Vec4`]), matrices ([`Mat3`],
//! [`Mat4`]), quaternions ([`Quat`]) and affine transforms ([`Affine3`]), with one definition of
//! each floating-point result on every supported platform.
//!
//! Specification: `docs/specs/liminal_math.md`.
//!
//! **Conventions** (engine-wide, spec §4): right-handed (`X × Y = Z`); +X right, +Y up, −Z
//! forward; angles in radians; column vectors (`m * v`); in `a * b` the right-hand factor
//! applies first; column-major storage; clip-space depth 0 (near) to 1 (far).
//!
//! **Bit-identical results.** Every result is computed with IEEE 754 binary32 and binary64
//! basic operations only, in the order the spec's exact expressions give, with no fused
//! multiply-add (MATH-R004); `clippy.toml` bans the platform math functions. The same inputs
//! give the same bits on every supported platform, except the sign and payload of NaN results.
//!
//! The crate is `deterministic = true` (ADR-0009): no clock, environment, randomness or global
//! state. Every public type implements `liminal_core::hash::StateHash`.
//!
//! Not yet implemented: the `det` module and the functions that call it
//! (`Mat4::perspective`, `Quat::from_axis_angle`, `Quat::slerp`; task M1-T021), SIMD paths
//! (task M1-T022) and the geometry types (task M1-T023).
//!
//! # Examples
//!
//! ```
//! use liminal_math::{Affine3, Mat4, Quat, Vec3};
//!
//! let transform = Affine3::from_scale_rotation_translation(
//!     Vec3::ONE,
//!     Quat::IDENTITY,
//!     Vec3::new(0.0, 1.0, 0.0),
//! );
//! let matrix = Mat4::from(transform);
//! assert_eq!(matrix.transform_point3(Vec3::X), Vec3::new(1.0, 1.0, 0.0));
//! assert_eq!(matrix.inverse()?.transform_point3(Vec3::new(1.0, 1.0, 0.0)), Vec3::X);
//! # Ok::<(), liminal_math::MathError>(())
//! ```

// `unsafe` is allowed in this crate for SIMD intrinsics only (architecture.toml, ADR-0018); no
// module needs it yet. The SIMD modules of task M1-T022 allow it locally.
#![deny(unsafe_code)]

mod affine3;
mod error;
mod mat3;
mod mat4;
mod quat;
pub mod reference;
mod vector;

pub use affine3::Affine3;
pub use error::MathError;
pub use mat3::Mat3;
pub use mat4::Mat4;
pub use quat::Quat;
pub use vector::{Vec2, Vec3, Vec4};
