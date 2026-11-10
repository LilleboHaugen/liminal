---
id: M1-T061
title: Fix the path-check definition of REFLD-R003 (follow-up of the M1-T057 review)
milestone: M1
status: todo
crates: [liminal_reflect_derive]
depends_on: [M1-T057]
requirements: []
---

# M1-T061: Fix the path-check definition of REFLD-R003 (follow-up of the M1-T057 review)

## Goal

Test (1) of REFLD-R003 is defined so that a correct expansion passes it and an expansion that
names another crate fails it.

## Context

- Contract-changing follow-up under ADR-0029 Decision 2.3, written by the review of M1-T057 (see
  that task's "Review"). It changes REFLD-R003, which M1-T032 (`todo`) implements; M1-T032
  lists this task in `depends_on`.
- `docs/specs/liminal_reflect_derive.md` `spec_version` 2, REFLD-R003 and §10.
- The defect. REFLD-R003 test (1) says a path "starts with `::`" when its first `::` follows
  neither an identifier nor `>`. In a `proc_macro2` token stream keywords are identifiers and
  `->`, `=>` and the end of `impl<…>` are `>` tokens. So the leading `::` of each of these is
  not counted as a path start:

  | Generated code | Token before the leading `::` |
  |---|---|
  | `impl ::liminal_reflect::value::ReflectFields for X` | `impl` |
  | `impl<T: …> ::liminal_reflect::value::Typed for Pair<T>` | `>` |
  | `fn type_path() -> ::std::string::String` | `>` (of `->`) |
  | `0usize => ::core::option::Option::Some(…)` | `>` (of `=>`) |
  | `<T as ::liminal_reflect::value::Typed>::type_path()` | `as` |
  | `&dyn ::liminal_reflect::value::Reflect` | `dyn` |

  Checked by the review with `proc-macro2` 1.0.107 and `quote` 1.0.47 (offline probe, not
  committed): a walk over such an expansion reports exactly these predecessors. Under the
  definition each of these paths then breaks the second rule (a path of two or more segments
  that does not start with `::` has `Self` or a generic parameter first) and the fourth
  (`Reflect`, `Typed`, `Option`, `Some`, `None` outside a path that starts with `::`). Every
  expansion contains `impl … for`, so no expansion can pass the test as written.
- Second, smaller point in the same test: its inputs have "primitive type names" as field
  types. `String` is a primitive of `liminal_reflect` (REFL-R003) and is also one of the
  identifiers the fourth rule forbids outside an absolute path; a field of type `String` puts
  the user's `String` token into the expansion.

## Scope

**In:**

1. Redefine the check of REFLD-R003 test (1). Two candidates; pick one and state it exactly:
   (a) token walk: a `::` is leading when the token before it is not a non-keyword identifier
   and not a `>` that closes a generic argument list (`->` and `=>` are recognisable by the
   joint `-` or `=` before the `>`; say how `impl<…>` and `Vec<T>::new` are told apart, or
   forbid the latter form in generated code); or
   (b) parse the expansion with `syn` and visit every `syn::Path`, `syn::Macro` and
   `syn::QSelf`: `leading_colon` is set and the first segment is `liminal_reflect`, `core` or
   `std`, or the path is `Self…`, a generic parameter, or a single segment that is not one of
   the listed identifiers. (b) needs `syn` features beyond the four of §3 (`full`, `visit`);
   record them in §3.
2. Exclude `String` from the field types of the test inputs, or exempt tokens copied from the
   input from the fourth rule, and say which.
3. Check the new definition against a hand-written expansion of the four input kinds the
   requirement lists (probe outside the repository, or in the crate if M1-T032 has started),
   and against one expansion that names `liminal_core` or a bare `Option`, which must fail.

**Out:** Any other change to REFLD-R003 test (2) or to the derive's design.

## Acceptance criteria

- [ ] Items 1 to 3 are done; the handoff says how item 3 was run.
- [ ] `docs/specs/liminal_reflect_derive.md` has a `spec_version` bump with a history row.
- [ ] `cargo xtask ci` passes.

## Test requirements

None (documentation task). REFLD-R003 is tested by M1-T032.

## Notes

_None._

## Handoff

_Filled in by the implementing session(s): date, what is done, what is not, the exact next step,
traps found._

## Review

_Filled in by the reviewing session: date, what was checked, outcome._
