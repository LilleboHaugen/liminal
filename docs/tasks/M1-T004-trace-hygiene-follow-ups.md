---
id: M1-T004
title: trace and hygiene follow-ups from the M0-T005 review
milestone: M1
status: review
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

- [x] Item 1 decided and written into the spec (if changed) and the code; a test covers what
      remains.
- [x] Numeric literal cases of item 2 are lexer tests; a hygiene case asserts that a forbidden
      macro after a range (`0..dbg!(x)`) is an R050 finding.
- [x] A string literal not directly after `extern` makes an annotation unattached (R041), tested.
- [x] `cargo xtask ci` passes.

## Test requirements

`Verifies: XTASK-R050` on the hygiene tests, `Verifies: XTASK-R041` on the annotation test.

## Notes

- Inputs that contain forbidden patterns go in files under `xtask/tests/fixtures/` (M0-T005
  notes), never in string literals. `cases/*.rs` line numbers are asserted.

## Handoff

2026-11-09, implementation sub-agent (parent: `docs/journal/` entry of session
`session/2026-11-09-m1-subagents`). Status `in-progress` → `review`.

Done:
1. Item 1 decided: `(` dropped from the spec and the code. `ignore` counts only directly after
   `[` or `,` (`docs/specs/xtask.md` `spec_version` 18, "Ignore attributes"). Reason: no
   test-ignore attribute puts `ignore` directly after `(` (in `cfg_attr` the first argument is
   the predicate), so `(` matched only non-test attributes. Test: line 21 `#[foo(ignore)]` of
   `xtask/tests/fixtures/hygiene/cases/ignores.rs` is no finding
   (`hygiene::tests::ignore_attributes_need_a_reason_naming_an_existing_task`); re-adding `'('`
   to the match fails it.
2. `xtask/src/rust_lexer.rs`: new `Lexer::number`. A `.` belongs to a decimal literal only once,
   before any letter, and not if followed by `.`, `_` or a letter; a `+`/`-` after a decimal
   exponent letter (the first letter) followed by a digit is part of it; a literal right after a
   single `.` (tuple field) takes no `.` (state `after_dot`/`after_field_dot`, updated in
   `Lexer::next`). Test `numeric_literals_end_at_ranges_method_calls_and_tuple_fields` covers
   `1.5e3`, `1_000u32`, `0x1F`, `1e-3` (one literal) and `0..n`, `1.max(2)`, `x.0.1`, `1..2.5`,
   `0x1E-3`, `1e-x` (split). Hygiene case: line 16 `0..dbg!(x)` of `cases/macros.rs` is an R050
   finding.
3. `xtask/src/trace/annotations.rs::next_function`: a string literal is skipped only after
   `extern` (comments between allowed). Fixture `cases/annotations.rs` line 35 (`pub "x" fn g`)
   is unattached (R041); line 37 (`extern /* ABI */ "C" fn h`) attaches.
- Spec `docs/specs/xtask.md` 17 → 18 (Lexing, Annotations, Ignore attributes, History).

Verified: `cargo xtask ci` on macOS 27.2, Rust 1.99.1: 11 steps pass, no skips. xtask coverage
98.06 % (5127/5228), floor 97.6 unchanged. Not verified on Windows or Linux (branch CI not run
by this sub-agent).

Not done: nothing in scope. Traps: the lexer is shared by `check-arch`, `trace` and `hygiene`;
a new lexer rule that reads the previous token must update state in `Lexer::next`, not
`next_token` (the shebang probe also uses `next`).

## Review

_Filled in by the reviewing session._
