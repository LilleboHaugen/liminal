---
id: M0-T005
title: trace and hygiene
milestone: M0
status: in-progress
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

- [x] `cargo xtask trace` on the real repository lists all `XTASK-R*` requirements and the tests that
      verify them. Every requirement implemented by M0-T001 … M0-T005 shows at least one test.
- [x] `cargo xtask hygiene` passes on the real repository. xtask's own source contains the forbidden
      patterns as string literals. Handle this with an exclusion in `ci/hygiene.toml` that covers the
      scanner's pattern file only, with a reason, or by building the patterns so they never appear
      literally. Choose one and justify it in a code comment.
- [x] Fixtures: unknown ID annotation fails (R043); duplicate IDs fail (R045); a `usable` spec with an
      untested MUST fails (R044) while the same spec at `experimental` passes; withdrawn requirements
      are excluded; untracked `TODO` fails; `TODO(M9-T999)` referencing a nonexistent task fails;
      `#[ignore]` without reason fails; `#[ignore = "M0-T005: reason"]` passes.
- [x] `trace` and `hygiene` are steps in `ci`. R028 is added to `check-docs`.

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

## Decisions that differ from the task text

Written here so the reviewer can judge them (AGENTS.md rule 2):
1. **Criterion 2 (patterns as string literals in xtask).** The criterion offers an exclusion of
   the pattern file or patterns built so they never appear literally. Neither is used. The
   scanners lex Rust (`xtask/src/rust_lexer.rs`: identifiers, punctuation, string literals,
   comments; not a parser), so string literal contents are never searched for macros or markers.
   No xtask file is excluded, and the pattern tables are plain literals. Reason: an exclusion
   would hide real violations in the excluded file, and obfuscation only hides the patterns from
   a scan that cannot tell code from strings. Justified in the module comment of
   `xtask/src/hygiene.rs`. Comments in xtask still must not contain a marker word.
2. **Notes: "attached to the next `fn` … within 10 lines".** Replaced by an exact token rule:
   after comments and outer attributes (any number of lines), the next item must be a function
   header, otherwise R041. A line-count limit would reject a test behind a long multi-line
   `#[expect(…)]` and accept an annotation above a `struct` followed by a function within 10
   lines. Spec v10 states the rule.
3. **Out of scope "anything needing a Rust parser; scanning is line-based".** The lexer above is
   token-based, not line-based. It is the R035 lexer from M0-T004, moved to a shared module and
   extended with comments, string contents and line numbers; R035 behaviour and tests are
   unchanged.

## Handoff

2026-11-08, implementation session (after review 2 of M0-T004 in the same session). Status →
`review`.

**Done (all acceptance criteria met):**
- `xtask/src/rust_lexer.rs`: shared lexer (moved from `check_arch/attributes.rs`, which now uses
  it). Tokens carry their line and whether they start a line; comments and string contents are
  kept.
- `xtask/src/requirements.rs`: section 6 extraction (R040 malformed rows and foreign prefixes,
  R045 duplicates). Works without valid frontmatter (crate = file name). `(withdrawn` counts only
  outside inline code spans: R040's own text names the marker in code and was read as withdrawn
  by the literal rule (found on the real spec; spec v10 changes R040's text).
- `check-docs` R028 (`check_docs.rs::task_requirements`); malformed-row IDs count as known.
- `xtask/src/repo/sources.rs`: `.rs` files of workspace members (nested members: the inner one
  owns a file), `ci/hygiene.toml` (`[[hygiene.exclude]]`, `[[trace.exclude]]`; shape errors are
  LIM-XTASK-0004).
- `xtask/src/trace.rs` (+ `trace/annotations.rs`): R041–R044, matrix in human and `--json`,
  `--crate`. `xtask/src/hygiene.rs`: R050–R052 (R052 also validates the trace section).
- `ci/hygiene.toml`: `xtask/tests/fixtures` excluded from both scans, with reasons.
- `trace` and `hygiene` registered as commands and as `ci` steps after `check-docs`.
- Fixtures: `xtask/tests/fixtures/trace/`, `xtask/tests/fixtures/hygiene/` (each with README),
  `check_docs/r028_task_requirements`. `check_docs/valid` and its `r022` overlay got a §6 table
  with `DEMO-R001`, which `valid`'s task M0-T001 already listed (R028 would otherwise fail).
- Spec v10 ("`trace` and `hygiene` details", R040 text, LIM-XTASK-0004). `docs/process/TESTING.md`
  §Requirement traceability: where annotations go, what fails.

**Tests:** 194 lib tests (was 141). `trace --crate xtask`: MUST 36/46 tested; the 10 untested
MUSTs are R060–R081 (M0-T006, M0-T008, M0-T009). Every requirement in the `requirements` of
M0-T001 … M0-T005 has a test.

**Verified (macOS 27.2, Apple M5 Pro, Rust 1.99.1):** `cargo xtask ci` passes (fmt, clippy, test,
doc, check-arch, check-docs, trace 28 ms, hygiene 27 ms). 28 single-line mutants of the new code
(scripted, each reverted): all killed (one survived the first run, `dedup` of a test listed twice
for one ID; a case was added). Faults injected into `xtask/src/main.rs` (reverted): an untracked
marker (R050), a marker naming the `done` task M0-T004 (R051), an annotation naming `XTASK-R999`
(R043); `ci --json --only trace,hygiene` gave both steps `fail` with those findings. Not verified:
Windows, Linux (M0-T007).

**Traps:**
- Inputs with forbidden patterns or fake annotations go in files under `xtask/tests/fixtures/`
  (read with `include_str!` or loaded as fixture repositories), never in string literals. The
  scanners would not flag a literal, but the task note requires it and it keeps xtask's source
  free of near-miss patterns.
- xtask comments must not contain a marker word: write "task marker".
- `cases/*.rs` line numbers are asserted; editing a case file changes the expected lists.
- zsh does not word-split `$var` in `for` loops (`set -- $t` does nothing); one fixture setup
  script went wrong that way.

**Next:** review (fresh session). Then the first `todo` in M0 with all dependencies done:
M0-T006 (coverage).

## Review

2026-11-08, new owner-started session (fresh context; did not write M0-T005). Outcome:
**changes required** (first "changes required" outcome of this task). Two blocking findings.

Checked, macOS 27.2, Apple M5 Pro, Rust 1.99.1:
1. `cargo xtask ci`: pass (fmt, clippy, test, doc, check-arch, check-docs, trace 26 ms,
   hygiene 25 ms).
2. Acceptance criteria, each by reading or running: `trace` lists all `XTASK-R*` requirements;
   `hygiene` passes on the repository with no xtask file excluded (decision 1 judged: lexing
   string literals out is a stronger answer to criterion 2 than either offered option, and the
   module comment of `hygiene.rs` justifies it); every fixture named in criterion 3 exists and is
   asserted (`cases/*.rs`, `r0xx_*` overlays, `r044_untested_must` at `usable` vs. the `valid`
   spec at `experimental`); `trace` and `hygiene` are `ci` steps; R028 is in `check-docs`.
   Decisions 2 and 3 (exact token rule instead of "within 10 lines", lexer instead of lines)
   accepted: both are stated in spec v10 and are stricter than the task text.
3. Code read in full: `rust_lexer.rs`, `requirements.rs`, `repo/sources.rs`, `trace.rs`,
   `trace/annotations.rs`, `hygiene.rs`, the `check_docs.rs`, `check.rs` and `ci.rs` diffs, the
   hygiene and trace case fixtures and tests. Checked against spec v10 "`trace` and `hygiene`
   details" paragraph by paragraph: agrees.
4. Own mutants (16, each reverted, full `cargo test -p xtask`): 13 killed, 3 survived (findings 1
   and 2, follow-up 1).
5. No placeholders, no weakened tests. Oracle changes (`check_docs/valid` §6 table, `ci/tests.rs`
   step list) are justified in the journal. No `unwrap`/`expect` in non-test code except
   `expect("invariant: …")`. No unsafe. Determinism: n/a.
6. Architecture: no new dependency. Session boundaries: the M0-T005 journal shows an
   owner-started session (after review 2 of M0-T004), no sub-agents.

Required changes:
1. **B4: nested-member file ownership is untested.** The handoff claims "`.rs` files of workspace
   members (nested members: the inner one owns a file)". Mutant in `repo/sources.rs::load`:
   `*owner < prefix.len()` → `*owner > prefix.len()` (the outer member owns the file): all tests
   pass. The owner is `SourceFile::crate_name`, which decides which `--crate` report an
   annotation finding belongs to. Add a test with a member nested inside another member's
   directory that asserts the crate of a file below the inner member (and of one below only the
   outer member).
2. **B4: the `## 6 ` heading form of section 6 is untested.** Spec "`trace` and `hygiene`
   details": section 6 starts at "the level-2 heading that starts with `6.` or `6 `". Mutant in
   `requirements.rs::section_6`: dropping `|| heading.starts_with("6 ")`: all tests pass, and a
   spec headed `## 6 Requirements` then has no requirements (every annotation of it becomes an
   R043 finding). Add a test for a `## 6 …` heading.

Follow-ups (non-blocking) → [M1-T004](M1-T004-trace-hygiene-follow-ups.md).
