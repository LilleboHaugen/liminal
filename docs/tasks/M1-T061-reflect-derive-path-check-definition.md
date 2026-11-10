---
id: M1-T061
title: Fix the path-check definition of REFLD-R003 (follow-up of the M1-T057 review)
milestone: M1
status: done
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

- [x] Items 1 to 3 are done; the handoff says how item 3 was run.
- [x] `docs/specs/liminal_reflect_derive.md` has a `spec_version` bump with a history row.
- [x] `cargo xtask ci` passes.

## Test requirements

None (documentation task). REFLD-R003 is tested by M1-T032.

## Notes

_None._

## Handoff

2026-11-10 (implementation sub-agent, journal `docs/journal/2026-11-10-M1-T061.md`).

**Done.** `docs/specs/liminal_reflect_derive.md` `spec_version` 2 → 3 (history row). No
requirement ID added or removed; only REFLD-R003 test (1), §3, §10 and two "not in
`spec_version`" references changed.

| Item | Result |
|---|---|
| 1 | Candidate (b), `syn`. The expansion is parsed with `syn::parse2::<syn::File>` and visited with `syn::visit::Visit`. It fails on (a) `use` / `extern crate` items, (b) a `QSelf` without `as`, (c) a macro whose path is not absolute with root `liminal_reflect`, `core` or `std`, or whose body is not a comma-separated expression list (the expressions are visited too), (d) a `syn::Path` that is none of: absolute with a root first; relative with `Self` or a generic parameter first; relative, one segment, not a listed identifier. §3 records `syn` features `full` and `visit` on the normal dependency, with the reason and the unmeasured compile-time effect. Rules (a) and (b) are new: (a) because a `use` tree is not a `syn::Path`; (b) because `syn` stores `<T>::f` as an *absolute* path with first segment `f` (probe output below), which (d) would reject with a misleading message. The test also runs the checker on two failing streams so it cannot be vacuous. |
| 2 | `String` excluded from the field types of the test inputs (field types are primitive types other than `String`, or generic parameters). No exemption for copied tokens. |
| 3 | Probe, see below. |

**How item 3 was run.** A throwaway Cargo binary in the session scratchpad (not committed):
`syn` =3.0.9 (features `derive`, `parsing`, `printing`, `proc-macro`, `full`, `visit`), `quote`
=1.0.47, `proc-macro2` =1.0.107, `cargo run --offline`, macOS arm64, Rust 1.99.1. It holds
hand-written expansions (`impl Reflect`, `Typed`, `ReflectFields`, and `ReflectEnum` for the
enum, every path absolute) of `struct Health { current: u32, max: u32 }`,
`struct Pos(f32, #[reflect(ignore)] u8, f64)`, `enum Figure { Empty, Point(f32, f32), Rect { w:
f32, h: f32 } }` and `struct Pair<T, U> where T: Clone { a: T, b: U }`, the checker of the new
definition, and the token rule of `spec_version` 2.

| Check | Result |
|---|---|
| `spec_version` 2 rule on the four correct expansions | 32, 32, 45, 39 listed identifiers reported outside a `::` path; leading `::` not counted after `impl`, `as`, `dyn`, `->`, `=>`, and `>` closing `impl<…>` (the review's finding reproduced) |
| New check on the four correct expansions | all pass |
| New check, 12 wrong expansions | all fail: `::liminal_core::…` path; bare `None`; bare `Option` in a return type; bare `Some`; `<Self as Typed>::…`; bound `T: Typed`; `format!` without path; `::liminal_core::…` inside `::std::format!` arguments; `Vec::new()`; `use ::liminal_core::x as _;` in a fn body; `<U>::type_path()`; field type `String` |
| `syn` representation | `<T as ::liminal_reflect::value::Typed>::type_path` → absolute, segments `liminal_reflect, value, Typed, type_path`, qself position 3; `<T as Typed>::type_path` → relative `Typed, type_path`; `<T>::type_path` → absolute (!) `type_path`, position 0 |

The checker in the probe, for M1-T032 to reuse: a `Visit` impl that overrides `visit_path`
(rule d, then `syn::visit::visit_path`), `visit_macro` (rule c on `m.path`, then visit the
path's segment arguments, then `m.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated)`
and `visit_expr` on each), `visit_qself` (rule b, then `syn::visit::visit_qself`),
`visit_item_use` and `visit_item_extern_crate` (rule a). `visit_macro` must not call
`visit_path` on the macro path, or a correct `::core::concat!` path is checked twice (harmless)
and a relative macro path is reported as a plain path instead of as a macro.

**CI.** `cargo xtask ci` on macOS arm64: pass, 12 of 12 steps, none skipped.

**Not done / not verified.** The hand-written expansions were parsed by `syn` but not compiled
against a `liminal_reflect` crate (none exists). The compile-time effect of `full` and `visit`
is not measured. Not verified on Windows or Linux.

**Traps.** Generated code that needs `vec![x; n]` or another non-expression macro body fails
rule (c); write such code without that macro form, or change the spec.

**Next step.** Review of M1-T061; then M1-T032 (its other dependency, M1-T031, must be `done`).

## Review

2026-11-10, review sub-agent (ADR-0026, ADR-0030; journal `docs/journal/2026-11-10-M1-T061-review.md`),
rules of `origin/main` at `c891029`. macOS 27.2 arm64, Rust 1.99.1.

**Outcome: done.** No blocking finding.

| Check | Result |
|---|---|
| Item 1 | Candidate (b) chosen and stated exactly in REFLD-R003 test (1): rules (a) to (d), the definition of *absolute* and *root*, the `QSelf` storage, which identifiers are not checked. §3 records `full` and `visit` with the reason. `syn-3.0.9` (the `Cargo.lock` version) declares both features (read in its `Cargo.toml`). |
| Item 2 | `String` excluded from the field types of the test inputs, with the reason; no exemption for copied tokens. |
| Item 3 | Re-run independently: own probe (session scratchpad, not committed; `syn` =3.0.9 with the six features, `quote` =1.0.47, `proc-macro2` =1.0.107, `--offline`), checker written from the spec text, not from the handoff. A hand-written expansion of `Pair<T, U> where T: Clone` (impl generics with bounds, `-> ::std::string::String`, `=>` arms with `::core::option::Option::Some`, `&dyn ::liminal_reflect::…`, `<T as ::liminal_reflect::value::Typed>::type_path()`, `::std::format!`, `::core::module_path!`, `::std::vec!`, `Self { a, .. }` pattern, `#[automatically_derived]`) passes. Fail, each with the expected rule: `::liminal_core::x()`, bare `Option<u8>`, `None::<u8>`, `<T as Typed>::…`, `<T>::…`, `::liminal_core` inside `::std::format!`, bare `format!`, `use … as _;`, `Vec::new()`, field type `String`, bound `T: Typed`. `syn` representation of `<T>::f` (absolute, 1 segment, position 0) and of `<T as ::liminal_reflect::value::Typed>::f` (absolute, 4 segments, position 3) confirmed as the spec says. |
| Criteria 2, 3 | `spec_version` 2 → 3 with history row. `cargo xtask ci` on the task branch with `origin/main` `c891029` merged and the review edits: pass, 12 of 12 steps, none skipped. |
| Scope | Diff touches only REFLD-R003 test (1), §3, §10, two `spec_version` mentions in §2 and §12, the history row, the M1-T032 Notes and `docs/STATUS.md`. Test (2) and the derive design unchanged. |
| Spec agreement | M1-T032 (`todo`) Notes and `depends_on` point at this task; §10 row matches the requirement. |
| Session boundaries | Implementation sub-agent of the parent on `session/2026-11-10-m1-t014-review` (its journal is written at the parent's handoff, not yet on any branch: not verifiable here); this reviewer did not work on the task; review prompt is the ADR-0030 Decision 7 template verbatim. No install, no workflow change. |

Not checked: the compile-time effect of `full` and `visit` (the handoff says unmeasured; the
spec says so too); Windows and Linux.

Backlog: 2 entries under "M1-T061 — review 2026-11-10". Follow-up task: none.
