---
id: M2-T006
title: Determinism lint follow-ups from the M1-T011 review
milestone: M2
status: todo
crates: [xtask]
depends_on: [M1-T011]
requirements: [XTASK-R039]
---

# M2-T006: Determinism lint follow-ups from the M1-T011 review

## Goal

The gaps between "a deterministic crate's clippy configuration bans the CODE_STYLE.md list"
(XTASK-R039) and "a deterministic crate cannot call non-deterministic functions" (ADR-0009) are
each closed by a check or stated in CODE_STYLE.md as not enforced.

## Context

- Follow-up task under ADR-0021, written by the review of M1-T011 (see that task's "Review").
- `xtask/src/repo/clippy.rs`, `xtask/src/check_arch/determinism.rs`, `xtask/src/repo.rs`
  (`load_arch`), `docs/specs/xtask.md` §check-arch (R039 and its details),
  `docs/process/CODE_STYLE.md` §Lints, ADR-0009 item 7.
- No M1 exit criterion needs this, so it is in M2 (ADR-0021). Items 2 and 3 matter more the
  more deterministic crates exist; an earlier session MAY pick the task up if the owner moves it.

## Scope

**In:**
1. **First registry entry.** The R039 details say a crate is judged by "the first `[[crate]]`
   entry of the name". Replacing `find` by `rfind` in `repo::load_arch` passes every test. A
   duplicate name is already an XTASK-R030 finding. Decide: add a `load_arch` test with two
   entries of one name and different `deterministic` values, or remove the sentence from the
   spec (bump `spec_version`).
2. **Lint level.** R039 checks the configuration file only. With a complete file,
   `#![allow(clippy::disallowed_methods)]` at a crate root (or the same lint set to `allow` in
   the crate's `[lints.clippy]`) makes clippy report nothing. Decide: a rule (`check-arch` or
   `hygiene`) that reports an `allow` of `clippy::disallowed_methods` in a deterministic crate
   outside test code, or a sentence in CODE_STYLE.md that says it is not enforced and why.
   A new rule needs a requirement ID, fixtures and a `spec_version` bump.
3. **Banned list.** With the complete list of CODE_STYLE.md, clippy 0.1.99 accepts these calls
   in a deterministic crate (probe in the M1-T011 review): `f32::sin_cos`, `f32::powi`,
   `f32::atan2`, `f32::log2`, `std::collections::HashSet::new`,
   `std::collections::HashMap::default`, `std::time::SystemTime::now`. `disallowed-types`,
   which ADR-0009 item 7 names, is not used. Decide which paths and types the list gains
   (candidates: every `f32`/`f64` method whose result is not required to be correctly rounded,
   `HashMap` and `HashSet` as types, `Instant` and `SystemTime` as types). A change is one
   edit of CODE_STYLE.md, `BANNED_METHODS`, the R039 row and the fixtures (the test
   `banned_methods_match_code_style` fails on a mismatch); crates that already have a
   `clippy.toml` are updated in the same change.
4. **Root settings.** CODE_STYLE.md says a deterministic crate's file repeats every setting
   of the root `clippy.toml`. Nothing checks it. Decide: R039 also reports a root key that is
   missing or different in the crate's file, or CODE_STYLE.md says the repetition is not
   checked.
5. **Rule range text.** `xtask/src/cli.rs` summary says "XTASK-R030 … R039"; R038 is not
   implemented. Use the wording of `xtask/src/check_arch.rs` ("R030 … R037, R039").

**Out:** writing `clippy.toml` for real crates (each crate's first implementation task).

## Acceptance criteria

- [ ] Each of items 1 to 4 has either a test that fails when the behaviour is broken, or a
      spec or CODE_STYLE.md sentence that states the decision; the journal names which.
- [ ] If the banned list changes: CODE_STYLE.md, `BANNED_METHODS`, the R039 row and every
      existing deterministic crate's `clippy.toml` agree, and a scratch-crate clippy run for
      each new path is recorded in the journal.
- [ ] Item 5 is done.
- [ ] `cargo xtask ci` passes.

## Test requirements

Fixture tests in the style of `xtask/tests/fixtures/check_arch/r039_*`; unit tests in
`xtask/src/repo/clippy.rs` and `xtask/src/check_arch/determinism.rs`.

## Notes

Written by the review of M1-T011 (2026-11-10). The mutants and probes the review ran are listed
in that task's "Review".

## Handoff

_Empty._

## Review

_Empty._
