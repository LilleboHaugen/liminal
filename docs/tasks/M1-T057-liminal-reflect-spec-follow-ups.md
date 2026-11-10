---
id: M1-T057
title: liminal_reflect and liminal_reflect_derive spec follow-ups from the M1-T030 review
milestone: M1
status: done
crates: [liminal_reflect, liminal_reflect_derive]
depends_on: [M1-T030]
requirements: []
---

# M1-T057: `liminal_reflect` and `liminal_reflect_derive` spec follow-ups from the M1-T030 review

## Goal

The two reflect specs leave no point open to a wrong reading by their implementation tasks, and
their normative example compiles.

## Context

- Follow-up task under ADR-0021, written by the review of M1-T030 (see that task's "Review").
- `docs/specs/liminal_reflect.md` v1, `docs/specs/liminal_reflect_derive.md` v1,
  `docs/specs/liminal_core.md` (on `main` since the M1-T012 merge).
- In M1, not M2: every item amends a contract that M1-T031 and M1-T032 implement, and the
  ROADMAP M1 exit criteria need both crates at `usable`. Best done before M1-T031 starts. If
  M1-T031 or M1-T032 is already in progress or done, each item also checks the code.

## Scope

**In:**

1. **The §5.2 example does not compile (`liminal_reflect`).** `r.field("z").unwrap_err()` needs
   `&dyn Reflect: Debug` (`Result::unwrap_err` requires `T: Debug`), and §5.2 gives `dyn Reflect`
   no `Debug` impl. Checked by the review with rustc 1.99.1 on a reduced copy of the API:
   error E0277, "`dyn Reflect` doesn't implement `Debug`". The example is normative and §10 makes
   it the `# Examples` doc test of `Reflect`. Either change the example (for example
   `match r.field("z") { Err(e) => …, Ok(_) => … }`), or add `impl core::fmt::Debug for dyn
   Reflect` to §5.2 with its exact output and a requirement. Also say that the example runs in a
   function that returns `Result<(), ReflectError>` (it uses `?`).
2. **`core::any::TypeId` in the registry (§5.3, §11).** §11 says `TypeId` is "never stored, hashed,
   ordered or printed". §5.3 stores it in `TypeRegistry` (REFL-R062 needs it) and derives `Debug`
   for `TypeRegistry`, which would print it; `TypeId`'s `Debug` output differs between builds.
   Reword §11 (not persisted, not hashed, not ordered), and state that `TypeRegistry`'s `Debug`
   output does not contain a `TypeId` (hand-written impl, with a test), or drop the `Debug` impl.
3. **Inconsistent hand-written impls in `write_reflect` and `field` (REFL-R100, REFL-R043).** The
   spec does not say what happens when `field_at(i)` or `ReflectList::get(i)` returns `None` for
   `i` below the reported count, or when `field_name(i)` matches but `field_at(i)` is `None`.
   AGENTS.md rule 12 forbids `unwrap`. State the behaviour (for example: `write_reflect` writes
   nothing for that field; `field` returns LIM-REFL-0010; `validate_reflect` is the check that
   reports it), with a test using a deliberately wrong impl.
4. **Check order in `register` (REFL-R060 to REFL-R062).** State whether the collision check of
   REFL-R062 runs after all checks of REFL-R061. In REFL-R061, "struct fields first, then variants
   in order with their fields" does not say whether an enum's variant names are checked before
   every variant's field names or variant by variant; fix one order (example to decide:
   `enum E { A { x, x }, B, A }` built by a hand-written impl).
5. **Hash collision case of REFL-R062.** "Two paths with the same hash" cannot be built in a test:
   REFL-R061 rejects a descriptor whose `id` is not the hash of its path before the collision
   check, and no XXH64 collision pair of type paths is known. Say that the test covers the
   same-path case (two Rust types, one path) and that the other case takes the same code path.
6. **"Depends on `liminal_reflect` only" (REFL-R120, REFLD-R003).** An integration test in
   `crates/liminal_reflect/tests/` is compiled with `liminal_core` and `proptest` available, so
   it cannot show that a dependency is absent. Reword to what is testable (the test file names no
   crate other than `liminal_reflect`), and add the stronger check to the `expand` unit tests:
   every path in the generated tokens starts with `::liminal_reflect`, `::core` or `::std`.
7. **Differential test and nested type paths (REFLD-R033).** "Equal `type_info()` apart from
   `type_path` and `id`" fails for the recursive and the generic test type if the twin has another
   path: the `TypeRef` of `children: Vec<Node>` contains the type's own path and ID. State how
   the twins are compared (for example: the hand-written twin returns the derived type's path
   and the two are never put into one registry, so the descriptors are equal without exception).
8. **Span assertions in `expand` unit tests (REFLD-R050).** Assumption, not checked in the review
   (no `proc-macro2` source in the repository yet): outside the compiler, `proc_macro2::Span`
   reports line and column only with the crate feature `span-locations`. State how the unit test
   identifies the span (that feature, or comparing the error's tokens with the input's), record
   a needed feature in §3 next to the `syn` features, and check it against
   `architecture.toml` (`dev_external = []` for the derive) and `docs/process/DEPENDENCIES.md`.
9. **Small text items.**
   (a) `liminal_reflect` §4 byte example: `State` is not defined there; `01 00 00 00` is correct
   only if `Fleeing` is the second variant, while `State<T>` in the derive spec §5 has it third.
   Define the example's types.
   (b) `liminal_reflect` §12 item 7 says "Tests cover depths up to 1,000"; no requirement asks
   for it. Add it to a requirement (REFL-R021 or REFL-R100) or delete the sentence.
   (c) `docs/specs/liminal_core.md` is on `main` now: turn the code-span references into links,
   and re-check the cited CORE requirements against the core spec's current `spec_version`
   (M1-T055 changes it).
   (d) REFL-R080: for the root value `at` is `""`, so the LIM-REFL-0005 message reads
   "at `` does not match". State that this is intended or name the root (for example `(root)`).
   (e) State once that reflection reads and writes private fields of a derived type from any
   crate (`field_mut`), and that `#[reflect(ignore)]` is the way to keep a field out; this is
   the general form of §12 item 2.
   (f) `liminal_reflect` §9: three rows say "Not scheduled"; name the task or promotion review
   that adds them, or drop the rows (same finding as M1-T055 item 4 and M1-T056 item 9b). Add
   that `validate_reflect` builds one `TypeInfo` per reachable value (list elements included),
   so its cost is linear in the value's size with one allocation group per element.

**Out:** Any other change to the reflect design (shapes, encoding, type ID algorithm). Code in
`crates/liminal_reflect` and `crates/liminal_reflect_derive` (M1-T031, M1-T032), unless one of
those tasks is already done and an item changes its contract. `docs/specs/liminal_reflect.md`
§12 item 1 (`liminal_math` edge) stays with M1-T037.

## Acceptance criteria

- [x] Items 1 to 9 are done, or a declined item is written into this task with the reason.
- [x] The §5.2 example, as changed by item 1, was compiled against a reduced copy of the §5 API
      (or against the crate, if M1-T031 exists), and the handoff says how.
- [x] Spec changes bump `spec_version` in each changed spec with a history row; requirement IDs
      added or moved are in exactly one task's `requirements` frontmatter.
- [x] `cargo xtask ci` passes.

## Test requirements

None (documentation task). The requirements it touches are tested by M1-T031 and M1-T032.

## Handoff

2026-11-10, implementation sub-agent (ADR-0026/0027) of parent session
`session/2026-11-10-m1-reviews` (journal `2026-11-10-session-m1-reviews.md`). Journal:
`docs/journal/2026-11-10-M1-T057.md`. No code exists for either crate (M1-T031 and M1-T032 are
`todo`), so every item is a spec change only.

**Done.** `docs/specs/liminal_reflect.md` and `docs/specs/liminal_reflect_derive.md` are at
`spec_version` 2, each with a history row. No item is declined.

| Item | Change |
|---|---|
| 1 | §5.2 example: wrapped in `fn main() -> Result<(), ReflectError>`; the last assertion is `assert!(matches!(r.field("z"), Err(ReflectError::NoSuchField { .. })))`. No `Debug` impl for `dyn Reflect` was added (the example was changed instead). A paragraph says which calls do not compile on `Result<&dyn Reflect, _>` |
| 2 | §5.3: `Debug` removed from the derive list of `TypeRegistry`, hand-written impl; new REFL-R065 fixes its output (`TypeRegistry { types: [...] }`, descriptors in `iter()` order) with a test. §11 reworded: a `TypeId` is kept in memory by the registry, never persisted, hashed, ordered or printed |
| 3 | REFL-R043: lookup order; `field_at(i)` `None` for a matching name gives LIM-REFL-0010. REFL-R100: nothing is written for a missing field or element, the rest is written. REFL-R080: `get(i)` must be `Some` below `len()`; `expected` / `actual` of the missing-value case. Each with a test using a deliberately wrong impl. INV-4 limited to values that pass `validate_reflect` |
| 4 | REFL-R060, REFL-R062: the collision check runs after all REFL-R061 checks. REFL-R061: one walk in declaration order, variant by variant (name, then that variant's fields). `enum E { A { x, x }, B, A }` reports `x`; `enum F { A, A { x, x } }` reports `A` |
| 5 | REFL-R062: the test covers the same-path case; the equal-hash case takes the same code path and cannot be built |
| 6 | REFL-R120 and REFLD-R003 reworded (the test file names no crate other than `liminal_reflect`; what that does and does not show). REFLD-R003 gains a token-walk unit test on the expansion |
| 7 | REFLD-R033: the hand-written twin returns the derived type's path; descriptors are equal without exception; twins are never in one registry with the derived type |
| 8 | Derive spec §3, REFLD-R050: `proc-macro2` feature `span-locations` on the normal dependency; why not a dev-dependency (XTASK-R034, scope `build`); no `architecture.toml` change, no ADR |
| 9a | §4 example defines `Boid` and `Mode` (`Fleeing` is variant 1), `bits(v)`, total 33 bytes |
| 9b | §12 item 7: the sentence is deleted; no requirement fixes a depth |
| 9c | Link to `liminal_core.md` in §3 (the `spec_version` 1 text had no path reference to the core spec, only CORE IDs). CORE-R050, R053, R054, R056, R098 compared with core `spec_version` 1 (`main`) and 2 (branch `task/M1-T055-…`, in `review`): identical text. References between the two reflect specs in prose are links now |
| 9d | REFL-R080, §8: `at` is `(root)` for the root value; `at` is stated to be the location of the value in violation |
| 9e | §4 "Visibility" (reflect), §4 and REFLD-R004 (derive) |
| 9f | §9: the three rows are dropped with a note (same form as M1-T055 and M1-T056); cost of `validate_reflect` stated |

**Scope additions** (same findings, other place): the derive spec's §9 had one "Not scheduled"
row, removed like item 9f. `requirements` of M1-T031 gained REFL-R065; M1-T031 and M1-T032
each got a "Notes" entry that points at the `spec_version` 2 changes. No requirement ID was
moved or withdrawn.

**How the claims were checked** (session scratch directory, nothing committed):
- Item 1: `rustc --edition 2024` 1.99.1 on a single file holding a reduced copy of the §5 API
  (`Reflect: Any`, `Typed`, `ReflectFields`, `ReflectRef`, `ReflectMut`, the four inherent
  methods on `dyn Reflect`, `ReflectError` with three variants, impls for `f32`) under
  `#![forbid(unsafe_code)]` and `#![deny(warnings)]`, a hand-written impl for `Velocity` in
  place of the derive, and the example's `main` verbatim. It compiles and runs (exit 0). The same
  file with `r.field("z").unwrap_err()` fails with E0277 as the review found. Not checked: the
  example with the real derive and as a doc test (neither crate exists).
- Item 1, found on the way: `code()` is a method of the trait `liminal_core::error::EngineError`,
  so the `spec_version` 1 example would also have needed that trait in scope. The new example
  does not call it.
- Item 8: read `proc-macro2` 1.0.107 (the version in `Cargo.lock`) in the local Cargo registry:
  `Span::start()` is `#[cfg(span_locations)]`; the fallback `Span`'s `Debug` prints `Span`
  without the feature. A probe crate built with `cargo run --offline` (`proc-macro2` 1.0.107,
  `syn` 3.0.9, `quote` 1.0.47, all from the local registry, nothing installed) parsed
  `"#[derive(Reflect)]\nunion U { … }"`, made a `syn::Error` at the `union` token and printed the
  tokens of `to_compile_error()`: without the feature every span prints `Span`; with it every
  token up to the group reports line 2, column 0. Not measured: the compile-time cost of the
  feature. Not checked: `syn` features (M1-T032 fixes them).
- XTASK-R034 read in `xtask/src/check_arch/rules.rs`: a dev-dependency must have scope `dev`.
- `cargo xtask ci`: pass, 11 of 11 steps, none skipped, macOS 27.2, Rust 1.99.1.

**Traps.**
- `docs/specs/liminal_core.md` is at `spec_version` 1 on this branch and 2 on the M1-T055
  branch. This task does not touch that file, so there is no conflict; the §3 sentence names
  both versions.
- §9 of both specs has a table header without rows (the template asks for the table);
  `check-docs` accepts it.
- `StableTypeId(…)` contains the text `TypeId(`: a test of REFL-R065 must compare the whole
  `Debug` string, not search for `TypeId(`.

**Next.** Review of this task. Then M1-T031 (needs M1-T014 `done`).

## Review

2026-11-10, review sub-agent of parent session `2026-11-10-session-m1-reviews.md`, fresh context
(journal `2026-11-10-M1-T057-review.md`). This agent wrote no part of M1-T057. The author was an
implementation sub-agent of the same parent; ADR-0029 Decision 1 allows that (see "Session
boundaries"). macOS 27.2 (arm64), Rust 1.99.1, worktree `liminal-worktrees/M1-T057-review`,
branch at `9903681` before the review commits. Rules read from `main` at `f8b9660` (ADR-0029
wording of `AGENTS.md` and `REVIEW.md`); the branch itself predated them.

Outcome: **done**, no blocking finding. Backlog entries: 3 (`docs/BACKLOG.md`, heading
"M1-T057 — review 2026-11-10"). Follow-up task: M1-T061 (contract-changing: it changes
REFLD-R003, which M1-T032 implements; M1-T032 has it in `depends_on` now).

Checked (REVIEW.md §1; the changed spec text also against §2):
1. Acceptance criteria. The diff `ae6bdf7..9903681` was read in full.
   - Item 1: the §5.2 example was compiled and run by this review: `rustc --edition 2024`
     1.99.1 on one file with a reduced copy of the §5 API (`Reflect: Any`, `Typed`,
     `ReflectFields`, `ReflectRef`, `ReflectMut`, the four inherent methods of `dyn Reflect`,
     three `ReflectError` variants, impls for `f32`, a hand-written impl for `Velocity`) and the
     example's `main` verbatim, under `#![forbid(unsafe_code)]` and `#![deny(warnings)]`: exit
     0. With `r.field("z").unwrap_err()` the same file fails with E0277. `EngineError::code`
     is a trait method in `docs/specs/liminal_core.md` §5. Not checked: the example with the
     real derive or as a doc test (no crate yet).
   - Item 2: §5.3, §11, REFL-R065 agree with each other; `TypeInfo` and its parts derive
     `Debug` (§5.1), so the stated output is defined; REFL-R065 is in M1-T031's `requirements`
     and in no other task's.
   - Item 3: REFL-R043, REFL-R080, REFL-R100, INV-4. The wrong impls of R043 and R100 reach
     the missing-value check of R080 (the descriptor checks before it pass for them).
   - Item 4: both examples walked by hand against the stated order: `E` gives `x`, `F` gives
     `A`. The "invalid descriptor whose ID is already stored" test of §10 can be built for 0002
     and 0003.
   - Item 5, 7, 9a to 9f: read. 9a: 8 + 4 + 4 + 4 + 4 + 8 + 1 = 33 bytes. 9c: CORE-R050, R053,
     R054, R056, R098 are textually identical on the branch and in the core spec at
     `spec_version` 3 on `main` (diffed row by row).
   - Item 6: done as the item asks, but the definition of the new token check is wrong
     (finding 1, M1-T061).
   - Item 8: probe crate built offline by this review (`proc-macro2` 1.0.107, `syn` 3.0.9,
     `quote` 1.0.47 from the local registry, nothing installed): with `span-locations`, every
     token of `syn::Error::to_compile_error()` for an error at the `union` token of
     `"#[derive(Reflect)]\nunion U { … }"` reports line 2, column 0. `Span::start()` is
     `#[cfg(span_locations)]` in the source, and its doc comment says what §3 says about stable
     toolchains. XTASK-R034 read in `xtask/src/check_arch/rules.rs`: a dev-dependency needs
     scope `dev`. `architecture.toml`, DEPENDENCIES.md and ADR-0003 have no rule on features.
   - `spec_version` 2 with a history row in both specs.
2. `cargo xtask ci` on the merge of `main` (`f8b9660`) into the branch with the review edits:
   pass, 12 steps, none skipped. Not run separately on `9903681`: branch CI run 42870451446
   on that commit is a success (`gh run list`).
3. Tests: none in this task (documentation). 4. No fake progress: no item declined or shrunk; no
   oracle touched. 5. Spec agreement: no code exists for either crate. 6, 7, 9: no code, no
   dependency change. 8. Docs: M1-T031 and M1-T032 notes point at the v2 changes.
10. Session boundaries: below. 11. Public repository: `.github/workflows/` unchanged by the
    task; no issue or pull request acted on.

Findings (none is B1 to B5):
1. **REFLD-R003 test (1) cannot pass as defined** (contract-changing, M1-T061). A path "starts
   with `::`" only if the `::` follows neither an identifier nor `>`. In a `proc_macro2` stream
   the leading `::` of `impl ::liminal_reflect::… for X` follows the identifier `impl`, that of
   `-> ::std::string::String` and `=> ::core::option::Option::Some(…)` follows `>`, and those
   in `<T as ::liminal_reflect::…>` and `&dyn ::liminal_reflect::…` follow `as` and `dyn`
   (probe by this review, table in M1-T061). Each such path then breaks the second and fourth
   rule of the test, and every expansion has an `impl … for`. Also: `String` is a "primitive
   type name" (allowed field type of the test inputs) and one of the identifiers the fourth
   rule forbids. Why not blocking: item 6 asked for the rewording and for the check, and both
   were written; the defect is in the wording of a test that M1-T032 implements, none of B1
   to B5 names a spec defect, and the M1-T030 review treated a normative example that does not
   compile the same way. It is recorded as a task that M1-T032 depends on, so M1-T032 cannot
   start against it.
2. Backlog: reflect §3 names core `spec_version` 1 and 2 (core is at 3; text identical).
3. Backlog: derive §3 writes `workspace = true` for `proc-macro2`; no workspace entry exists.
4. Backlog: REFL-R080 fixes `expected` / `actual` only for the missing-value case.

Merge edits by this review (ADR-0026 Decision 6, no task work): `docs/STATUS.md` regenerated
with `cargo xtask status` after the merge of `main`.

Session boundaries (item 10), stated as facts:
- The author's journal names its parent `2026-11-10-session-m1-reviews.md`. That entry (branch
  `session/2026-11-10-m1-reviews`, "in progress", read at `b32bbab`) lists the author as row 12,
  "impl template". It does not list this review yet.
- This review has the same parent. My prompt was the ADR-0026 review template verbatim, with
  nothing added. The worktree's `AGENTS.md` and the copy in my context forbade a review by a
  sibling; `main` carries ADR-0029, which allows it, and the text on `main` is in force. I read
  that from the repository.
- ADR-0029 was written and merged by this parent session on an owner instruction that the ADR
  quotes. I cannot check the quote. If the owner did not give it, this review is void under
  ADR-0026 Decision 5.1 and the task needs a review by another session.
- M1-T057 had one implementation sub-agent and this one review sub-agent. Neither ran an
  install; I spawned no agent.
