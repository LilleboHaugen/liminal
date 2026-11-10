# Code style

Rules for Rust code in Liminal. Where a lint can enforce a rule, it is enforced through
`[workspace.lints]` in the root `Cargo.toml` (task M0-T001). Everything else is checked in review.

## Toolchain and edition

- Rust stable, pinned in `rust-toolchain.toml`. Edition 2024. Nightly only for Miri, fuzzing and the
  API snapshot tool, never needed to build the engine.
- Toolchain upgrades are dedicated tasks: update the pin, fix new lints, and record it in the journal.

## Lints (workspace level)

- All warnings are errors in CI (`-D warnings`).
- `missing_docs = "deny"`: every public item is documented from the first commit.
- `unsafe_op_in_unsafe_fn = "deny"`, `clippy::undocumented_unsafe_blocks = "deny"`.
- `clippy::unwrap_used = "deny"` (allowed in tests through `clippy.toml`). `clippy::expect_used` is
  **not** enabled. Instead, every non-test `expect` message MUST start with `"invariant: "` (checked in
  review; may be added to `hygiene` later).
- `clippy::pedantic = "warn"` at priority -1, with specific exceptions listed and justified in
  `Cargo.toml` comments.
- `clippy::dbg_macro`, `clippy::print_stdout`, `clippy::print_stderr = "deny"` at workspace level
  (use `liminal_telemetry`). `[workspace.lints]` cannot tell libraries from binaries, so binary crates
  whose job is to print (`xtask`, `liminal_cli`) allow these two lints at their binary's crate root, with
  `#![allow(clippy::print_stdout, clippy::print_stderr, reason = "…")]`. Cargo does not allow
  per-crate overrides together with `[lints] workspace = true`, so the attribute is the mechanism.
- Deterministic crates (`deterministic = true` in `architecture.toml`): clippy `disallowed-methods`
  bans `f32::sin`, `f32::cos`, `f32::tan`, `f32::powf`, `f32::exp`, `f32::ln` (and `f64`
  equivalents), `std::collections::HashMap::new`, and `std::time::Instant::now`. Use
  `liminal_math::det` and `liminal_core` types instead (ADR-0009).
  **Where it lives:** in the crate's own `<manifest directory>/clippy.toml`. Clippy reads only one
  file per crate, the nearest `.clippy.toml` or `clippy.toml` in the crate's directory or a parent,
  and does not merge it with the root `clippy.toml` (verified with clippy 0.1.99, M1-T011). The
  crate's file therefore repeats every setting of the root `clippy.toml` (the `allow-*-in-tests`
  lines) and adds the `disallowed-methods` list, each entry as
  `{ path = "f32::sin", reason = "ADR-0009: use liminal_math::det" }`. Do not create `.clippy.toml`
  files: within one directory it takes precedence over `clippy.toml`. The crate's first
  implementation task writes the file; `cargo xtask check-arch` (XTASK-R039) fails while a
  deterministic workspace crate's effective configuration misses a path of this list. The root
  `clippy.toml` does not ban these paths, because `xtask` and the non-deterministic crates may use
  them.

## Errors (ADR-0017)

- Recoverable failures return `Result<T, E>` where `E` is the crate's error enum.
- Each error variant has a stable code `LIM-<AREA>-<NNNN>`, a message, structured fields, and a `help`
  hint where a fix is knowable. Codes are never reused or renumbered. Codes are listed in the crate spec.
- Messages say what happened, why, and how to fix it. Example:
  `LIM-ECS-0003: component type `Health` is not registered. help: call `app.register_component::<Health>()` in your module's `build`.`
- Panics only for violated invariants (bugs). Use `expect("invariant: <why this cannot happen>")`.
- No `Box<dyn Error>` and no stringly-typed errors in public APIs.

## `unsafe` (ADR-0018)

- Allowed only in crates with `unsafe = "allow"` in `architecture.toml`. All others use
  `#![forbid(unsafe_code)]`.
- Every `unsafe` block has a `// SAFETY:` comment that names the invariants relied on and why they hold.
- Every `unsafe fn` has a `# Safety` doc section listing the caller's obligations.
- Keep unsafe regions minimal, wrapped in safe abstractions, and tested under Miri. Lock-free structures
  also get loom tests.
- Prefer safe code until a benchmark proves the unsafe version is needed. Record the benchmark in the
  commit message or spec.

## API design for agents

The primary user of every API is an agent with limited context. Optimize for being able to use the
API correctly from its docs alone.

- **One obvious way.** No aliases or duplicate entry points for the same thing.
- **Explicit names.** `translation`, not `pos` or `t`. Units in names or types where ambiguous
  (`duration_secs`, `Meters`).
- **No hidden ordering.** If something must happen before something else, the API either enforces it
  in the type system or returns a clear error. System order is declared explicitly.
- **Simple generics in public signatures.** Avoid deep trait-bound chains and type-level tricks that
  produce unreadable compiler errors. Prefer concrete types and small traits.
- **Validate at boundaries.** Builders and constructors validate and return errors with codes. Hot inner
  loops assume validated data and `debug_assert!` their invariants.
- **Documented defaults.** Every default value is stated in the docs, along with why it was chosen.
- **Examples are tests.** Primary types and functions have `# Examples` doc tests. Agents copy these.

## Performance conventions

- No allocation in per-frame hot paths after warm-up, unless the spec allows it. Use `liminal_memory`
  frame allocators. Allocation counts are tracked by telemetry and tested where specs require it.
- Data-oriented layouts: structure-of-arrays where iteration dominates. Keep hot and cold data apart.
- No optimization without a benchmark showing the gain. Keep the reference implementation (AGENTS.md
  rule 4).
- `#[inline]` only with a benchmark or a cross-crate generic reason.

## Formatting and structure

- `rustfmt` with the repository `rustfmt.toml`. No manual formatting debates.
- Module files: `foo.rs` with submodules in `foo/` (no `mod.rs`).
- Order inside a file: module docs, `use`, public types, impls, private helpers, `#[cfg(test)] mod tests`.
- Crate root (`lib.rs`) contains crate-level docs linking to the spec, `#![forbid(unsafe_code)]` where
  applicable, module declarations and re-exports, and no logic.
- Feature flags are documented in the crate root docs and in the spec. Avoid feature combinations
  that change behaviour silently. Every feature must build alone (`--no-default-features --features x`).
