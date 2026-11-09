---
id: M1-T004
title: trace and hygiene follow-ups from the M0-T005 review
milestone: M1
status: in-progress
crates: [xtask]
depends_on: [M0-T005]
requirements: [XTASK-R041, XTASK-R050]
---

# M1-T004: `trace` and `hygiene` follow-ups from the M0-T005 review

## Goal

The non-blocking findings of the first M0-T005 review are fixed: the `ignore` position rule says
what the code can test, and the shared Rust lexer splits numeric literals and range operators
correctly.

## Context

- Follow-up task under ADR-0021, written by the first review of M0-T005 (see that task's
  "Review").
- `docs/specs/xtask.md` §6 "`trace` and `hygiene` details" (Lexing, Ignore attributes,
  Annotations). `xtask/src/rust_lexer.rs`, `xtask/src/hygiene.rs`,
  `xtask/src/trace/annotations.rs`.
- No M0 exit criterion needs this, so it is in M1. The M1 kickoff (M1-T001) places it in the M1
  order; it touches only `xtask`.

## Scope

**In:**
1. `hygiene.rs::ignores`: the spec counts `ignore` "directly after `[`, `(` or `,`". The `(` case
   is untested (review mutant: dropping `'('` from the match leaves every test passing). No valid
   test-ignore attribute puts `ignore` directly after `(`: in `cfg_attr(pred, attr…)` the first
   argument is the predicate. The `(` case only matches attributes like `#[foo(ignore)]`, which
   are not test ignores. Decide: either drop `(` from the spec and the code (bump `spec_version`),
   or keep it with a stated reason and a test.
2. `rust_lexer.rs` numeric literals: the loop consumes `.` and alphanumerics, so `0..n` is one
   `Literal` token and the identifier `n` is lost; `0..dbg!(x)` is then not an R050 finding.
   Not realistic in current code, but the lexer is shared (R035, R041, R050). Lex a numeric
   literal so that `..` and `.ident` (method call, field) end it: `1.5e3`, `1_000u32`, `0x1F`,
   `1e-3` stay one literal; `0..n`, `1.max(2)`, `x.0.1` split. Test each.
3. `trace/annotations.rs::next_function`: the qualifier loop skips a string literal at any
   position (meant for `extern "ABI"`), so `"x" fn f` would be accepted. Accept a string only
   directly after `extern`, and test it.

**Out:** other behaviour changes to `trace` and `hygiene`.

## Acceptance criteria

- [ ] Item 1 decided and written into the spec (if changed) and the code; a test covers what
      remains.
- [ ] Numeric literal cases of item 2 are lexer tests; a hygiene case asserts that a forbidden
      macro after a range (`0..dbg!(x)`) is an R050 finding.
- [ ] A string literal not directly after `extern` makes an annotation unattached (R041), tested.
- [ ] `cargo xtask ci` passes.

## Test requirements

`Verifies: XTASK-R050` on the hygiene tests, `Verifies: XTASK-R041` on the annotation test.

## Notes

- Inputs that contain forbidden patterns go in files under `xtask/tests/fixtures/` (M0-T005
  notes), never in string literals. `cases/*.rs` line numbers are asserted.

## Handoff

_Filled in by the implementing session(s)._

## Review

_Filled in by the reviewing session._
