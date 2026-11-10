# Backlog

Minor review findings that are not worth a task of their own (ADR-0029 Decision 2). A reviewer
appends its non-blocking, non-contract-changing findings here. Findings that change a contract a
pending task implements go into a follow-up task instead (`docs/process/REVIEW.md`).

Rules:
- One heading per review: `## <task ID> — review YYYY-MM-DD`. One list item per finding: what is
  wrong and the file it concerns, in one to three lines.
- An implementation task MAY fix entries in files it changes anyway. It deletes the entry and
  says so in its handoff.
- Milestone kickoff and exit reviews work through the file: each entry becomes a task or is
  deleted, with a one-line reason in that session's journal entry.
- On merge, keep both sides' entries.

Follow-up tasks written before 2026-11-10 (M1-T055 … M1-T058, M2-T001 … M2-T007) stay tasks.

## M1-T013 — review 2026-11-10

- `crates/liminal_core/tests/handles.rs`: no test fails when `HandleMap::new()` uses a slot
  limit other than `u32::MAX` (mutant `u32::MAX - 1` survives). Compare its `Debug` form with
  `with_slot_limit(u32::MAX)`, as the allocator test does.
- `crates/liminal_core/src/error/code.rs`: the `compile_fail` doc test for CORE-R003 passes on
  any compile error, not only the const panic. No stable way to match the message; say so in
  the spec's test plan or add a UI test when a harness for one exists.
- `crates/liminal_core/tests/error_model.proptest-regressions`, `handles.proptest-regressions`:
  four committed seeds whose causes `docs/journal/2026-11-10-M1-T013.md` does not explain (it
  says no regression file was kept). Record what each seed found, or delete seeds that came
  from test bugs.
- `docs/specs/liminal_core.md` §7 INV-1 says `debug_assert!` in `allocate` and `free`;
  `crates/liminal_core/src/handle/allocator.rs` asserts the slot state only in `allocate`
  (`free` has the INV-2 count check). Add the assertion or correct the spec sentence.
- `docs/specs/liminal_core.md` CORE-R012 "the first repeated key" has two readings for keys
  `a, b, b, a`. Code and tests use "the first field whose key occurred earlier" (`b`). State
  that reading in the requirement.

## M1-T055 — review 2026-11-10

- `docs/specs/liminal_core.md` §5.1, §5.6: `ErrorContext`, `Tick` and `TickDuration` derive
  `Default`, and no requirement states the value (empty context; count 0). The derived values
  are the only reasonable ones and `ErrorContext::default()` is already tested as empty; add
  the statement to CORE-R007 / CORE-R111 when those rows are next edited, or leave it.
- `docs/process/DOCUMENTATION.md`, `docs/specs/_TEMPLATE.md`: the spec frontmatter field
  `last_reviewed` has no stated meaning (date of the last spec review, or of the last change?).
  `docs/specs/liminal_core.md` v2 keeps 2026-11-09, the review date of v1. Define the field.

## M1-T019 — review 2026-11-10

- `docs/specs/liminal_math.md` §3, MATH-R004, §12 item 10: written while M1-T011 was `todo`.
  M1-T011 is `done` (mechanism: the crate's own `clippy.toml`, CODE_STYLE.md "Lints"); the
  wider ban list for other deterministic crates has no owner task (M2-T006 covers other
  M1-T011 follow-ups). Update the wording and name a task for item 10.
- `docs/specs/liminal_math.md` MATH-R004: the banned intrinsics exist on one architecture only
  (`_mm_rsqrt_ps` on x86_64, `vrsqrteq_f32` on aarch64). Unverified: whether clippy with
  `-D warnings` rejects a `disallowed-methods` path that does not resolve on the host. M1-T020
  must check this on macOS and on a CI x86_64 runner before relying on the requirement.
- `docs/specs/liminal_math.md` §12 item 7: says the `liminal_reflect` spec (M1-T030) resolves
  reflection of math types. M1-T030 is `done` and left it open (`docs/specs/liminal_reflect.md`
  §12 item 1). Point both items at one resolving task or milestone.
- `docs/specs/liminal_math.md` §12 items 8 and 9: the review accepted the `reference` item
  names (`mat4_mul`, `mat4_mul_vec4`, `quat_mul`, `affine3_transform_points`).
  `docs/process/PERFORMANCE.md` rule 1 ("same name and signature") needs one sentence for
  operator counterparts. Item 9 (an ADR for handedness, axes, matrix layout, angle unit, clip
  depth before the M3 kickoff) is the owner's decision and is still open.
- `docs/specs/liminal_math.md` MATH-R155: does not say that `Sphere::center()` and `radius()`
  return the constructor arguments (MATH-R150 and MATH-R165 say it for `Aabb` and `Ray`).
- `docs/specs/liminal_math.md` MATH-R049: a finite `up` with components near 1e30 makes
  `f.cross(up).normalize()` fail with a non-finite length, reported as LIM-MATH-0005 ("`up`
  is zero or parallel to `direction`"). Report LIM-MATH-0002 for it or widen the message.
- `docs/specs/liminal_math.md` §5.8, §10 "Validation errors": `MathError` derives `PartialEq`
  and holds `f32` fields, so `SingularMatrix { determinant: NaN }` is unequal to itself; the
  planned "hand-written expected `MathError` values" need `matches!` for NaN cases. Say so.
- `docs/specs/liminal_math.md` MATH-R166: an origin on the plane can return `Some(-0.0)`
  (`−0.0 ÷ denominator`), which the `geometry_queries` workload hashes by bits. Deterministic,
  but state it or return `+0.0`.
- `docs/tasks/M1-T020-implement-liminal-math-scalar-vector-matrix-and.md`: scope "Out" says
  geometry and `det` only; the spec also moves `perspective`, `from_axis_angle` and `slerp` to
  M1-T021. Only M1-T021's "Notes" record it; add the same line to M1-T020's "Notes".

## M1-T059 — review 2026-11-10

- `crates/liminal_core/tests/handles.rs`: the five CORE-R041 tests were inserted between the
  doc comment of `operations_on_many_slots_do_not_scan` ("CORE-R040 is a SHOULD …") and that
  function. The paragraph now heads `default_allocator_is_in_the_state_of_new`, and the
  CORE-R040 test has only its `Verifies:` line. Move the paragraph back. Trace output is correct.
