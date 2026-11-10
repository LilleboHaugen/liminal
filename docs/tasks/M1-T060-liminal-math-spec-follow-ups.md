---
id: M1-T060
title: liminal_math spec follow-ups from the M1-T019 review
milestone: M1
status: todo
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

- [ ] MATH-R023's antisymmetry clause holds for every input in its stated domain, including
      `a = b = (0, 3e19, 3e19)` if that input stays in the domain.
- [ ] For every input that `Plane::from_normal_and_point`, `Plane::from_points`, `Ray::new` and
      `Frustum::from_view_projection` accept under the requirements, INV-3 to INV-5 hold as
      written in §7; the spec names the option chosen in item 2 and the test cases
      `(1e-22, 5e-23, 0)` and `(1e-21, 5e-22, 0)`.
- [ ] MATH-R004 covers every operation an `f64`-evaluated `det` function needs.
- [ ] MATH-R072 states the domain of the identity clause; §4 "Stable case" names the grown and
      shrunk shape for two-shape cases.
- [ ] Requirement IDs are unchanged (no reuse, no renumbering); new requirements, if any, are
      assigned to exactly one of M1-T020 to M1-T023 in spec §6 and in that task's
      `requirements` frontmatter.
- [ ] `spec_version: 2` with a history row; `cargo xtask ci` passes with no failed or skipped
      step.

## Test requirements

None (documentation task). The review is a spec review of the changed rows (REVIEW.md §2).

## Notes

_None._

## Handoff

_Empty._

## Review

_Empty._
