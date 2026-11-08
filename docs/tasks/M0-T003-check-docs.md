---
id: M0-T003
title: Frontmatter parser and check-docs
milestone: M0
status: review
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

- [x] `cargo xtask check-docs` passes on the real repository. If it finds real problems in the
      planning docs, fix the docs (not the checker), and list the fixes in the journal.
- [x] One fixture per rule fails with exactly that rule ID. One complete valid fixture passes.
- [x] Property test: frontmatter generated from random valid key/value data parses back to the same
      data. Random lines outside the subset are rejected, never misparsed.
- [x] `check-docs` is a step in `ci`.

## Test requirements

`Verifies:` tests for R020–R027. Fixtures under `xtask/tests/fixtures/check_docs/<case>/`.

## Notes

- Parse dates as strings matching `^\d{4}-\d{2}-\d{2}$`. Do not validate calendar dates.
- Links: handle `[text](path)` and `[text](path#anchor)`. Ignore links inside fenced code blocks and
  inline code.
- Keep the parser small and strict. Frontmatter syntax errors must point at file and line.

## Handoff

2026-11-08, implementation session (after reviewing M0-T002 in the same session). Status → `review`.

**Done (all acceptance criteria met):**
- `xtask/src/frontmatter.rs`: strict parser for the restricted YAML subset. Errors carry the line
  number. The exact grammar is now in spec §6 "`check-docs` details" (`spec_version` 3).
- Shared repo-model loader `xtask/src/repo.rs`: `RepoModel` (root and `docs/` Markdown, every
  repository path except `.git/` and `target/`, `architecture.toml`, workspace members),
  `adr_files`/`spec_files`/`task_files` selectors, and `LoadError` (spec §8 codes).
  `repo/architecture.rs` deserializes `architecture.toml` in full with `deny_unknown_fields` (shape
  only; the fields that only M0-T004 reads carry `expect(dead_code)`, so M0-T004 must remove those
  attributes when it reads them). `repo/workspace.rs` reads `[workspace].members` without cargo
  (paths and `<dir>/*`).
- `xtask/src/markdown.rs`: link targets (inline links, images, reference definitions; fenced code
  and inline code ignored) and tables.
- `xtask/src/check_docs.rs` (+ `fields.rs`, `rules.rs`, `indexes.rs`): R020–R027 as a pure function
  of the model. `xtask/src/check.rs`: shared check-command runner (`--json`, report, exit code,
  finding order) for M0-T004/T005/T006/T009 to reuse.
- `ci`: `Step.action` is now `Action::Cargo` or `Action::Check` (native step, runs in-process).
  `check-docs` is in `STEPS` after `doc`. `ProcessRunner` renamed `RealRunner`.
- `CommandError::Internal` for LIM-XTASK-nnnn errors. New code LIM-XTASK-0004 (invalid `Cargo.toml`).
- `proptest` added as a dev-dependency (approved: `architecture.toml` `dev_external`, ADR-0010).
  `toml` used (approved, `tool` scope).
- Fixtures `xtask/tests/fixtures/check_docs/`: `valid/` plus one overlay per rule (see its
  README). The fixture tests are unit tests (`src/check_docs/fixture_tests.rs`), because the binary
  always checks the compiled-in repository root.
- `docs/process/DOCUMENTATION.md` links to the exact grammar.

**Spec decisions (spec v3), beyond the literal R020–R027 text:** unknown frontmatter fields are
errors; spec `crate` must equal the file name; task `milestone` must equal the ID's milestone;
values starting with a YAML indicator or containing `: ` are rejected; an ADR's index status
for a superseded ADR is `superseded by <IDs>`; findings are sorted by path, line, rule, message.
R027 (SHOULD) findings are errors like the others: `Severity::Warning` does not exist yet.

**Tests:** 98 lib tests in total, plus integration tests (was 47 + 7). Every rule has a fixture
that fails with exactly that rule, and unit tests cover each sub-condition. Three property tests
cover the parser: round trip, invalid line rejected at its line, and arbitrary input.

**Verified (macOS 27.2, Apple M5 Pro, Rust 1.99.1):** `cargo xtask ci` passes (fmt, clippy, test,
doc, check-docs). `cargo xtask check-docs` on the real repository: no findings, so the planning
docs needed no fixes, in 0.14 s. Four faults injected into real docs, one each for R021, R023,
R024 and R025, were each reported, then reverted. `ci --json --only check-docs` with a broken
link: step `fail`, finding in the summary, exit 1. 16 single-line mutations of the new code: all
killed. Not verified: Windows and Linux (M0-T007). On Windows, `paths` uses `/` separators
(built from path components), but no test has run there.

**Traps:**
- A blank line ends a Markdown table. Removing an index row with a blank line left in place makes
  every later row "not listed". This is correct (GitHub renders it the same way), but the
  findings list is long.
- Mutation testing creates `xtask/proptest-regressions/` when a property fails. Delete it unless
  it records a real bug.
- After mutation testing with `cp`/`mv`, `touch` the restored file. Otherwise cargo's mtime
  fingerprint keeps the last mutant's binary (see the journal).
- `xtask/tests/fixtures/check_docs/valid/` contains `Cargo.toml` files. They are fixtures, not
  workspace members. Do not add them to the workspace.

**Next:** review (fresh session). Then the first `todo` with all dependencies done is M0-T004
(check-arch). It extends `repo::Architecture` (drop the `expect(dead_code)`) and adds `cargo
metadata` loading to `repo`.

**Review changes (2026-11-08, second session; same session that wrote the review below).** Status →
`review`. The next review must be done by a new session: this session wrote both the review and
these changes.
1. `frontmatter.rs`: a value or list item starting with `-`, `?` or `:` followed by whitespace or
   the end, or ending with `:`, is rejected (`INDICATORS_BEFORE_SPACE`). `-1`, `--all`, `?x`, `a:b`
   stay plain strings (test `accepts_indicator_characters_that_yaml_reads_as_plain_text`). New
   rejection cases in the unit test; the `invalid_line()` generator and the arbitrary-input
   property cover the new forms; `plain_text()` no longer generates values ending with `:`.
2. Test `r025_adr_status_cell_must_not_be_in_backticks`.
3. Footnote definitions specified (spec v4) and tested
   (`markdown::tests::footnote_definitions_are_not_reference_definitions`).
4. `rules.rs`: task files without a valid `id` (R020 failure, or missing/invalid `id`) exist for
   `depends_on` under their file-name ID (`file_task_id`). Test
   `r023_a_task_without_a_valid_id_still_exists_by_its_file_name`.
- Spec `spec_version` 4 (§6 details, History). `docs/process/DOCUMENTATION.md` names the new rejected forms.
- Verified (macOS 27.2, Rust 1.99.1): `cargo xtask ci` passes; 102 lib tests. The review's three
  injected `title:` values now each give exactly one R020 finding, and no false R023 on M0-T010.
  The two surviving mutants of the review and two new mutants (ends-with-`:` check removed,
  file-name fallback removed) are each killed. Not verified: Windows, Linux (M0-T007).

**Review 2 changes (2026-11-08, third session; same session that wrote Review 2 below).**
Status → `review`. The next review must be done by a new session: this session wrote both
Review 2 and these changes.
1. `frontmatter.rs`: `,` and `]` added to `FORBIDDEN_START`. With `[` (list), `#` (comment) and
   `INDICATORS_BEFORE_SPACE`, every YAML indicator character is now handled.
2. `frontmatter.rs`: list items must not contain `FLOW_INDICATORS` (`[`, `]`, `{`, `}`) anywhere.
   Scalars may still contain `{`/`}` after the first character (YAML reads `a{b}` as plain text).
3. Spec: a well-formed but duplicate `id` is a valid `id`; the file is not also known by its
   file-name ID. Test `r023_a_task_with_a_duplicate_id_does_not_exist_by_its_file_name`.
- New rejection cases in the unit test; `invalid_line()` generates list items with `{`/`}`; the
  arbitrary-input property asserts list items contain no `[]{},`.
- Spec `spec_version` 5 (§6 details: also states that YAML type resolution is not modelled;
  History). `docs/process/DOCUMENTATION.md`: forbidden list-item characters.
- Verified (macOS 27.2, Rust 1.99.1): `cargo xtask ci` passes; 103 lib tests. Mutants killed: the
  review's surviving mutant (`fields_have_valid_id` guard removed), `]` removed from
  `FORBIDDEN_START`, `}` removed from `FLOW_INDICATORS`. YAML differential (Ruby Psych) re-run:
  the remaining differences are scalar typing, Psych's `:x` symbol extension, and libyaml's YAML 1.1
  reading of `[a:b]`, `[:x]`, `[a ?b]`, all out of scope by the spec. Not verified: Windows,
  Linux (M0-T007).

**Review 3 changes (2026-11-08, fourth session; same session that wrote Review 3 below).**
Status → `review`. The next review must be done by a new session: this session wrote both
Review 3 and these changes.
1. `frontmatter.rs`: `WHITESPACE` (space, tab) replaces Unicode trimming in `normalize`,
   `parse_line`, `parse_list` and the indicator-before-space check. The indented-line check uses it too.
2. `frontmatter.rs`: `is_allowed_character` / `check_characters` reject, anywhere on a frontmatter
   line, C0 controls except tab, a `\r` that is not part of a CRLF ending, DEL, C1 controls (U+0085
   included), U+2028, U+2029, U+FFFE, U+FFFF. The error names the code point (`U+000D`).
3. Tests: `only_space_and_tab_are_trimmed_and_other_printable_characters_are_text` (U+00A0,
   U+3000, U+FEFF, tab inside a value, `-`/`?` before U+00A0, U+00A0 before `#`); 19 new rejection
   cases; `plain_text()` generates U+00A0, U+3000, `é` and an emoji, edges included;
   `invalid_line()` generates every `REJECTED_CHARACTERS` sample in values and list items; the
   arbitrary-input property draws from a dense alphabet of indicators, whitespace and rejected
   characters and no longer asserts with Unicode `trim`.
- Spec `spec_version` 6 (§6 details: definition of whitespace, rejected characters; History).
  `docs/process/DOCUMENTATION.md` updated.
- Verified (macOS 27.2, Rust 1.99.1): `cargo xtask ci` passes; 104 lib tests. Review 3's 12-value
  differential re-run on the real repository: the 8 values YAML rejects each give one R020 finding
  naming the code point. The 4 non-ASCII whitespace values are accepted and keep their characters
  (unit-tested), as YAML reads them. 15 single-line mutants of the new code: all killed. Not verified:
  Windows, Linux (M0-T007).
- Observation, not in scope: `markdown.rs` table cells use Unicode `trim`, but GFM trims only ASCII
  whitespace. An ADR title with a trailing U+00A0 would give a false R025 title mismatch. Not
  fixed: no rule requires it and the case is unlikely. A later task that touches `markdown.rs` should fix it.

## Review

2026-11-08, review session (owner-started, fresh context). Outcome: **changes required**, status →
`in-progress`.

Checked (REVIEW.md §1), macOS 27.2, Apple M5 Pro, Rust 1.99.1:
1. Acceptance criteria: `cargo xtask check-docs` on the real repository: ok (6 ms in `ci`). Every
   fixture fails with exactly its rule (`fixture_tests.rs`, read and run). Property tests read and
   run. `check-docs` is a `ci` step.
2. `cargo xtask ci`: pass (fmt, clippy, test, doc, check-docs).
3. Tests: read all of `check_docs/tests.rs`, `fixture_tests.rs`, `frontmatter/tests.rs`,
   `markdown.rs` tests. Own mutations (4): two killed (supersession iff in `rules.rs`, duplicate
   task ID), **two survived** (items 2 and 3 below).
4. No placeholders, no weakened tests; new fixtures are new oracles.
5. Spec agreement: checked `frontmatter.rs`, `check_docs/*.rs`, `markdown.rs`, `repo/*.rs`
   against spec v3 §6 details; gaps in items 1, 3 and 4.
6. Unsafe: none. 7. Determinism: n/a (BTree collections, sorted findings).
8. Docs: public items documented; STATUS.md and DOCUMENTATION.md current.
9. Architecture: `toml` (tool) and `proptest` (dev) are approved in `architecture.toml`.
10. Session boundaries: journal shows an owner-started session, no sub-agents.

Required changes:
1. **Parser accepts values YAML reads differently** (violates the intent of the acceptance criterion
   "rejected, never misparsed", and the spec's own rationale). Verified by injecting into a real
   task file; each was accepted with no finding:
   `title: Ends with colon:` (YAML: error), `title: - dash item` (YAML: block sequence, error),
   `title: ? question` (YAML: complex mapping key). In flow lists the same holds for items starting
   with `- `/`? ` and items ending with `:` (`[a:, b]` is a mapping in YAML). Reject a value or list
   item that starts with `-`, `?` or `:` followed by whitespace or the end of the value, or that
   ends with `:`. Add these forms to the spec's rejected list (bump `spec_version`), to the
   rejection unit test and to the `invalid_line()` property generator.
2. **ADR index status cell: backticks are accepted untested.** Mutant `backticks_allowed: true` for
   the ADR index survives. Spec says the ADR status cell *is* the status (only the spec index allows
   backticks). Add a test that `` `accepted` `` in the ADR index is an R025 finding.
3. **Footnote definitions: unspecified, untested behaviour.** `markdown::reference_definition`
   skips `[^label]: …`; removing that clause survives all tests. Either specify it (spec: footnote
   definitions are not reference definitions) and test it, or remove it.
4. **Misleading R023 cascade.** A task whose frontmatter fails R020 is dropped from the task set, so
   every task depending on it also gets "depends on M0-T009, which is not an existing task"
   (observed: a tab after `:` in M0-T009's title produced an R020 plus a false R023 on M0-T010).
   A task file whose frontmatter fails R020 still exists for `depends_on` purposes, by the task ID
   prefix of its file name. Specify this in the spec details and test it.

## Review 2

2026-11-08, third session (owner-started, fresh context; did not write M0-T003 or review 1).
Outcome: **changes required**, status → `in-progress`.

Checked (REVIEW.md §1), macOS 27.2, Apple M5 Pro, Rust 1.99.1:
1. Acceptance criteria: re-run, not trusted. `check-docs` on the real repository: ok. Fixture and
   property tests read and run. `check-docs` is a `ci` step.
2. `cargo xtask ci`: pass (fmt, clippy, test, doc, check-docs). 102 lib tests.
3. Review-1 changes: all four present and tested (diff of `edb37c5` read in full).
4. **Differential test against a real YAML parser** (Ruby 3.1 Psych/libyaml, YAML 1.1): 70 values
   and list items injected into `docs/tasks/M0-T009-deny-and-doctor.md` (reverted), `check-docs`
   result compared with what YAML reads. Script:
   scratchpad `diff.rb` (not committed). Findings in items 1 and 2 below. Differences that are not
   defects: scalar typing (`null`, `~`, `yes`, `.inf`, `0x1F`): the spec says "every other value is a
   string", so YAML type resolution is out of scope by design; `[a:b]`, `[:x]`, `[a ?b]`: libyaml
   (YAML 1.1) rejects them, YAML 1.2 reads them as plain strings, and the spec chooses the 1.2 reading.
5. Mutants (2): `chars.next().is_none_or(char::is_whitespace)` → `is_none()`: killed.
   `if id.is_none() && !fields_have_valid_id(..)` → `if id.is_none()`: **survived** (item 3).
6. No placeholders, no weakened tests, no oracle changes. Unsafe: none. Determinism: n/a.
7. Docs: spec v4 and DOCUMENTATION.md agree with the code, except for the gaps below.
8. Architecture: no new dependencies since review 1.
9. Session boundaries: journals show owner-started sessions, no sub-agents.

Required changes:
1. **Scalar values starting with `,` or `]` are accepted; YAML rejects them.** `title: ]x` and
   `title: ,x` give no finding (Psych: parse error). Both are YAML indicators (c-indicator); the
   forbidden-start list has every other one (`[` starts a list, `#` a comment). Add `,` and `]`
   to the rejected first characters (spec §6 details, DOCUMENTATION.md, unit test, `invalid_line()`
   generator, arbitrary-input property).
2. **List items containing `{` or `}` are accepted; YAML rejects them.** `[a{b}]` and `[a}]` give no
   finding (Psych: parse error). In a flow sequence a plain scalar must not contain a flow indicator
   (`,[]{}`) anywhere; the parser rejects only `[`, `]` (and `,` splits). Reject `{` and `}` anywhere
   in a list item, and state it where the spec and DOCUMENTATION.md list the forbidden item characters.
3. **Duplicate-ID branch of the file-name fallback is untested.** Removing
   `!fields_have_valid_id(&frontmatter)` in `check_docs/rules.rs` survives all tests. The spec says
   "a task file without a valid `id`" but does not say whether a well-formed `id` that duplicates
   another task's ID is "valid". Specify it (the code says yes: such a file is not added under its
   file-name ID) and add a test that fails under the mutant.

## Review 3

2026-11-08, fourth session (owner-started, fresh context; did not write M0-T003, review 1 or
review 2). Outcome: **changes required**, status → `in-progress`.

Checked (REVIEW.md §1), macOS 27.2, Apple M5 Pro, Rust 1.99.1:
1. Acceptance criteria: re-run, not trusted. `check-docs` on the real repository: ok (8 ms in
   `ci`). Fixture tests and the three property tests read. `check-docs` is a `ci` step.
2. `cargo xtask ci`: pass (fmt, clippy, test, doc, check-docs).
3. Review-2 changes: all three present and tested (diff of `ca36af0` read in full).
4. **Differential test with non-ASCII whitespace and control characters** against Ruby 2.6 Psych
   (libyaml): 12 `title:` values injected into `docs/tasks/M0-T009-deny-and-doctor.md` (reverted).
   Earlier differentials and every property generator used ASCII text only. Findings in item 1.
5. No placeholders, no weakened tests, no oracle changes. Unsafe: none. Determinism: n/a.
6. Docs: spec v5 and DOCUMENTATION.md agree with the code, except that "whitespace" and "trimmed"
   do not say which characters count (item 1).
7. Architecture: no new dependencies. Session boundaries: journals show owner-started sessions, no
   sub-agents.

Required changes:
1. **Non-ASCII whitespace is trimmed and control characters are accepted. YAML reads both
   differently.** `normalize` (`trim_end`), `parse_line` (`trim`) and `parse_list` (`trim`) strip
   every Unicode `White_Space` character. YAML's whitespace is only space and tab (`s-white`).
   Observed, each accepted by `check-docs` with no finding:
   - `title: x<U+00A0>` → parsed `x`, YAML `"x "`. Same for a leading U+00A0, a trailing
     U+3000, and `[a<U+00A0>, b]` (items parsed `a`, YAML `"a "`).
   - `title: a<U+0001>b`, `a<U+007F>b`, `x<U+000B>`, `a<U+000C>b`: YAML error (outside
     `c-printable`).
   - `title: a<CR>b` (bare `\r` inside a line): YAML reads `\r` as a line break. Error in YAML 1.1 and 1.2.

   Fix:
   (a) Trim only space and tab: trailing whitespace on a line, around values, around list items.
   `\r` before the line end of a CRLF file stays allowed.
   (b) Reject any other character outside YAML 1.2 `c-printable` with its line: C0 controls except
   tab, DEL, C1 controls, U+FFFE/U+FFFF, and a `\r` that is not part of a line ending. Also reject
   U+0085, U+2028 and U+2029: YAML 1.1 reads them as line breaks and 1.2 does not (libyaml errors
   on them), so the subset must not depend on the version.
   (c) Spec §6 details and DOCUMENTATION.md: state which whitespace is trimmed and which
   characters are rejected. Bump `spec_version`.
   (d) Tests: unit cases for each class above, and an `invalid_line()` generator case. The
   arbitrary-input property's `text.trim() == text` uses Unicode trim, the same mistake the code
   makes. Assert edge characters are neither space nor tab, and no rejected character appears.
   Include non-ASCII and control characters in its input; `.{0,40}` rarely produces them.
