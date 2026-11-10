---
id: M1-T004
title: trace and hygiene follow-ups from the M0-T005 review
milestone: M1
status: done
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

Reviewer: review sub-agent (ADR-0026), 2026-11-10, parent
`docs/journal/2026-11-10-session-m1-reviews.md`; did not write M1-T004 (its author was sub-agent
no. 4 of `2026-11-09-session-m1-subagents.md`). Prompt: the ADR-0026 review template verbatim.
Outcome: **done**, no blocking findings. Follow-up task:
`docs/tasks/M2-T005-rust-lexer-number-follow-ups.md`.

Checked, macOS 27.2, Rust 1.99.1, on `b8031d0`:
1. Acceptance criteria (by reading the diff `d6bc2b6..b8031d0` and by running):
   - Item 1: `(` is dropped from `hygiene.rs::ignores` and from the spec ("Ignore attributes"),
     with the reason. `cases/ignores.rs:21` (`#[foo(ignore)]`) is no finding. Mutant: re-adding
     `'('` fails `ignore_attributes_need_a_reason_naming_an_existing_task`. Met.
   - Item 2: `1.5e3`, `1_000u32`, `0x1F`, `1e-3` are one literal; `0..n`, `1.max(2)`, `x.0.1`
     split (lexer test). `cases/macros.rs:16` (`0..dbg!(x)`) is an R050 finding. Met.
   - Item 3: `cases/annotations.rs:35` (`pub "x" fn g`) is unattached (R041); line 37
     (`extern /* ABI */ "C" fn h`) attaches. Met.
   - `cargo xtask ci`: pass, 11 steps, none skipped. Met.
2. Tests prove behaviour: 17 single-edit mutants, each run against `cargo test -p xtask`.
   Killed (12): `(` re-added; digit check after the exponent sign dropped; `+` sign dropped;
   `after_field_dot` set on every `.`; `after_field_dot` ignored; each of `.`, `_`, letter
   dropped from the "`.` is not a fraction" test; exponent recognised after any letter and in a
   radix literal (both conditions dropped together); string literal skipped anywhere in the
   header; comment skip between `extern` and the ABI string dropped; ABI string skip dropped.
   Survived (5), all in `Lexer::number`, all non-blocking (below).
3. Realistic input (B3): a temporary lexer probe (not committed) over 24 inputs, among them
   `a[1..]`, `v[..2].len()`, `0..n.len()`, `0..=n`, `1.0..2.0`, `..=1.5`, `a.0..b.1`,
   `x.0.0.max(1)`, a tuple field on the next line, `2.0_f32.sqrt()`, `1.5.max(2.5)`,
   `1u8.pow(2)`, `1.0e-3f32`, `1usize-3`, `0xdead_beef-1`, `(1.)`, `1.e3`: every token stream is
   the one rustc's lexer gives. No wrong result found.
4. No fake progress: no placeholder; scope items 1 to 3 all done. Oracle changes are the
   expected-finding lists of three case-file tests, extended for the new fixture lines and named
   in the journal entry. Coverage floor unchanged (97.6).
5. Spec agreement: "Lexing", "Annotations" and "Ignore attributes" say what the code does. The
   branch numbers the spec change `spec_version` 18; `main` is at 21, so the merge renumbers it
   to 22 (ADR-0026 Decision 4.3), in the history row and in the four places that name the
   number (three in the spec, one comment in `xtask/src/hygiene/tests.rs`). The handoff above
   and the task's journal entry keep "18".
6. Unsafe, determinism, architecture: no `unsafe`, no dependency, no `architecture.toml` change.
   Docs: no other document states the old `(` rule (grep).
7. Session boundaries (REVIEW.md item 10): the implementation sub-agent is no. 4 in
   `docs/journal/2026-11-09-session-m1-subagents.md` (owner-started parent, "impl template").
   The parent pushed the branch (recorded there); no install, no spawned agent. This review
   belongs to a different owner-started session.
8. Public repository (REVIEW.md item 11 on `main`): no workflow change; no issue, pull request
   or outside code acted on; no credential or personal data in the diff.
9. Branch CI: run 42856863374 on `b8031d0`: success.

Non-blocking findings (follow-up M2-T005):
- `Lexer::number`: `radix` is redundant. The prefix letter (`x`, `b`, `o`) already sets
  `letter_seen`, which alone forbids the fraction and the exponent sign. Dropping `!radix` from
  either condition passes every test (2 surviving mutants, equivalent). Because of it, dropping
  `!letter_seen` from the exponent condition also passes every test, although it changes
  `1usize-3` from three tokens to one.
- "One `.`, before any letter" is untested: dropping `fraction_allowed = false` after a letter,
  or after the `.`, passes every test (2 surviving mutants). The token stream then differs only
  for input rustc rejects (`1e3.5`, `1.2.3`) or a `.` followed by white space (`1u8. pow(2)`).
  Not B4: no input changes a finding or result of `check-arch`, `trace` or `hygiene` unless it
  fails to compile, and for `1usize-3` only `-` and a digit move into the literal.
- The lexer test carries `Verifies: XTASK-R050`. It checks token streams, not a `hygiene`
  report; the R050 evidence is `cases/macros.rs:16`.
- Spec wording: in "Annotations (R041)" the inserted clause separates "otherwise the annotation
  is an R041 finding" from the condition it belongs to; that line, the "Ignore attributes" line
  and the doc comment of `next_function` are longer than 100 columns.
- The task's journal entry names its parent by branch, not by file (already M2-T004 item 5; not
  repeated in M2-T005).
