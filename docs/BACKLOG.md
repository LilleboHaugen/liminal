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

- `docs/specs/liminal_core.md` §5.1: `ErrorContext` derives `Default`, and no requirement
  states the value (the empty context). `ErrorContext::default()` is already tested as empty;
  add the statement to CORE-R007 when that row is next edited, or leave it. (The `Tick` and
  `TickDuration` part of this entry was fixed by M1-T014: CORE-R114, `spec_version` 4.)
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

## M1-T034 — review 2026-11-10

- `docs/specs/liminal_jobs.md` §10 (JOBS-R024, JOBS-R064): a process-wide allocation counter
  with "one lock" does not isolate tests in one libtest binary: the harness thread allocates
  when it reports a finished test, while the next test is already counting. Require one
  `#[test]` per allocation binary (as JOBS-R064 words it), or a counter that ignores the
  harness thread.
- `docs/specs/liminal_jobs.md` JOBS-R064: a worker that ran no job during the 10 warm-up calls
  can do its first lazy per-thread initialization in the measured calls. Say that workers
  initialize their per-thread state before `ThreadPool::new` returns, or that the warm-up
  must reach every worker.
- `docs/specs/liminal_jobs.md` JOBS-R087, §12 item 10: with §3 (`graph` forbids `unsafe` and
  is built on `scope`) and §5.1 (`Scope::spawn` allocates once per call), `run_graph`
  allocates per node. Running ready nodes in waves on `join` avoids that but breaks the order
  of JOBS-R085. Record this in open question 10 before the SHOULD becomes a MUST.
- `docs/specs/liminal_jobs.md` JOBS-R082, JOBS-R083: `add_dependency(x, x)` with `x` out of
  range matches both. State that the range check comes first (`DependencyCycle` needs names
  that do not exist).
- `docs/specs/liminal_jobs.md` §3, JOBS-R120, JOBS-R122: the loop functions and `run_graph`
  are "built on `scope`", and the public `scope` records `jobs.scope`. State that they use an
  entry without a span, so that each call records exactly one span.
- `docs/specs/liminal_jobs.md` JOBS-R011: checked in review only. The number of park calls can
  be asserted through the private park function (unit test or the loom model of JOBS-R010).
  JOBS-R004: "the third worker of five" does not say whether `threads` is 5 or 6.
- `docs/specs/liminal_jobs.md` §12 items 4, 5, 7, 12: resolved "at the `stable` promotion",
  which is neither a task nor a milestone (`_TEMPLATE.md` §12). Name the milestone.
- `docs/specs/liminal_jobs.md` frontmatter: `adrs` omits ADR-0024, whose Decision 2
  (`tests/determinism.rs`, two runs in one process) JOBS-R111 and §11 follow.
- `docs/specs/liminal_jobs.md` §4 "Panic rule", JOBS-R100 to JOBS-R104: nothing says what
  holds under `panic = "abort"` (the process aborts; the requirements apply to unwinding
  builds only). §4 "Helping": spans of an unrelated job run by a waiting thread nest under
  that thread's open span; say so for readers of traces.

## M1-T060 — review 2026-11-10

- `docs/specs/liminal_math.md` §10 "Validation errors": the row lists the vectors for MATH-R160,
  MATH-R165 and MATH-R175 together. As frustum plane coefficients `(1e-30, 0, 0)` gives
  LIM-MATH-0014, not LIM-MATH-0001, and the accepted boundary `(2^−63, 0, 0)` needs a matrix
  whose other five planes are valid; MATH-R175 states neither case, and MATH-R165 does not
  list `(2^−64, 0, 0)` and `(1e-30, 0, 0)`. State the cases per constructor.
- `docs/specs/liminal_math.md` MATH-R160: `Plane::from_points` with finite points whose
  difference or edge cross product overflows (edge lengths above about 1.8e19) reports
  LIM-MATH-0013, whose message says the points are collinear or too close. Same class as the
  MATH-R049 entry under "M1-T019 — review 2026-11-10"; decide both together.
- `docs/specs/liminal_math.md` MATH-R073, MATH-R049: `Quat::from_axis_angle` and `Mat4::look_to`
  normalize without the `length_squared() < f32::MIN_POSITIVE` check of MATH-R160, so an axis
  such as `(1e-22, 5e-23, 0)` gives `Ok` with a quaternion of length about 0.9956. No invariant
  covers them and their property domains use unit vectors. Reject as in MATH-R160 or state it.

## M1-T056 — review 2026-11-10

- `docs/specs/liminal_telemetry.md` §2 and §12 item 7: two references to
  `docs/specs/liminal_memory.md` still say "on its branch". The file is on `main` (M1-T024);
  turn them into links (rest of M1-T056 item 9c).
- `docs/specs/xtask.md` "`test-features-off` details", `xtask/src/features_off.rs` (`test_args`
  doc): "`--package` selects one package, so no other workspace member can turn a feature on"
  does not hold for the package's own dependency graph. If a dev-dependency (for example a
  future `liminal_testkit`) depends on the tested crate with the feature enabled, Cargo unifies
  it on again and the `cfg(not(feature))` tests silently do not run. By Cargo's rules; not run.
  State the limit, and check it when `liminal_telemetry` gets dev-dependencies (M1-T028).
- `docs/specs/xtask.md` XTASK-R014, `docs/process/CODE_STYLE.md`: no command builds each
  feature alone (`--no-default-features --features x`), and no task covers it (M1-T056 handoff,
  "For the reviewer" 2). XTASK-R014 also lacks the "revisit at M3" note of XTASK-R012: a crate
  that needs exactly one backend feature would fail with all features off.
- `docs/specs/liminal_telemetry.md` §10 "Ring buffer and its reference": the assumption gives
  "atomics hold one machine word" as the reason no safe ring exists. A 48-byte record can be
  spread over six `AtomicU64`; the obstacle is that `&'static str` names and the log `Box`
  cannot be rebuilt from integers without `unsafe`. Correct the reason; M1-T028's review checks
  the `unsafe` against it (REVIEW.md item 6).
- `docs/specs/liminal_telemetry.md` TELE-R011: "so no frame record holds a span with a
  `start_ns` before its collector's `install`" holds only if no collector is dropped and
  installed on another thread between the guard's identity load and its push. Qualify the
  sentence (same window as for `counter!`, `gauge!`, `log!`), or state it for one thread.
- `docs/tasks/M1-T029-implement-liminal-telemetry-frame-stats-and-expo.md`: title and file
  name say "frame stats", which are M1-T028's since M1-T056 (handoff, "For the reviewer" 5).

## M1-T057 — review 2026-11-10

- `docs/specs/liminal_reflect.md` §3: says the cited CORE requirements were compared with the
  core spec at `spec_version` 1 and 2. `docs/specs/liminal_core.md` is at 3 on `main`; the
  review compared CORE-R050, R053, R054, R056 and R098 with it (identical text). Update the
  sentence when the spec is next edited.
- `docs/specs/liminal_reflect_derive.md` §3: the manifest line
  `proc-macro2 = { workspace = true, … }` assumes a `[workspace.dependencies]` entry that the
  root `Cargo.toml` does not have. M1-T032 adds it or writes a version.
- `docs/specs/liminal_reflect.md` REFL-R080: `expected` / `actual` are fixed only for the
  missing-value case; for the other checks (field count, name, kind, variant index, array
  length, field ID) the strings are examples, while §10 asks for hand-written expected values.
  M1-T031 fixes the strings in the requirement when it writes the tests.

## M1-T014 — review 2026-11-10

- `crates/liminal_core/src/collections/set.rs`: `DetHashSet::insert` hashes and probes the
  value twice (`contains_key`, then `DetHashMap::insert`). Still expected constant time
  (CORE-R075); one probe would halve the cost. Revisit with the `liminal_benches` tasks.
- `crates/liminal_core/examples/seeded_world.rs`: `cargo test` compiles it, but no CI step runs
  it, so its own check (two runs with one seed give one hash) runs only by hand.
- `crates/liminal_core/tests/reference/kat.py`: not run by CI. A literal regenerated without it
  would go unnoticed; consider a CI step (Python 3 is on the hosted runners) when CI next
  changes.

## M1-T037 — review 2026-11-10

- `docs/specs/liminal_ecs.md` ECS-R210 and `docs/specs/liminal_jobs.md` JOBS-R121: if
  `run_parallel` names each `JobGraph` node with the system name, `run_graph` already records a
  span of that name, and an ECS span of its own makes two. M1-T042 picks distinct node names or
  relies on the node span; say which in the spec.
- `docs/specs/liminal_ecs.md` ECS-R210: schedule and system span names (`"fixed_update"`,
  `"integrate"`) do not follow the telemetry name convention (SHOULD, `liminal_telemetry.md`
  §5.1: lowercase, dot-separated, crate area first). Decide whether callers or the ECS prefix them.
- `docs/specs/liminal_ecs.md` §11 rule 3 and ECS-R221: an `Rng` in a registered resource needs
  `Rng: Typed`, which is M1-T062's decision; M1-T043 (`schedule_ticks`) does not depend on
  M1-T062, so its `Rng` resource stays unregistered (not hashed) unless the order changes.
- `docs/specs/liminal_ecs.md` §10 "loom" and §11: the world identity of ECS-R155 most likely
  needs a process-global counter (an atomic of the crate's own); §10 says the crate has no
  shared mutable state of its own. State the exception or the mechanism.
- `docs/specs/liminal_ecs.md` ECS-R191: "equal event buffers" does not say that the sets of
  registered event types must be equal too; one extra registered event type with empty buffers
  changes the hash by ECS-R190 (4). Name it among the conditions.
- `docs/tasks/README.md` has no row for M1-T059, and `depends_on` of
  `docs/tasks/M1-T053-m1-exit-review.md` lists none of M1-T055 to M1-T061 (only M1-T062).

## M1-T061 — review 2026-11-10

- `docs/specs/liminal_reflect_derive.md` REFLD-R003 test (1) rule (d): a one-segment path that is
  not in the list passes, so bare prelude names (`Clone`, `Default`, `Box`, `Into`) in generated
  code pass test (1), and test (2) does not shadow them. Same gap as in `spec_version` 2; M1-T032
  may extend the list or the shadowing test.
- `docs/tasks/README.md`: no row for M1-T061 (task created by the M1-T057 review).
