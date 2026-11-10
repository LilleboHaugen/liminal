---
crate: liminal_math
req_prefix: MATH
layer: 0
maturity: planned
spec_version: 2
last_reviewed: 2026-11-10
adrs: [ADR-0009, ADR-0017, ADR-0018, ADR-0023, ADR-0024]
---

# Spec: `liminal_math`

## 1. Purpose

`liminal_math` is the engine's `f32` math: vectors, matrices, quaternions and affine transforms,
the geometry primitives and intersection tests built on them, and `det`, the transcendental
functions that deterministic crates use instead of the platform math library (ADR-0009 item 5).
It is a separate crate at layer 0 so that every crate shares one set of conventions (handedness,
matrix layout, angle unit) and one definition of each floating-point result: the crate fixes which
IEEE 754 operations compute a value, so the same inputs give the same bits on every supported
platform, with or without SIMD.

## 2. Scope

**In scope:**
- Linear algebra types over `f32`: `Vec2`, `Vec3`, `Vec4`, `Mat3`, `Mat4`, `Quat`, `Affine3`,
  their operators and the operations of §5.1 to §5.4.
- The engine-wide conventions of §4: right-handed, +Y up, −Z forward, column vectors,
  column-major storage, radians, clip-space depth 0 to 1.
- `det`: `sin`, `cos`, `sin_cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `exp`, `ln`, `powf`
  for `f32`, with an accuracy bound in ULP and bit-identical results on every supported platform.
- SIMD paths (SSE2 on x86_64, NEON on aarch64) for the four operations of §5.6, and the public
  `reference` module with their scalar reference implementations (PERFORMANCE.md "Reference
  implementations reachable from benchmarks").
- Geometry primitives `Aabb`, `Sphere`, `Plane`, `Ray`, `Frustum` and the intersection tests of
  §5.7.
- The crate's error enum `MathError` (codes `LIM-MATH-NNNN`, §8) and `StateHash` implementations
  for every public value type.

**Out of scope / non-goals:**
- `f64` vector types and integer vector types (`IVec3` for voxel coordinates): not in
  `spec_version` 1 (open question 6).
- Euler angles, quaternion from matrix, quaternion to axis and angle, `nlerp`, rotation between two
  vectors: not in `spec_version` 1 (open question 6).
- Transform hierarchies, a translation-rotation-scale `Transform` component and its propagation:
  `liminal_scene` (M4).
- Cameras, viewports, clip-space differences between Metal and Vulkan: `liminal_render`,
  `liminal_gpu`. This crate provides only the projection matrices of §5.2.
- Swept tests, triangles, capsules, oriented boxes, bounding-volume hierarchies, contact
  generation: `liminal_physics` (M6).
- Noise functions: `liminal_procgen` (M5).
- Random numbers: `liminal_core::rng`. Reflection of the math types: `liminal_reflect` (open
  question 7).
- SIMD versions of `det`, wide (structure-of-arrays) vector types, AVX or other target features
  above the baseline, runtime feature detection.
- `f64` transcendental functions.
- 32-bit targets and targets other than x86_64 and aarch64 get the scalar paths only (ADR-0005).

## 3. Dependencies

- Engine crates: `liminal_core` (`architecture.toml`: layer 0, `same_layer_deps =
  ["liminal_core"]`). Used for `liminal_core::error` (`ErrorCode`, `EngineError`, `ErrorContext`,
  `write_display`, `validate_error`) and `liminal_core::hash` (`StateHash`, `StateHasher`). Tests
  also use `liminal_core::rng::Rng` for seeded inputs.
- External crates: none at runtime (`external = []`). Dev-dependency `proptest` only
  (`dev_external`, ADR-0010).
- `unsafe = "allow"` (ADR-0018), for `core::arch` intrinsics only. `unsafe` is confined to the
  private SIMD modules: the `reference` module contains none (MATH-R130), and `det`, the scalar
  types and the geometry code need none (checked in review, REVIEW.md §1 item 6).
- `deterministic = true`: the determinism lint configuration of CODE_STYLE.md (mechanism chosen
  by M1-T011) applies, extended by the paths of MATH-R004, and the crate meets §11.
- No cargo features.
- Structure: private modules hold the types; the crate root re-exports each type once
  (`liminal_math::Vec3`, `liminal_math::Aabb`, `liminal_math::MathError`), so every item has
  exactly one public path (CODE_STYLE.md "One obvious way"). The only public modules are
  `liminal_math::det` and `liminal_math::reference`. The root contains no logic.

## 4. Concepts

**Conventions** (engine-wide; every crate that uses `liminal_math` inherits them):

| Convention | Value |
|---|---|
| Handedness | Right-handed: `X × Y = Z` |
| Axes | +X right, +Y up, −Z forward (a camera with the identity view matrix looks along −Z) |
| Angle unit | Radians, `f32`. Parameters that take an angle end in `_radians` |
| Rotation direction | A positive angle rotates counter-clockwise when the rotation axis points at the viewer (right-hand rule): rotating +X by +π/2 about +Z gives +Y |
| Vectors | Column vectors. A matrix transforms a vector as `M * v` |
| Composition | `(A * B) * v = A * (B * v)`: the right-hand factor applies first. The same holds for `Quat` and `Affine3` |
| Matrix storage | Column-major: a matrix is a list of column vectors; element (row `r`, column `c`) of a `Mat4` is at index `4 × c + r` of `to_cols_array` |
| Clip space | After the perspective divide: x in −1 (left) to 1 (right), y in −1 (bottom) to 1 (top), depth in 0 (near plane) to 1 (far plane) |
| Scalar type | `f32` everywhere; no `f64` types |

**Exact expression.** Where this spec gives a formula for a result "exactly", each `+`, `−`, `×`,
`÷` and `sqrt` in it is one IEEE 754 binary32 operation, rounded to nearest (ties to even),
evaluated in the order the parentheses give, with no fused multiply-add and no reassociation. A
vector expression such as `a × s + b` (vector `a`, scalar `s`) applies to each component
separately. Two implementations of an exact expression return the same bits for every input,
except NaN results (below).

**NaN bits.** Rust does not define the sign and payload of a NaN that an arithmetic operation
produces, and x86_64 and aarch64 differ. Throughout this spec, "the same bits" and "bit-identical"
mean: where one result is NaN the other is NaN (any sign and payload), and all other results have
equal `to_bits()`. `det` functions are stricter: every NaN they return is the canonical NaN
`f32::from_bits(0x7FC0_0000)`.

**ULP error.** For an `f32` result `y` and the exact real result `r` of a `det` function:
`ulp(r) = 2^(max(e, −126) − 23)` where `e = floor(log2(abs(r)))`, with `e = 127` if
`abs(r) ≥ 2^128`, and `ulp(0) = 2^−149`. The error of `y` is `abs(y − r) ÷ ulp(r)`, where an
infinite `y` counts as `±2^128`. Tests compute `r` with the `f64` function of the Rust standard
library applied to the input widened to `f64`, and evaluate the error in `f64` (assumption: the
platform `f64` function is within 1 `f64` ULP, which is 2^−29 of an `f32` ULP and is ignored).

**Approximately equal at scale `s`.** Two `f32` values `a` and `b` are approximately equal at
scale `s` when `abs(a − b) ≤ 1e-5 × max(1, s)`, evaluated in `f64`. Vectors, quaternions and
matrices are approximately equal when every pair of components is. The tolerances in §6 are
upper bounds for a correct implementation; they come from a prototype measurement, not from the
implementation (open question 1).

**Test domains.** Property requirements name these input domains:
- *D-vec:* vectors whose components are each in −100 to 100.
- *D-unit:* unit quaternions made by normalizing four components in −1 to 1 whose length is at
  least 0.1.
- *D-trs:* scale components in 0.5 to 2, rotation in D-unit, translation components in −10 to 10.
- *D-geo:* points with components in −100 to 100; unit directions made by normalizing a vector
  with components in −1 to 1 and length at least 0.1; radii and half extents in 0.01 to 100.

**Stable case.** A geometric test case (two shapes, a shape and a point, or a ray and a shape) is
*stable* when a reference evaluation in `f64` gives the same answer (true or false, hit or miss)
three times: with the *varied shape* as given, grown by 0.01, and shrunk by 0.01. Exactly one
shape of a case is varied; every other shape, point or ray stays as given.

| Test | Varied shape | Stays as given |
|---|---|---|
| `Sphere::contains_point`, `Sphere::intersects_sphere`, `Sphere::intersects_aabb` (MATH-R157) | The receiver sphere (`self`) | The argument (point, sphere or `Aabb`) |
| `Ray::intersect_sphere`, `Ray::intersect_aabb`, `Ray::intersect_plane` (MATH-R169) | The argument (sphere, `Aabb` or plane) | The ray |

Growing a sphere adds 0.01 to its radius; growing an `Aabb` adds 0.01 to each half extent; growing
a plane, for a ray test, moves it by 0.01 along its normal in either direction. Shrinking
subtracts the same amounts. A case whose shrunk shape would have a negative radius or half extent
is not stable. Only stable cases have a required answer under rounding; exactly representable
cases are covered by unit tests with hand-computed results.

**Solid shapes, closed sets.** `Aabb` and `Sphere` are solid and include their boundary: touching
counts as intersecting, and a point on the surface is contained. A ray that starts inside a solid
shape hits it at distance 0.

**Reference and SIMD path.** For the four operations of §5.6 the crate has a scalar reference
implementation (public, in `liminal_math::reference`) and, on x86_64 and aarch64, a SIMD path
behind the public item. The public item returns the same bits as the reference (MATH-R131).

## 5. Public API

Signatures are normative; bodies and private items are not. Derives listed are part of the API.
All types are `Clone`, `Copy`, `PartialEq` (component-wise `f32` equality: NaN is unequal to
itself, `−0.0 == 0.0`) and `Debug`. None is `Eq`, `Ord` or `Hash`. Only the vector types implement
`Default` (the zero vector); matrices, `Quat` and `Affine3` have `IDENTITY` and no `Default`,
because zero and identity are both plausible defaults.

### 5.1 Vectors

```rust
#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[repr(C)]
pub struct Vec2 { pub x: f32, pub y: f32 }

#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[repr(C)]
pub struct Vec3 { pub x: f32, pub y: f32, pub z: f32 }

#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[repr(C, align(16))]
pub struct Vec4 { pub x: f32, pub y: f32, pub z: f32, pub w: f32 }

impl Vec3 {
    pub const ZERO: Vec3;   // (0, 0, 0)
    pub const ONE: Vec3;    // (1, 1, 1)
    pub const X: Vec3;      // (1, 0, 0)
    pub const Y: Vec3;      // (0, 1, 0)
    pub const Z: Vec3;      // (0, 0, 1)
    pub const fn new(x: f32, y: f32, z: f32) -> Vec3;
    pub const fn splat(value: f32) -> Vec3;
    pub const fn from_array(components: [f32; 3]) -> Vec3;
    pub const fn to_array(self) -> [f32; 3];
    pub fn dot(self, other: Vec3) -> f32;
    pub fn cross(self, other: Vec3) -> Vec3;             // Vec3 only
    pub fn length_squared(self) -> f32;
    pub fn length(self) -> f32;
    /// Errors: LIM-MATH-0002 (length not finite), LIM-MATH-0001 (length is zero).
    pub fn normalize(self) -> Result<Vec3, MathError>;
    /// `normalize()`, or `fallback` if it fails.
    pub fn normalize_or(self, fallback: Vec3) -> Vec3;
    /// Component-wise; a component is `self`'s if it is less than `other`'s, else `other`'s.
    pub fn min(self, other: Vec3) -> Vec3;
    /// Component-wise; a component is `self`'s if it is greater than `other`'s, else `other`'s.
    pub fn max(self, other: Vec3) -> Vec3;
    pub fn abs(self) -> Vec3;
    /// `self + (end - self) * t`; `t` is not clamped.
    pub fn lerp(self, end: Vec3, t: f32) -> Vec3;
    /// True if every component is finite.
    pub fn is_finite(self) -> bool;
    pub const fn extend(self, w: f32) -> Vec4;           // Vec3 only
}
// Operators for Vec3: Add<Vec3>, Sub<Vec3>, Neg, Mul<Vec3> (component-wise), Mul<f32>,
// Div<f32>, Mul<Vec3> for f32 (scalar * vector), AddAssign<Vec3>, SubAssign<Vec3>,
// MulAssign<f32>, DivAssign<f32>.
```

`Vec2` and `Vec4` have the same constants (`Vec2`: `ZERO`, `ONE`, `X`, `Y`; `Vec4`: `ZERO`, `ONE`,
`X`, `Y`, `Z`, `W`), methods and operators with their own component count, except `cross` and
`extend`. `Vec4` also has `pub const fn truncate(self) -> Vec3` (drops `w`).

### 5.2 Matrices

```rust
/// 3×3 matrix, columns `x_axis`, `y_axis`, `z_axis`.
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct Mat3 { pub x_axis: Vec3, pub y_axis: Vec3, pub z_axis: Vec3 }

impl Mat3 {
    pub const IDENTITY: Mat3;
    pub const fn from_cols(x_axis: Vec3, y_axis: Vec3, z_axis: Vec3) -> Mat3;
    /// Column-major: `elements[3 * c + r]` is row `r`, column `c`.
    pub const fn from_cols_array(elements: &[f32; 9]) -> Mat3;
    pub const fn to_cols_array(&self) -> [f32; 9];
    /// Diagonal matrix with `scale` on the diagonal.
    pub fn from_scale(scale: Vec3) -> Mat3;
    /// Rotation matrix of a unit quaternion.
    pub fn from_quat(rotation: Quat) -> Mat3;
    pub fn transpose(&self) -> Mat3;
    pub fn determinant(&self) -> f32;
    /// Errors: LIM-MATH-0003 if the determinant is zero or not finite.
    pub fn inverse(&self) -> Result<Mat3, MathError>;
}
// Operators: Mul<Mat3> for Mat3, Mul<Vec3> for Mat3.

/// 4×4 matrix, columns `x_axis`, `y_axis`, `z_axis`, `w_axis`. 64 bytes, alignment 16.
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct Mat4 { pub x_axis: Vec4, pub y_axis: Vec4, pub z_axis: Vec4, pub w_axis: Vec4 }

impl Mat4 {
    pub const IDENTITY: Mat4;
    pub const fn from_cols(x_axis: Vec4, y_axis: Vec4, z_axis: Vec4, w_axis: Vec4) -> Mat4;
    /// Column-major: `elements[4 * c + r]` is row `r`, column `c`.
    pub const fn from_cols_array(elements: &[f32; 16]) -> Mat4;
    pub const fn to_cols_array(&self) -> [f32; 16];
    pub fn from_translation(translation: Vec3) -> Mat4;
    pub fn from_scale(scale: Vec3) -> Mat4;
    pub fn from_quat(rotation: Quat) -> Mat4;
    /// Scales, then rotates, then translates: `T * R * S`.
    pub fn from_scale_rotation_translation(scale: Vec3, rotation: Quat, translation: Vec3) -> Mat4;
    /// View matrix: maps `eye` to the origin, `direction` to −Z and `up` into the +Y half of
    /// the YZ plane. Errors: LIM-MATH-0002, LIM-MATH-0001 (`direction`), LIM-MATH-0005.
    pub fn look_to(eye: Vec3, direction: Vec3, up: Vec3) -> Result<Mat4, MathError>;
    /// Perspective projection, depth 0 at `z_near` and 1 at `z_far`. Errors: LIM-MATH-0004.
    pub fn perspective(fov_y_radians: f32, aspect_ratio: f32, z_near: f32, z_far: f32)
        -> Result<Mat4, MathError>;
    /// Orthographic projection, depth 0 at `z_near` and 1 at `z_far`. Errors: LIM-MATH-0004.
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, z_near: f32, z_far: f32)
        -> Result<Mat4, MathError>;
    pub fn transpose(&self) -> Mat4;
    pub fn determinant(&self) -> f32;
    /// Errors: LIM-MATH-0003 if the determinant is zero or not finite.
    pub fn inverse(&self) -> Result<Mat4, MathError>;
    /// `(self * point.extend(1.0)).truncate()`; no perspective divide.
    pub fn transform_point3(&self, point: Vec3) -> Vec3;
    /// `(self * vector.extend(0.0)).truncate()`; ignores translation.
    pub fn transform_vector3(&self, vector: Vec3) -> Vec3;
    /// `c = self * point.extend(1.0)`, then `c.truncate() / c.w` (perspective divide).
    pub fn project_point3(&self, point: Vec3) -> Vec3;
}
// Operators: Mul<Mat4> for Mat4 (SIMD path, §5.6), Mul<Vec4> for Mat4 (SIMD path, §5.6).
impl From<Affine3> for Mat4 { /* MATH-R090 */ }
```

### 5.3 Quaternions

```rust
/// Quaternion `x i + y j + z k + w`. A rotation when its length is 1 (unit quaternion).
/// 16 bytes, alignment 16.
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C, align(16))]
pub struct Quat { pub x: f32, pub y: f32, pub z: f32, pub w: f32 }

impl Quat {
    pub const IDENTITY: Quat;   // (0, 0, 0, 1)
    /// No normalization.
    pub const fn from_xyzw(x: f32, y: f32, z: f32, w: f32) -> Quat;
    /// Rotation by `angle_radians` about `axis` (normalized here).
    /// Errors: LIM-MATH-0002 (`angle_radians` or `axis`), LIM-MATH-0001 (`axis`).
    pub fn from_axis_angle(axis: Vec3, angle_radians: f32) -> Result<Quat, MathError>;
    pub fn dot(self, other: Quat) -> f32;
    pub fn length(self) -> f32;
    /// Errors: LIM-MATH-0002, LIM-MATH-0001.
    pub fn normalize(self) -> Result<Quat, MathError>;
    /// `(-x, -y, -z, w)`: the inverse rotation of a unit quaternion.
    pub fn conjugate(self) -> Quat;
    /// Spherical interpolation along the shorter arc; `t` is not clamped.
    pub fn slerp(self, end: Quat, t: f32) -> Quat;
}
// Operators: Mul<Quat> for Quat (Hamilton product; SIMD path, §5.6),
// Mul<Vec3> for Quat (rotates the vector; meaningful for unit quaternions).
```

Operations on a non-unit `Quat` return what their formulas give; nothing panics or asserts
(MATH-R005).

### 5.4 Affine transforms

```rust
/// `point -> matrix3 * point + translation`. 48 bytes, alignment 4.
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct Affine3 { pub matrix3: Mat3, pub translation: Vec3 }

impl Affine3 {
    pub const IDENTITY: Affine3;
    pub fn from_scale_rotation_translation(scale: Vec3, rotation: Quat, translation: Vec3) -> Affine3;
    pub fn transform_point3(&self, point: Vec3) -> Vec3;
    /// `matrix3 * vector`; ignores translation.
    pub fn transform_vector3(&self, vector: Vec3) -> Vec3;
    /// Replaces every element `p` of `points` with `transform_point3(p)` (SIMD path, §5.6).
    pub fn transform_points(&self, points: &mut [Vec3]);
    /// Errors: LIM-MATH-0003 if `matrix3` has a zero or non-finite determinant.
    pub fn inverse(&self) -> Result<Affine3, MathError>;
}
// Operators: Mul<Affine3> for Affine3.
```

### 5.5 `liminal_math::det`

```rust
pub fn sin(x: f32) -> f32;
pub fn cos(x: f32) -> f32;
/// `(sin(x), cos(x))`.
pub fn sin_cos(x: f32) -> (f32, f32);
pub fn tan(x: f32) -> f32;
/// Result in −π/2 to π/2.
pub fn asin(x: f32) -> f32;
/// Result in 0 to π.
pub fn acos(x: f32) -> f32;
/// Result in −π/2 to π/2.
pub fn atan(x: f32) -> f32;
/// Angle of the point (x, y) from the +X axis, in −π to π. Argument order as `f32::atan2`:
/// `y` first.
pub fn atan2(y: f32, x: f32) -> f32;
/// e^x.
pub fn exp(x: f32) -> f32;
/// Natural logarithm.
pub fn ln(x: f32) -> f32;
/// base^exponent.
pub fn powf(base: f32, exponent: f32) -> f32;
```

Special values (normative, referenced by MATH-R102 to MATH-R109). "NaN" is the canonical NaN.
π, π/2, π/4 and 3π/4 stand for the `f32` nearest to each (`core::f32::consts::PI`, `FRAC_PI_2`,
`FRAC_PI_4`, and `2.3561945`).

| Function | Input | Result |
|---|---|---|
| every function | any argument NaN (except the two `powf` rows below) | NaN |
| `sin`, `tan`, `asin`, `atan` | `±0` | `±0` (same sign) |
| `sin`, `cos`, `tan` | `±∞` | NaN |
| `cos` | `±0` | `1` |
| `asin`, `acos` | `x < −1` or `x > 1` | NaN |
| `acos` | `1` | `+0` |
| `atan` | `±∞` | `±π/2` |
| `atan2` | `y = ±0`, `x = +0` or `x > 0` | `±0` |
| `atan2` | `y = ±0`, `x = −0` or `x < 0` | `±π` |
| `atan2` | `y > 0`, `x = ±0` | `π/2`; `y < 0`: `−π/2` |
| `atan2` | `y` finite and `> 0` or `< 0`, `x = +∞` | `±0` (sign of `y`) |
| `atan2` | `y` finite and `> 0` or `< 0`, `x = −∞` | `±π` (sign of `y`) |
| `atan2` | `y = ±∞`, `x` finite | `±π/2` |
| `atan2` | `y = ±∞`, `x = +∞` | `±π/4` |
| `atan2` | `y = ±∞`, `x = −∞` | `±3π/4` |
| `exp` | `±0` | `1` |
| `exp` | `−∞` | `+0` |
| `exp` | `+∞` | `+∞` |
| `ln` | `±0` | `−∞` |
| `ln` | `1` | `+0` |
| `ln` | `x < 0` (including `−∞`) | NaN |
| `ln` | `+∞` | `+∞` |
| `powf` | `exponent = ±0`, any `base` (also NaN) | `1` |
| `powf` | `base = 1`, any `exponent` (also NaN) | `1` |
| `powf` | `base = ±0`, `exponent` an odd integer `< 0` | `±∞` (sign of `base`) |
| `powf` | `base = ±0`, `exponent < 0` otherwise (also `−∞`) | `+∞` |
| `powf` | `base = ±0`, `exponent` an odd integer `> 0` | `±0` (sign of `base`) |
| `powf` | `base = ±0`, `exponent > 0` otherwise (also `+∞`) | `+0` |
| `powf` | `base = −1`, `exponent = ±∞` | `1` |
| `powf` | `abs(base) < 1`, `exponent = +∞` | `+0`; `exponent = −∞`: `+∞` |
| `powf` | `abs(base) > 1`, `exponent = +∞` | `+∞`; `exponent = −∞`: `+0` |
| `powf` | `base = +∞`, `exponent < 0` | `+0`; `exponent > 0`: `+∞` |
| `powf` | `base = −∞`, `exponent` an odd integer `< 0` | `−0`; other `exponent < 0`: `+0` |
| `powf` | `base = −∞`, `exponent` an odd integer `> 0` | `−∞`; other `exponent > 0`: `+∞` |
| `powf` | `base < 0` finite, `exponent` finite and not an integer | NaN |
| `powf` | `base < 0` finite, `exponent` an integer | `powf(−base, exponent)`, negated if `exponent` is odd |

These are the special cases of IEEE 754-2019 §9.2 (`sin`, `cos`, `tan`, `asin`, `acos`, `atan`,
`atan2`, `exp`, `log`, `pow`). For the "odd integer" rows: every `f32` with `abs(x) ≥ 2^24` is an
even integer.

### 5.6 `liminal_math::reference` and SIMD paths

```rust
/// Oracle for `<Mat4 as Mul<Mat4>>::mul`.
pub fn mat4_mul(a: Mat4, b: Mat4) -> Mat4;
/// Oracle for `<Mat4 as Mul<Vec4>>::mul`.
pub fn mat4_mul_vec4(matrix: Mat4, vector: Vec4) -> Vec4;
/// Oracle for `<Quat as Mul<Quat>>::mul`.
pub fn quat_mul(a: Quat, b: Quat) -> Quat;
/// Oracle for `Affine3::transform_points`.
pub fn affine3_transform_points(affine: &Affine3, points: &mut [Vec3]);
```

| Public item (SIMD path on x86_64 and aarch64) | Reference item | Differential test in `crates/liminal_math/tests/simd_differential.rs` |
|---|---|---|
| `Mat4 * Mat4` | `reference::mat4_mul` | `mat4_mul_matches_reference` |
| `Mat4 * Vec4` | `reference::mat4_mul_vec4` | `mat4_mul_vec4_matches_reference` |
| `Quat * Quat` | `reference::quat_mul` | `quat_mul_matches_reference` |
| `Affine3::transform_points` | `reference::affine3_transform_points` | `affine3_transform_points_matches_reference` |

Three counterparts are operator implementations, which cannot share a name with a free function;
the reference items therefore have the operand and result types of their counterpart and a name
made of the type and the operation (PERFORMANCE.md rule 1 asks for the same name and signature;
open question 8). The reference items contain no `unsafe`, are always compiled, and are the
implementation the public items use on targets without a SIMD path. A SIMD path is kept on a
target only while a benchmark shows it faster than the reference there (ADR-0018; §9, open
question 3); a path that is dropped is removed from this table by a `spec_version` bump, and its
reference item leaves the `reference` module (PERFORMANCE.md rule 3).

### 5.7 Geometry

Fields are private; constructors establish the invariants of §7.

```rust
/// Axis-aligned box, solid, closed. `min <= max` on every axis.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Aabb { /* min: Vec3, max: Vec3 */ }

impl Aabb {
    /// Errors: LIM-MATH-0002, LIM-MATH-0010.
    pub fn new(min: Vec3, max: Vec3) -> Result<Aabb, MathError>;
    /// Errors: LIM-MATH-0002, LIM-MATH-0011.
    pub fn from_center_half_extents(center: Vec3, half_extents: Vec3) -> Result<Aabb, MathError>;
    /// Smallest box containing all points. Errors: LIM-MATH-0012 (empty slice), LIM-MATH-0002.
    pub fn from_points(points: &[Vec3]) -> Result<Aabb, MathError>;
    pub fn min(&self) -> Vec3;
    pub fn max(&self) -> Vec3;
    pub fn center(&self) -> Vec3;
    pub fn half_extents(&self) -> Vec3;
    pub fn contains_point(&self, point: Vec3) -> bool;
    pub fn intersects_aabb(&self, other: &Aabb) -> bool;
    /// The point of the box nearest to `point` (`point` itself if inside).
    pub fn closest_point(&self, point: Vec3) -> Vec3;
    /// Smallest box containing both.
    pub fn union(&self, other: &Aabb) -> Aabb;
    /// Smallest box containing the eight transformed corners. Errors: LIM-MATH-0002.
    pub fn transformed(&self, affine: &Affine3) -> Result<Aabb, MathError>;
}

/// Solid, closed ball. `radius >= 0`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Sphere { /* center: Vec3, radius: f32 */ }

impl Sphere {
    /// Errors: LIM-MATH-0002, LIM-MATH-0011.
    pub fn new(center: Vec3, radius: f32) -> Result<Sphere, MathError>;
    pub fn center(&self) -> Vec3;
    pub fn radius(&self) -> f32;
    pub fn contains_point(&self, point: Vec3) -> bool;
    pub fn intersects_sphere(&self, other: &Sphere) -> bool;
    pub fn intersects_aabb(&self, aabb: &Aabb) -> bool;
}

/// The points `p` with `normal · p + d = 0`; `normal` has length 1. The positive side is the
/// side `normal` points to.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Plane { /* normal: Vec3, d: f32 */ }

impl Plane {
    /// Plane through `point` with the direction of `normal` (normalized here).
    /// Errors: LIM-MATH-0002, LIM-MATH-0001 (`normal`), LIM-MATH-0015 (`normal`).
    pub fn from_normal_and_point(normal: Vec3, point: Vec3) -> Result<Plane, MathError>;
    /// Plane through three points; the normal is `(b - a) × (c - a)` normalized, so the points
    /// are counter-clockwise seen from the positive side. Errors: LIM-MATH-0002, LIM-MATH-0013.
    pub fn from_points(a: Vec3, b: Vec3, c: Vec3) -> Result<Plane, MathError>;
    pub fn normal(&self) -> Vec3;
    pub fn d(&self) -> f32;
    /// `normal · point + d`: positive on the positive side.
    pub fn signed_distance(&self, point: Vec3) -> f32;
}

/// Half-line from `origin` along `direction` (length 1).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Ray { /* origin: Vec3, direction: Vec3 */ }

impl Ray {
    /// `direction` is normalized here.
    /// Errors: LIM-MATH-0002, LIM-MATH-0001 (`direction`), LIM-MATH-0015 (`direction`).
    pub fn new(origin: Vec3, direction: Vec3) -> Result<Ray, MathError>;
    pub fn origin(&self) -> Vec3;
    pub fn direction(&self) -> Vec3;
    /// `origin + direction * distance`.
    pub fn at(&self, distance: f32) -> Vec3;
    /// Distance (>= 0) along the ray to the plane, or `None`.
    pub fn intersect_plane(&self, plane: &Plane) -> Option<f32>;
    /// Distance (>= 0) to the first point of the solid sphere, or `None`.
    pub fn intersect_sphere(&self, sphere: &Sphere) -> Option<f32>;
    /// Distance (>= 0) to the first point of the solid box, or `None`.
    pub fn intersect_aabb(&self, aabb: &Aabb) -> Option<f32>;
}

/// Six planes with normals pointing into the volume, in the order left, right, bottom, top,
/// near, far.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Frustum { /* planes: [Plane; 6] */ }

impl Frustum {
    /// From a matrix that maps world space to clip space (projection * view).
    /// Errors: LIM-MATH-0002, LIM-MATH-0014.
    pub fn from_view_projection(view_projection: &Mat4) -> Result<Frustum, MathError>;
    pub fn planes(&self) -> &[Plane; 6];
    pub fn contains_point(&self, point: Vec3) -> bool;
    /// Conservative: never false for a sphere that touches the volume; may be true for a
    /// sphere outside it near an edge or corner.
    pub fn intersects_sphere(&self, sphere: &Sphere) -> bool;
    /// Conservative, as `intersects_sphere`.
    pub fn intersects_aabb(&self, aabb: &Aabb) -> bool;
}
```

### 5.8 Errors

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum MathError {
    ZeroLengthVector { parameter: &'static str },                 // LIM-MATH-0001
    NonFiniteInput { parameter: &'static str },                   // LIM-MATH-0002
    SingularMatrix { determinant: f32 },                          // LIM-MATH-0003
    InvalidProjection { parameter: &'static str, value: f32 },    // LIM-MATH-0004
    InvalidViewBasis,                                             // LIM-MATH-0005
    InvalidAabb { axis: &'static str, min: f32, max: f32 },       // LIM-MATH-0010
    NegativeExtent { parameter: &'static str, value: f32 },       // LIM-MATH-0011
    EmptyPointSet,                                                // LIM-MATH-0012
    CollinearPoints,                                              // LIM-MATH-0013
    DegenerateFrustum { plane: &'static str },                    // LIM-MATH-0014
    VectorTooShort { parameter: &'static str, length_squared: f32 }, // LIM-MATH-0015
}
impl liminal_core::error::EngineError for MathError { /* … */ }
impl std::error::Error for MathError {}
impl core::fmt::Display for MathError { /* liminal_core::error::write_display */ }
```

`parameter` is the name of the function parameter as written in this section (`"axis"`,
`"direction"`, `"radius"`), or `"self"` for a method receiver. `axis` is `"x"`, `"y"` or `"z"`.
`plane` is `"left"`, `"right"`, `"bottom"`, `"top"`, `"near"` or `"far"`.

`StateHash` (`liminal_core::hash`) is implemented for `Vec2`, `Vec3`, `Vec4`, `Mat3`, `Mat4`,
`Quat`, `Affine3`, `Aabb`, `Sphere`, `Plane`, `Ray` and `Frustum` (MATH-R007, MATH-R170).

## 6. Requirements

In requirement text, `abs(x)` is the absolute value, `length`, `dot` and `cross` are the public
methods, "exactly" refers to §4 "Exact expression", "approximately equal at scale" and the domains
D-vec, D-unit, D-trs and D-geo to §4.

### Crate-wide (R001–R019)

| ID | Level | Requirement |
|---|---|---|
| MATH-R001 | MUST | Sizes and alignments in bytes: `Vec2` 8 and 4; `Vec3` 12 and 4; `Vec4` 16 and 16; `Quat` 16 and 16; `Mat3` 36 and 4; `Mat4` 64 and 16; `Affine3` 48 and 4. Each type is `#[repr(C)]` with its `f32` fields at offsets `4 × i` in the order of §5 (columns in order for matrices; `matrix3` then `translation` for `Affine3`). `Vec2::default()`, `Vec3::default()` and `Vec4::default()` equal their `ZERO`. |
| MATH-R002 | MUST | Conventions of §4: `Vec3::X.cross(Vec3::Y) == Vec3::Z`, `Vec3::Y.cross(Vec3::Z) == Vec3::X` and `Vec3::Z.cross(Vec3::X) == Vec3::Y` exactly; with `h = core::f32::consts::FRAC_1_SQRT_2`, `Quat::from_xyzw(0, 0, h, h) * Vec3::X` (a rotation by +π/2 about +Z) is approximately equal at scale 1 to `Vec3::Y`, and likewise `from_xyzw(h, 0, 0, h)` takes `Vec3::Y` to `Vec3::Z` and `from_xyzw(0, h, 0, h)` takes `Vec3::Z` to `Vec3::X`; `Mat4::from_translation(t) * p.extend(1.0)` equals `(p + t).extend(1.0)` for exactly representable sums; for matrices `a`, `b` and a vector `v` with small integer components (no rounding), `(a * b) * v == a * (b * v)`. |
| MATH-R003 | MUST | `Mat4::from_cols_array(&e)` builds the matrix whose element at row `r`, column `c` is `e[4 × c + r]`, and `Mat3::from_cols_array(&e)` uses `e[3 × c + r]`. `to_cols_array` is the inverse: `from_cols_array(&m.to_cols_array()) == m` and the array of `from_cols_array(&e)` has the bits of `e`. `from_cols` stores its arguments as the columns in order. |
| MATH-R004 | MUST | The crate computes floating-point results only with IEEE 754 binary32 and binary64 addition, subtraction, multiplication, division, square root, negation, absolute value, comparison, `floor`, `ceil`, `trunc`, conversions between integers and floats (Rust `as`: integer to float rounds to nearest, ties to even; float to integer truncates toward zero and saturates, NaN gives 0), conversion from binary32 to binary64 (exact; `f64::from` or `as`), conversion from binary64 to binary32 (Rust `as`: rounded to nearest, ties to even, overflow gives an infinity), and bit manipulation of `to_bits` values (`to_bits`, `from_bits`). A `det` function may therefore evaluate in binary64 and round the result once to binary32. Its clippy `disallowed-methods` configuration (mechanism of M1-T011) bans, for `f32` and `f64`: the CODE_STYLE.md list (`sin`, `cos`, `tan`, `powf`, `exp`, `ln`) and also `mul_add`, `sin_cos`, `asin`, `acos`, `atan`, `atan2`, `sinh`, `cosh`, `tanh`, `asinh`, `acosh`, `atanh`, `powi`, `exp2`, `exp_m1`, `ln_1p`, `log`, `log2`, `log10`, `hypot`, `cbrt`; and the estimate and fused intrinsics `_mm_rsqrt_ps`, `_mm_rsqrt_ss`, `_mm_rcp_ps`, `_mm_rcp_ss`, `_mm_fmadd_ps`, `vrsqrteq_f32`, `vrecpeq_f32`, `vfmaq_f32`, `vfmsq_f32`, `vmlaq_f32`, `vmlsq_f32`. A test reads that configuration and fails if a listed path is missing. Tests may use the banned `f64` functions as oracles. |
| MATH-R005 | MUST | No public function or operator of the crate panics, for arguments of any bit pattern (NaN, infinities and subnormal numbers included) and slices of any length. Invalid input to a function that returns `Result` gives the `Err` named in its requirement; other functions return the value their formula gives. Each implementation task extends the property test of this requirement to the functions it adds. |
| MATH-R006 | MUST | `MathError` implements `liminal_core::error::EngineError`. Each variant has the code, message, help and context fields of §8, its `Display` is `liminal_core::error::write_display`, and one value of every variant passes `liminal_core::error::validate_error(&e, "MATH")`. |
| MATH-R007 | MUST | `StateHash` encodings, each as consecutive `write_f32` calls with no length prefix: `Vec2` `x, y`; `Vec3` `x, y, z`; `Vec4` and `Quat` `x, y, z, w`; `Mat3` and `Mat4` their columns in order; `Affine3` `matrix3` then `translation`. The hash of a value equals the hash of its `to_array` or `to_cols_array` elements written one by one. |

### Vectors (R020–R039)

Each requirement applies to `Vec2`, `Vec3` and `Vec4` unless it names one type.

| ID | Level | Requirement |
|---|---|---|
| MATH-R020 | MUST | `new` stores its arguments in the fields of the same name; `splat(v)` sets every component to `v`; `from_array(a).to_array()` has the bits of `a`, with `x` at index 0. The constants have the values of §5.1 (`X`, `Y`, `Z`, `W` have 1 in the named component and 0 elsewhere). `Vec3::extend(w)` returns `(x, y, z, w)` and `Vec4::truncate()` returns `(x, y, z)`. |
| MATH-R021 | MUST | The operators compute each component with exactly one IEEE 754 operation: `a + b`, `a - b`, `a * b` (component-wise) on the pair of components; `-a` negates (flips the sign bit); `a * s` and `s * a` multiply each component by the scalar `s`; `a / s` divides each component by `s` (no multiplication by a reciprocal). `+=`, `-=`, `*=` and `/=` store the result of the matching binary operator. |
| MATH-R022 | MUST | `dot` returns exactly `a.x × b.x + a.y × b.y` (`Vec2`), `(a.x × b.x + a.y × b.y) + a.z × b.z` (`Vec3`), `((a.x × b.x + a.y × b.y) + a.z × b.z) + a.w × b.w` (`Vec4`). `length_squared()` is `self.dot(self)` and `length()` is exactly `sqrt(length_squared())`. Neither rescales its input: a component with `abs` above 1.8e19 can overflow to infinity and one below 1e-19 can underflow to zero. |
| MATH-R023 | MUST | `Vec3::cross` returns exactly `(a.y × b.z − a.z × b.y, a.z × b.x − a.x × b.z, a.x × b.y − a.y × b.x)`. For inputs of every bit pattern (NaN and infinities included), each component of `a.cross(b)` and the same component of `-(b.cross(a))` are either both NaN or equal under `==`. Two cases make this weaker than vector `==` and than equal bits: a product can overflow, so that with `a = b = (0, 3e19, 3e19)` both `x` components are NaN (`∞ − ∞`); and a zero component can differ in sign (`p − p` is `+0`, `−(p − p)` is `−0`). For `a`, `b` in D-vec, `a.cross(b).dot(a)` and `a.cross(b).dot(b)` are approximately equal to 0 at scale `length(a)² × length(b)` and `length(a) × length(b)²`. |
| MATH-R024 | MUST | `normalize()` computes `l = length()`. If `l` is NaN or infinite it returns `Err(MathError::NonFiniteInput { parameter: "self" })` (LIM-MATH-0002); if `l == 0.0` it returns `Err(MathError::ZeroLengthVector { parameter: "self" })` (LIM-MATH-0001); otherwise `Ok(self / l)`. `normalize_or(fallback)` returns the `Ok` value of `normalize()`, or `fallback` unchanged if it is `Err`. |
| MATH-R025 | MUST | For a vector whose non-zero components have `abs` in 1e-10 to 1e10 and that has at least one non-zero component, `normalize()` is `Ok(n)` with `abs(n.length() − 1) ≤ 4 × f32::EPSILON`, and each component of `n` has the sign of the input component. |
| MATH-R026 | MUST | `min` returns, per component, `self`'s component if it is less than `other`'s, else `other`'s; `max` likewise with greater than (so a NaN component in `self` gives `other`'s component). `abs` clears the sign bit of every component. `lerp(end, t)` returns exactly `self + (end − self) × t`; `lerp(end, 0.0) == self` for inputs in D-vec. `is_finite` is true exactly when no component is NaN or infinite. |

### Matrices (R040–R069)

| ID | Level | Requirement |
|---|---|---|
| MATH-R040 | MUST | `Mat3::IDENTITY` and `Mat4::IDENTITY` have 1 on the diagonal and 0 elsewhere. `Mat3::from_scale(s)` and `Mat4::from_scale(s)` are diagonal with `s.x, s.y, s.z` (and 1 for `Mat4`). `Mat4::from_translation(t)` is the identity with `w_axis = (t.x, t.y, t.z, 1)`. Off-diagonal and unset elements are `+0.0`. |
| MATH-R041 | MUST | `Mat4 * Vec4` returns exactly `((x_axis × v.x + y_axis × v.y) + z_axis × v.z) + w_axis × v.w`. `Mat3 * Vec3` returns exactly `(x_axis × v.x + y_axis × v.y) + z_axis × v.z`. |
| MATH-R042 | MUST | `a * b` for `Mat4` returns the matrix whose columns are `a * b.x_axis`, `a * b.y_axis`, `a * b.z_axis`, `a * b.w_axis` (MATH-R041); for `Mat3` likewise with three columns. `m * IDENTITY == m` and `IDENTITY * m == m` for finite `m`. |
| MATH-R043 | MUST | `transpose()` returns the matrix whose element at row `r`, column `c` has the bits of the element at row `c`, column `r`. |
| MATH-R044 | MUST | `determinant()` returns the determinant. For a matrix whose elements are integers with `abs` at most 8 the result is exact (every intermediate value is an integer below 2^24). `IDENTITY.determinant() == 1.0`. For `m` built by `from_scale_rotation_translation` from D-trs, `Mat4::determinant` is approximately equal at scale 8 to `scale.x × scale.y × scale.z`; the same holds for `Mat3::from_quat(r) * Mat3::from_scale(s)`. |
| MATH-R045 | MUST | `inverse()` computes `d = determinant()`. If `d == 0.0`, or `d` is NaN or infinite, it returns `Err(MathError::SingularMatrix { determinant: d })` (LIM-MATH-0003); otherwise `Ok` of the adjugate divided by `d`. `IDENTITY.inverse() == Ok(IDENTITY)`. For `Mat4` `m` built from D-trs, and for `Mat3` `m = Mat3::from_quat(r) * Mat3::from_scale(s)` from D-trs, every element of `m * m.inverse()` and of `m.inverse() * m` differs from the identity element by at most 1e-4. |
| MATH-R046 | MUST | For `q` in D-unit and `v` in D-vec, `Mat3::from_quat(q) * v` is approximately equal to `q * v` at scale `length(v)`. `Mat4::from_quat(q)` has `Mat3::from_quat(q)` in its upper-left 3×3 block (same bits), `w_axis = (0, 0, 0, 1)` and 0 in the fourth row of the other columns. `Mat3::from_quat(Quat::IDENTITY) == Mat3::IDENTITY`. |
| MATH-R047 | MUST | `Mat4::from_scale_rotation_translation(s, r, t)` returns the matrix with columns `(R.x_axis × s.x, 0)`, `(R.y_axis × s.y, 0)`, `(R.z_axis × s.z, 0)`, `(t, 1)` exactly, where `R = Mat3::from_quat(r)`. |
| MATH-R048 | MUST | `transform_point3(p)` returns `(self * p.extend(1.0)).truncate()`, `transform_vector3(v)` returns `(self * v.extend(0.0)).truncate()`, and `project_point3(p)` returns `c.truncate() / c.w` with `c = self * p.extend(1.0)`, each with the same bits as that expression. `project_point3` does not check `c.w`: a zero gives infinite or NaN components. |
| MATH-R049 | MUST | `Mat4::look_to(eye, direction, up)`: if a component of any argument is NaN or infinite, `Err(NonFiniteInput)` naming the first such parameter in signature order; else `f = direction.normalize()`, with its error returned with `parameter: "direction"`; else `s = f.cross(up).normalize()`, and if that fails `Err(MathError::InvalidViewBasis)` (LIM-MATH-0005); else, with `u = s.cross(f)`, `Ok` of the matrix with columns `(s.x, u.x, −f.x, 0)`, `(s.y, u.y, −f.y, 0)`, `(s.z, u.z, −f.z, 0)`, `(−s.dot(eye), −u.dot(eye), f.dot(eye), 1)`. For `eye` in D-vec, a unit `direction` and a unit `up` whose angle to `direction` is between 0.1 and π − 0.1 radians: `transform_point3(eye)` is approximately 0 at scale `length(eye)`, `transform_vector3(direction)` is approximately `(0, 0, −1)` at scale 1, and `transform_vector3(up)` has `x` approximately 0 at scale 1 and `y > 0`. |
| MATH-R050 | MUST | `Mat4::perspective(fov_y_radians, aspect_ratio, z_near, z_far)` returns `Err(MathError::InvalidProjection)` (LIM-MATH-0004) naming the first parameter, in signature order, that fails its check, with its value: `fov_y_radians` finite with `0 < fov_y_radians < core::f32::consts::PI`; `aspect_ratio` finite and `> 0`; `z_near` finite and `> 0`; `z_far` finite and `> z_near`. Otherwise, with `f = 1 ÷ det::tan(fov_y_radians × 0.5)`, `Ok` of the matrix with columns `(f ÷ aspect_ratio, 0, 0, 0)`, `(0, f, 0, 0)`, `(0, 0, z_far ÷ (z_near − z_far), −1)`, `(0, 0, (z_near × z_far) ÷ (z_near − z_far), 0)` exactly. For `fov_y_radians` in 0.2 to 2.5, `aspect_ratio` in 0.5 to 3, `z_near` in 0.05 to 1 and `z_far` in 10 to 500: `project_point3((0, 0, −z_near))` is approximately `(0, 0, 0)` and `project_point3((0, 0, −z_far))` approximately `(0, 0, 1)` at scale 1; a point with `x > 0`, `y > 0`, `z < 0` projects to `x > 0` and `y > 0`. |
| MATH-R051 | MUST | `Mat4::orthographic(left, right, bottom, top, z_near, z_far)` returns `Err(MathError::InvalidProjection)` (LIM-MATH-0004) with the parameter's name and value for the first of these that applies: a parameter that is NaN or infinite (the first in signature order); `right` if `right == left`; `top` if `top == bottom`; `z_far` if `z_far == z_near`. Otherwise `Ok` of the matrix with columns `(2 ÷ (right − left), 0, 0, 0)`, `(0, 2 ÷ (top − bottom), 0, 0)`, `(0, 0, 1 ÷ (z_near − z_far), 0)`, `(−(right + left) ÷ (right − left), −(top + bottom) ÷ (top − bottom), z_near ÷ (z_near − z_far), 1)` exactly. For `left`, `right`, `bottom`, `top` in −10 to 10 with `right − left ≥ 1` and `top − bottom ≥ 1`, `z_near` in 0.05 to 1 and `z_far` in 10 to 500, each component of `transform_point3((left, bottom, −z_near))` differs from `(−1, −1, 0)` by at most 1e-4, and each component of `transform_point3((right, top, −z_far))` from `(1, 1, 1)` by at most 1e-4. |

### Quaternions (R070–R089)

| ID | Level | Requirement |
|---|---|---|
| MATH-R070 | MUST | `Quat::IDENTITY` is `(0, 0, 0, 1)`. `from_xyzw` stores its arguments unchanged. `dot` returns exactly `((a.x × b.x + a.y × b.y) + a.z × b.z) + a.w × b.w`; `length()` is exactly `sqrt(self.dot(self))`. `conjugate()` negates `x`, `y`, `z` (sign bit) and keeps `w`. `normalize()` follows MATH-R024 and MATH-R025 with four components. |
| MATH-R071 | MUST | `a * b` for `Quat` returns exactly `x = ((a.w × b.x + a.x × b.w) + a.y × b.z) − a.z × b.y`, `y = ((a.w × b.y − a.x × b.z) + a.y × b.w) + a.z × b.x`, `z = ((a.w × b.z + a.x × b.y) − a.y × b.x) + a.z × b.w`, `w = ((a.w × b.w − a.x × b.x) − a.y × b.y) − a.z × b.z`. `q * Quat::IDENTITY == q` and `Quat::IDENTITY * q == q` for finite `q`. |
| MATH-R072 | MUST | `q * v` for `Quat` and `Vec3` returns exactly `(v + t × q.w) + u.cross(t)` with `u = (q.x, q.y, q.z)` and `t = u.cross(v) × 2`. For `a`, `b` in D-unit and `v` in D-vec, at scale `length(v)`: `length(a * v)` is approximately `length(v)`; `(a * b) * v` is approximately `a * (b * v)`; `a.conjugate() * (a * v)` is approximately `v`. `Quat::IDENTITY * v == v` for every finite `v` (an infinite component of `v` gives NaN components from `0 × ∞`). |
| MATH-R073 | MUST | `Quat::from_axis_angle(axis, angle_radians)`: if `angle_radians` is NaN or infinite, `Err(NonFiniteInput { parameter: "angle_radians" })`; else `n = axis.normalize()`, with its error returned with `parameter: "axis"`; else, with `(s, c) = det::sin_cos(angle_radians × 0.5)`, `Ok(Quat::from_xyzw(n.x × s, n.y × s, n.z × s, c))`. `from_axis_angle(Vec3::Z, FRAC_PI_2)` applied to `Vec3::X` is approximately `Vec3::Y` at scale 1 (rotation direction of §4). For a unit `axis`, `angle_radians` in −2π to 2π and `v` in D-vec: `q * axis` is approximately `axis` at scale 1, and `(q * v).dot(v)` is approximately `v.dot(v) × cos(angle) + (axis.dot(v))² × (1 − cos(angle))` (evaluated in `f64`) at scale `length(v)²`. |
| MATH-R074 | MUST | `a.slerp(end, t)`: `d = a.dot(end)`; if `d < 0`, `b = −end` (all four components negated) and `d = −d`, else `b = end`. If `d > 0.9995` the result is `r ÷ length(r)` with `r = a + (b − a) × t` (four components). Otherwise, with `θ = det::acos(d)`, the result is `(a × det::sin((1 − t) × θ) + b × det::sin(t × θ)) ÷ det::sin(θ)`. For `a`, `end` in D-unit and `t` in 0 to 1: the result's length differs from 1 by at most 1e-5; `a.slerp(end, 0.0)` and `a` differ by at most 1e-6 per component, as do `a.slerp(end, 1.0)` and `b`; and `a.dot(result)` differs from `cos(t × acos(abs(d)))` (evaluated in `f64`) by at most 1e-5. |

### Affine transforms (R090–R099)

| ID | Level | Requirement |
|---|---|---|
| MATH-R090 | MUST | `Affine3::IDENTITY` is `Mat3::IDENTITY` with zero translation. `Affine3::from_scale_rotation_translation(s, r, t)` has `matrix3` with columns `R.x_axis × s.x`, `R.y_axis × s.y`, `R.z_axis × s.z` (`R = Mat3::from_quat(r)`) and `translation = t`, the same bits as the upper-left block and `w_axis` of the `Mat4` function of MATH-R047. `Mat4::from(a)` has columns `(a.matrix3.x_axis, 0)`, `(a.matrix3.y_axis, 0)`, `(a.matrix3.z_axis, 0)`, `(a.translation, 1)`. |
| MATH-R091 | MUST | `transform_point3(p)` returns exactly `((matrix3.x_axis × p.x + matrix3.y_axis × p.y) + matrix3.z_axis × p.z) + translation`. `transform_vector3(v)` returns `matrix3 * v` (MATH-R041). |
| MATH-R092 | MUST | `a * b` for `Affine3` returns `Affine3 { matrix3: a.matrix3 * b.matrix3, translation: a.transform_point3(b.translation) }`. For `a`, `b` from D-trs and `p` with components in −10 to 10, `(a * b).transform_point3(p)` is approximately `a.transform_point3(b.transform_point3(p))` at scale 100. |
| MATH-R093 | MUST | `inverse()` returns the error of `matrix3.inverse()` (LIM-MATH-0003) if it fails; otherwise `Ok(Affine3 { matrix3: m, translation: −(m * translation) })` with `m` the inverse of `matrix3`. For `a` from D-trs and `p` with components in −10 to 10, each component of `a.inverse().transform_point3(a.transform_point3(p))` differs from that of `p` by at most `1e-4 × max(1, length(p))`. |
| MATH-R094 | MUST | `transform_points(points)` replaces each element `p` with a value that has the same bits as `transform_point3(p)`, for slices of every length including 0, and changes nothing else. |

### `det` (R100–R129)

| ID | Level | Requirement |
|---|---|---|
| MATH-R100 | MUST | Every `det` function is a pure function of its arguments' bits, computed only with the operations of MATH-R004, and returns the same bits on every supported platform for every input. Every NaN result has the bits `0x7FC0_0000`. No `det` function reads or changes the floating-point environment. |
| MATH-R101 | MUST | Every `det` function returns the result of §5.5 "Special values" for the inputs listed there, with the stated sign of zero. |
| MATH-R102 | MUST | `det::sin` and `det::cos`: ULP error (§4) at most 1.0 for every `x` with `abs(x) ≤ 1048576` (2^20). For every finite `x` (also above 2^20, where no accuracy is required) the result is in −1 to 1 and not NaN. |
| MATH-R103 | MUST | For every `x`, `det::sin_cos(x)` returns the same bits as `(det::sin(x), det::cos(x))`. For every finite `x`, `det::sin(−x)` has the bits of `−det::sin(x)` and `det::cos(−x)` has the bits of `det::cos(x)`. |
| MATH-R104 | MUST | `det::tan`: ULP error at most 1.0 for every `x` with `abs(x) ≤ 1048576`; `det::tan(−x)` has the bits of `−det::tan(x)` for every finite `x`; no finite `x` gives NaN. |
| MATH-R105 | MUST | `det::asin` and `det::acos`: ULP error at most 1.0 for every `x` in −1 to 1. `det::asin(−x)` has the bits of `−det::asin(x)`. |
| MATH-R106 | MUST | `det::atan`: ULP error at most 1.0 for every finite `x`. `det::atan2`: ULP error at most 1.0 for every pair of finite arguments that are not both zero. Results never exceed the `f32` nearest to π/2 (`atan`) or π (`atan2`) in absolute value. |
| MATH-R107 | MUST | `det::exp`: ULP error at most 1.0 for every finite `x`, including results that are subnormal, zero (`x < −104`) or infinite (`x > 88.73`). |
| MATH-R108 | MUST | `det::ln`: ULP error at most 1.0 for every finite `x > 0`, subnormal inputs included. |
| MATH-R109 | MUST | `det::powf`: ULP error at most 1.0 for every pair of finite arguments whose exact result is a real number (`base > 0`, or `base < 0` with an integer `exponent`, or `base = ±0` with `exponent > 0`), including results that are subnormal, zero or infinite. `powf(x, 1.0)` has the bits of `x` for every finite `x`. |
| MATH-R110 | MUST | Workload `det_functions` in `crates/liminal_math/tests/determinism.rs` (§11) gives the same state hash in two runs in one process and equals its recorded hash literal on every supported platform. |

### Reference and SIMD paths (R130–R149)

| ID | Level | Requirement |
|---|---|---|
| MATH-R130 | MUST | `liminal_math::reference` contains exactly the four functions of §5.6. They contain no `unsafe` and no `core::arch` item, and return exactly: `mat4_mul` the product of MATH-R042; `mat4_mul_vec4` the product of MATH-R041; `quat_mul` the product of MATH-R071; `affine3_transform_points` the per-element result of MATH-R091. |
| MATH-R131 | MUST | On x86_64 and on aarch64, `Mat4 * Mat4`, `Mat4 * Vec4`, `Quat * Quat` and `Affine3::transform_points` return the same bits (§4 "NaN bits") as their `reference` items for inputs of every bit pattern, including NaN, infinities, subnormal numbers and `−0.0`, and, for `transform_points`, every slice length from 0 to 67 at any start offset within a larger buffer. The differential tests of §5.6 check this on both architectures. |
| MATH-R132 | MUST | SIMD paths use only the target's baseline features (SSE2 on x86_64, NEON on aarch64) and are selected at compile time with `cfg(target_arch)`. The crate contains no runtime feature detection (`is_x86_feature_detected!`, `is_aarch64_feature_detected!`) and no `#[target_feature]` attribute; a test scans `crates/liminal_math/src` for these. On every other architecture the public items call the `reference` items. |

### Geometry (R150–R189)

| ID | Level | Requirement |
|---|---|---|
| MATH-R150 | MUST | `Aabb::new(min, max)`: `Err(NonFiniteInput)` naming the first parameter with a NaN or infinite component; else, for the first axis in the order x, y, z with `min > max`, `Err(MathError::InvalidAabb { axis, min, max })` (LIM-MATH-0010) with that axis's values; else `Ok`, and `min()` and `max()` return the arguments. `min == max` on an axis is valid (a flat box or a point). |
| MATH-R151 | MUST | `Aabb::from_center_half_extents(center, half_extents)`: `Err(NonFiniteInput)` as in MATH-R150; else, for the first axis with a negative half extent, `Err(MathError::NegativeExtent { parameter: "half_extents", value })` (LIM-MATH-0011); else the box with `min = center − half_extents` and `max = center + half_extents`, or `Err(NonFiniteInput { parameter: "half_extents" })` if a result component is infinite. `Aabb::from_points(points)`: `Err(MathError::EmptyPointSet)` (LIM-MATH-0012) for an empty slice; `Err(NonFiniteInput { parameter: "points" })` if any component is NaN or infinite; else the box whose `min` and `max` are the component-wise minimum and maximum over all points. |
| MATH-R152 | MUST | `center()` returns exactly `(min + max) × 0.5` and `half_extents()` exactly `(max − min) × 0.5`. `contains_point(p)` is true exactly when `min.c ≤ p.c` and `p.c ≤ max.c` on every axis `c` (false if a component of `p` is NaN). `a.intersects_aabb(&b)` is true exactly when `a.min.c ≤ b.max.c` and `b.min.c ≤ a.max.c` on every axis. `closest_point(p)` returns `p.max(min).min(max)` (MATH-R026). `a.union(&b)` has `min = a.min.min(b.min)` and `max = a.max.max(b.max)`. These use comparisons only and have no rounding error. |
| MATH-R153 | MUST | `transformed(affine)` returns the box whose `min` and `max` are the component-wise minimum and maximum of `affine.transform_point3(corner)` over the eight corners, or `Err(NonFiniteInput { parameter: "affine" })` if any transformed component is NaN or infinite. With `Affine3::IDENTITY` it returns the same box. |
| MATH-R155 | MUST | `Sphere::new(center, radius)`: `Err(NonFiniteInput)` naming the first parameter that is or contains NaN or an infinity; else `Err(MathError::NegativeExtent { parameter: "radius", value: radius })` (LIM-MATH-0011) if `radius < 0`; else `Ok`. A radius of 0 is valid. |
| MATH-R156 | MUST | `Sphere::contains_point(p)` returns `(p − center).length_squared() ≤ radius × radius`. `a.intersects_sphere(&b)` returns `(a.center − b.center).length_squared() ≤ (a.radius + b.radius) × (a.radius + b.radius)`. `s.intersects_aabb(&b)` returns `(s.center − b.closest_point(s.center)).length_squared() ≤ s.radius × s.radius`. Each has the value of the expression as written (exact expression, §4). |
| MATH-R157 | MUST | For shapes from D-geo, `Sphere::contains_point`, `Sphere::intersects_sphere` and `Sphere::intersects_aabb` return the answer of the `f64` reference in every stable case (§4). |
| MATH-R160 | MUST | `Plane::from_normal_and_point(normal, point)`: `Err(NonFiniteInput)` naming the first parameter with a NaN or infinite component; else `n = normal.normalize()`, with its error returned with `parameter: "normal"`; else, with `s = normal.length_squared()`, `Err(MathError::VectorTooShort { parameter: "normal", length_squared: s })` (LIM-MATH-0015) if `s < f32::MIN_POSITIVE` (2^−126, 1.1754944e-38; `s` is then a subnormal number); else the plane with normal `n` and `d = −n.dot(point)`, or `Err(NonFiniteInput { parameter: "point" })` if `d` is infinite. `Plane::from_points(a, b, c)`: `Err(NonFiniteInput)` as above; else, with `m = (b − a).cross(c − a)`, `n = m.normalize()`, and if that fails or `m.length_squared() < f32::MIN_POSITIVE`, `Err(MathError::CollinearPoints)` (LIM-MATH-0013); else normal `n` and `d = −n.dot(a)`, with the same check of `d` (`parameter: "a"`). `from_points((0,0,0), (1,0,0), (0,1,0))` has normal `(0, 0, 1)` and `d = 0`. Every plane either function returns as `Ok`, for arguments of any bit pattern, has a finite `d()` and `abs(normal().length() − 1) ≤ 4 × f32::EPSILON` (INV-3). Cases for `from_normal_and_point` with `point = (0, 0, 0)`: the normals `(1e-22, 5e-23, 0)`, `(1e-21, 5e-22, 0)` and `(2^−64, 0, 0)` give `Err(VectorTooShort)`; `(1e-30, 0, 0)` gives `Err(ZeroLengthVector)` (its squared length underflows to zero); `(2^−63, 0, 0)` gives the normal `(1, 0, 0)`. `from_points((0,0,0), (1e-11,0,0), (0,1e-11,0))` gives `Err(CollinearPoints)`. |
| MATH-R161 | MUST | `Plane::signed_distance(p)` returns exactly `normal.dot(p) + d`. For a plane from `from_normal_and_point(n, q)` with `n`, `q`, `p` from D-geo, it is approximately equal at scale 100 to the `f64` value of `(p − q) · n ÷ length(n)`; `signed_distance(q)` is approximately 0 at scale 100. |
| MATH-R165 | MUST | `Ray::new(origin, direction)`: `Err(NonFiniteInput)` naming the first parameter with a NaN or infinite component; else `d = direction.normalize()`, with its error returned with `parameter: "direction"`; else, with `s = direction.length_squared()`, `Err(MathError::VectorTooShort { parameter: "direction", length_squared: s })` (LIM-MATH-0015) if `s < f32::MIN_POSITIVE`; else `Ok`, and `direction()` returns `d`, `origin()` the argument. Every `Ok` ray, for arguments of any bit pattern, has `abs(direction().length() − 1) ≤ 4 × f32::EPSILON` (INV-4). The directions `(1e-22, 5e-23, 0)` and `(1e-21, 5e-22, 0)` give `Err(VectorTooShort)`; `(2^−63, 0, 0)` gives the direction `(1, 0, 0)`. `at(distance)` returns exactly `origin + direction × distance`. |
| MATH-R166 | MUST | `Ray::intersect_plane(plane)`: with `denominator = plane.normal().dot(direction)`, returns `None` if `denominator == 0.0` (the ray is parallel to the plane, also when it lies in it); else, with `t = −plane.signed_distance(origin) ÷ denominator`, `Some(t)` if `t ≥ 0.0` and `None` otherwise. Both sides of the plane are hit. |
| MATH-R167 | MUST | `Ray::intersect_sphere(sphere)` returns `Some(0.0)` if `sphere.contains_point(origin)`; otherwise `Some(t)` for the smallest `t > 0` at which `at(t)` is on the sphere's surface, or `None` if the ray misses the sphere or the sphere is behind the origin. |
| MATH-R168 | MUST | `Ray::intersect_aabb(aabb)` returns `Some(0.0)` if `aabb.contains_point(origin)`; otherwise `Some(t)` for the smallest `t > 0` at which `at(t)` is on the box's surface, or `None`. On an axis where the direction component is `0.0` (either sign), the ray misses if the origin component is outside `min` to `max` on that axis, and that axis otherwise puts no bound on `t`; no NaN from `0 × ∞` decides the result. |
| MATH-R169 | MUST | For rays and shapes from D-geo, in every stable case (§4): `intersect_sphere` and `intersect_aabb` return `Some` exactly when the `f64` reference does, and `intersect_plane` does so when also `abs(denominator) ≥ 0.01`; every returned `t` satisfies `t ≥ 0` and `abs(t − t_ref) ≤ 1e-3 × max(1, t_ref)`. Exactly representable cases with hand-computed results cover: a ray touching a sphere and a box at one point (hit), a ray along a box face (hit), an origin on the surface (`Some(0.0)`), a zero-radius sphere on the ray (hit), a shape behind the origin (`None`), a ray parallel to a plane (`None`). |
| MATH-R170 | MUST | `StateHash` encodings, as consecutive `write_f32` calls: `Aabb` `min` then `max`; `Sphere` `center` then `radius`; `Plane` `normal` then `d`; `Ray` `origin` then `direction`; `Frustum` its six planes in order. |
| MATH-R175 | MUST | `Frustum::from_view_projection(m)`: `Err(NonFiniteInput { parameter: "view_projection" })` if an element is NaN or infinite. Else, with `r0` to `r3` the rows of `m` as 4-vectors, the plane coefficient vectors are left `r3 + r0`, right `r3 − r0`, bottom `r3 + r1`, top `r3 − r1`, near `r2`, far `r3 − r2`. Each `(a, b, c, e)` is divided by `l = length((a, b, c))` to give the plane with normal `(a, b, c) ÷ l` and `d = e ÷ l`. If, with `s = length_squared((a, b, c))`, `s` is NaN, infinite or below `f32::MIN_POSITIVE` (zero included), or `e ÷ l` is NaN or infinite (a sum or difference of two finite rows can overflow), the result is `Err(MathError::DegenerateFrustum { plane })` (LIM-MATH-0014) naming the first such plane in the order above. `planes()` returns the six planes in that order. Every plane of an `Ok` frustum, for a matrix of any bit pattern, has a finite `d()` and `abs(normal().length() − 1) ≤ 4 × f32::EPSILON` (INV-5). A matrix whose rows are `r0 = (1e-22, 5e-23, 0, 0)` (or `(1e-21, 5e-22, 0, 0)`) and `r1 = r2 = r3 = 0` gives `Err(DegenerateFrustum { plane: "left" })`. |
| MATH-R176 | MUST | `Frustum::contains_point(p)` is true exactly when `plane.signed_distance(p) ≥ 0.0` for all six planes. `intersects_sphere(s)` is true exactly when `plane.signed_distance(s.center()) ≥ −s.radius()` for all six planes. `intersects_aabb(b)` is true exactly when, for all six planes, `plane.signed_distance(v) ≥ 0.0` for the corner `v` of `b` that takes `b.max()`'s component on each axis where the plane normal's component is `≥ 0.0` and `b.min()`'s component elsewhere. |
| MATH-R177 | MUST | For a frustum from `Mat4::perspective(…) * Mat4::look_to(…)` with the parameter ranges of MATH-R050, `eye` in D-geo, and a unit `direction` and unit `up` as in MATH-R049: the point `eye + direction × (z_near + z_far) × 0.5` is contained; `eye − direction` is not; `eye + direction × (z_far × 1.01 + 0.1)` is not. No false negatives: for every point `p` of a grid of at least 5 × 5 × 5 points spanning an `Aabb` (or the bounding box of a `Sphere`) from D-geo, if `p` is inside the shape and `contains_point(p)` holds with every signed distance at least 0.01, then `intersects_aabb` (or `intersects_sphere`) is true. |

### Determinism (R190–R199)

| ID | Level | Requirement |
|---|---|---|
| MATH-R190 | MUST | Workload `linear_algebra` in `crates/liminal_math/tests/determinism.rs` (§11) gives the same state hash in two runs in one process and equals its recorded hash literal on every supported platform. |
| MATH-R191 | MUST | Workload `simd_paths` (§11) gives the same state hash in two runs in one process, the same hash when computed through the `reference` items, and equals its recorded hash literal on every supported platform. |
| MATH-R192 | MUST | Workload `geometry_queries` (§11) gives the same state hash in two runs in one process and equals its recorded hash literal on every supported platform. |

### Assignment to implementation tasks

Every requirement is assigned to exactly one task; the IDs are in the tasks' `requirements`
frontmatter.

| Task | Requirements |
|---|---|
| M1-T020 (scalar types, errors, `reference` items) | MATH-R001 to MATH-R007, MATH-R020 to MATH-R026, MATH-R040 to MATH-R049, MATH-R051, MATH-R070 to MATH-R072, MATH-R090 to MATH-R094, MATH-R130, MATH-R190 |
| M1-T021 (`det` and the three operations that call it) | MATH-R050, MATH-R073, MATH-R074, MATH-R100 to MATH-R110 |
| M1-T022 (SIMD paths) | MATH-R131, MATH-R132, MATH-R191 |
| M1-T023 (geometry) | MATH-R150 to MATH-R153, MATH-R155 to MATH-R157, MATH-R160, MATH-R161, MATH-R165 to MATH-R170, MATH-R175 to MATH-R177, MATH-R192 |

Notes on the assignment:
- `Mat4::perspective` (MATH-R050), `Quat::from_axis_angle` (MATH-R073) and `Quat::slerp`
  (MATH-R074) call `det` functions, so they are implemented with `det` in M1-T021, not in M1-T020.
  M1-T020 does not add these three functions (no placeholder bodies). This widens M1-T021's scope
  beyond its task text; the task's "Notes" record it.
- M1-T020 implements all of `MathError` (MATH-R006), including the variants whose producers come
  in M1-T021 and M1-T023.
- M1-T020 implements the four operations of §5.6 by calling the `reference` items (MATH-R130);
  M1-T022 adds the SIMD paths behind the same public items.

## 7. Invariants

- INV-1: An `Aabb` has finite `min` and `max` with `min ≤ max` on every axis. (Private fields;
  every constructor and `union`, `transformed` keep it; `debug_assert!` in the constructors;
  property test over generated inputs.)
- INV-2: A `Sphere` has a finite center and a finite `radius ≥ 0`. (Private fields; constructor.)
- INV-3: A `Plane` has a finite `d` and a normal whose length differs from 1 by at most
  `4 × f32::EPSILON`. (Private fields; every constructor normalizes and rejects a vector to
  normalize whose `length_squared()` is below `f32::MIN_POSITIVE`; property test of MATH-R160
  over arbitrary bit patterns.)
- INV-4: A `Ray` has a finite origin and a direction whose length differs from 1 by at most
  `4 × f32::EPSILON`. (Private fields; the constructor rejects as for INV-3; property test of
  MATH-R165 over arbitrary bit patterns.)
- INV-5: A `Frustum` holds six planes that each satisfy INV-3. (Private fields; the constructor
  rejects as for INV-3 and rejects a non-finite `d`; property test of MATH-R175 over arbitrary
  bit patterns.)
- INV-6: A `det` function never returns a NaN with bits other than `0x7FC0_0000`. (Property test
  of MATH-R100 over arbitrary bit patterns.)
- INV-7: A public item with a SIMD path returns the bits of its `reference` item (MATH-R131).
  (Differential property tests on both architectures; workload `simd_paths`.)

In INV-3 to INV-5 "length" is the public `Vec3::length()` (MATH-R022), as in MATH-R025.

How INV-3 to INV-5 hold for every accepted input (`spec_version` 2, option (a) of M1-T060 item
2): `Vec3::normalize` (MATH-R024) returns `Ok` for every non-zero finite length, and when
`length_squared()` is a subnormal number (lengths from about 2.6e-23 to 1.1e-19) its result is not
a unit vector: `(1e-22, 5e-23, 0)` normalizes to length 0.9956 and `(1e-21, 5e-22, 0)` to length
1 + 1.7e-5. `Plane::from_normal_and_point`, `Plane::from_points`, `Ray::new` and
`Frustum::from_view_projection` therefore reject a vector to normalize whose `length_squared()`
is below `f32::MIN_POSITIVE` (MATH-R160, MATH-R165, MATH-R175). `normalize` itself, MATH-R022,
MATH-R024 and the bit-exact contract are unchanged. Rejected alternatives: (b) rescaling inside
`normalize` before squaring changes the exact expressions of MATH-R022 and MATH-R024 for every
caller; (c) invariants that hold only in an input range leave planes and rays with non-unit
normals in circulation, and every query that assumes a unit normal would need the range as a
precondition. The bound `4 × f32::EPSILON` for accepted inputs is measured, not proven (open
question 1), so the constructors do not `debug_assert!` INV-3 to INV-5: a wrong bound would
turn into a panic, against MATH-R005. The property tests check it instead.

`Vec2`, `Vec3`, `Vec4`, `Mat3`, `Mat4`, `Quat` and `Affine3` have public fields and no invariant:
every bit pattern is a valid value.

## 8. Error codes

`liminal_math` has recoverable errors: input that a constructor or a normalizing or inverting
function cannot use. Messages and help below are normative; `{x}` is the value of context field
`x`, and the backticks shown are part of the text.

| Code | Variant | Meaning (message) | Help | Context |
|---|---|---|---|---|
| LIM-MATH-0001 | `ZeroLengthVector` | `{parameter}` has length zero and cannot be normalized. | pass a non-zero vector; components with absolute value below 1e-19 can underflow to a zero length. Use `normalize_or` to supply a fallback direction. | `parameter` |
| LIM-MATH-0002 | `NonFiniteInput` | `{parameter}` is not finite: it contains NaN or an infinity, or the result computed from it overflows. | check the value for NaN and infinities before the call; components with absolute value above 1.8e19 overflow when squared. | `parameter` |
| LIM-MATH-0003 | `SingularMatrix` | the matrix has determinant `{determinant}` and cannot be inverted. | a matrix with a zero scale component or with linearly dependent columns has no inverse; check the scale passed to `from_scale_rotation_translation`. | `determinant` |
| LIM-MATH-0004 | `InvalidProjection` | projection parameter `{parameter}` has the invalid value `{value}`. | `perspective` needs 0 < fov_y_radians < π, aspect_ratio > 0 and 0 < z_near < z_far; `orthographic` needs left != right, bottom != top and z_near != z_far; all values finite. | `parameter`, `value` |
| LIM-MATH-0005 | `InvalidViewBasis` | `up` is zero or parallel to `direction`, so no view basis exists. | pass an `up` vector that is not parallel to `direction`, for example `Vec3::Z` when looking straight up or down. | none |
| LIM-MATH-0010 | `InvalidAabb` | box minimum `{min}` is greater than maximum `{max}` on axis `{axis}`. | pass `min <= max` on every axis, or build the box with `Aabb::from_points`. | `axis`, `min`, `max` |
| LIM-MATH-0011 | `NegativeExtent` | `{parameter}` has the negative value `{value}`. | pass a radius or half extent of zero or more. | `parameter`, `value` |
| LIM-MATH-0012 | `EmptyPointSet` | cannot build a bounding box from zero points. | pass at least one point. | none |
| LIM-MATH-0013 | `CollinearPoints` | the three points lie on one line, coincide or span too small a triangle, so they define no plane. | pass three points that form a triangle with non-zero area; the cross product of two of its edges needs a length of at least 1.1e-19 (two perpendicular edges of length 1e-9 are enough). | none |
| LIM-MATH-0014 | `DegenerateFrustum` | the `{plane}` plane of the view-projection matrix has a normal that is zero, shorter than 1.1e-19 or not finite, or a distance that is not finite. | pass the product `projection * view` of matrices from `Mat4::perspective` or `Mat4::orthographic` and `Mat4::look_to`. | `plane` |
| LIM-MATH-0015 | `VectorTooShort` | `{parameter}` has the squared length `{length_squared}`, which is below 1.1754944e-38, and cannot be normalized to unit length. | pass a longer vector; a length of at least 1.1e-19 is enough. Below that the squared length is a subnormal `f32`, too coarse to give a unit vector. | `parameter`, `length_squared` |

Each message ends with `.`. `parameter`, `axis` and `plane` are `Text` context values; `value`,
`min`, `max`, `determinant` and `length_squared` are `Float` (`f32` widened exactly; NaN and
infinities render as `null` in JSON, CORE-R010). Codes 0006 to 0009 and 0016 onward are unassigned; new codes take the
next free number in their group (0001–0009 linear algebra, 0010–0019 geometry).

## 9. Performance budgets

None yet: set at `stable` from baselines. No number is set before its measurement
(PERFORMANCE.md). The scenarios below are measured on the reference machine with `liminal_benches`
(ADR-0023); each SIMD scenario measures the public item and its `reference` item, and the pair is
the evidence ADR-0018 requires for keeping the `unsafe` path on aarch64.

| Scenario | Budget | Machine | Source of number |
|---|---|---|---|
| 1,000,000 × `Mat4 * Mat4`, public item and `reference::mat4_mul` | none yet | Reference | Baseline `ci/perf/<machine-id>/liminal_math.json` from M1-T022 |
| 1,000,000 × `Mat4 * Vec4`, public item and `reference::mat4_mul_vec4` | none yet | Reference | Same baseline (M1-T022) |
| 1,000,000 × `Quat * Quat`, public item and `reference::quat_mul` | none yet | Reference | Same baseline (M1-T022) |
| `Affine3::transform_points` over 100,000 points, public item and `reference::affine3_transform_points` | none yet | Reference | Same baseline (M1-T022) |
| 1,000,000 calls each of `det::sin`, `det::cos`, `det::exp`, `det::ln`, `det::powf`, `det::atan2` on seeded inputs | none yet | Reference | Same baseline (M1-T022, the first `liminal_math` task that depends on the harness; noted in that task) |
| The `glam` comparison workloads (defined by M1-T051) | none (parity not required in M1) | Reference | `ci/perf/competitive/liminal_math-<date>.json` from M1-T051; summary added here by that task |

x86_64 has no wall-clock machine (PERFORMANCE.md "Machines"). The evidence for the SSE2 paths is
the instruction count per iteration of the same cases on the CI Linux runner (ADR-0023 item 4):
open question 3. Geometry scenarios are added with the `stable` promotion, when a task for it
exists; none is listed before that.

## 10. Test plan

| Area | Test kinds | Oracle |
|---|---|---|
| Layout, conventions, constructors (R001–R003, R020, R040) | unit, doc test | Hand-written values; `size_of`, `align_of`, `offset_of` |
| Operation set (R004), SIMD selection (R132) | integration (reads the clippy configuration; scans `src/`) | The lists in the requirements |
| No panics (R005) | property (arbitrary bit patterns for every public function; slices of length 0 to 67) | Completion without panic; `Result` variants as specified |
| `MathError` (R006) | unit, snapshot (`crates/liminal_math/tests/snapshots/math_errors.txt`: Display and JSON of one value of every variant) | Approved snapshot; `liminal_core::error::validate_error` |
| `StateHash` (R007, R170) | unit | Hash of the component list written with `write_f32` by hand |
| Exact expressions (R021–R023, R026, R041–R043, R047, R048, R071, R072, R090, R091) | unit (exactly representable inputs, hand-computed results; inputs chosen so that a different association order gives different bits), property | Hand-computed values; for association order, literals computed once by hand from the expression and recorded with the derivation |
| Algebraic properties (R023, R025, R044–R046, R049–R051, R072–R074, R092, R093) | property (proptest over D-vec, D-unit, D-trs; arbitrary bit patterns for the antisymmetry clause of R023, arbitrary finite `v` for the identity clause of R072), unit (R023: `a = b = (0, 3e19, 3e19)`) | The identities and tolerances of the requirements; `f64` reference in `tests/common/` where the requirement says "evaluated in `f64`" |
| Validation errors (R024, R045, R049–R051, R073, R150, R151, R155, R160, R165, R175) | unit (one case per error and per parameter position). For R160, R165 and R175 also the too-short vectors `(1e-22, 5e-23, 0)`, `(1e-21, 5e-22, 0)` and `(2^−64, 0, 0)` (rejected: LIM-MATH-0015, or LIM-MATH-0014 as the coefficients of a frustum plane), `(1e-30, 0, 0)` (LIM-MATH-0001) and the boundary `(2^−63, 0, 0)` (accepted, result `(1, 0, 0)`); for R160 the triangle `(0,0,0), (1e-11,0,0), (0,1e-11,0)` (LIM-MATH-0013) | Hand-written expected `MathError` values |
| Geometry invariants INV-3 to INV-5 (R160, R165, R175) | property (arguments of arbitrary bit patterns, and vectors scaled so that `length_squared()` is within a factor 16 of `f32::MIN_POSITIVE` on either side, with component ratios down to 2^−12) | Every `Ok` value has a finite `d` and `abs(length − 1) ≤ 4 × f32::EPSILON` for its normal or direction |
| `det` special values (R101) | unit (every row of §5.5) | The table of §5.5 |
| `det` accuracy (R102, R104–R109) | differential, three input sets per function: (a) fixed: every special value, ±0, ±1, powers of two, the `f32` nearest to k × π/2 for k in 1 to 1000 and their neighbours, domain bounds and their neighbours, subnormal bounds; (b) 1,000,000 seeded inputs (seeded pairs for `atan2` and `powf`) uniform over the bit patterns of the domain, run in `cargo xtask ci`; (c) exhaustive over every `f32` in the domain (one-argument functions) or 100,000,000 seeded pairs, in tests marked `#[ignore = "M1-T021: exhaustive det sweep"]`, run once by M1-T021 and again whenever a `det` function changes | Rust standard library `f64` function on the widened input (§4 "ULP error") |
| `det` symmetry and purity (R100, R103) | property (arbitrary bit patterns), determinism | Bit equality; canonical NaN bits |
| `reference` items (R130) | unit | Hand-computed exact results, as for exact expressions |
| SIMD paths (R131) | differential property tests named in §5.6, run on aarch64 (reference machine, macOS CI) and x86_64 (Windows and Linux CI) | `liminal_math::reference` items; bit comparison with the NaN rule of §4 |
| `Aabb`, `Sphere`, `Plane`, `Ray`, `Frustum` (R150–R177) | unit (touching, degenerate, parallel, origin inside, zero radius, flat box), property (differential over D-geo) | Hand-computed values; `f64` reference implementation in `tests/common/` with the stable-case rule of §4; brute-force point grid for MATH-R177 |
| Determinism (R110, R190–R192) | determinism (`tests/determinism.rs`, ADR-0024) | Run twice in-process; recorded hash literals |
| `unsafe` (SIMD modules) | Miri (`cargo +nightly miri test -p liminal_math`, from M1-T018) | Undefined-behaviour detection. Assumption, unverified: Miri supports the SSE2 intrinsics used and not all NEON intrinsics. M1-T022 lists here every test Miri cannot run, with the reason (ADR-0018) |

Measured maximum `det` errors are recorded here by M1-T021 (AC of that task); the table is empty
until then.

| Function | Domain | Maximum ULP error, set (b) | Maximum ULP error, set (c) | Measured by |
|---|---|---|---|---|
| (none yet) | | | | M1-T021 |

Every requirement's verifying test carries `/// Verifies: MATH-Rnnn`. Primary entry points with
`# Examples`: `Vec3`, `Mat4` (a model-view-projection product applied to a point), `Quat`
(`from_axis_angle` and rotating a vector), `Affine3`, `det::sin_cos`, `Aabb`, `Ray::intersect_aabb`,
`Frustum::from_view_projection`, `MathError` (matching on a variant and printing its code).

Tolerances are part of the contract. If a correct implementation exceeds one, the spec is changed
explicitly (WORKFLOW.md §6); the test is not weakened on its own.

## 11. Determinism

`liminal_math` is `deterministic = true` (ADR-0009):

- It reads no clock, no environment and no randomness, has no global or thread-local state and no
  interior mutability. Every function is a pure function of its arguments.
- Every result is computed with the IEEE 754 operations of MATH-R004. Rust does not contract
  `a × b + c` into a fused multiply-add and evaluates `f32` and `f64` arithmetic without extended
  precision on the supported targets (x86_64 with SSE2, aarch64), so a fixed sequence of these
  operations gives the same non-NaN bits on every supported platform.
- The transcendental functions are the crate's own (`det`), never the platform math library.
- SIMD paths return the bits of the scalar reference (MATH-R131), so results do not depend on the
  architecture or on whether a SIMD path exists.
- NaN sign and payload are not deterministic for arithmetic results (§4 "NaN bits"). `StateHash`
  hashes `to_bits()` without canonicalization (CORE-R053), so a NaN in hashed simulation state can
  break hash equality; callers keep NaN out of simulation state. `det` returns canonical NaN.
- The determinism lint configuration of CODE_STYLE.md applies, extended by MATH-R004.

**Workloads of `crates/liminal_math/tests/determinism.rs`** (ADR-0024). Each workload runs at
least twice in the same process, feeds its results into a `liminal_core::hash::StateHasher`, and
compares the final hashes with each other and with a hash literal recorded in the test. Inputs
come from `liminal_core::rng::Rng` with fixed seeds and are kept in ranges that produce no NaN.
Exact parameters are fixed in the test; changing them changes the literal and is an oracle change
(AGENTS.md rule 5).

| Workload | What it does | Cross-platform bit-identity claimed |
|---|---|---|
| `linear_algebra` (MATH-R190) | 10,000 rounds over seeded values from D-vec, D-unit and D-trs: every operation of §5.1 to §5.4 (including `normalize`, `inverse`, `look_to`, `orthographic`) except the four of §5.6 and the three that call `det`; every result hashed | Yes (IEEE 754 operations of MATH-R004 only) |
| `det_functions` (MATH-R110) | Every `det` function over input set (a) of §10 and 10,000 seeded inputs, every result hashed by bits (NaN results are canonical); then 10,000 seeded calls each of `Quat::from_axis_angle`, `Quat::slerp` and `Mat4::perspective` | Yes (MATH-R100) |
| `simd_paths` (MATH-R191) | 10,000 seeded `Mat4 * Mat4`, `Mat4 * Vec4`, `Quat * Quat` products, chained so results feed later inputs and renormalized to stay finite, and `Affine3::transform_points` over slices of 0 to 67 points; computed once through the public items and once through the `reference` items | Yes (MATH-R131: the same literal on SSE2, NEON and scalar) |
| `geometry_queries` (MATH-R192) | 10,000 seeded shapes and rays from D-geo and frusta with the ranges of MATH-R050: every query of §5.7; booleans, `Option` distances (by bits) and error codes hashed | Yes (operations of MATH-R004 only) |

Cross-platform bit-identity has **not been verified yet**: it is claimed for all four workloads and
verified when each workload's task (M1-T020 to M1-T023) passes on the three CI operating systems
(macOS arm64 with NEON, Windows and Linux x86_64 with SSE2).

## 12. Open questions

1. **Tolerances come from a prototype.** The tolerances of §4 and §6 (`1e-5 × max(1, s)`,
   `4 × f32::EPSILON`, `1e-4` for inverses, `1e-3 × max(1, t)` for ray distances, the stable-case
   margin 0.01) were chosen from a throwaway scalar prototype run by M1-T019 on aarch64 over
   2,000,000 seeded cases per property (maximum observed errors and the factors of headroom are
   in the journal entry `2026-11-10-M1-T019.md`), not from the implementation. MATH-R161,
   MATH-R049 to MATH-R051, MATH-R044, MATH-R073, MATH-R092 and MATH-R177 were not prototyped; their
   tolerances are estimates. Added in `spec_version` 2 by M1-T060, from a second throwaway
   prototype (journal entry `2026-11-10-M1-T060.md`), also not engine code: the bound
   `4 × f32::EPSILON` of INV-3 to INV-5 for every vector with `length_squared()` from
   `f32::MIN_POSITIVE` up (maximum observed `1.5 × f32::EPSILON`), and the varied shape of §4
   "Stable case" for MATH-R157 (the receiver sphere). Which shape the M1-T019 prototype varied for
   its sphere–sphere and sphere–box figures is not recorded. Resolve: M1-T020 and M1-T023 record
   the maximum observed error of each property in their journals and change a tolerance by a
   `spec_version` bump if needed.
2. **`det::sin`, `cos`, `tan` above 2^20.** Accuracy is required only for `abs(x) ≤ 2^20`; larger
   arguments need an argument reduction with more than 53 bits of π. Results there are defined
   (same bits everywhere, in range) but not accurate. Resolve: M1-T021 either meets the 1 ULP
   bound for all finite inputs and widens MATH-R102 and MATH-R104, or leaves it and records the
   measured error above 2^20.
3. **Benchmark evidence for the x86_64 SIMD paths.** ADR-0018 requires a benchmark before `unsafe`
   is kept for speed. The reference machine is aarch64; x86_64 exists only on CI, where only
   instruction counts are trusted (ADR-0023), and the instruction-count job (M1-T033) depends on
   M1-T022. Resolve in M1-T022: either measure the x86_64 cases with the `k`/`2k` cachegrind
   method in a CI run of its branch and record the counts, or ship the x86_64 paths in M1-T033.
   The decision is recorded in M1-T022's handoff.
4. **Miri and intrinsics.** Which SSE2 and NEON intrinsics Miri can run is unverified. Resolve in
   M1-T022 (exceptions listed in §10).
5. **Clip-space conventions and the renderer.** Depth 0 to 1 and y up in clip space match Metal;
   Vulkan's clip-space y points down. Whether the renderer flips y in the projection, the
   viewport or the shader, and whether it uses reversed depth or an infinite far plane, is decided
   by the `liminal_gpu` / `liminal_render` specs (M3, M4). New projection functions are added
   here by a `spec_version` bump; the existing ones keep their definition.
6. **Operations left out.** Euler angles, `Quat` from `Mat3`, `Quat` to axis and angle, `nlerp`,
   rotation between two vectors, `Vec2` perpendicular dot product, component-wise `clamp`,
   integer vectors (`IVec3`, voxel coordinates) and `f64` types (large worlds). Each is added by
   a `spec_version` bump when a dependent crate's spec needs it: `liminal_scene` (M4),
   `liminal_voxel` (M5 kickoff for integer vectors), `liminal_anim` (M8).
7. **Reflection of math types.** `liminal_math` cannot depend on `liminal_reflect`
   (`same_layer_deps = ["liminal_core"]`), yet components containing `Vec3` or `Quat` must be
   reflected and serialized. Resolve in the `liminal_reflect` spec (M1-T030): either
   `liminal_reflect` depends on `liminal_math` and implements reflection for its types, or
   `architecture.toml` changes.
8. **Names of `reference` items.** PERFORMANCE.md rule 1 says a reference item has the same name
   and signature as its optimized counterpart. Three counterparts here are operators. This spec
   uses `mat4_mul`, `mat4_mul_vec4`, `quat_mul` (the PERFORMANCE.md example is
   `liminal_math::reference::mat4_mul`). Resolve: the review of M1-T019 accepts this reading or
   asks for named methods; PERFORMANCE.md may need one clarifying sentence.
9. **Conventions without an ADR.** Handedness, axes, matrix layout and the angle unit (§4) are
   engine-wide decisions taken in this spec, because M1-T019's scope assigns them to it and no
   ADR covers them. Resolve: the spec review of M1-T019; if the owner or the reviewer wants them
   in an ADR, it is written before the M3 kickoff (first renderer and platform specs).
10. **CODE_STYLE.md ban list is incomplete.** It bans `sin`, `cos`, `tan`, `powf`, `exp`, `ln`
    but not the other platform-dependent functions listed in MATH-R004 (`asin`, `atan2`, `powi`,
    `hypot`, `mul_add`, …). `liminal_math` bans them for itself; other deterministic crates
    would still be allowed to call them. Resolve in M1-T011 (still `todo`) or a follow-up to it.
11. **Batch operations.** Only `Affine3::transform_points` works on slices. Whether
    `headless_boids` (M1-T050) or culling in `liminal_render` needs more batch operations
    (`Frustum` against many boxes, structure-of-arrays vectors) is unknown before measurements.
    Resolve: M1-T050's handoff, then a `spec_version` bump.
12. **More `det` functions.** `exp2`, `log2`, `log10`, hyperbolic functions and `f64` versions are
    not provided. Resolve when a deterministic crate needs one (`liminal_procgen`, M5;
    `liminal_physics`, M6).

## 13. Maturity

Current: `planned`. Next target: `usable` (via `experimental`), evidence prepared by M1-T023 and
checked by the session that reviews it.

- [ ] Spec follows `docs/specs/_TEMPLATE.md` and passed a spec review (REVIEW.md §2) (evidence:
      review of M1-T019).
- [ ] Compiles, all lints pass, determinism lint configuration accepted by `check-arch`,
      registered in `ci/coverage.toml` (evidence: M1-T020).
- [ ] All MUST requirements traced: `cargo xtask trace --crate liminal_math` with
      `maturity: usable` applied on a scratch copy reports every MUST tested (evidence: M1-T023).
- [ ] Line coverage ≥ 85 % (layer 0 pure logic, TESTING.md) (evidence: M1-T023).
- [ ] Every public item documented; the primary entry points of §10 have runnable `# Examples`.
- [ ] Integration tests in `crates/liminal_math/tests/` use only the public API.
- [ ] Miri passes on the test suite; every test Miri cannot run is listed in §10 with a reason;
      every `unsafe` block has a `// SAFETY:` comment checked by a reviewer (evidence: M1-T022,
      M1-T023).
- [ ] Every kept SIMD path has a differential test passing on aarch64 and x86_64 and benchmark
      evidence (§9, open question 3) (evidence: M1-T022).
- [ ] `tests/determinism.rs` with the four workloads of §11 and recorded literals, passing on the
      three CI operating systems.
- [ ] Measured `det` errors recorded in §10 (evidence: M1-T021).
- [ ] At least one example or dependent crate uses it (expected: `headless_boids`, M1-T050, or
      `liminal_ecs` tests; if neither exists when M1-T023 finishes, M1-T023 adds a runnable
      example under `crates/liminal_math/examples/`).

## 14. History

| spec_version | Date | Change | By task |
|---|---|---|---|
| 1 | 2026-11-10 | Initial spec | M1-T019 |
| 2 | 2026-11-10 | MATH-R023: antisymmetry stated per component as "both NaN or equal under `==`", for every bit pattern (was `==` for all finite inputs, false when products overflow). MATH-R160, MATH-R165, MATH-R175, §7: the plane, ray and frustum constructors reject a vector to normalize whose `length_squared()` is below `f32::MIN_POSITIVE`, so INV-3 to INV-5 hold for every accepted input; new error code LIM-MATH-0015 `VectorTooShort`; messages of LIM-MATH-0013 and LIM-MATH-0014 widened; `Frustum::from_view_projection` also rejects a non-finite plane `d`. MATH-R004: conversions between binary32 and binary64 allowed. MATH-R072: identity clause for finite `v`. §4 "Stable case": names the varied shape. No requirement ID added, removed or renumbered | M1-T060 |
