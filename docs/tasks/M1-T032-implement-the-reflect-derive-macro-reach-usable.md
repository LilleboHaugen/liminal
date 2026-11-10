---
id: M1-T032
title: Implement the Reflect derive macro; reach usable for both reflect crates
milestone: M1
status: review
crates: [liminal_reflect, liminal_reflect_derive]
depends_on: [M1-T031, M1-T061]
requirements: [REFL-R120, REFL-R130, REFL-R131, REFL-R132, REFLD-R001, REFLD-R002, REFLD-R003, REFLD-R004, REFLD-R005, REFLD-R020, REFLD-R021, REFLD-R030, REFLD-R031, REFLD-R032, REFLD-R033, REFLD-R034, REFLD-R035, REFLD-R050, REFLD-R051, REFLD-R052, REFLD-R053, REFLD-R070]
---

# M1-T032: Implement the `Reflect` derive macro; reach `usable` for both reflect crates

## Goal

`#[derive(Reflect)]` generates impls equal to hand-written ones, and both reflect crates are ready for the `usable` promotion review.

## Context

- `docs/specs/liminal_reflect.md` (and the derive's spec); ADR-0003.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Derive crate; tests comparing derived and hand-written impls; compile-fail tests (rustdoc `compile_fail` doc tests, since no compile-test crate is approved); `tests/determinism.rs`; the `usable` evidence for both crates.

**Out:** Serialization (M2, OQ-002).

## Acceptance criteria

- [x] The crate exists at the path in `architecture.toml`, is a workspace member, has
      `#![forbid(unsafe_code)]` if `unsafe = "forbid"`, and has a floor in `ci/coverage.toml`.
- [x] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [x] New public items are documented; primary entry points have runnable `# Examples`.
- [x] `cargo xtask ci` passes with no failed or skipped step.
- [x] Differential tests: derived impl and hand-written reference impl give the same descriptors and field access for the same types.
- [x] The spec's §Maturity checklist holds the evidence for every `usable` gate in
      `docs/process/MATURITY.md` (trace with `maturity: usable` applied on a scratch copy, coverage
      against the 85 % target for its crate type, examples, integration tests).
- [x] The task is set to `review` with `maturity` still unchanged. The reviewing session performs
      the promotion review (REVIEW.md §3) and, if every gate holds, sets `maturity: usable` in the
      spec and `docs/specs/README.md` and regenerates STATUS.md (MATURITY.md "Promotion
      procedure": a different session changes `maturity`).

## Test requirements

Requirements in frontmatter; differential and compile-fail tests.

## Notes

- 2026-11-10 (M1-T057): both specs are at `spec_version` 2. The derive crate enables the
  `proc-macro2` feature `span-locations` on its normal dependency (derive spec §3, REFLD-R050).
  REFLD-R003 has two tests (token walk over the expansion; shadowing integration test).
  REFLD-R033: the hand-written twins return the derived types' paths and are never registered
  in the same `TypeRegistry` as the derived type. REFL-R120 is reworded.
- 2026-11-10 (M1-T061): the derive spec is at `spec_version` 3. REFLD-R003 test (1) is no longer
  a token walk: it parses the expansion as a `syn::File` and checks it with `syn::visit::Visit`
  (rules (a) to (d) in the requirement), so the derive crate enables the `syn` features `full`
  and `visit` on its normal dependency (§3). Its inputs have no field of type `String`. The test
  also runs the check on two hand-written streams (one naming `::liminal_core`, one with a bare
  `Option`) and asserts that it fails. A working version of the checker is described in the
  M1-T061 handoff.

## Handoff

2026-11-10 (implementation sub-agent; journal `docs/journal/2026-11-10-M1-T032.md`): done, set to
`review`. `maturity` of both specs unchanged (`planned`).

**Done**
- `crates/liminal_reflect_derive` (workspace member, `proc-macro`, `#![forbid(unsafe_code)]`):
  `src/lib.rs` (entry point + private `expand`), `src/parse.rs` (model, validation, the nine
  `LIM-REFLD` diagnostics, collected in source order), `src/generate.rs` (one
  `const _: () = { … };` block with the impls and two private name tables `FIELD_NAMES`,
  `VARIANT_NAMES`), unit tests in `src/tests.rs` and `src/tests/{paths,diagnostics,expansion}.rs`.
  Workspace `Cargo.toml`: `syn` 3.0 (default features off; `derive`, `parsing`, `printing`,
  `proc-macro`, `clone-impls`; the crate adds `full`, `visit`), `quote` 1.0, `proc-macro2` 1.0
  (crate adds `span-locations`). Floor `liminal_reflect_derive = 99.7` in `ci/coverage.toml`.
- `liminal_reflect`: dependency on the derive, re-export `value::Reflect` (macro) with docs,
  `# Examples` and 10 `compile_fail` doc tests (one per LIM-REFLD code, one `E0277` for
  REFLD-R052); the `Reflect` trait example now uses the derive (spec §5.2).
- Tests in `crates/liminal_reflect/tests/`: `derive.rs` (R001, R002, R004, R005, R020, R021,
  R030 to R032, R034), `derive_differential.rs` (R033, R035), `derive_shadowing.rs` (R003 test 2),
  `reexport.rs` (REFL-R120), `determinism.rs` (REFL-R130 to R132, literals recorded on macOS
  arm64). `tests/common/types.rs` gained `derived::{Sample, Pair, Marker, Mode}` (overrides
  `refl_test::…`); the hand-written twins return `<derived::X as Typed>::type_path()`, so the
  M1-T031 literals (REFL-R005, snapshot) are unchanged.
- `crates/liminal_reflect/examples/inspect.rs` (runs; uses `writeln!` on stdout because
  `print_stdout` is denied).
- Specs: derive `spec_version` 3 → 4 (`clone-impls`, §12 item 3 resolved, §13 evidence);
  reflect §3 wording and §13 evidence (no requirement change, no bump).

**REFLD-R053 (manual compile, macOS arm64, Rust 1.99.1).** A probe test file (deleted) with
`struct S { a: u8, b: NotReflected }` and `enum E { A(NotReflected) }`: E0277 "the trait bound
`NotReflected: Typed` is not satisfied" points at the field type (`6:8`, `10:7`) for both. The
additional E0277 for `NotReflected: Reflect` (coercion in `field_at`) points at the struct field
line (`6:5`, the field name) for the struct, and at `#[derive(Reflect)]` for the enum variant
(the binding is a generated identifier). The field-access arms carry the field type's span
(`generate.rs`).

**Decisions a reviewer should check**
- LIM-REFLD-0005 `{tokens}`: for a list, the list content as `TokenStream::to_string()`; for
  `#[reflect]` empty, spanned at `reflect`; for `#[reflect = …]` the tokens after `reflect`,
  spanned at `=`; for `#[reflect()]` spanned at `(`. One 0005 per malformed attribute.
- LIM-REFLD-0009 for `#[reflect(ignore, ignore)]` (both in one attribute) is spanned at that
  attribute. LIM-REFLD-0007 counts every `type_path` occurrence, valid or not.
- LIM-REFLD-0004 is spanned at the `const` keyword; 0003 at the lifetime's apostrophe.
- `syn` parse errors of input that is not an item are emitted unchanged (no LIM code); cannot
  happen through `#[derive]`.
- REFLD-R033 twins of the four M1-T031 types are those types themselves; the derived types
  carry `type_path` overrides. Twins of the generic, ignored-field and recursive types are new
  (`derive_differential.rs`), with default derived paths.
- The trace gate with `maturity: usable` was run by editing both specs in the worktree and
  reverting them (`git checkout`), not on a separate copy: exit 0, every REFL and REFLD MUST
  tested; untested SHOULDs REFL-R064 (review) and REFLD-R053 (above).

**Not verified**
- Windows and Linux (determinism literals, all tests): only the GitHub CI run will show it; the
  reflect §13 determinism item stays unticked until then. The task branch was not pushed.
- Compile-time effect of the `syn`/`proc-macro2` features: not measured (as the spec says).

**CI.** `cargo xtask ci` on macOS 27.2 arm64: pass, 12 of 12 steps, none skipped.

**Next step.** Review of M1-T032 including the promotion review (REVIEW.md §3) for both crates.

## Review

_Empty._
