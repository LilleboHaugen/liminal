---
id: M1-T013
title: Implement liminal_core — error model and generational handles
milestone: M1
status: review
crates: [liminal_core]
depends_on: [M1-T012, M1-T011, M1-T003, M1-T004, M1-T005]
requirements: [CORE-R001, CORE-R002, CORE-R003, CORE-R004, CORE-R005, CORE-R006, CORE-R007, CORE-R008, CORE-R009, CORE-R010, CORE-R011, CORE-R012, CORE-R013, CORE-R030, CORE-R031, CORE-R032, CORE-R033, CORE-R034, CORE-R035, CORE-R036, CORE-R037, CORE-R038, CORE-R039, CORE-R040]
---

# M1-T013: Implement `liminal_core`: error model and generational handles

## Goal

`liminal_core` exists with the error model and generational IDs/handles specified in its spec.

## Context

- `docs/specs/liminal_core.md` (requirements assigned to this task).
- ADR-0017; CODE_STYLE.md §Errors. Follow-ups M1-T003–T005 harden the gates this first engine crate meets.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** The crate, its error model and its handle types, with unit, integration and property tests.

**Out:** Hashing, RNG, collections, time types: the next `liminal_core` task.

## Acceptance criteria

- [x] The crate exists at the path in `architecture.toml`, is a workspace member, has
      `#![forbid(unsafe_code)]` if `unsafe = "forbid"`, and has a floor in `ci/coverage.toml`.
- [x] The crate's clippy determinism configuration is in place and `check-arch` accepts it.
- [x] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [x] New public items are documented; primary entry points have runnable `# Examples`.
- [x] `cargo xtask ci` passes with no failed or skipped step.

## Test requirements

Requirements in frontmatter (assigned by the spec task). Property tests for handle generation and reuse rules.

## Notes

_None._

## Handoff

**2026-11-10, implementation sub-agent (journal `2026-11-10-M1-T013.md`). Status: `review`.**

Done:
- `crates/liminal_core` (workspace member, `#![forbid(unsafe_code)]`, no dependencies,
  dev-dependency `proptest`), floor `liminal_core = 98.9` in `ci/coverage.toml` (measured
  98.96 %, 1052 of 1063 lines, macOS).
- `crates/liminal_core/clippy.toml`: root settings repeated plus the 14 `disallowed-methods`
  paths of XTASK-R039; `check-arch` passes.
- `liminal_core::error` (spec §5.1): `ErrorCode`, `EngineError`, `write_display`, `render_json`,
  `validate_error`, `ErrorContext`, `ContextValue`, `CoreError` with all 11 variants of §8
  (also the variants whose producers come in M1-T014).
- `liminal_core::handle` (spec §5.2): `RawHandle`, `Handle<T>`, `HandleAllocator`,
  `HandleMap<T>` (the map is a `HandleAllocator` plus `Vec<Option<T>>`).
- Tests: 26 unit, 29 integration (13 of them property tests), 12 doc tests and 1 `compile_fail`
  doc test. Every ID in `requirements` has a `Verifies:` test (`cargo xtask trace --crate
  liminal_core`). Oracles: `tests/common/json.rs` (strict JSON decoder written from RFC 8259),
  `tests/common/handle_model.rs` (linear-search model), `tests/snapshots/core_errors.txt`
  (written by hand from spec §8 before the first run; it passed unchanged).
- `# Examples`: crate root, `ErrorCode`, `EngineError` (complete enum), `write_display`,
  `render_json`, `validate_error`, `ErrorContext`, `CoreError`, `RawHandle`, `Handle`,
  `HandleAllocator`, `HandleMap`.
- Spec §13: second checklist item ticked with evidence. No requirement, API or `spec_version`
  change.

Outside `crates/liminal_core` (needed for CI to pass with a second workspace member):
- `xtask/src/repo.rs` (`loads_the_real_repository`) and `xtask/src/repo/sources.rs`
  (`loads_the_rust_files_of_the_real_workspace`) asserted that `xtask` is the only member and
  owns every source file. They now check membership (`xtask` and `liminal_core` present, every
  source under its crate's directory). The fixture tests next to them keep their exact lists.
- `Cargo.toml` (member), `Cargo.lock`, `ci/coverage.toml`, `docs/STATUS_NOTES.md` (module notes).

Not done / limits (none is in scope):
- `hash`, `collections`, `rng`, `time`, `tests/determinism.rs`, the `StateHash` impls for
  handles: M1-T014.
- Spec `maturity` is still `planned`; a reviewing session changes it.
- CORE-R040 (SHOULD, "checked by review"): its test has no timing assertion (TESTING.md). It runs
  800,000 operations on 200,000 slots, which a per-operation scan would turn into about 10^11
  slot visits (minutes). A reviewer still has to read `allocator.rs` and `map.rs` for the O(1)
  claim.
- CORE-R003 "compile error in a `const` item": checked by a `compile_fail` doc test on
  `ErrorCode`, which passes for any compile error, not only this one. The `Verifies:` tests
  check the panic at run time.
- CORE-R011 and CORE-R013 "every supported platform": the snapshot literals ran on macOS arm64
  only in this session. Windows and Linux: not verified until the branch CI run.
- CORE-R037 "`new()` has the slot limit `u32::MAX`" is tested through the `Debug` form
  (2^32 allocations are not feasible in a test).

Verified: `cargo xtask ci` on macOS 27.2, Rust 1.99.1: 11 steps pass, none skipped. 20 hand-made
mutations of the implementation (list in the journal) each made `cargo test -p liminal_core`
fail.

Traps:
- `allow-unwrap-in-tests` covers only `#[test]` functions and `#[cfg(test)]` modules. Helper
  functions in `tests/*.rs` and `tests/common/` need `expect("test setup: …")`.
- Shared test helpers are included per test file with `#[path = "common/<file>.rs"] mod …;`. A
  single `tests/common/mod.rs` gives dead-code errors in every test binary that does not use
  all of it.
- `cargo xtask coverage --update-floors` raises every crate's floor, also `xtask`'s (97.6 →
  98.1 here). That line was put back to 97.6: the `xtask` floor is not this task's, and it was
  measured on macOS while the `coverage` CI job runs on Linux.
- `HandleMap::iter` zips `HandleAllocator::iter_live` with the `Some` entries of the value
  vector; this relies on INV-2. Keep both in step when changing `insert` or `remove`.

Next step: a new session reviews M1-T013. Then M1-T014 can start.

## Review

_Empty._
