---
id: M1-T060
title: liminal_math spec follow-ups from the M1-T019 review
milestone: M1
status: review
crates: [liminal_math]
depends_on: [M1-T019]
requirements: []
---

# M1-T060: `liminal_math` spec follow-ups from the M1-T019 review

## Goal

`docs/specs/liminal_math.md` contains no requirement that a correct implementation cannot meet
and no invariant that an accepted input breaks, before M1-T020 and M1-T023 implement it.

## Context

- Follow-up task under ADR-0029 Decision 2.3, written by the review of M1-T019 (see that task's
  "Review"). Every item changes a requirement or invariant that M1-T020 or M1-T023 (`todo`)
  implements, so both list this task in `depends_on`.
- `docs/specs/liminal_math.md` v1: §4, MATH-R004, MATH-R005, MATH-R023, MATH-R024, MATH-R025,
  MATH-R072, MATH-R157, MATH-R160, MATH-R165, MATH-R175, §7 INV-3 to INV-5.
- The numbers below were computed by the reviewer with a scratch `rustc -O` program on aarch64
  (`f32` arithmetic as the spec's exact expressions; not engine code, not committed).

## Scope

**In:**
1. **MATH-R023 is false as written.** "For all finite inputs `a.cross(b) == -(b.cross(a))`"
   fails when two products overflow to the same infinity: `a = (0, 3e19, 3e19)`,
   `b = (0, 3e19, 3e19)` gives `x = ∞ − ∞ = NaN` on both sides, and `NaN == NaN` is false. A
   property test over all finite inputs fails for a correct implementation. Restrict the
   domain (for example components with `abs` at most 1e18, or D-vec) or state the equality
   with the §4 "NaN bits" rule.
2. **INV-3, INV-4, INV-5 contradict MATH-R160, MATH-R165, MATH-R175.** The constructors
   normalize with MATH-R024, which returns `Ok(self / l)` for every non-zero finite length.
   MATH-R025 bounds the result's length only for components with `abs` in 1e-10 to 1e10. When
   `length_squared()` is subnormal (lengths from about 2.6e-23 to 1.1e-19) the result is not
   a unit vector: `(1e-22, 5e-23, 0)` normalizes to a vector of length 0.9956,
   `(1e-21, 5e-22, 0)` to length 1 + 1.7e-5; the invariants allow `4 × f32::EPSILON`
   (4.8e-7). So `Plane::from_normal_and_point((1e-22, 5e-23, 0), p)` is `Ok` by MATH-R160 and
   breaks INV-3, and a `debug_assert!` of INV-3 (§7) would panic against MATH-R005. The same
   holds for `Ray::new` (INV-4) and for a plane of `Frustum::from_view_projection` with a
   small coefficient length (INV-5). Decide one and write it into the requirements and §7:
   (a) the three constructors reject a normal or direction whose `length_squared()` is below
   `f32::MIN_POSITIVE` (which error code; LIM-MATH-0001 exists, its message says "length
   zero"); (b) `normalize` rescales before squaring, which changes MATH-R022/R024 and the
   bit-exact contract; or (c) the invariants state the input range in which they hold and
   the `debug_assert!` is dropped. Add the unit cases to the test plan.
3. **MATH-R004 does not list conversion between `f32` and `f64`.** The allowed operations name
   "conversions between integers and floats" only. `det` evaluated in `f64` and rounded once
   (the route the M1-T019 handoff names for the 1.0 ULP bound) needs widening and
   `f64 → f32` rounding (round to nearest, ties to even). Add both, or say why not.
4. **MATH-R072: `Quat::IDENTITY * v == v` has no domain.** With `v = (∞, 1, 2)` the formula
   gives `t = u.cross(v) × 2 = (0, NaN, NaN)` and a NaN result. State "for finite `v`", as
   MATH-R042 and MATH-R071 do.
5. **§4 "Stable case" does not say which shape is grown for two shapes.** MATH-R157 uses it
   for `Sphere::intersects_sphere` and `Sphere::intersects_aabb`; "the shape grown by 0.01"
   can mean the receiver, the argument or both. State it (the journal's prototype numbers
   were measured with one reading; record which).
6. Bump `spec_version` to 2 with a history row; journal entry explains each change.

**Out:** Any other design change to `liminal_math`. Code. Items in `docs/BACKLOG.md` under
"M1-T019 — review 2026-11-10" (they MAY be fixed here in passing; delete the fixed entries).

## Acceptance criteria

- [x] MATH-R023's antisymmetry clause holds for every input in its stated domain, including
      `a = b = (0, 3e19, 3e19)` if that input stays in the domain.
- [x] For every input that `Plane::from_normal_and_point`, `Plane::from_points`, `Ray::new` and
      `Frustum::from_view_projection` accept under the requirements, INV-3 to INV-5 hold as
      written in §7; the spec names the option chosen in item 2 and the test cases
      `(1e-22, 5e-23, 0)` and `(1e-21, 5e-22, 0)`.
- [x] MATH-R004 covers every operation an `f64`-evaluated `det` function needs.
- [x] MATH-R072 states the domain of the identity clause; §4 "Stable case" names the grown and
      shrunk shape for two-shape cases.
- [x] Requirement IDs are unchanged (no reuse, no renumbering); new requirements, if any, are
      assigned to exactly one of M1-T020 to M1-T023 in spec §6 and in that task's
      `requirements` frontmatter.
- [x] `spec_version: 2` with a history row; `cargo xtask ci` passes with no failed or skipped
      step.

## Test requirements

None (documentation task). The review is a spec review of the changed rows (REVIEW.md §2).

## Notes

_None._

## Handoff

2026-11-10, implementation sub-agent (ADR-0026, ADR-0029) on branch
`task/M1-T060-liminal-math-spec-follow-ups`, worktree `liminal-worktrees/M1-T060`. Journal:
`2026-11-10-M1-T060.md`. Status: `review`.

**Done** (`docs/specs/liminal_math.md`, `spec_version` 2; no requirement ID added, removed or
renumbered: 71 before and after, compared by script):

| Item | Change |
|---|---|
| 1, MATH-R023 | Antisymmetry is stated per component: `a.cross(b)` and `-(b.cross(a))` are both NaN or equal under `==`, for every bit pattern. `a = b = (0, 3e19, 3e19)` stays in the domain (both `x` are NaN). "Same bits" (§4 "NaN bits") was not usable: `p − p` is `+0` and `−(p − p)` is `−0`. |
| 2, INV-3 to INV-5 | **Option (a).** `Plane::from_normal_and_point`, `Plane::from_points`, `Ray::new` and `Frustum::from_view_projection` reject a vector to normalize whose `length_squared()` is below `f32::MIN_POSITIVE` (MATH-R160, R165, R175; §7 names the option and why (b) and (c) were rejected). Error: new code **LIM-MATH-0015 `VectorTooShort { parameter, length_squared }`** for `normal` and `direction` (LIM-MATH-0001's "length zero" would be false); `from_points` reports LIM-MATH-0013 and the frustum LIM-MATH-0014, both messages widened. `Vec3::normalize`, MATH-R022, R024, R025 are unchanged. §7 says the constructors do not `debug_assert!` the unit length. Unit cases `(1e-22, 5e-23, 0)`, `(1e-21, 5e-22, 0)`, `(2^−64, 0, 0)`, `(1e-30, 0, 0)`, `(2^−63, 0, 0)` are in MATH-R160, R165, R175 and §10. |
| 2, extra | `Frustum::from_view_projection` accepted a plane with an infinite `d` (`e ÷ l` overflows, or a row sum overflows in the fourth component), which breaks INV-3 "finite `d`" through INV-5. MATH-R175 now rejects it with LIM-MATH-0014. Needed for acceptance criterion 2. |
| 3, MATH-R004 | Adds `f32 → f64` (exact) and `f64 → f32` (`as`, round to nearest, ties to even), states the rounding of integer and float conversions, names `from_bits`. |
| 4, MATH-R072 | `Quat::IDENTITY * v == v` "for every finite `v`". |
| 5, §4 "Stable case" | Table of the varied shape: for MATH-R157 the receiver sphere; for MATH-R169 the shape the ray is tested against. Everything else stays as given. |
| 6 | `spec_version: 2`, history row, journal entry. |

**For M1-T020 and M1-T023:** `MathError` has 11 variants now; M1-T020 implements all of them
(spec §6 note), M1-T023 produces `VectorTooShort`. No `requirements` frontmatter changed.

**Evidence (not engine code):** scratch `rustc -O` program on aarch64, not committed; numbers
in the journal. It checked the new MATH-R023 clause (20,000,000 random bit patterns), the unit
bound for accepted vectors (about 235,000,000 accepted vectors, maximum `1.5 × f32::EPSILON`),
the MATH-R072 identity clause (19,766,818 finite `v`), the five unit cases, and MATH-R157 under
the receiver-varied reading (0 mismatches in about 2,000,000 stable cases per test). The bound
`4 × f32::EPSILON` is measured, not proven.

**Not done / not known:**
- Item 5 asks to record which reading the M1-T019 prototype used. That prototype is not in the
  repository and neither its journal nor its handoff says. The spec states this (§12 item 1)
  and records the reading measured here.
- No entry of `docs/BACKLOG.md` "M1-T019 — review 2026-11-10" was fixed; the file is unchanged.
- Seen, not changed (outside the five items): `Plane::from_points` with finite points near
  `f32::MAX` overflows in `b − a` or the cross product and reports LIM-MATH-0013 ("lie on one
  line …"), the same class as the MATH-R049 backlog entry. `Quat::from_axis_angle` and
  `Mat4::look_to` normalize without the new check; they have no invariant, and their property
  domains use unit vectors.
- `last_reviewed` in the spec frontmatter is unchanged (its meaning is undefined, backlog entry
  under M1-T055).

**Verification:** `cargo xtask ci` on macOS 27.2 (arm64): pass, 11 of 11 steps, none skipped.
Not verified on Windows or Linux: the branch was pushed after this note was written, and the
result of its CI run is not recorded here; the reviewer checks it.

**Traps:** a threshold on `length()` instead of `length_squared()` is not equivalent at the
boundary; the requirement compares `length_squared()` with `f32::MIN_POSITIVE` (2^−126), and
`(2^−63, 0, 0)` is the smallest accepted axis-aligned vector.

## Review

_Empty._
