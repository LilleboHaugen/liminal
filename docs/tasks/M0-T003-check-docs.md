---
id: M0-T003
title: Frontmatter parser and check-docs
milestone: M0
status: todo
crates: [xtask]
depends_on: [M0-T002]
requirements: [XTASK-R009, XTASK-R020, XTASK-R021, XTASK-R022, XTASK-R023, XTASK-R024, XTASK-R025, XTASK-R026, XTASK-R027]
---

# M0-T003: Frontmatter parser and `check-docs`

## Goal

The documentation system is checked by machine: frontmatter is valid, references resolve, indexes are
complete. Agents cannot silently break the docs that every later session depends on.

## Context

- `docs/process/DOCUMENTATION.md` (restricted YAML subset, field lists).
- `docs/specs/xtask.md` R020–R027, §4 (repo model: loaders separate from pure checks).
- R028 (task requirement IDs exist) needs requirement extraction and is done in M0-T005.

## Scope

**In:** restricted-YAML frontmatter parser; the **shared repo-model loader module** named in the
xtask spec §3 (later checks reuse and extend it, never duplicate it), with loaders for ADRs, specs,
tasks, `architecture.toml` (deserialized fully with `serde`; semantic validation is M0-T004), and
workspace members from `[workspace].members` in the root `Cargo.toml` (no cargo invocation); checks
R020–R027; the `check-docs` command with `--json` (R009) and the `ci` step; fixtures.

If M0-T004 has already landed, reuse its loader code instead of adding a second one.

**Out:** anchor (`#heading`) validation in links (may be added later as SHOULD).

## Acceptance criteria

- [ ] `cargo xtask check-docs` passes on the real repository. If it finds real problems in the
      planning docs, fix the docs (not the checker), and list the fixes in the journal.
- [ ] One fixture per rule fails with exactly that rule ID. One complete valid fixture passes.
- [ ] Property test: frontmatter generated from random valid key/value data parses back to the same
      data. Random lines outside the subset are rejected, never misparsed.
- [ ] `check-docs` is a step in `ci`.

## Test requirements

`Verifies:` tests for R020–R027. Fixtures under `xtask/tests/fixtures/check_docs/<case>/`.

## Notes

- Parse dates as strings matching `^\d{4}-\d{2}-\d{2}$`. Do not validate calendar dates.
- Links: handle `[text](path)` and `[text](path#anchor)`. Ignore links inside fenced code blocks and
  inline code.
- Keep the parser small and strict. Frontmatter syntax errors must point at file and line.

## Handoff

_Empty._

## Review

_Empty._
