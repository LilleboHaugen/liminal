# Dependencies

Decision record: ADR-0003. The machine-readable registry is the `[external.*]` table in
`architecture.toml`, enforced by `cargo xtask check-arch`.

## Default: build it ourselves

Liminal builds its own engine systems so each can be designed and optimized for this engine. An
external crate is used only when one of these holds:

1. **Boundary code:** it wraps an OS or driver interface where writing our own adds cost but no speed or
   control (windowing, raw API bindings, audio device access).
2. **Compile-time only:** proc-macro infrastructure (`syn`, `quote`) that never runs in a game.
3. **Testing and tooling:** dev-dependencies and tool-only crates (`proptest`, `loom`, `serde` in
   `xtask`) that never ship in a game.
4. **Not yet worth replacing:** a runtime crate that is not a bottleneck, used behind a Liminal-owned
   interface so it can be swapped later without touching callers.

An external crate is replaced by our own implementation when either:
- a benchmark or profile shows it is a bottleneck, or
- an agent can show, through a prototype with tests and benchmarks, that our own version is better on
  a criterion that matters (speed, control, determinism, compile time).

The replacement follows the normal process: ADR that supersedes the original, spec, tests, and a
differential test against the old implementation where possible.

## Scopes and their bars

| Scope | Meaning | Bar |
|---|---|---|
| `runtime` | Compiled into games | ADR required. Must be behind a Liminal interface. Licence-compatible. Maintained. Audited for `unsafe` and transitive dependencies |
| `build` | Proc-macros and build scripts only | ADR required. Licence-compatible |
| `dev` | Tests and benches only | ADR (may be a shared ADR such as ADR-0010). Licence-compatible |
| `tool` | Layer 5 crates only | ADR (may be shared). Licence-compatible |

## Adding a dependency (procedure)

1. Check that no approved crate already covers the need.
2. Write an ADR (or extend a shared one, for `dev` and `tool` scopes) covering the need, alternatives
   including building it ourselves, licence, maintenance status, transitive dependency count, and
   `unsafe` usage.
3. Add the `[external.<name>]` entry and add the name to the using crate's `external` or
   `dev_external` list in `architecture.toml`.
4. Add it to `[workspace.dependencies]` in the root `Cargo.toml` as a caret requirement on
   `major.minor` (e.g. `serde = "1.0"`), and inherit it in the crate with `{ workspace = true }`.
   `Cargo.lock` is committed. CI builds with `--locked`, so resolved versions change only through a
   commit.
5. `cargo deny check` must pass (licences, advisories, duplicate versions).

## Licence policy

Liminal is MIT OR Apache-2.0. Allowed dependency licences: MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause,
ISC, Zlib, Unicode-3.0, CC0-1.0, MPL-2.0 (only for tool and dev scopes). Copyleft licences (GPL, LGPL,
AGPL) are not allowed in any scope that is distributed. Enforced through `deny.toml` (task M0-T009).

## External tools (not crates)

Tools the checks need (installed only with the owner's approval, ADR-0022): `cargo-llvm-cov`, `cargo-deny`, `cargo-nextest` (optional), Miri
(nightly component), `cargo-fuzz`, Vulkan SDK (for validation layers), and later the shader toolchain
(OQ-001). `liminal doctor` (M2) and `cargo xtask doctor` (M0) report which tools are missing and how
to install them. An agent asks the owner before installing any of them, giving the tool, the exact
command, the reason, where it installs and how to undo it.
