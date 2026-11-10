---
id: M1-T030
title: Write the spec for liminal_reflect
milestone: M1
status: done
crates: [liminal_reflect, liminal_reflect_derive]
depends_on: [M1-T001]
requirements: []
---

# M1-T030: Write the spec for `liminal_reflect`

## Goal

`docs/specs/liminal_reflect.md` is a complete, reviewable contract for `liminal_reflect`, so the implementation tasks can be done without guessing.

## Context

- `docs/ARCHITECTURE.md` §2 (module map: runtime type information, field access by name, stable type IDs; `#[derive(Reflect)]`).
- `docs/specs/_TEMPLATE.md`; `docs/specs/xtask.md` as the quality bar.
- `architecture.toml` entry for `liminal_reflect` (layer, `unsafe`, `deterministic`, allowed deps).
- ADR-0009 item 6; ADR-0003 (`syn`, `quote`, `proc-macro2` approved for the derive).
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Spec for `liminal_reflect` **and** `liminal_reflect_derive` (one spec file each, or one file covering both if `check-docs` R022/R026 allow it; decide and record): type descriptors, stable type IDs (how they stay stable across builds and platforms), field access by name, supported shapes (structs, tuple structs, enums, generics), how state hashing (ADR-0009 item 6) walks reflected values in canonical order, and the derive macro's accepted input and compile errors.

**Out:** Code. The implementation tasks (M1-T031, M1-T032) write it.

## Acceptance criteria

- [x] `docs/specs/liminal_reflect.md` exists, follows `docs/specs/_TEMPLATE.md` (all sections), with
      `maturity: planned`, `spec_version: 1` and `adrs` listing every ADR it implements.
- [x] Every requirement has an ID `REFL-Rnnn`, an RFC 2119 level and a testable statement with
      units, ranges and failure behaviour. No vague words ("fast", "robust", "efficient").
- [x] §5 Public API covers the scope below and is consistent with `docs/ARCHITECTURE.md` and the
      specs of the crates it depends on.
- [x] Error codes `LIM-REFL-NNNN` are listed with message and `help` (ADR-0017), or the spec says
      why the crate has no recoverable errors.
- [x] The test plan names the test kinds that apply (TESTING.md) and, for every optimized path, the
      reference implementation it is checked against (AGENTS.md rule 4).
- [x] Determinism requirements follow ADR-0024: the spec names the workloads of
      `tests/determinism.rs` and says whether cross-platform bit-identity is claimed for each.
- [x] Performance budgets name scenarios and machines; numbers that need a measurement say which
      task measures them (PERFORMANCE.md: budgets are never invented).
- [x] Every MUST requirement is assigned to exactly one implementation task (M1-T031, M1-T032): the spec
      lists the assignment, and this session writes the IDs into those tasks' `requirements`
      frontmatter (`check-docs` R028 validates them).
- [x] Open questions are listed in the spec instead of being decided silently.
- [x] `docs/specs/README.md` lists the spec (XTASK-R027); `cargo xtask ci` passes.

## Test requirements

None (documentation task). The review is a spec review (REVIEW.md §2).

## Notes

Requirement IDs leave gaps between groups (template §6). Keep the API small: every public item has to be documented, tested and later snapshotted.

## Handoff

**Done (2026-11-10, implementation sub-agent).** Status `review`.

- Decision (Scope: one file or two): **two spec files.** XTASK-R026 requires a spec at
  `docs/specs/<crate>.md` for every workspace member, and the file-name rule of XTASK-R022 makes a
  spec's `crate` equal its file name, so one file cannot cover both crates once
  `liminal_reflect_derive` is a member. `req_prefix` must be unique (XTASK-R022), so the derive has
  the new prefix `REFLD`; `docs/specs/README.md` now reserves `REFL` for `liminal_reflect` only and
  `REFLD` for `liminal_reflect_derive`.
- `docs/specs/liminal_reflect.md` (`REFL`, 33 requirements: 32 MUST, 1 SHOULD; error codes
  LIM-REFL-0001 to 0005 and 0010 to 0013; three determinism workloads, all claiming
  cross-platform bit-identity).
- `docs/specs/liminal_reflect_derive.md` (`REFLD`, 14 requirements: 12 MUST, 2 SHOULD; compile
  error codes LIM-REFLD-0001 to 0009, no runtime errors).
- Key design choices (reasons in the specs): stable type ID = XXH64 seed 0 of the type path
  (`module_path!()` + identifier + generic arguments, overridable with
  `#[reflect(type_path = "…")]`), never `core::any::TypeId`; descriptors reference field types
  by `TypeRef` (path + ID), so recursive types work; `Option<T>` is reflected as an enum; the
  state encoding writes values only (field order, `u32` variant index, `u64` list length) through
  `liminal_core::hash::StateHasher`; `TypeRegistry` iterates in ascending ID order.
- `requirements` frontmatter of M1-T031 (25 IDs) and M1-T032 (22 IDs): every spec ID is in
  exactly one task (checked by script: 47 spec IDs, 47 task entries, no overlap).
- `xtask/src/repo.rs` test `loads_the_real_repository` asserted that the repository has exactly
  one spec. The change is byte-identical to M1-T012's fix (not merged to `main` yet); it was
  already in this worktree when this session reached that step (modification time 17:30, before
  any edit of this session; presumably the parent). Identical changes merge without conflict.
- This task's `crates` now lists `liminal_reflect_derive` too (the task wrote its spec).

**Not verified:** the specs depend on `docs/specs/liminal_core.md` (`StateHasher`, `StateHash`,
`EngineError`, `validate_error`), which is reviewed (`done`) on branch
`task/M1-T012-write-the-spec-for-liminal-core` but not merged to `main`. The specs therefore name
it in code spans, not links (a link would fail XTASK-R024 on this branch).

**For the reviewer and the next tasks:**
- Spec §12 item 1 (`Reflect` for `liminal_math` types) must be resolved before ECS components hold
  math vectors (M1-T037 at the latest, before M1-T038); it needs an `architecture.toml` edge.
- M1-T031 builds the crate without the derive dependency; M1-T032 adds it and the re-export
  (REFL-R120).

## Review

2026-11-10, review sub-agent (ADR-0026/0027) of parent session `session/2026-11-10-m1-reviews`
(journal `2026-11-10-session-m1-reviews.md`), fresh context, prompt = the ADR-0026 review template.
This session implemented no part of M1-T030; the author was a sub-agent (no. 14; no. 16 was a
restart in the same worktree) of the other parent session `2026-11-09-session-m1-subagents.md`.
macOS 27.2 (Darwin), Rust 1.99.1, worktree `liminal-worktrees/M1-T030-review`, branch at
`576eb66`. Task review (REVIEW.md §1) plus spec review (REVIEW.md §2), as "Test requirements"
says.

Outcome: **done**, no blocking findings (B1–B5). Follow-ups: M1-T057.

Checked:
- AC 1: both spec files have the 14 template sections in order; `maturity: planned`,
  `spec_version: 1`; `adrs` `[ADR-0003, ADR-0009, ADR-0017, ADR-0018, ADR-0024]` and
  `[ADR-0003, ADR-0017, ADR-0018]`. The one-file-or-two decision is recorded in the handoff
  (XTASK-R022, XTASK-R026).
- AC 2: 47 requirement rows (script): REFL 33 (32 MUST, 1 SHOULD), REFLD 14 (12 MUST, 2 SHOULD),
  IDs unique, each with a level. Word search for "fast", "robust", "efficient", "seamless",
  "powerful", "simple", "easy", "reasonable", "appropriate": no hit in a requirement. Points
  where failure behaviour or testability is not fully stated are follow-up items 3 to 8.
- AC 3: §5 covers every scope item (descriptors, stable IDs and why they are stable, field
  access by name, named/tuple/unit structs, enums, generics, canonical hashing order, derive
  input and compile errors). Compared with `docs/ARCHITECTURE.md` §2 rows, `architecture.toml`
  (both crates layer 0, `unsafe = "forbid"`; reflect `deterministic = true`, deps `liminal_core`
  and the derive, dev `proptest`; derive `proc-macro`, `syn`/`quote`/`proc-macro2`, no dev
  deps), ADR-0009 item 6, ADR-0017, ADR-0024, and `docs/specs/liminal_core.md` v1 (as on `main`
  and on the M1-T012 branch): `StateHasher` methods and CORE-R050/R053/R054/R056, `StateHash`,
  `state_hash_of`, `EngineError`, `write_display`, `validate_error(&e, "REFL")`, context key
  form (all REFL keys match `^[a-z][a-z0-9_]*$`), area length (`REFL`, `REFLD` ≤ 8 letters),
  `Rng::next_f32`/`next_f64` and CORE-R098. REFL-R102 agrees with CORE-R054 byte for byte for
  primitives, `String`, `Vec<T>` and `[T; N]`, and the `Option<T>` exception is as stated.
- AC 3, API shape compiled (rustc 1.99.1, a reduced copy in the session scratch directory, not
  committed): `trait Reflect: Any`, `trait Typed: Reflect + Sized`, inherent methods on
  `dyn Reflect` with `T: Typed`, and the downcast through trait upcasting compile under
  `#![forbid(unsafe_code)]`. The §5.2 example's `unwrap_err()` does not (follow-up item 1).
  `rustdoc --test` runs a `compile_fail` doc test placed on a `pub use` item, so the doc tests of
  REFLD-R050 and REFLD-R052 on the re-export are possible.
- AC 4: nine REFL codes with message, help and context; nine REFLD compile-error codes with
  message, help and span, and the reason the derive has no runtime errors.
- AC 5: test kinds per area are those of TESTING.md; both specs state that no path is an
  optimized variant of another; generated impls are checked against hand-written twins
  (REFLD-R033).
- AC 6: three workloads of `tests/determinism.rs`, each with "cross-platform bit-identity
  claimed: yes" and "not verified yet"; the derive spec says why it has no determinism test.
- AC 7: four scenarios, no number; all say "Not scheduled; added with the `stable` promotion"
  (follow-up item 9f, the same finding as in the M1-T012 and M1-T027 reviews).
- AC 8: script over both specs and all task files: the 44 MUST IDs (and the 3 SHOULD) are in
  exactly one of M1-T031 (25 IDs) and M1-T032 (22 IDs), none unknown; the §6 tables agree.
- AC 9: seven and three open questions, each with the task, milestone or condition that
  resolves it.
- AC 10: `docs/specs/README.md` rows and prefix table (`REFL`, new `REFLD`). `cargo xtask ci` on
  `576eb66`: pass, 11 of 11 steps, none skipped. Branch CI run 42856110407 on `576eb66`:
  success.
- Read for correctness: the §4 byte example against REFL-R100; type-path grammar (derived paths
  always contain `::` before any `<`, container paths never do, so they cannot coincide);
  REFL-R061/R062/R080 check order; the REFLD-R021 pattern; REFLD-R004 tuple names with ignored
  fields; REFLD-R005 (no `type_info()` recursion).
- Oracle change: `xtask/src/repo.rs` `loads_the_real_repository`, exact spec list replaced by
  membership and exclusion checks. Justified in the journal; the hunk is identical to the one
  already on `main` (`git diff` of the file against `origin/main` shows only M1-T011's
  additions), so nothing new is weakened.
- No fake progress: no code; `crates` gained `liminal_reflect_derive` with the reason in the
  handoff; nothing reduced.
- Session boundaries (item 10): the author is row 14 of its parent's journal; row 16 is a
  restart for the same task, which deviates from ADR-0027 Decision 2.3 on the owner's quoted
  instruction 3. For some minutes both agents worked in one worktree; no. 16 left the status
  change and the `repo.rs` hunk, which the author kept and recorded. The author's journal does
  not name its parent's file (ADR-0026 Decision 8.2); the parent's entry records that. No
  install or nested spawn recorded.
- ADR-0028 (item 11): no workflow change; no credentials or personal data in the diff; no
  issue, pull request or outside code acted on.

Follow-ups (M1-T057): the §5.2 example does not compile (`unwrap_err` needs `dyn Reflect:
Debug`); `TypeId` stored and printed by `TypeRegistry` against §11; behaviour of `write_reflect`
and `field` on inconsistent hand-written impls; check order in `register`; the untestable hash
collision case of REFL-R062; "depends on `liminal_reflect` only" in an integration test; nested
type paths in the differential test; span assertions outside the compiler (assumption); small
text items.

Whoever starts M1-T031 before M1-T057 is done: do not copy the §5.2 example as it is (item 1).
