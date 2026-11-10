---
crate: liminal_reflect_derive
req_prefix: REFLD
layer: 0
maturity: planned
spec_version: 3
last_reviewed: 2026-11-10
adrs: [ADR-0003, ADR-0017, ADR-0018]
---

# Spec: `liminal_reflect_derive`

## 1. Purpose

`liminal_reflect_derive` provides `#[derive(Reflect)]`, which generates the `liminal_reflect` trait
impls (`Reflect`, `Typed`, `ReflectFields`, and `ReflectEnum` for enums) for structs and enums, so
that game and engine types become reflectable without hand-written boilerplate. It is a separate
crate because Rust requires procedural macros to live in a `proc-macro` crate. Its contract is the
behaviour of the generated code, which [`docs/specs/liminal_reflect.md`](liminal_reflect.md) defines for impls in
general, plus the input the macro accepts and the compile errors it reports. Users never depend on
this crate directly: `liminal_reflect::value::Reflect` re-exports the macro (REFL-R120).

## 2. Scope

**In scope:**
- The derive macro `Reflect` for named structs, tuple structs, unit structs and enums (unit, tuple
  and struct variants), with generic type parameters.
- The attributes `#[reflect(type_path = "…")]` (on the type) and `#[reflect(ignore)]` (on a field).
- Default type paths of derived types (REFLD-R020).
- Compile errors for rejected input, each with a stable code `LIM-REFLD-NNNN` (§8).

**Out of scope / non-goals:**
- The traits, descriptors, field access semantics, registry and state encoding:
  `liminal_reflect` ([`docs/specs/liminal_reflect.md`](liminal_reflect.md)). The derive generates impls that meet those
  requirements; it defines none of its own runtime behaviour.
- Unions, lifetime parameters and const generic parameters (rejected, §8; §12 item 1).
- Other derives (`Default`, serialization): not in this crate.
- Renaming the `liminal_reflect` dependency in a user's `Cargo.toml` (`#[reflect(crate = …)]`):
  not in `spec_version` 3 (§12 item 2).

## 3. Dependencies

- Engine crates: none (`architecture.toml`: layer 0, `same_layer_deps = []`). The generated code
  names `::liminal_reflect`, which the user's crate depends on; the derive crate itself does not.
  It cannot have `liminal_reflect` as a dev-dependency (XTASK-R032, no same-layer edge), so tests
  that compile derived types live in `liminal_reflect` (§10).
- External crates: `syn`, `quote`, `proc-macro2`, scope `build` (ADR-0003, `architecture.toml`
  `external`). `syn` features: `derive`, `parsing`, `printing`, `proc-macro`, and `full` and `visit`
  (`spec_version` 3), all on the crate's normal dependency.
  M1-T032 records any further feature here by a spec update.
- `syn` features `full` and `visit`: the unit test of REFLD-R003 test (1) parses the expansion as
  a `syn::File` (needs `full`: items, expressions, statements, patterns) and walks it with
  `syn::visit::Visit` (needs `visit`). The derive's own parsing of `syn::DeriveInput` needs only
  `derive`. They are on the normal dependency for the reason given for `span-locations` below
  (XTASK-R034: no dev-dependency on a crate of scope `build`); no registry change and no ADR, as
  for `span-locations`. Consequence: Cargo unifies features, so every build that contains the
  derive compiles its `syn` with `full` and `visit`, which enlarges that compile; the effect has
  not been measured. Checked with `syn` 3.0.9 (the version in `Cargo.lock`), `quote` 1.0.47 and
  `proc-macro2` 1.0.107 by a probe outside the repository (M1-T061 handoff): `syn` 3.0.9 has both
  features, and the check passes hand-written expansions of the four input kinds of REFLD-R003
  and fails expansions that name `::liminal_core` or bare `Option`, `Some`, `None`, `Vec`,
  `String`, a relative trait or macro, or contain a `use` item.
- `proc-macro2` feature `span-locations`: enabled, on the crate's normal dependency
  (`proc-macro2 = { workspace = true, features = ["span-locations"] }`). The unit tests of
  REFLD-R050 need it: they run outside the compiler, where `proc_macro2::Span::start()` exists
  only with this feature. Checked with `proc-macro2` 1.0.107 (the version in `Cargo.lock`), source
  and a probe program: without the feature `Span` has no `start()`, its `Debug` output is `Span`
  for every span, and it has no `PartialEq`, so the tokens of an error cannot be matched with the
  input's tokens by span either. With the feature, the tokens of
  `syn::Error::to_compile_error()` report the line and column of the token the error was created
  with. Why not a dev-dependency: XTASK-R034 accepts only crates of scope `dev` as
  dev-dependencies, `proc-macro2` has scope `build`, and `dev_external` stays `[]`.
  `architecture.toml` and `docs/process/DEPENDENCIES.md` have no rule for features of an approved
  crate, and ADR-0003 approves the crate, so no registry change and no ADR are needed.
  Consequences: Cargo unifies features, so every build that contains the derive compiles its one
  `proc-macro2` with the feature on, for all proc macros of that build. The effect on compile
  time has not been measured. Inside the compiler on a stable toolchain `start()` carries no
  usable value (`proc-macro2` documentation), so the derive itself never calls it; only tests do.
- `unsafe = "forbid"` (ADR-0018): the crate root has `#![forbid(unsafe_code)]`.
- `deterministic = false` in `architecture.toml` (it runs at compile time, not in simulation). Its
  output is still a pure function of its input (REFLD-R070).
- Structure: `src/lib.rs` holds only the `#[proc_macro_derive(Reflect, attributes(reflect))]` entry
  point, which converts `proc_macro::TokenStream` to `proc_macro2::TokenStream` and calls a private
  `expand(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream`. Parsing, validation and
  generation are in modules below `lib.rs`, operate on `proc_macro2` types only, and are unit-tested
  in the crate, so `cargo llvm-cov` measures them (proc-macro code that runs inside the compiler is
  not measured).

## 4. Concepts

- **Accepted input:** a `struct` (named, tuple, unit) or an `enum` with at least one variant, with
  zero or more generic type parameters, an optional `where` clause, and the `reflect` attributes of
  §5. Everything else is rejected with a code (§8).
- **Reflected field:** a field without `#[reflect(ignore)]`, whatever its visibility (`pub`,
  `pub(crate)`, private). Reflected fields keep their declaration order and get consecutive indices
  from 0 (per struct, or per enum variant). Reflection thus reads and writes private fields of a
  derived type from any crate (`field_mut`); `#[reflect(ignore)]` is the only way to keep a field
  out (`liminal_reflect` spec §4 "Visibility").
- **Base path:** the part of the type path before the generic arguments: `module_path!()` + `::` +
  the type's identifier without `r#`, or the `type_path` attribute's value.
- **Generated block:** all generated items are trait impls inside one `const _: () = { … };` block,
  so no names are added to the user's module.

## 5. Public API

The crate's only public item is the derive macro. Its accepted syntax is normative:

```rust
#[proc_macro_derive(Reflect, attributes(reflect))]
pub fn derive_reflect(input: proc_macro::TokenStream) -> proc_macro::TokenStream;
```

```rust
use liminal_reflect::value::Reflect;

#[derive(Reflect)]
#[reflect(type_path = "my_game::Health")]     // optional, at most once, on the type
struct Health {
    current: u32,
    max: u32,
    #[reflect(ignore)]                          // optional, on any field (named or tuple)
    cache: SomeNonReflectType,
}

#[derive(Reflect)]
enum State<T> {                                 // generic type parameters: `T: Typed` is added
    Idle,                                       // unit variant
    Moving(T, f32),                             // tuple variant: fields "0", "1"
    Fleeing { from: u32 },                      // struct variant
}
```

Generated for a type `X` with generic type parameters `P1 … Pn` (n ≥ 0): `impl<…> Reflect for X<…>`,
`impl<…> Typed for X<…>`, `impl<…> ReflectFields for X<…>`, and for enums also
`impl<…> ReflectEnum for X<…>`, each with the type's own generics and `where` clause plus the bound
`Pi: ::liminal_reflect::value::Typed` for every type parameter. Paths in generated code are absolute
(`::liminal_reflect::…`, `::core::…`, `::std::…`).

## 6. Requirements

### Accepted input and generated items (R001–R019)

| ID | Level | Requirement |
|---|---|---|
| REFLD-R001 | MUST | `#[derive(Reflect)]` on a named struct, tuple struct, unit struct, or an enum with at least one variant (any mix of unit, tuple and struct variants) generates impls of `Reflect`, `Typed` and `ReflectFields`, and for enums also `ReflectEnum`, and nothing else; the impls are inside one `const _: () = { … };` block. The generated code compiles without warnings under the workspace lints (`cargo clippy --all-targets -- -D warnings`), checked by derived test types in `liminal_reflect`'s tests. |
| REFLD-R002 | MUST | For each generic type parameter `P` of the input, the generated impls carry the bound `P: ::liminal_reflect::value::Typed` in addition to the input's own bounds and `where` clause, which are kept unchanged. A type with two type parameters, one of them with its own bound (`T: Copy`), derives and works for `T = u8, U = String`. |
| REFLD-R003 | MUST | Generated code names items only through absolute paths starting with `::liminal_reflect`, `::core` or `::std`. Two tests. (1) Unit test of `expand`, on inputs whose field types are Rust primitive types other than `String` (`bool`, the integer types, `f32`, `f64`, `char`) or the input's own generic type parameters (a named struct, a tuple struct, an enum with all three variant kinds, a generic type with two parameters); `String` is excluded because field types and `where` clauses are copied into the expansion as written, so a field of type `String` puts a bare `String` into it. The test parses the output with `syn::parse2::<syn::File>` (a parse error fails the test) and visits the file with `syn::visit::Visit` (`syn` features `full` and `visit`, §3). A path is *absolute* when its `leading_colon` is set, and a *root* is one of `liminal_reflect`, `core`, `std`. The test fails, naming the offending item, on: (a) any `use` or `extern crate` item (`ItemUse`, `ItemExternCrate`; a `use` tree is not a `syn::Path`, so rule (d) would not see it); (b) any `syn::QSelf` without `as` (`<T>::f`: `syn` stores it as an absolute path whose first segment is `f`); (c) any `syn::Macro` whose path is not absolute with a root as first segment, or whose body does not parse as `Punctuated<syn::Expr, syn::Token![,]>::parse_terminated`; the parsed expressions are visited by rules (a) to (d), so paths inside `::std::format!(…)` and `::std::vec![…]` are checked; (d) any other `syn::Path` (every path `Visit::visit_path` reaches: types, expressions, patterns, trait bounds, attributes, visibilities) that is none of: absolute with a root as first segment; not absolute with `Self` or one of the input's generic type parameters as first segment; not absolute, of one segment, and not one of the identifiers `Reflect`, `Typed`, `TypeInfo`, `Shape`, `Option`, `Some`, `None`, `Result`, `Ok`, `Err`, `Vec`, `String`. `syn` stores the trait of a qualified path as the leading segments of the path, so `<T as ::liminal_reflect::value::Typed>::type_path()` is the absolute path `::liminal_reflect::value::Typed::type_path` and passes, and `<T as Typed>::type_path()` is the path `Typed::type_path` and fails. Identifiers that are not part of a path (field names, method names, bindings, lifetimes) name no item and are not checked. The same unit test also runs the check on two hand-written token streams, one that names `::liminal_core::…` and one that contains a bare `Option`, and asserts that it fails on each, so the check is shown not to be vacuous. This is the check that the generated code depends on no crate but `liminal_reflect` and the standard library. (2) Integration test in `crates/liminal_reflect/tests/`: a file that names no crate other than `liminal_reflect` (no path, `use` or `extern crate` naming `liminal_reflect_derive`, `liminal_core` or `proptest`) declares local items named `Reflect`, `Typed`, `TypeInfo`, `Shape`, `Option`, `Some`, `None`, `Result`, `Ok`, `Err`, `Vec` and `String` and derives without errors. Test (2) shows that shadowing does not break the generated code; it cannot show that a dependency is absent, because an integration test is compiled with every dependency and dev-dependency of `liminal_reflect` available. |
| REFLD-R004 | MUST | Field names: a named field's identifier with any `r#` prefix removed (`r#type` → `"type"`); a tuple field's position among the input's fields in decimal, counted before ignored fields are removed (`struct T(u8, #[reflect(ignore)] C, u16)` has fields `"0"` and `"2"`). Reflected fields keep declaration order and have consecutive indices from 0. Visibility plays no part: a private field without `#[reflect(ignore)]` is reflected like a `pub` field (test type with private fields defined in a submodule of the test file, read and written by name from outside that module). |
| REFLD-R005 | MUST | Recursive types derive and their `Typed::type_info()` returns (it does not call `type_info()` of field types): `struct Node { value: u32, children: Vec<Node> }` has a descriptor whose `children` field has `TypeRef` path `"Vec<" + Node::type_path() + ">"`. |

### Type paths (R020–R029)

| ID | Level | Requirement |
|---|---|---|
| REFLD-R020 | MUST | Without a `type_path` attribute, `Typed::type_path()` returns `module_path!()` of the module that contains the type definition, `::`, and the type's identifier without `r#`. With generic type parameters `P1 … Pn` (n ≥ 1) it appends `<`, then `P1::type_path()` … `Pn::type_path()` joined by `", "`, then `>`. Example: `Pair<u8, String>` defined in module `game::grid` → `game::grid::Pair<u8, String>`. |
| REFLD-R021 | MUST | `#[reflect(type_path = "…")]` on the type replaces the base path (generic arguments are still appended by REFLD-R020). The string literal is accepted exactly when it matches `^[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_]*)+$` (at least two segments, so it never equals a primitive or container path of `liminal_reflect`); otherwise LIM-REFLD-0006. |

### Generated behaviour (R030–R049)

| ID | Level | Requirement |
|---|---|---|
| REFLD-R030 | MUST | `Typed::type_info()` returns `TypeInfo { type_path: Self::type_path(), id: Self::stable_type_id(), shape }` with `shape` `Struct { kind, fields }` for structs (`kind` `Named`, `Tuple` or `Unit` by the struct's syntax) or `Enum { variants }` for enums (variants in declaration order; each `VariantInfo` has the variant's identifier without `r#`, its kind by syntax, and its reflected fields). Each `FieldInfo` has the name of REFLD-R004 and `ty: TypeRef::of::<F>()` for the field's type `F`. `Reflect::reflect_type_info` returns `Self::type_info()`. |
| REFLD-R031 | MUST | `reflect_ref` and `reflect_mut` return `Struct(self)` for structs and `Enum(self)` for enums. For enums, `variant_index()` is the current variant's 0-based declaration position (explicit discriminants are ignored) and `variant_name()` its identifier without `r#`. |
| REFLD-R032 | MUST | `ReflectFields` of a derived struct, or of an enum's current variant: `field_count()` is the number of reflected fields; `field_name(i)`, `field_at(i)` and `field_at_mut(i)` return the reflected field with index `i` for `i < field_count()` and `None` otherwise. A unit struct and a unit variant have 0 fields. |
| REFLD-R033 | MUST | Differential test: for each derived test type in `crates/liminal_reflect/tests/` (named, tuple and unit struct; enum with all three variant kinds; generic type with two parameters; type with ignored fields; recursive type), a hand-written reference impl of a structurally identical twin type (written per [`docs/specs/liminal_reflect.md`](liminal_reflect.md), kept from M1-T031) gives a `type_info()` equal to the derived type's in every part, `type_path` and `id` included. To make that hold, the twin's hand-written `Typed::type_path()` returns the derived type's path: for a non-generic twin `<Derived as Typed>::type_path()`, for a generic twin `PairTwin<T, U>` the path of `Pair<T, U>` with the same arguments. A twin that refers to itself refers to its twin type (`NodeTwin { value: u32, children: Vec<NodeTwin> }`), so the `TypeRef` of `children` has the path `"Vec<" + Node::type_path() + ">"` and the same ID as in the derived descriptor. A derived type and its twin share a stable type ID and are therefore never registered in one `TypeRegistry` (that would be LIM-REFL-0001, REFL-R062); each test registers them in separate registries. `validate_reflect` returns `Ok` for both, and for proptest-generated values `reflect_state_hash`, `field_count`, `field_name`, and the result of every `field(name)` / `field_mut(name)` request (value read back, or the whole `ReflectError` value, which is equal because the paths are) are equal. |
| REFLD-R034 | MUST | A field with `#[reflect(ignore)]` is absent from `type_info()`, has no index, is not returned by `field_at`, `field`, or `field_mut` (requests for its name return LIM-REFL-0010 or LIM-REFL-0013 per REFL-R043), and is not written by `write_reflect`: changing only that field leaves `reflect_state_hash` unchanged. Its type need not implement `Typed`. |
| REFLD-R035 | MUST | The derived impls meet every requirement that [`docs/specs/liminal_reflect.md`](liminal_reflect.md) states for impls of `Reflect` and `Typed` (REFL-R002 identities, REFL-R041, REFL-R043 to REFL-R045 field access, REFL-R061 descriptor validity, REFL-R080 consistency): property tests run `validate_reflect`, the field round trip of REFL-R045 and `TypeRegistry::register` on every derived test type of REFLD-R033. |

### Diagnostics (R050–R069)

| ID | Level | Requirement |
|---|---|---|
| REFLD-R050 | MUST | Each rejected input of §8 makes the derive emit a compile error whose message starts with its code and `: ` (for example `LIM-REFLD-0001: `) followed by the message of §8, spanned at the token named in §8's "Span" column. Verified twice: unit tests of `expand` assert the message and span start for each code (the test parses the input from a string literal with `proc_macro2::TokenStream::from_str`, calls `expand`, and for each `compile_error` invocation in the output compares the message literal with the expected text and `span().start()` of the `compile_error` identifier token with the hand-written line (1-based) and column (0-based) at which the token (or the first token) of §8's "Span" column starts in the input string; this needs the `proc-macro2` feature `span-locations`, §3); one rustdoc `compile_fail` doc test per code in `liminal_reflect` (on the `Reflect` re-export) proves that compilation fails. |
| REFLD-R051 | SHOULD | All diagnostics of one derive input are reported together, in source order (one compile run shows every attribute error of the type), not only the first. |
| REFLD-R052 | MUST | A reflected field whose type does not implement `Typed` makes compilation fail (rustc trait-bound error E0277 naming `Typed`), and the derive emits no LIM-REFLD error for it. Proved by a `compile_fail` doc test in `liminal_reflect`. |
| REFLD-R053 | SHOULD | The E0277 error of REFLD-R052 points at the field's type (generated bounds and calls carry the field type's span). Checked by review of the expansion and one manual compile, recorded in M1-T032's handoff. |

### Determinism of the expansion (R070–R079)

| ID | Level | Requirement |
|---|---|---|
| REFLD-R070 | MUST | `expand` is a pure function of its input tokens: expanding the same input twice gives token streams with identical `to_string()` output, and the output does not depend on environment variables, the file system, time or iteration order of a hash map (unit test over every derived test input, run twice; review checks that no `HashMap`/`HashSet` is used). |

### Assignment to implementation tasks

Every requirement is assigned to exactly one task; the IDs are in the task's `requirements`
frontmatter.

| Task | Requirements |
|---|---|
| M1-T032 (derive, determinism, `usable` evidence) | REFLD-R001 to REFLD-R005, REFLD-R020, REFLD-R021, REFLD-R030 to REFLD-R035, REFLD-R050 to REFLD-R053, REFLD-R070 |

## 7. Invariants

- INV-1: For every accepted input, the generated `type_info()` lists exactly the reflected fields
  that `field_at` returns, with the same names and order. (Generated from one field list in `expand`;
  REFLD-R033, REFLD-R035.)
- INV-2: Every rejected input produces at least one `LIM-REFLD` diagnostic and no impls (the
  derive emits only the errors). (Unit tests of REFLD-R050.)
- INV-3: The output of `expand` for an accepted input contains no items outside the
  `const _: () = { … };` block. (Unit test on the output tokens; REFLD-R001.)

## 8. Error codes

`liminal_reflect_derive` has no runtime errors: it runs inside the compiler and has no
`liminal_core` dependency, so it does not implement `EngineError`. Its rejections are compile
errors. They carry stable codes in the ADR-0017 form (`LIM-<req_prefix>-NNNN`) so an agent can look
them up here. Messages are normative; `{x}` is filled from the input.

| Code | Rejected input | Message (after `LIM-REFLD-NNNN: `) | Help (appended as ` help: …`) | Span |
|---|---|---|---|---|
| LIM-REFLD-0001 | `union` | `#[derive(Reflect)]` does not support unions. | use a struct or an enum. | `union` keyword |
| LIM-REFLD-0002 | enum with no variants | `#[derive(Reflect)]` needs at least one variant in enum `{name}`. | add a variant, or do not derive `Reflect` for an uninhabited type. | enum identifier |
| LIM-REFLD-0003 | lifetime parameter | `#[derive(Reflect)]` does not support lifetime parameter `{lifetime}`: reflected types are `'static`. | store owned data (`String`, `Vec<T>`) instead of references. | the lifetime parameter |
| LIM-REFLD-0004 | const generic parameter | `#[derive(Reflect)]` does not support const parameter `{name}`. | use a fixed-size array field `[T; N]` with a literal `N`, or a `Vec<T>`. | the const parameter |
| LIM-REFLD-0005 | unknown or malformed `reflect` attribute (unknown key, missing `= "…"` for `type_path`, value given to `ignore`, not a list) | unknown or malformed attribute `#[reflect({tokens})]`. | the accepted forms are `#[reflect(type_path = "a::B")]` on the type and `#[reflect(ignore)]` on a field. | the attribute's content |
| LIM-REFLD-0006 | `type_path` value not matching REFLD-R021 | type path `{value}` is not a Rust path with at least two segments. | write segments of letters, digits and `_` separated by `::`, for example `my_game::Health`. | the string literal |
| LIM-REFLD-0007 | `type_path` given more than once | `type_path` is given more than once. | keep one `#[reflect(type_path = "…")]`. | the second occurrence |
| LIM-REFLD-0008 | `reflect` attribute in the wrong place (`ignore` on the type or a variant, `type_path` on a field or a variant, any `reflect` attribute on a variant or generic parameter) | `#[reflect({key})]` is not allowed on {place}. | put `type_path` on the type and `ignore` on fields. | the attribute |
| LIM-REFLD-0009 | the same field marked `ignore` twice | field `{name}` is marked `#[reflect(ignore)]` more than once. | keep one `#[reflect(ignore)]`. | the second attribute |

`{place}` is one of `a type`, `a variant`, `a field`, `a generic parameter`. Codes 0010 onward are
unassigned; new codes take the next free number.

## 9. Performance budgets

None yet: set at `stable` from baselines. The derive runs at compile time; its cost shows in build
times, not frame times.

| Scenario | Budget | Machine | Source of number |
|---|---|---|---|

The table lists only scenarios that a task measures, and no existing task measures one for this
crate, so it has no row (one "Not scheduled" row was removed in `spec_version` 2: `cargo build`
time of a crate with 100 derived types of 10 fields each, against the same crate with hand-written
impls). The task that prepares the `stable` promotion of `liminal_reflect_derive` (not written
yet) adds scenarios and rows by a `spec_version` bump; the compile-time effect of the
`proc-macro2` feature `span-locations` (§3) belongs to that measurement.

## 10. Test plan

No optimized path. The oracle for generated code is the hand-written reference impls of
`liminal_reflect`'s tests (REFLD-R033).

| Area | Test kinds | Oracle |
|---|---|---|
| Parsing and validation (R001, R003 test 1, R004, R020, R021, R050, R051, R070) | unit (in `liminal_reflect_derive`, on `expand` with `proc_macro2` token input parsed from a string) | Hand-written expected messages, span starts as line and column in the input string (`span-locations`, §3), field lists and type-path expressions; for R003 test 1, a `syn::visit::Visit` walk over the output parsed as a `syn::File` (§3), checked against two hand-written failing streams; second expansion (R070) |
| Generated behaviour (R002, R003 test 2, R004 visibility, R005, R030–R035) | integration and property tests in `crates/liminal_reflect/tests/` | Hand-written reference impls of twin types that return the derived types' paths (M1-T031, REFLD-R033); `liminal_reflect` requirements |
| Rejections (R050, R052) | rustdoc `compile_fail` doc tests on `liminal_reflect::value::Reflect` (no compile-test crate is approved, M1-T032) | The compiler rejects the input |
| Spans (R053) | review, one manual compile | Compiler output recorded in the M1-T032 handoff |

Every requirement's verifying test carries `/// Verifies: REFLD-Rnnn`. `trace` finds annotations in
every workspace crate, so tests in `liminal_reflect` count for this spec. A `compile_fail` doc test
cannot carry a `Verifies:` line attached to a test function; REFLD-R050 and REFLD-R052 therefore
also have `#[test]` functions (the `expand` unit tests for R050; for R052, a unit test asserting that
`expand` emits no `compile_error!` for a field of a non-`Typed` type) that carry the annotation, and
the doc tests add the compiler-level check. Primary entry point with `# Examples`: the `Reflect`
derive (documented and exemplified on the `liminal_reflect::value::Reflect` re-export).

## 11. Determinism

`architecture.toml` marks the crate `deterministic = false`: it runs in the compiler, not in the
simulation, and the determinism lints of CODE_STYLE.md do not apply. The generated code runs in the
user's (possibly deterministic) crate and contains no clock, RNG, hash map or floating-point
arithmetic; its runtime behaviour is covered by `liminal_reflect` §11. The expansion itself is a
pure function of its input (REFLD-R070), so a type's derived impl is the same in every build and on
every platform. There is no `tests/determinism.rs` for this crate: ADR-0024 applies to crates with
`deterministic = true`; `liminal_reflect`'s workloads use derived types and cover their runtime
behaviour.

## 12. Open questions

1. **Const generics.** `struct Grid<const W: usize>` would need `W` in the type path. Rejected in
   `spec_version` 3 (LIM-REFLD-0004). Revisit when a consumer needs it (voxel chunk types, M6).
2. **Renamed dependency.** Generated code names `::liminal_reflect`; a crate that renames the
   dependency cannot use the derive. Add `#[reflect(crate = "…")]` if a consumer (for example the
   `liminal_engine` facade re-exporting reflection) needs it; resolve in the facade's spec.
3. **Coverage measurement.** The `usable` gate needs 85 % line coverage. This spec assumes that
   unit tests of `expand` in the derive crate are measured by `cargo llvm-cov` like any library
   code. M1-T032 confirms it; if the proc-macro crate type prevents measurement, it records the
   finding and the resolution (a floor on the measured part, or an ADR) before the promotion.

## 13. Maturity

Current: `planned`. Next target: `usable` (via `experimental`), evidence prepared by M1-T032 and
checked by the session that reviews it.

- [ ] Spec follows `docs/specs/_TEMPLATE.md` and passed a spec review (REVIEW.md §2) (evidence:
      review of M1-T030).
- [ ] Compiles, all lints pass, registered in `ci/coverage.toml` (evidence: M1-T032).
- [ ] All MUST requirements traced: `cargo xtask trace` with `maturity: usable` applied on a
      scratch copy reports every REFLD MUST tested (evidence: M1-T032).
- [ ] Line coverage ≥ 85 % (layer 0 pure logic, TESTING.md) (evidence: M1-T032; §12 item 3).
- [ ] Every public item documented; the derive has runnable `# Examples` (on the re-export).
- [ ] Integration tests (in `crates/liminal_reflect/tests/`) use only the public API.
- [ ] Determinism: not a `deterministic = true` crate; REFLD-R070 covers the expansion.
- [ ] At least one example or dependent crate uses it: `liminal_reflect` re-exports and tests it.

## 14. History

| spec_version | Date | Change | By task |
|---|---|---|---|
| 1 | 2026-11-10 | Initial spec | M1-T030 |
| 2 | 2026-11-10 | Follow-ups of the M1-T030 review. REFLD-R003: split into a token check on the expansion (paths start with `::liminal_reflect`, `::core` or `::std`) and a shadowing integration test, worded as what each can show. REFLD-R033: twins return the derived type's path, descriptors are equal without exception, twins are never registered in one registry. REFLD-R050, §3: span assertions use the `proc-macro2` feature `span-locations`, enabled on the normal dependency. §4, REFLD-R004: private fields are reflected. §9: the row without a measuring task removed. References to the `liminal_reflect` spec are links | M1-T057 |
| 3 | 2026-11-10 | Follow-up of the M1-T057 review. REFLD-R003 test (1) redefined: the spec_version 2 token rule (a path starts with `::` when its first `::` follows neither an identifier nor `>`) did not count the leading `::` after `impl`, `as`, `dyn`, `->`, `=>` or the end of `impl<…>`, so no expansion could pass. The test now parses the expansion with `syn` and checks every `syn::Path`, `syn::Macro` and `syn::QSelf`, forbids `use` and `extern crate` items, and is run on two failing streams. `String` is excluded from the field types of its inputs. §3: `syn` features `full` and `visit`. §10 row updated | M1-T061 |
