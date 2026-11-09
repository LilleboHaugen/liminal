---
id: M0-T001
title: Bootstrap the Cargo workspace
milestone: M0
status: done
crates: [xtask]
depends_on: []
requirements: [XTASK-R001, XTASK-R002, XTASK-R011]
---

# M0-T001: Bootstrap the Cargo workspace

## Goal

A Cargo workspace with pinned toolchain, workspace-wide lints and formatting rules, and an `xtask`
crate that answers `cargo xtask help`. Every later task builds on this.

## Context

- ADR-0005 (toolchain, edition 2024, resolver 3), ADR-0015 (naming, licence), ADR-0018 (unsafe policy).
- `docs/process/CODE_STYLE.md` (lints), `docs/specs/xtask.md` (R001, R002, R011).
- Prerequisite handled by the project owner: Rust installed through `rustup`, `git init` done.
  If either is missing, set this task to `blocked` and say so in the handoff.

## Scope

**In:**
- Root `Cargo.toml`: `[workspace]` with `members = ["xtask"]` and `resolver = "3"`;
  `[workspace.package]` (edition 2024, `license = "MIT OR Apache-2.0"`, `rust-version` equal to the
  pinned version, `repository = "https://github.com/LilleboHaugen/liminal"`); `[workspace.lints]` implementing CODE_STYLE.md §Lints; `[workspace.dependencies]`
  with the four `tool`-scope crates xtask will use (exact minor versions).
- `rust-toolchain.toml`: `channel` = the **current stable release at the time of this task, as an exact
  version** (find it with `rustup check` or the Rust release notes). Components `rustfmt`, `clippy`,
  `llvm-tools` (formerly `llvm-tools-preview`). `profile = "minimal"`.
- `Cargo.lock` committed (CI uses `--locked`, XTASK-R012).
- `.gitattributes` already exists (LF line endings, created in planning). Keep it.
- `rustfmt.toml`: `edition = "2024"`, `max_width = 100`, `newline_style = "Unix"`. Stable options only.
- `clippy.toml`: `allow-unwrap-in-tests = true`, `allow-expect-in-tests = true`,
  `allow-print-in-tests = true`.
- `.cargo/config.toml`: `[alias] xtask = "run --package xtask --"` (R001).
- `xtask/`: `Cargo.toml` (inherits workspace package and lints, `publish = false`); a library target
  `src/lib.rs` holding all logic (hand-written argument parsing, a command table later commands plug
  into, a working `help` command, usage errors with exit code 2, R002); a thin `src/main.rs`. Both
  roots have `#![forbid(unsafe_code)]`. `main.rs` (and only it) has
  `#![allow(clippy::print_stdout, clippy::print_stderr, reason = "CLI output is xtask's job")]`.
  Library code returns output as strings or writes to a passed-in `impl Write`, so tests can capture it
  (see CODE_STYLE.md §Lints).

**Out:** the `ci` command and all checks (M0-T002 onward). Engine crates: none are created in M0.
No `liminal_*` crate exists before its spec does. Release profile tuning (M1, with benchmarks).

## Acceptance criteria

- [x] `cargo build --workspace`, `cargo test --workspace`,
      `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check` and
      `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` all pass on macOS.
- [x] `cargo xtask` and `cargo xtask help` print the command list (currently: `help`) with exit code 0.
- [x] `cargo xtask nonsense` prints usage to stderr and exits with code 2.
- [x] `xtask/Cargo.toml` has no dependency on any `liminal_*` crate (R011).
- [x] The pinned Rust version is recorded in the journal entry.
- [x] Every lint configured in `[workspace.lints]` that deviates from CODE_STYLE.md is justified in a
      comment. Pedantic lints allowed at workspace level are listed with reasons.

## Test requirements

- `Verifies: XTASK-R002`: tests for argument parsing (help, no arguments, unknown command, unknown
  option), done as unit tests on the parser plus one integration test that runs the binary
  (`std::process::Command` with `env!("CARGO_BIN_EXE_xtask")`) and checks exit codes.
- XTASK-R001 and XTASK-R011 are configuration facts. Verify them with an integration test that reads
  `.cargo/config.toml` and `xtask/Cargo.toml` and asserts the alias exists and no `liminal_` dependency
  is present.

## Notes

- `[workspace.lints]` requires each member to opt in with `[lints] workspace = true`.
- Edition 2024 defaults to resolver 3. State it explicitly anyway, for readers.
- Do not add a `liminal_core` placeholder crate to "have something to build". The xtask crate is
  enough for CI, and placeholders break the "spec before code" rule.

## Handoff

2026-11-08, implementation session. Status → `review`.

**Done (all acceptance criteria met):**
- Pinned toolchain `1.99.1` (current stable per `rustup check` on 2026-11-08) in
  `rust-toolchain.toml`, components `rustfmt`, `clippy`, `llvm-tools`, profile `minimal`.
  `rust-version = "1.99.1"` in `[workspace.package]`.
- Root `Cargo.toml`, `Cargo.lock`, `rustfmt.toml`, `clippy.toml`, `.cargo/config.toml` as scoped.
- `[workspace.lints]` follows CODE_STYLE.md §Lints with no deviations. Pedantic lints allowed at
  workspace level: none.
- `[workspace.dependencies]`: `serde = "1.0"`, `serde_json = "1.0"`, `toml = "1.2"`,
  `cargo_metadata = "0.23"` (latest on crates.io 2026-11-08: 1.0.229, 1.0.154, 1.2.1, 0.23.1).
  `xtask` does not inherit any of them yet, so they are not in `Cargo.lock` and have not been compiled.
  `serde` has no `derive` feature yet; the task that first deserializes decides that.
- `xtask` crate: `src/lib.rs` (crate root only) + `src/cli.rs` (parser, `COMMANDS` table, `help`,
  `Exit` codes 0/1/2, `CommandError`) + thin `src/main.rs` (converts OS args; non-UTF-8 argument →
  exit 2). New commands plug in by adding a `Command { name, summary, run }` entry to `COMMANDS`;
  each command parses its own arguments and returns `CommandError::Usage` for unknown ones.
- Tests: 9 unit tests in `cli.rs`, `tests/cli.rs` (runs the binary, checks exit codes and streams),
  `tests/config.rs` (R001 alias, R011 scan of `xtask/Cargo.toml` + scanner tested on synthetic
  manifests), 1 doc test on `xtask::run`.

**Verified (macOS 27.2, Apple M5 Pro):** `cargo build --workspace --locked`, `cargo test --workspace
--all-features --locked` (16 tests incl. doc test), `cargo clippy --workspace --all-targets
--all-features --locked -- -D warnings`, `cargo fmt --all --check`, `RUSTDOCFLAGS="-D warnings" cargo
doc --workspace --no-deps --all-features --locked`: all pass. `cargo xtask` / `cargo xtask help` exit 0
(also from `docs/specs/`); `cargo xtask nonsense` → usage on stderr, empty stdout, exit 2.
Mutation checks: changing the usage exit code to 3 fails 2 unit tests; removing the alias fails the
R001 test. Not verified on Windows or Linux (CI arrives with M0-T007).

**Traps:**
- In non-interactive shells `cargo` may not be on `PATH`; prepend `/opt/homebrew/opt/rustup/bin`.
- Integration-test helper functions are not `#[test]` functions, so `clippy::unwrap_used` applies to
  them despite `allow-unwrap-in-tests`. Use `expect("test setup: …")` or `unwrap_or_else(panic!)`.

**Next:** review this task (fresh session, `docs/process/REVIEW.md`). Then M0-T002.

## Review

2026-11-08, review session (fresh context, did not write this work). macOS 27.2, Apple M5 Pro,
Rust 1.99.1. **Outcome: `done`.**

Checked:
1. **Acceptance criteria**, each re-run rather than trusted: `cargo build --workspace --locked`,
   `cargo test --workspace --all-features --locked` (16 tests: 9 unit, 3 CLI, 3 config, 1 doc test),
   `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`,
   `cargo fmt --all --check`, `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
   --all-features --locked`: all pass. `cargo xtask` and `cargo xtask help` exit 0 (also from
   `docs/specs/`). `cargo xtask nonsense` exits 2 with empty stdout and usage on stderr.
   `xtask/Cargo.toml` has no dependencies. The journal records 1.99.1, which `rustup check` confirms
   is the current stable. `[workspace.lints]` matches CODE_STYLE.md §Lints with no deviations, so no
   justification comments are needed.
2. **`cargo xtask ci`** does not exist yet (M0-T002). The plain cargo commands listed in AGENTS.md
   were run instead (item 1).
3. **Tests prove behaviour.** These mutations were each applied to one line, and each one made the
   suite fail: no arguments → usage error; option-in-command-position check removed; usage text
   without summaries; usage error without the usage text; `help` ignoring extra arguments; non-UTF-8
   argument → exit 0; a `liminal_core` dependency added to `xtask/Cargo.toml`. The last one also
   breaks the build because the path does not exist, so the R011 scanner is backed by its own test
   on synthetic manifests (`liminal_dependency_scan_finds_every_dependency_form`). No test lacks
   assertions. The `Verifies:` tags (R001, R002, R011) match what each test checks.
4. **No fake progress:** no `todo!`, `unimplemented!`, `dbg!`, TODO, or `#[ignore]` in `xtask/`. No
   non-test `expect`. The scope matches the task. No oracles exist yet.
5. **Spec agreement:** the code matches `docs/specs/xtask.md` §3 (lib + thin bin, hand-written
   parsing), §5 (`help`, only implemented commands listed), R001, R002, R007 (exit codes) and R011.
6. **Unsafe:** none. Both crate roots have `#![forbid(unsafe_code)]`. Miri is not applicable.
7. **Determinism:** not applicable (xtask is `deterministic = false`).
8. **Docs:** every public item is documented. `xtask::run` has a runnable `# Examples` doc test.
   STATUS.md and M0-T006 (`llvm-tools`) were updated. No stale references to `llvm-tools-preview`
   remain outside historical notes.
9. **Architecture:** the four `[workspace.dependencies]` entries match `architecture.toml`
   `[[crate]] xtask` `external` and their `[external.*]` entries (scope `tool`, ADR-0003). They are
   caret `major.minor` requirements per DEPENDENCIES.md. The `llvm-tools` component is installed.

Non-blocking observations (no change required):
- `help_lists_every_command_with_its_summary` finds a command's line with `starts_with(name)`. Once a
  command name is a prefix of another (e.g. `check` and `check-arch`), it could match the wrong line.
  The task that adds such a pair should match the exact name column.
- `tests/config.rs` checks R001 and R011 by scanning text lines. The file says so, and M0-T004
  (`check-arch`) is the authoritative R011 check.
