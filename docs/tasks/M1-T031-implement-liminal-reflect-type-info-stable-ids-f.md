---
id: M1-T031
title: Implement liminal_reflect — type info, stable IDs, field access
milestone: M1
status: done
crates: [liminal_reflect]
depends_on: [M1-T030, M1-T014]
requirements: [REFL-R001, REFL-R002, REFL-R003, REFL-R004, REFL-R005, REFL-R020, REFL-R021, REFL-R040, REFL-R041, REFL-R042, REFL-R043, REFL-R044, REFL-R045, REFL-R060, REFL-R061, REFL-R062, REFL-R063, REFL-R064, REFL-R065, REFL-R080, REFL-R100, REFL-R101, REFL-R102, REFL-R103, REFL-R104, REFL-R140]
---

# M1-T031: Implement `liminal_reflect`: type info, stable IDs, field access

## Goal

Types can describe themselves at runtime and be read and written by field name, with hand-written impls.

## Context

- `docs/specs/liminal_reflect.md`.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Crate, type descriptors, stable type IDs, field access, hand-written impls for primitives and `liminal_math` types if the spec asks for them.

**Out:** The derive macro: next task.

## Acceptance criteria

- [x] The crate exists at the path in `architecture.toml`, is a workspace member, has
      `#![forbid(unsafe_code)]` if `unsafe = "forbid"`, and has a floor in `ci/coverage.toml`.
- [x] The crate's clippy determinism configuration is in place.
- [x] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [x] New public items are documented; primary entry points have runnable `# Examples`.
- [x] `cargo xtask ci` passes with no failed or skipped step.

## Test requirements

Requirements in frontmatter; property tests for get/set round trips.

## Notes

- 2026-11-10 (M1-T057): the spec is at `spec_version` 2. New requirement REFL-R065
  (`TypeRegistry` `Debug` without `TypeId`) is in this task's `requirements`. REFL-R043,
  REFL-R080 and REFL-R100 now ask for tests with deliberately wrong hand-written impls
  (`field_at(1)` / `get(1)` returning `None` below the count); REFL-R061 and REFL-R062 fix the
  check order; the §5.2 example changed (the `spec_version` 1 form did not compile).

## Handoff

2026-11-10 (implementation sub-agent): done, set to `review`.

**Done**
- Crate `crates/liminal_reflect` (workspace member, `#![forbid(unsafe_code)]`,
  `extern crate self as liminal_reflect;`), modules `info`, `value` (`value/access.rs`,
  `value/impls.rs`, `value/validate.rs`), `registry`, `hash`, `error`, per spec §5 except the
  derive re-export (M1-T032). Dependency `liminal_core` only; dev-dependency `proptest`.
- `crates/liminal_reflect/clippy.toml`: the determinism `disallowed-methods` list (copy of
  `liminal_core`'s); `check-arch` accepts it. Coverage floor `liminal_reflect = 98.8` in
  `ci/coverage.toml` (measured 98.84 %).
- Tests in `crates/liminal_reflect/tests/`: `info.rs` (R001–R005, R020), `access.rs`
  (R040–R045), `registry.rs` (R060–R063, R065), `validate.rs` (R021, R080), `hashing.rs`
  (R100–R104), `errors.rs` (R140) with the hand-written snapshot
  `tests/snapshots/reflect_errors.txt`. Hand-written test types in `tests/common/types.rs`
  (`Sample` = `refl_test::Sample`, `Pair`, `Marker`, `Mode`, and `Fake`, a value whose
  descriptor and views the test sets, for the deliberately wrong impls).
- `# Examples` on `Reflect` (hand-written impl, the §5.2 behaviour), `Typed`, `StableTypeId`,
  `TypeRegistry`, `validate_reflect`, `reflect_state_hash`, `write_reflect`, `field`,
  `downcast_ref`, and the crate root.

**Not covered by a `Verifies:` test, by the spec's own wording**
- REFL-R064 (SHOULD, "Checked by review"): `register`, `get`, `get_by_path` use `BTreeMap`
  lookups only (`src/registry.rs`).

**Decisions a reviewer should check**
- `TypeRegistry::register` returns the stored descriptor's `id` (checked equal to
  `of_path(type_path)`), not a second call of `T::stable_type_id()`. The two are equal for every
  impl that meets REFL-R002.
- `<dyn Reflect>::field_mut` builds the value's `TypeInfo` once on the success path: the
  error for an impl whose `field_at_mut(i)` returns `None` needs the type path, and the borrow
  checker (NLL, no Polonius) rejects using `self` in the `None` branch while the `Some` branch
  returns a borrow of it. Error paths build it once too.
- `downcast_mut` checks `Any::is::<T>()` first and then downcasts with
  `expect("invariant: …")` (same borrow-checker reason).
- The `expected`/`actual` texts of LIM-REFL-0005 beyond the spec's examples are chosen here
  (`"field 1 named y"`, `"variant index below 2"`, `"array of length 4"`,
  `"field x of type `u64` (ID …)"`, …); see `src/value/validate.rs` and `tests/validate.rs`.
- REFL-R005 literals were computed with `crates/liminal_core/tests/reference/kat.py xxh64`
  (independent Python XXH64); `xxhsum` is not installed on this machine. Verified on macOS
  arm64 only; Windows and Linux are checked by GitHub CI.

**Traps**
- With `liminal_reflect::value::ReflectList` in scope, `vec.get(i)` on a `Vec<T: Typed>`
  resolves to `ReflectList::get` (returns `Option<&dyn Reflect>`), not the slice method. Use
  `vec.as_slice().get(i)`. Hit in `tests/common/types.rs`; the derive (M1-T032) should
  generate code that does not depend on method resolution of `get`/`len`.
- `StableTypeId`'s `Debug` contains the text `TypeId`; the R065 test strips `StableTypeId(`
  before looking for `TypeId`.

**Next:** review (REVIEW.md). Then M1-T032 (derive): it switches the `Reflect` doc example to
`#[derive(Reflect)]` (spec §5.2), adds the re-export and changes the hand-written test types'
`type_path()` as REFLD-R033 says. The spec's maturity checklist item "Compiles, all lints
pass, … registered in `ci/coverage.toml`" is ticked with this task as evidence; `maturity`
stays `planned` (promotion is the reviewer's call, MATURITY.md).

## Review

2026-11-10, review sub-agent of the parent session on `session/2026-11-10-m1-t020-t031-review`
(ADR-0026, ADR-0029, ADR-0030; fresh context, did no work on this task). Rules as on
`origin/main` `115b556`. Outcome: **done**, no blocking finding.

Checked:
- Acceptance criteria: crate at `crates/liminal_reflect`, workspace member,
  `#![forbid(unsafe_code)]`, floor `liminal_reflect = 98.8` in `ci/coverage.toml`;
  `clippy.toml` equals `liminal_core`'s apart from the crate name in the comment; every
  requirement of the frontmatter except REFL-R064 has a `Verifies:` test that checks it (REFL-R064
  is "checked by review" in the spec: `register`, `get`, `get_by_path` use `BTreeMap` lookups only,
  confirmed in `src/registry.rs`); public items documented, `# Examples` on the §10 entry points
  (the `Reflect` example is hand-written until M1-T032 adds the derive).
- `cargo xtask ci` on the task branch with `origin/main` `115b556` merged in: pass, 12 of 12
  steps, none skipped, macOS 27.2 arm64. GitHub CI on the task branch (run 42909152699): success.
- REFL-R005 literals re-computed independently with
  `python3 crates/liminal_core/tests/reference/kat.py xxh64 …`: all eight match. `xxhsum` is not
  installed here either.
- Code against spec §5 and §6 line by line: `info`, `value` (access, impls, validate),
  `registry`, `hash`, `error`; snapshot `tests/snapshots/reflect_errors.txt` against §8 (codes,
  messages, help, context names). Lookup order and the wrong-impl paths of REFL-R043, the check
  order of REFL-R061/R062, the walk order and `at` of REFL-R080, the byte layout of REFL-R100.
- Mutations considered (each caught by a named test): variant index written as `u8`
  (`struct_enum_list_and_primitive_bytes`); highest instead of lowest matching index
  (`field_by_name_on_a_wrong_impl_returns_no_such_field`); empty `at` instead of `(root)` and
  child before parent ID check (`tests/validate.rs`); collision check before descriptor checks
  (`two_rust_types_with_one_path_collide_in_both_orders`); `TypeId` in `Debug`
  (`debug_prints_the_descriptors_only`); `downcast` by stable ID
  (`downcasts_check_the_concrete_type`).
- Determinism: no clock, environment, OS randomness, `HashMap`; `TypeId` used only for equality.
  No `unsafe`. No new dependency (`liminal_core`; dev `proptest`, both in `architecture.toml`).
- Session boundaries: the implementation sub-agent is row 11 of
  `docs/journal/2026-11-10-session-m1-t014-review.md` (spawned 22:36, cutoff 22:40); it did not
  review the task.

Decisions of the handoff: `register` returning the stored descriptor's `id` differs from the
REFL-R060 wording `Ok(T::stable_type_id())` only for an impl that breaks REFL-R002
(`type_path()` and `type_info().type_path` disagree); there the spec's value would name an ID
under which nothing is stored. Treated as a follow-up (spec wording), not B5. The other
decisions (`field_mut` builds a `TypeInfo` on success, `downcast_mut` with
`expect("invariant: …")`, the chosen `expected`/`actual` texts) match the spec.

Backlog entries: 4 in `docs/BACKLOG.md` under "M1-T031 — review 2026-11-10". Follow-up task:
none (no `todo` or `in-progress` task implements the affected requirements). Maturity stays
`planned`.
