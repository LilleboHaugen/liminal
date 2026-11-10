---
id: M2-T005
title: Rust lexer numeric literal follow-ups from the M1-T004 review
milestone: M2
status: todo
crates: [xtask]
depends_on: [M1-T004]
requirements: [XTASK-R041, XTASK-R050]
---

# M2-T005: Rust lexer numeric literal follow-ups from the M1-T004 review

## Goal

Every condition in `Lexer::number` is either tested or removed, and the spec text M1-T004 added
reads unambiguously.

## Context

- Follow-up task under ADR-0021, written by the review of M1-T004 (see that task's "Review").
- `xtask/src/rust_lexer.rs` (`Lexer::number` and the test
  `numeric_literals_end_at_ranges_method_calls_and_tuple_fields`), `docs/specs/xtask.md` §6
  "`trace` and `hygiene` details" (Lexing, Annotations, Ignore attributes),
  `xtask/src/trace/annotations.rs::next_function`.
- No M1 exit criterion needs this, so it is in M2 (ADR-0021). The lexer is shared by
  `check-arch` (R035), `trace` (R041) and `hygiene` (R050).

## Scope

**In:**
1. **Redundant `radix`.** In `Lexer::number`, the prefix letter of `0x`, `0b`, `0o` sets
   `letter_seen`, which already forbids the fraction `.` and the exponent sign. Removing
   `!radix` from either condition passes every test. Remove `radix` (or state why it stays),
   so that the `0x1E-3` case tests the `letter_seen` condition. Add the lexer case `1usize-3`
   (three tokens): today, dropping `!letter_seen` from the exponent condition passes every test
   and makes it one literal.
2. **"One `.`, before any letter".** Dropping `fraction_allowed = false` after a letter, or
   after the `.`, passes every test. The results differ only for input that does not compile
   (`1e3.5`, `1.2.3`) or has white space after the `.` (`1u8. pow(2)`). Decide: add lexer cases
   that pin the stated rule, or simplify the rule in code and spec (bump `spec_version`) to what
   the tests can show.
3. **`Verifies:` tag.** The lexer test carries `Verifies: XTASK-R050` but checks token streams,
   not a `hygiene` report. Lexing has no requirement ID. Decide: remove the tag (the R050
   evidence is `xtask/tests/fixtures/hygiene/cases/macros.rs` line 16), or keep it with a
   stated reason in the test's doc comment.
4. **Spec wording.** In "Annotations (R041)" the clause added by M1-T004 ("a string literal
   anywhere but directly after `extern` … ends the header") stands between the condition and
   "otherwise the annotation is an R041 finding". Reorder so that "otherwise" follows its
   condition. Re-wrap that line, the "Ignore attributes" line and the doc comment of
   `next_function` to 100 columns. A text-only spec change still bumps `spec_version`.

**Out:** other behaviour changes to the lexer, `trace` or `hygiene`.

## Acceptance criteria

- [ ] `Lexer::number` has no condition whose removal passes every test, or the journal names
      each remaining one and why it is equivalent.
- [ ] `1usize-3` is a lexer case.
- [ ] Scope items 3 and 4 are done.
- [ ] `cargo xtask ci` passes.

## Test requirements

Lexer unit tests in `xtask/src/rust_lexer.rs`. Inputs that contain forbidden patterns go in files
under `xtask/tests/fixtures/` (M0-T005 notes), never in string literals.

## Notes

Written by the review of M1-T004 (2026-11-10). Mutants the review ran are listed in that task's
"Review".

## Handoff

_Empty._

## Review

_Empty._
