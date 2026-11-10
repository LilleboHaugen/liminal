---
id: M1-T057
title: liminal_reflect and liminal_reflect_derive spec follow-ups from the M1-T030 review
milestone: M1
status: todo
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

- [ ] Items 1 to 9 are done, or a declined item is written into this task with the reason.
- [ ] The §5.2 example, as changed by item 1, was compiled against a reduced copy of the §5 API
      (or against the crate, if M1-T031 exists), and the handoff says how.
- [ ] Spec changes bump `spec_version` in each changed spec with a history row; requirement IDs
      added or moved are in exactly one task's `requirements` frontmatter.
- [ ] `cargo xtask ci` passes.

## Test requirements

None (documentation task). The requirements it touches are tested by M1-T031 and M1-T032.

## Handoff

_Empty._

## Review

_Empty._
