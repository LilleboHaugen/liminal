---
id: M0-T005
title: trace and hygiene
milestone: M0
status: todo
crates: [xtask]
depends_on: [M0-T003]
requirements: [XTASK-R009, XTASK-R028, XTASK-R040, XTASK-R041, XTASK-R042, XTASK-R043, XTASK-R044, XTASK-R045, XTASK-R050, XTASK-R051, XTASK-R052]
---

# M0-T005: `trace` and `hygiene`

## Goal

Requirement → test traceability and forbidden-pattern detection run automatically. These are the two
most important anti-fake-progress checks (AGENTS.md rules 1, 3, 13).

## Context

- ADR-0010. `docs/process/TESTING.md` §Requirement traceability. `docs/specs/xtask.md` R028,
  R040–R045, R050–R052.
- Builds on the repo model and spec loader from M0-T003.

## Scope

**In:** requirement extraction from spec §6 tables; `Verifies:` annotation scanning in all workspace
`.rs` files; trace matrix (human and JSON); the maturity-dependent gate (R044); R028 in `check-docs`;
`hygiene` scanner; `ci/hygiene.toml` with exclusions and reasons; both steps in `ci`; fixtures.

**Out:** anything needing a Rust parser. Scanning is line-based by design.

## Acceptance criteria

- [ ] `cargo xtask trace` on the real repository lists all `XTASK-R*` requirements and the tests that
      verify them. Every requirement implemented by M0-T001 … M0-T005 shows at least one test.
- [ ] `cargo xtask hygiene` passes on the real repository. xtask's own source contains the forbidden
      patterns as string literals. Handle this with an exclusion in `ci/hygiene.toml` that covers the
      scanner's pattern file only, with a reason, or by building the patterns so they never appear
      literally. Choose one and justify it in a code comment.
- [ ] Fixtures: unknown ID annotation fails (R043); duplicate IDs fail (R045); a `usable` spec with an
      untested MUST fails (R044) while the same spec at `experimental` passes; withdrawn requirements
      are excluded; untracked `TODO` fails; `TODO(M9-T999)` referencing a nonexistent task fails;
      `#[ignore]` without reason fails; `#[ignore = "M0-T005: reason"]` passes.
- [ ] `trace` and `hygiene` are steps in `ci`. R028 is added to `check-docs`.

## Test requirements

`Verifies:` tests for R028, R040–R045, R050–R052.

## Notes

- The annotation attaches to the next line that starts (after whitespace and attributes) with `fn`,
  `pub fn`, `async fn` or similar. If none follows within 10 lines, report a finding.
- Exclude `target/` and `xtask/tests/fixtures/` from scanning: fixtures deliberately contain
  violations. That exclusion belongs in `ci/hygiene.toml` with its reason (R052). The `trace` scanner
  needs the same fixture exclusion. Read it from the same file, under a separate key.
- MUST: every test input that contains a forbidden pattern or a fake `Verifies:` line (TODO markers,
  `dbg!(`, bare `#[ignore]`, unknown requirement IDs) lives in a file under `xtask/tests/fixtures/`.
  It is never an inline string literal in a `.rs` file, or the scanners would flag xtask's own tests.
  Doc comments explaining the rules must not spell the patterns in a matching form (e.g. write
  "TODO markers" in prose, not a literal marker followed by a colon).
- Use the shared repo model for the list of workspace crates to scan (xtask spec §3).

## Handoff

_Empty._

## Review

_Empty._
