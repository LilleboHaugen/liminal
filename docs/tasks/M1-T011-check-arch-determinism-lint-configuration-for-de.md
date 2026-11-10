---
id: M1-T011
title: check-arch — determinism lint configuration for deterministic crates
milestone: M1
status: done
crates: [xtask]
depends_on: [M1-T001]
requirements: [XTASK-R039]
---

# M1-T011: check-arch: determinism lint configuration for deterministic crates

## Goal

A crate marked `deterministic = true` cannot be added without the clippy `disallowed-methods` configuration that ADR-0009 item 7 and CODE_STYLE.md require.

## Context

- ADR-0009 item 7; `docs/process/CODE_STYLE.md` §Lints (the banned list).
- `docs/specs/xtask.md` §check-arch; the root `clippy.toml`.
- Assumption to verify first: clippy reads the `clippy.toml` nearest to the crate's manifest directory and does not merge it with the root file. If so, a per-crate file must repeat the root settings.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** 1. Choose the mechanism (per-crate `clippy.toml` or another that clippy really applies) and record it in CODE_STYLE.md.
2. New XTASK requirement: `check-arch` reports a deterministic workspace crate whose effective clippy configuration does not ban every path on the CODE_STYLE.md list. Spec `spec_version` bump and history row.
3. Fixture tests for the passing and failing case.

**Out:** Writing the configuration for real crates: each crate's first implementation task does that, and this check then enforces it.

## Acceptance criteria

- [x] The mechanism is shown to work: a fixture or scratch crate calling `f32::sin` fails `cargo clippy` with the configuration and passes without it (journal records the run).
- [x] New requirement in `docs/specs/xtask.md` with `Verifies:` fixture tests for: missing file, file missing one banned path, complete file, non-deterministic crate without file.
- [x] CODE_STYLE.md says where the configuration lives.
- [x] `cargo xtask ci` passes.

## Test requirements

The new XTASK requirement (MUST), fixture tests in the style of the existing `check-arch` fixtures.

## Notes

_None._

## Handoff

Done (2026-11-09, implementation sub-agent):

- Assumption verified (clippy 0.1.99, scratch workspaces outside the repository; journal
  `docs/journal/2026-11-09-M1-T011.md`): clippy uses the nearest `.clippy.toml` or `clippy.toml`
  (`.clippy.toml` wins in one directory) and does not merge it with a parent's file. Mechanism:
  each deterministic crate has its own `<manifest dir>/clippy.toml` that repeats the root
  settings and adds `disallowed-methods`. Recorded in `docs/process/CODE_STYLE.md` §Lints and a
  comment in the root `clippy.toml`.
- New XTASK-R039 (MUST) in `docs/specs/xtask.md`, `spec_version` 21, with details (lookup,
  entry forms, LIM-XTASK-0004, locations).
- Code: `xtask/src/repo/clippy.rs` (lookup and parsing, loader side), `ArchModel::clippy`
  filled by `repo::load_arch` for packages registered `deterministic = true`,
  `xtask/src/check_arch/determinism.rs` (rule and `BANNED_METHODS`, with a test that the constant
  equals the CODE_STYLE.md list). `CrateEntry::deterministic` lost its `expect(dead_code)`.
- Tests: unit tests in `determinism.rs` (complete, missing paths, no file, non-deterministic crate)
  and `repo/clippy.rs`; fixtures `valid/` (deterministic `runtime` with a complete file, root
  `clippy.toml` banning nothing), `r039_missing_clippy_config`, `r039_incomplete_clippy_config`,
  plus a no-file-at-all variant and a LIM-XTASK-0004 case in `fixture_tests.rs`.
- `cargo xtask ci` passes (11 steps, no skips), macOS 27.2.

Not done / traps:

- No real deterministic crate is in the workspace yet, so R039 judges nothing on the real
  repository today. Each deterministic crate's first implementation task (M1-T013 for
  `liminal_core`, etc.) must add the crate's `clippy.toml`; check-arch then fails until it does.
- `disallowed-types` (ADR-0009 item 7 mentions it) is not on the CODE_STYLE.md list and is not
  checked. `HashMap::default`/`with_capacity` are not banned by the list either; changing the list
  is a CODE_STYLE.md change plus an update of `BANNED_METHODS` (the test catches a mismatch).
- Not verified on Windows or Linux (path lookup uses `/`-joined repository-relative paths, as the
  rest of the loader does).

## Review

Reviewer: review sub-agent (ADR-0026), 2026-11-10, parent
`docs/journal/2026-11-10-session-m1-reviews.md`; did not write M1-T011 (its author was sub-agent
no. 12 of `2026-11-09-session-m1-subagents.md`). Prompt: the ADR-0026 review template verbatim.
Outcome: **done**, no blocking findings. Follow-up task:
`docs/tasks/M2-T006-determinism-lint-follow-ups.md`.

Checked, macOS 27.2, Rust 1.99.1, clippy 0.1.99, on `ecf7f24`:
1. Acceptance criteria (by reading the diff `73930b5..ecf7f24` and by running):
   - Mechanism: repeated in a scratch workspace outside the repository. Root `clippy.toml`
     banning `f32::cos`; member `det` with a copy of the fixture
     `valid/crates/runtime/clippy.toml`; member `plain` without a file. `cargo clippy
     --workspace`: `det` reports `f32::sin`, `f32::cos`, `f32::tan`, `f64::ln`, `f64::powf`,
     `f64::exp`, `HashMap::new`, `Instant::now` (every path the probe called); `plain` reports
     only `f32::cos`. After deleting `det/clippy.toml`, `det` reports only `f32::cos`. So the
     nearest file applies, a parent file applies when there is none, and files are not merged.
     Met.
   - XTASK-R039 is in `docs/specs/xtask.md` with `Verifies:` tests for: no file at all
     (`no_configuration_at_all_is_reported_at_the_manifest`,
     `r039_missing_clippy_config_without_a_root_file`), no own file with a root file that bans
     nothing (`r039_missing_clippy_config_fixture`), one path missing
     (`r039_incomplete_clippy_config_fixture`, unit test), complete file (`valid_fixture_passes`,
     unit test), non-deterministic crates without a file (`valid/`: `base`, `peer`, `cli` and
     others; unit test). Met.
   - CODE_STYLE.md §Lints says where the file lives and why. Met.
   - `cargo xtask ci`: pass, 11 steps, none skipped. Met.
2. Tests prove behaviour: 11 single-edit mutants, each run against `cargo test -p xtask`.
   Killed (10): `.clippy.toml`/`clippy.toml` preference swapped; parent walk jumps straight to
   the root; the root directory is not searched; every registered crate judged instead of
   deterministic ones; finding location of an incomplete file; finding location without a
   file; help path; a non-array `disallowed-methods` accepted; table entries not read; one
   path dropped from `BANNED_METHODS`. Survived (1): the registry entry is taken with `rfind`
   instead of `find` (non-blocking, below).
3. Realistic input (B3): `cargo xtask check-arch` on the real repository with `xtask`
   temporarily `deterministic = true`: one XTASK-R039 finding at `clippy.toml` naming all 14
   paths, help naming `xtask/clippy.toml`; reverted. A key spelled `disallowed_methods` is
   rejected by clippy itself ("unknown field name"), so R039's exact key cannot pass a file
   clippy reads differently. No wrong result found.
4. No fake progress: no placeholder; scope items 1 to 3 done. No oracle changed: three fixture
   registries mark `runtime` deterministic and `valid/` gained two clippy files; no expected
   finding of an existing test changed.
5. Spec agreement: the R039 row and details say what `xtask/src/repo/clippy.rs` and
   `xtask/src/check_arch/determinism.rs` do (lookup order, entry forms, LIM-XTASK-0004,
   locations, order of missing paths). The branch numbers the spec change `spec_version` 21;
   `main` is at 22, so the merge renumbers it to 23 (ADR-0026 Decision 4.3) in the frontmatter,
   the history row and the R039 row. The handoff above and the task's journal entry keep "21".
6. Unsafe, determinism, architecture: no `unsafe`, no new dependency (`toml` was approved for
   `xtask`), no `architecture.toml` change. Docs: no other document states where the
   deterministic clippy configuration lives (grep for `clippy.toml` and `disallowed`).
7. Session boundaries (REVIEW.md item 10): the implementation sub-agent is no. 12 in
   `docs/journal/2026-11-09-session-m1-subagents.md` (owner-started parent, "impl template").
   No install, no spawned agent. This review belongs to a different owner-started session.
8. Public repository (REVIEW.md item 11): no workflow change; no issue, pull request or
   outside code acted on; no credential or personal data in the diff.
9. Branch CI: run 42698214554 on `ecf7f24`: success.

Non-blocking findings (follow-up M2-T006):
- "The first `[[crate]]` entry of the name" (R039 details) is untested: `rfind` instead of
  `find` in `repo::load_arch` passes every test. Not B4: a duplicate name is already an
  XTASK-R030 finding, so `check-arch` fails either way; only the presence of an additional R039
  finding differs.
- R039 checks the configuration, not the lint level. With the complete fixture file,
  `#![allow(clippy::disallowed_methods)]` at the crate root makes clippy report nothing
  (probe run in this review). No check sees that.
- The banned list is narrow. With the complete fixture file clippy accepts `f32::sin_cos`,
  `f32::powi`, `f32::atan2`, `f32::log2`, `std::collections::HashSet::new`,
  `HashMap::default` and `std::time::SystemTime::now` (probe run in this review).
  `disallowed-types` (ADR-0009 item 7) is not used. The handoff names part of this. The list
  is CODE_STYLE.md's, outside this task's scope.
- CODE_STYLE.md says a crate's file repeats every setting of the root `clippy.toml`; nothing
  checks it. A crate file without the `allow-*-in-tests` lines only makes clippy stricter.
- `xtask/src/cli.rs` says "XTASK-R030 … R039"; R038 is not implemented (SHOULD, from M2).
  `xtask/src/check_arch.rs` says "R030 … R037, R039".
- The task's journal entry names its parent by description, not by file (already M2-T004
  item 5; not repeated in M2-T006).
