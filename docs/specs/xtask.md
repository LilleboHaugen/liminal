---
crate: xtask
req_prefix: XTASK
layer: 5
maturity: planned
spec_version: 7
last_reviewed: 2026-11-08
adrs: [ADR-0001, ADR-0003, ADR-0004, ADR-0010, ADR-0011, ADR-0018]
---

# Spec: `xtask`

## 1. Purpose

`xtask` is the repository's automation and quality-gate tool, run as `cargo xtask <command>`. It turns
the project's written rules (architecture, traceability, hygiene, coverage, doc consistency) into
checks that pass or fail, so they hold without anyone remembering them. It is written in Rust so it runs
the same on macOS, Windows and Linux, with no shell scripts.

## 2. Scope

**In scope:** running the full CI sequence locally and in CI; architecture enforcement against
`architecture.toml`; doc frontmatter and link validation; requirement → test traceability; forbidden
pattern detection; coverage floors; STATUS.md generation; tool availability report.

**Out of scope:** building or packaging games (`liminal_cli`, M7); benchmarks and perf baselines
(added under OQ-004 in M1); asset cooking (`liminal_cook`).

## 3. Dependencies

- Engine crates: **none**. `xtask` MUST work even when engine crates do not compile.
- External (`tool` scope, ADR-0003): `serde`, `serde_json`, `toml`, `cargo_metadata`.
- External programs invoked: `cargo` (fmt, clippy, test, doc, metadata), `cargo-llvm-cov`, `cargo-deny`.
- Argument parsing is hand-written (no CLI-parsing crate), to keep `xtask` fast to build.
- Structure: all logic in a library target (`xtask/src/lib.rs` and modules), with a thin
  `xtask/src/main.rs`. Integration tests in `xtask/tests/` call the library directly, and doc tests run
  on it.
- One shared repo-model loader module (`xtask/src/repo.rs` or `repo/`) is used by every check. Workspace
  members come from `[workspace].members` in the root `Cargo.toml` (no cargo needed). Full dependency
  data comes from `cargo metadata` (added by M0-T004).

## 4. Concepts

- **Step:** one check within `ci` (e.g. `fmt`, `clippy`, `check-arch`). Each step ends as `pass`,
  `fail` or `skipped` (with a reason).
- **Repo model:** the parsed set of `architecture.toml`, spec/task/ADR frontmatter, and the workspace
  metadata. Every check is a pure function from the repo model (plus file contents) to a list of
  findings. Only the loaders touch the file system or spawn processes. This keeps checks testable on
  fixtures without running cargo.
- **Finding:** `{ rule: XTASK-Rnnn, severity: error|warning, location: path[:line], message, help }`.

## 5. Public API (command line)

```
cargo xtask help
cargo xtask ci [--json] [--fail-fast] [--only <step>[,<step>…]] [--skip <step>[,<step>…]] [--allow-skip <step>[,…]]
cargo xtask check-arch [--json]
cargo xtask check-docs [--json]
cargo xtask trace [--json] [--crate <name>]
cargo xtask hygiene [--json]
cargo xtask coverage [--json] [--update-floors]
cargo xtask status [--check]
cargo xtask doctor [--json]
```

`ci` steps, in order: `fmt`, `clippy`, `test`, `doc`, `check-arch`, `check-docs`, `trace`, `hygiene`,
`status-check`, `coverage`, `deny`.

During M0 each step is added to `ci` by the task that implements it. A step that does not exist yet is
absent from `ci`. It is never stubbed as a pass. The order above is the final order.

JSON summary emitted by `ci --json` (and always written to `target/xtask/ci-summary.json`):

```json
{
  "version": 1,
  "ok": false,
  "steps": [
    { "name": "fmt", "status": "pass", "duration_ms": 412 },
    { "name": "coverage", "status": "skipped", "duration_ms": 3,
      "reason": "cargo-llvm-cov not installed", "help": "cargo install cargo-llvm-cov" },
    { "name": "trace", "status": "fail", "duration_ms": 51,
      "findings": [ { "rule": "XTASK-R044", "severity": "error",
                      "location": "docs/specs/liminal_ecs.md", "message": "...", "help": "..." } ] }
  ]
}
```

Summary details (added in `spec_version` 2):

- Step objects contain `name`, `status`, `duration_ms` (wall-clock milliseconds), and only the
  optional fields that apply: `reason` and `help` for `skipped`; `findings` (omitted when empty)
  and `output_tail` for `fail`.
- `output_tail`: the last 50 lines of a failed process step's combined stdout and stderr, in the
  order the child wrote them. Present only when output is captured, which is in `--json` mode. In
  human mode the child writes directly to the terminal and `output_tail` is absent.
- `--fail-fast` stops after the first step that makes `ok` false (a `fail`, or a `skipped` step not
  named in `--allow-skip`). Steps after it do not run and are absent from `steps`.
- List options accept `--only fmt,clippy` and `--only=fmt,clippy`, and may be repeated (names
  accumulate). Unknown names in `--only`, `--skip` or `--allow-skip` are usage errors (exit 2), and
  so is a selection that leaves no step to run. Selected steps always run in §5 order, whatever the
  order of names on the command line.

## 6. Requirements

### General and `ci` (R001–R019)

| ID | Level | Requirement |
|---|---|---|
| XTASK-R001 | MUST | The alias `xtask = "run --package xtask --"` is defined in `.cargo/config.toml`, so `cargo xtask <command>` works from any directory in the repository. |
| XTASK-R002 | MUST | `help` (and no arguments) prints the list of implemented commands with one-line descriptions. Unknown commands or options print usage to stderr and exit with code 2. |
| XTASK-R003 | MUST | `ci` runs its steps in the order listed in §5. |
| XTASK-R004 | MUST | `ci` runs every selected step even if an earlier step fails, unless `--fail-fast` is given. |
| XTASK-R005 | MUST | `ci` writes the JSON summary (§5 schema) to `target/xtask/ci-summary.json`, and to stdout when `--json` is given. |
| XTASK-R006 | MUST | A step that cannot run (missing tool, unsupported platform) is reported as `skipped` with `reason` and `help`. `ok` is `false` if any step failed or was skipped, unless that step is named in `--allow-skip`. A skip is never reported as a pass. |
| XTASK-R007 | MUST | Exit codes: 0 if `ok` is true, 1 if any step failed or was skipped without being allowed, 2 for usage errors. |
| XTASK-R008 | SHOULD | `--only` and `--skip` select steps by name. Unknown step names are a usage error (exit 2). Deselected steps are omitted from `steps` in the summary, and they do not affect `ok`. Deselecting is not skipping. |
| XTASK-R009 | MUST | Every check command (`check-arch`, `check-docs`, `trace`, `hygiene`, `coverage`, `doctor`) supports `--json`, which outputs its findings in the `findings` format of §5. Each command's implementing task provides this for that command. |
| XTASK-R010 | MUST | Check commands MUST NOT modify files outside `target/`. Only `status` (without `--check`) and `coverage --update-floors` write to the repository. |
| XTASK-R011 | MUST | `xtask` has no dependency on any `liminal_*` crate. |
| XTASK-R012 | MUST | The `clippy` step runs `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`. The `doc` step runs `cargo doc --workspace --no-deps --all-features --locked` with `RUSTDOCFLAGS=-D warnings`. The `test` step runs `cargo test --workspace --all-features --locked`. The `fmt` step runs `cargo fmt --all --check`. (`--all-features` is revisited at M3 kickoff, when backend and GPU-test features exist; see OQ-007.) |
| XTASK-R013 | SHOULD | The xtask-native checks (`check-arch`, `check-docs`, `trace`, `hygiene`, `status-check`) complete in under 5 seconds in total on the reference machine for the repository at its current size. |

### `check-docs` (R020–R029)

| ID | Level | Requirement |
|---|---|---|
| XTASK-R020 | MUST | Frontmatter is parsed according to the restricted YAML subset in `docs/process/DOCUMENTATION.md`. Anything outside the subset is an error reported with file and line. |
| XTASK-R021 | MUST | ADR files (`docs/adr/NNNN-*.md`, excluding `0000-template.md`): required fields `id, title, status, date, supersedes, superseded_by`. `id` matches the file number. `status` ∈ {proposed, accepted, superseded, rejected}. Every ID in `supersedes`/`superseded_by` exists, and the links are symmetric. `status: superseded` if and only if `superseded_by` is non-empty. |
| XTASK-R022 | MUST | Spec files (`docs/specs/*.md`, excluding `_TEMPLATE.md` and `README.md`): required fields `crate, req_prefix, layer, maturity, spec_version, last_reviewed, adrs`. `crate` exists in `architecture.toml`, and `layer` matches it. `maturity` ∈ {planned, experimental, usable, stable, optimized}. `req_prefix` is unique across specs. Each ADR listed exists. |
| XTASK-R023 | MUST | Task files (`docs/tasks/M*-T*.md`): required fields `id, title, milestone, status, crates, depends_on, requirements`. `id` matches `^M\d+-T\d{3}$` and is the filename prefix, and IDs are unique. `status` ∈ {todo, in-progress, blocked, review, done}. Each `depends_on` ID exists, and the dependency graph has no cycles. Each crate in `crates` exists in `architecture.toml`. |
| XTASK-R024 | MUST | Every relative Markdown link in `*.md` files at the repository root and under `docs/` resolves to an existing file or directory. External links (`http(s)://`) are not checked. |
| XTASK-R025 | MUST | `docs/adr/README.md` lists every ADR exactly once, with the same status as its frontmatter. |
| XTASK-R026 | MUST | Every workspace member crate has a spec at `docs/specs/<crate>.md`. |
| XTASK-R027 | SHOULD | `docs/specs/README.md` lists every spec file, with the same maturity as its frontmatter. |
| XTASK-R028 | MUST | Task `requirements` entries refer to requirement IDs that exist in some spec (see R040). |

`check-docs` details (added in `spec_version` 3):

- Frontmatter subset, exact form (R020). Line 1 of the file is `---`, and the next line that is
  exactly `---` closes the frontmatter. "Whitespace" in this grammar means space and tab only, as in
  YAML (`s-white`); U+00A0 and other Unicode whitespace are ordinary text, also at the edges of a
  value and before `#` (`spec_version` 6). Trailing whitespace and the `\r` of a CRLF line ending
  are ignored on every line. A frontmatter line (key, value and comment) must not contain any
  other `\r`, a C0 control other than tab, DEL, a C1 control (U+0080–U+009F, including U+0085),
  U+2028, U+2029, U+FEFF, U+FFFE or U+FFFF (`spec_version` 6; U+FEFF added in `spec_version` 7):
  YAML 1.2 rejects them, because keys, values and comments are made of `nb-char` (`c-printable`
  minus line breaks and the byte order mark U+FEFF), or reads them as a line break (`\r`; U+0085,
  U+2028, U+2029 in YAML 1.1). The subset must mean the same in both versions; YAML 1.1 reads
  U+FEFF inside a value as text, YAML 1.2 rejects it. A byte order mark before line 1's `---` is
  rejected too (line 1 is not `---`). Between the delimiters, every line is `key: value` with no indentation. A key matches
  `^[a-z][a-z0-9_]*$` and is followed by `:` and at least one space or tab. Each key appears at most
  once. A comment starts at a `#` preceded by a space or tab and runs to the end of the line. Rejected:
  blank lines, comment-only lines, an empty value (an empty list is `[]`), and a value or list item
  that contains `: `, starts with one of `"`, `'`, `|`, `>`, `&`, `*`, `!`, `{`, `}`, `%`, `@`,
  `` ` ``, `,` or `]` (`,` and `]` added in `spec_version` 5), starts with `-`, `?` or `:`
  followed by whitespace or the end of the text, or ends with `:` (added in `spec_version` 4).
  YAML would read these differently (quoting, block scalars, sequence entries, complex keys,
  mappings), and the subset has no quoting. `-1`, `--all`, `?x` and `a:b` are plain strings. A
  value that starts with `[` is a list. It ends with `]`, and its items, split on `,` and trimmed
  of whitespace,
  are non-empty and contain none of the flow indicators `[`, `]`, `{`, `}` (`{` and `}` added in
  `spec_version` 5; YAML rejects them inside a flow sequence item). Every other value is a string:
  YAML type resolution (`null`, `true`, `.inf`, …) is not modelled. Integers (`^\d+$`) and dates
  (`^\d{4}-\d{2}-\d{2}$`, calendar not checked) are strings that the field rules check.
- Field rules (R021–R023). A field not in the rule's required list is an error, which catches typos
  such as `superseeded_by`. Values have these forms: ADR IDs `^ADR-\d{4}$`; `title` non-empty; dates
  as above; `layer` and `spec_version` integers (`spec_version` ≥ 1); `req_prefix` `^[A-Z]+$`; task
  `milestone` `^M\d+$`, equal to the milestone part of `id`. A spec's `crate` equals its file name
  without `.md`. A file whose frontmatter fails R020 is excluded from R021–R023 and from the status
  and maturity comparisons of R025 and R027. For the `depends_on` existence check (R023), a task
  file without a valid `id` (including one that fails R020) exists under the task ID its file name
  starts with (`^M\d+-T\d{3}` before the next `-` or `.md`), if any. This avoids a second, false
  finding on every task that depends on it. Its own `depends_on` is not known and not checked.
  A task file whose `id` has the task ID form but duplicates another file's `id` has a valid `id`:
  it is known by that ID only, not also by its file-name ID (`spec_version` 5).
- Links (R024): inline links and images (`[text](target)`, `![alt](target)`, also
  `<target>` and an optional title) and reference definitions (`[label]: target`). Ignored: fenced
  code blocks (```` ``` ```` or `~~~`), inline code spans (on one line), footnote definitions
  (`[^label]: text` is not a reference definition; links inside its text count), targets with a URI scheme
  (`^[A-Za-z][A-Za-z0-9+.-]*:`, e.g. `https:`, `mailto:`), and pure anchors (`#…`). From the
  target, `#…` and `?…` are removed. The rest is resolved against the linking file's directory, or
  against the repository root if it starts with `/`. It is not percent-decoded. A target that
  leaves the repository is broken.
- Indexes (R025, R027): the index is every Markdown table in `docs/adr/README.md` whose header has
  first cell `ADR` and a `Status` column (in `docs/specs/README.md`: first cell `Spec` and a
  `Maturity` column). The row's first cell links to the file. The ADR status cell is the frontmatter
  `status`. For a superseded ADR it is `superseded by <IDs of superseded_by, joined by ", ">`. The
  spec maturity cell is the frontmatter `maturity`, optionally in backticks. A row that links to a
  file which is not an ADR (spec) file is an error.
- Workspace members (R026) come from `[workspace].members` in the root `Cargo.toml`. A member is a
  directory path, or `<dir>/*` (every direct subdirectory of `<dir>` containing a `Cargo.toml`).
  Other glob patterns are internal error LIM-XTASK-0004. The crate name is `[package].name` of the
  member's `Cargo.toml`.

Check command output (R009, all check commands): human mode prints, per finding,
`<severity> <rule> <location>: <message>` and an indented `help:` line, then
`<command>: ok` or `<command>: <n> finding(s)`. `--json` prints
`{ "version": 1, "ok": <bool>, "findings": [ … ] }` (pretty-printed, findings in the §5 format).
Exit code 0 without error findings, 1 otherwise. An internal error (§8) prints `error: <code> …` to
stderr and exits 1. Findings are sorted by location (path, then line number; a location without a line comes
first), then rule, then message.

In `ci`, a native check step (one that runs inside xtask, without spawning a process) reports its
findings in the step's `findings`. An internal error makes the step `fail`, with the error message
as `output_tail` in both output modes.

### `check-arch` (R030–R039)

| ID | Level | Requirement |
|---|---|---|
| XTASK-R030 | MUST | `architecture.toml` is validated: `schema_version = 1`; every `[[crate]]` has `name, path, layer, kind, unsafe, deterministic, same_layer_deps, external, dev_external` with valid values (layer 0–5, kind ∈ {lib, proc-macro, bin}, unsafe ∈ {forbid, allow}). Crate names are unique. Every `same_layer_deps` entry is a registered crate of the same layer. Every `external`/`dev_external` entry has an `[external.<name>]` entry with `scope` ∈ {runtime, build, dev, tool}, `adr` and `reason`. |
| XTASK-R031 | MUST | Every workspace member is registered, with a `path` equal to its manifest directory (relative to the repository root, compared with `/` separators on every OS) and a matching `kind` (`bin` = has at least one binary target, and may also have a library target; see the `architecture.toml` header). |
| XTASK-R032 | MUST | For every workspace crate, every dependency on another workspace crate (normal, build or dev) targets a strictly lower layer, or a crate listed in the dependent's `same_layer_deps`. |
| XTASK-R033 | MUST | Every external normal or build dependency is listed in the crate's `external`, and its scope is allowed: layers 0–4 may use `runtime` and `build` (`build` only as a build-dependency or from a `proc-macro` crate); layer 5 may use `runtime`, `build` and `tool`. |
| XTASK-R034 | MUST | Every external dev-dependency is listed in the crate's `dev_external`, with scope `dev` (layer 5 crates may also use `tool`). |
| XTASK-R035 | MUST | Every crate with `unsafe = "forbid"` contains `#![forbid(unsafe_code)]` in its crate root file. |
| XTASK-R036 | MUST | No crate in layers 0–4 depends, with any dependency kind, on a layer 5 crate. |
| XTASK-R037 | MUST | Each violation is reported with the crate, the dependency (if any), the rule ID, and a `help` text saying how to fix it. |
| XTASK-R038 | SHOULD | From M2: `liminal_engine` and crates under `games/` depend only on crates whose spec maturity is `usable` or higher (ADR-0011). |

### `trace` and `hygiene` (R040–R059)

| ID | Level | Requirement |
|---|---|---|
| XTASK-R040 | MUST | Requirements are extracted from tables in section 6 of each spec: a row whose first cell matches `^[A-Z]+-R\d{3}$`, whose second cell is MUST, SHOULD or MAY, and whose third cell is the text. A requirement is withdrawn if its text contains `(withdrawn`. The prefix MUST equal the spec's `req_prefix`. |
| XTASK-R041 | MUST | Test annotations are lines in `.rs` files of workspace crates matching `^\s*///\s*Verifies:\s*ID(\s*,\s*ID)*\s*$`. Each annotation is attached to the next `fn` item, and the function name is recorded. |
| XTASK-R042 | MUST | `trace` reports, per requirement, the verifying tests (`path:line fn_name`), and per crate the count of MUST/SHOULD/MAY requirements with and without tests. |
| XTASK-R043 | MUST | An annotation naming an unknown or withdrawn requirement ID is an error. |
| XTASK-R044 | MUST | For each spec whose `maturity` is `usable`, `stable` or `optimized`, every non-withdrawn MUST requirement without a verifying test is an error. For lower maturities it is reported as information only. |
| XTASK-R045 | MUST | Duplicate requirement IDs (within or across specs) are an error. |
| XTASK-R050 | MUST | `hygiene` reports, in `.rs` files of the workspace: `todo!(`, `unimplemented!(`, `dbg!(`; comments containing `TODO`, `FIXME`, `XXX` or `HACK` not immediately followed by `(M<n>-T<nnn>)`; and `#[ignore]` without a reason string containing a task ID. |
| XTASK-R051 | MUST | Task IDs referenced by TODO comments exist in `docs/tasks/` and are not `done` (a TODO pointing at a finished task is stale). Task IDs in `#[ignore]` reasons only need to exist: long-lived ignores (e.g. "needs network", "needs GPU") legitimately outlive the task that added them. |
| XTASK-R052 | MUST | Paths excluded from `hygiene` are listed in `ci/hygiene.toml`, each with a `reason` field. No other exclusion mechanism exists. |

### `coverage` (R060–R069)

| ID | Level | Requirement |
|---|---|---|
| XTASK-R060 | MUST | `coverage` runs `cargo llvm-cov --workspace --all-features --json --summary-only` and computes line coverage per workspace crate. |
| XTASK-R061 | MUST | Each crate's coverage is compared with its floor in `ci/coverage.toml`. Coverage below `floor − 0.5` percentage points is an error naming the crate, the floor and the actual value. |
| XTASK-R062 | MUST | A workspace crate without an entry in `ci/coverage.toml` is an error, with help text showing the line to add. |
| XTASK-R063 | MUST | `--update-floors` sets each floor to `max(current floor, measured value rounded down to 0.1)` and never lowers a floor. |
| XTASK-R064 | MUST | If `cargo-llvm-cov` is not installed, the step is `skipped` with an install hint (R006). |

### `status` (R070–R079)

| ID | Level | Requirement |
|---|---|---|
| XTASK-R070 | MUST | `status` generates `docs/STATUS.md` with: the current milestone (lowest milestone with any task not `done`); a table of that milestone's tasks (id, title, status, depends_on); a table of every crate in `architecture.toml` with layer and maturity (`planned` if no spec); a list of `blocked` tasks; and a link to the hand-written `docs/STATUS_NOTES.md`. |
| XTASK-R071 | MUST | Output is deterministic for a given repository state: sorted tables, no timestamps. The file starts with a notice that it is generated and must not be edited by hand. |
| XTASK-R072 | MUST | `status --check` (the `status-check` step in `ci`) fails if `docs/STATUS.md` differs from the generated content. |

### `doctor` and `deny` (R080–R089)

| ID | Level | Requirement |
|---|---|---|
| XTASK-R080 | MUST | `doctor` reports, for each tool (rustc, cargo, rustfmt, clippy, cargo-llvm-cov, cargo-deny, nightly toolchain with Miri), whether it is present, its version, and an install hint if missing. |
| XTASK-R081 | MUST | The `deny` step runs `cargo deny check` with the repository `deny.toml`. If `cargo-deny` is missing, the step is skipped (R006). |

## 7. Invariants

- INV-1: Check commands are pure with respect to the repository (R010). Running a check twice gives
  identical findings.
- INV-2: Every finding names a rule ID from this spec.
- INV-3: Check logic never spawns processes. Loaders do, and their output is parsed into the repo model.

## 8. Error codes

`xtask` reports findings by rule ID (`XTASK-Rnnn`) instead of `LIM-` error codes: its "errors" are
violations of repository rules, not runtime errors. Internal failures (cannot read file, tool output
unparseable) use:

| Code | Meaning | Help |
|---|---|---|
| LIM-XTASK-0001 | File could not be read | Check the path and permissions |
| LIM-XTASK-0002 | External tool output could not be parsed | Run `cargo xtask doctor`; tool version may be unsupported |
| LIM-XTASK-0003 | `architecture.toml` is not valid TOML | Fix the syntax at the reported line |
| LIM-XTASK-0004 | Another TOML file (`Cargo.toml`) is invalid, misses a field xtask needs, or uses an unsupported workspace member pattern | Fix the file at the reported location |

## 9. Performance budgets

None yet beyond XTASK-R013. Set at `stable` from measurements.

## 10. Test plan

| Area | Test kinds | Oracle |
|---|---|---|
| Frontmatter parser | unit, property (round-trip of generated valid frontmatter; random invalid lines are rejected, never accepted) | Subset definition in DOCUMENTATION.md |
| Each check rule | integration tests on fixture repositories in `xtask/tests/fixtures/<case>/` | One passing fixture plus one fixture per rule that must fail with exactly that rule ID |
| `check-arch` | unit tests on a hand-built repo model; integration on fixture `cargo metadata` JSON saved in fixtures | Rules R030–R037 (R038 from M2) |
| `ci` orchestration | unit tests with a fake step runner (steps are a trait; real runner spawns processes) | Order, continue-on-failure, skip semantics, exit codes |
| `status` | snapshot test of generated STATUS.md for a fixture | Committed snapshot |
| Real repository | `cargo xtask ci` on the actual repository in CI | Must pass |

The fake step runner is a test double for process spawning only, and is used solely to test
orchestration logic. Every real step is also exercised in CI against the real repository.

## 11. Determinism

Not a deterministic crate in the ADR-0009 sense, but all output MUST be deterministic for a given
repository state (R071, INV-1).

## 12. Open questions

- None blocking. Benchmark and perf-baseline commands are added after OQ-004 is resolved (M1).

## 13. Maturity

Current: `planned`. Next: `experimental` once M0-T001/T002 land.

Promotion to `usable` (M0 exit):
- [ ] All MUST requirements traced (`cargo xtask trace --crate xtask`)
- [ ] Coverage ≥ 75%
- [ ] Each check shown to fail on its fixture
- [ ] Review by a separate session

## 14. History

| spec_version | Date | Change | By task |
|---|---|---|---|
| 1 | 2026-11-08 | Initial spec | Planning session |
| 2 | 2026-11-08 | §5 summary details: optional `output_tail`, `--fail-fast` stopping rule, list option forms, empty selection is a usage error | M0-T002 |
| 3 | 2026-11-08 | `check-docs` details (exact frontmatter subset, field forms, unknown fields, link and index rules, member patterns), check command output, native `ci` steps, LIM-XTASK-0004 | M0-T003 |
| 4 | 2026-11-08 | Frontmatter: reject values starting with `- `, `? `, `: ` or ending with `:`. Links: footnote definitions. R023: a task file without a valid `id` exists by its file-name ID for `depends_on` | M0-T003 (review changes) |
| 5 | 2026-11-08 | Frontmatter: reject values starting with `,` or `]`, and list items containing `{` or `}`. R023: a duplicate but well-formed `id` is a valid `id` for the file-name fallback | M0-T003 (review 2 changes) |
| 6 | 2026-11-08 | Frontmatter: whitespace is space and tab only (U+00A0 etc. are text); reject control characters, a `\r` that is not a line ending, U+0085, U+2028, U+2029, U+FFFE, U+FFFF | M0-T003 (review 3 changes) |
| 7 | 2026-11-08 | Frontmatter: reject U+FEFF (not in YAML 1.2 `nb-char`) | M0-T003 (review 4 changes) |
