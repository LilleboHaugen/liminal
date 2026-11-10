---
id: M1-T032
title: Implement the Reflect derive macro; reach usable for both reflect crates
milestone: M1
status: in-progress
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

**2026-11-10, review sub-agent of the session on branch `session/2026-11-10-m1-t020-t031-review`
(ADR-0026, ADR-0030; did not work on this task). Outcome: changes required (round 1 of at most
2). `maturity` of both specs unchanged; the promotion review (REVIEW.md §3) is not done.**

Rules read from `origin/main` (`483f277`), which is an ancestor of the reviewed head `5580675`
(the reviewed tree is the final tree).

**Required changes**

1. **B1: `cargo xtask ci` fails on Windows because of this change.** GitHub run 42913316674 on
   `5580675`: `checks (windows-latest)` fails in step `test`
   (`cargo test --workspace --all-features --locked`) with
   `failed to remove file D:\a\liminal\liminal\target\debug\xtask.exe: Access is denied (os error 5)`;
   macOS, Linux, coverage and deny pass. Cause: `cargo xtask` (`run --package xtask`) resolves
   features for `xtask` alone, while `--workspace` unifies the `syn` features `full`, `visit`
   and the `proc-macro2` feature `span-locations` of `liminal_reflect_derive` (derive spec §3)
   into the `syn`/`proc-macro2` that `serde_derive` (a dependency of `xtask`) uses. The
   workspace build therefore makes a second `xtask` binary with another hash and uplifts it to
   `target/debug/xtask`, which Windows cannot replace while that file runs `ci`. Reproduced on
   macOS: alternating `cargo build -p xtask` and `cargo build --workspace` gives
   `target/debug/xtask` a new inode each time, and `target/debug/deps` holds two
   `serde_derive` dylibs. Merged as is, `main` turns red on Windows, and the reflect §13
   determinism gate (three CI operating systems) cannot be met, so the acceptance criterion on
   the `usable` evidence is not met either. The fix is the implementer's choice (examples:
   nested cargo calls of `xtask ci` use their own target directory; the alias runs a copied
   binary; the features stop reaching `serde_derive`'s dependencies); a change of an
   `xtask` requirement or of the CI workflow needs the matching spec/ADR-0028 rules. Done
   when a GitHub run on the task branch is green on all three operating systems, which also
   gives the evidence for the open reflect §13 determinism item (the literals already pass on
   `macos-latest` and `ubuntu-latest` in run 42913316674).

**Checked without a blocking finding**
- Acceptance criteria 1 to 3, 5 to 7 by reading and running: crate at the registered path,
  workspace member, `#![forbid(unsafe_code)]`, floor 99.7; every requirement of the
  frontmatter has a `Verifies:` test that checks it; docs and `# Examples` on the re-export and
  `Reflect`; differential tests (`derive_differential.rs`, twins return the derived paths,
  separate registries, ignored-field twin with different ignored values).
- `cargo xtask ci` on macOS 27.2 arm64, Rust 1.99.1: pass, 12 of 12 steps, none skipped.
  `cargo xtask coverage`: `liminal_reflect` 98.84 %, `liminal_reflect_derive` 99.78 %.
  `cargo run -p liminal_reflect --example inspect` runs.
- Code against the derive spec: §5 bounds and absolute paths, §8 messages and spans (unit
  tests compare line and column per code), REFLD-R004 tuple positions with ignored fields,
  REFLD-R031 declaration positions with explicit discriminants, INV-2/INV-3; REFLD-R070: no
  hash map, environment, file system or clock in `src/`. REFLD-R003 checker matches rules (a)
  to (d) and is shown non-vacuous. REFLD-R053 manual compile recorded in the handoff.
- Dependencies: `syn`, `quote`, `proc-macro2` are scope `build` under ADR-0003 in
  `architecture.toml`; the same-layer edge `liminal_reflect` → `liminal_reflect_derive` is
  registered; `check-arch` passes.
- Session boundaries: implementation sub-agent 4 and this review sub-agent are listed in the
  parent journal; spawn times before the cutoff; no install; no `.github/workflows/` change.

**Backlog entries:** 2 (`docs/BACKLOG.md`, "M1-T032 — review 2026-11-10"). **Follow-up task:**
none.
