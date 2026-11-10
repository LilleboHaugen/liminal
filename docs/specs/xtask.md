---
crate: xtask
req_prefix: XTASK
layer: 5
maturity: usable
spec_version: 27
last_reviewed: 2026-11-09
adrs: [ADR-0001, ADR-0003, ADR-0004, ADR-0009, ADR-0010, ADR-0011, ADR-0018]
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
(OQ-004 is resolved by ADR-0023; `cargo xtask bench` is specified here by M1-T017, which adds
it); asset cooking (`liminal_cook`).

## 3. Dependencies

- Engine crates: **none**. `xtask` MUST work even when engine crates do not compile.
- External (`tool` scope, ADR-0003): `serde`, `serde_json`, `toml`. `cargo_metadata` is approved
  but not used (`spec_version` 8): `check-arch` reads the few fields it needs from
  `cargo metadata` JSON with `serde_json`, which adds no transitive dependencies.
- External programs invoked: `cargo` (fmt, clippy, test, doc, metadata), `cargo-llvm-cov`, `cargo-deny`.
- Argument parsing is hand-written (no CLI-parsing crate), to keep `xtask` fast to build.
- Structure: all logic in a library target (`xtask/src/lib.rs` and modules), with a thin
  `xtask/src/main.rs`. Integration tests in `xtask/tests/` call the library directly or run the real
  binary (`CARGO_BIN_EXE_xtask`, one test per command at least), and doc tests run on it.
- One shared repo-model loader module (`xtask/src/repo.rs` or `repo/`) is used by every check. Workspace
  members come from `[workspace].members` in the root `Cargo.toml` (no cargo needed). Full dependency
  data comes from `cargo metadata --format-version 1 --no-deps --offline`, run in the repository
  root by the `check-arch` loader (M0-T004).

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

`ci` steps, in order: `fmt`, `clippy`, `test`, `test-features-off`, `doc`, `check-arch`, `check-docs`,
`trace`, `hygiene`, `status-check`, `coverage`, `deny`.

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
| XTASK-R014 | MUST | The `test-features-off` step runs `cargo test --package <name> --no-default-features --locked` once for each workspace package that has at least one Cargo feature, and for no other package. It passes if every such command passes, and also if no workspace package has a feature. (`spec_version` 24; the `test` step of R012 uses `--all-features`, which never compiles code under `cfg(not(feature = "…"))`.) |
| XTASK-R015 | MUST | Every command except `help` first checks that the repository root (the parent of the `xtask/` manifest directory, fixed when xtask is compiled) is a directory that contains `architecture.toml`. If not (for example, the checkout was moved or renamed after xtask was built, and cargo did not rebuild it), the command runs nothing else: it prints exactly one line, internal error LIM-XTASK-0007 naming that root and the fix, to stderr, nothing to stdout, and exits with code 1 (`spec_version` 27). |

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
  without `.md`, so `crate` is unique across specs: a second spec for one crate has another file
  name and is reported (`spec_version` 20). A file whose frontmatter fails R020 is excluded from R021–R023 and from the status
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
- Markdown tables and fences (R024, R025, R027; `spec_version` 18): as in GitHub Flavored
  Markdown, a table line starts with `|` after optional spaces and tabs, and each cell is trimmed
  of spaces and tabs only. U+00A0 and other Unicode whitespace are cell text, so an index cell
  `accepted` followed by U+00A0 does not match the status `accepted`. A closing code fence may be
  followed only by spaces and tabs.
- A file whose line 1 is U+FEFF followed by `---` (UTF-8 with byte order mark) is rejected at
  line 1 (R020); the message names U+FEFF (`spec_version` 18).
- Workspace members (R026) come from `[workspace].members` in the root `Cargo.toml`. A member is a
  directory path, or `<dir>/*` (every direct subdirectory of `<dir>` containing a `Cargo.toml`).
  Other glob patterns are internal error LIM-XTASK-0004. The crate name is `[package].name` of the
  member's `Cargo.toml`. A member entry is normalized before use (added in `spec_version` 19):
  `.` segments, empty segments and a trailing `/` are removed (`./xtask` and `xtask/./` are
  `xtask`), so member directories match repository-relative file paths in every command.

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

`test-features-off` details (R014; added in `spec_version` 24):

- Packages and their features come from `cargo metadata --format-version 1 --no-deps --offline`
  run in the repository root (the `check-arch` loader). A package "has a feature" if its
  `features` object in that output is not empty. Cargo lists there every key of `[features]`
  (`default` included) and the implicit feature of each optional dependency that no feature names
  with `dep:`. Packages are tested in ascending name order.
- One command per package, selected with `--package`, not one `--workspace` command: Cargo
  unifies features over the packages of one command, so with `--workspace` another member that
  depends on the package with a feature enabled would turn that feature on again.
- The step stops at the first command that fails; later packages are not tested. In `--json`
  mode `output_tail` is `cargo test --package <name> --no-default-features --locked` on its own
  line, followed by the last 50 lines of that command's output. In human mode the progress line
  is `==> test-features-off: cargo test --package <crate> --no-default-features --locked (once
  per workspace crate with features)`.
- If `cargo` cannot be started, the step is `skipped` (R006). If `cargo metadata` fails or its
  output cannot be parsed (LIM-XTASK-0005, LIM-XTASK-0002), the step fails with the error message
  as `output_tail` in both output modes.
- Not covered: building each feature alone (`--no-default-features --features x`, CODE_STYLE.md
  "Every feature must build alone") and combinations of features. No command checks them yet.
- Tests: the step's logic over a package list with a fake command function; the real step
  (real `cargo metadata` and `cargo test`) on a two-package workspace that the test writes to a
  temporary directory, where a package without features has a test that always fails, and the
  package with a default-on feature has one always-failing test that exists only with the
  feature and one test that exists only without it.

### `check-arch` (R030–R039)

| ID | Level | Requirement |
|---|---|---|
| XTASK-R030 | MUST | `architecture.toml` is validated: `schema_version = 1`; every `[[crate]]` has `name, path, layer, kind, unsafe, deterministic, same_layer_deps, external, dev_external` with valid values (layer 0–5, kind ∈ {lib, proc-macro, bin}, unsafe ∈ {forbid, allow}). Crate names are unique. Every `same_layer_deps` entry is a registered crate of the same layer. Every `external`/`dev_external` entry has an `[external.<name>]` entry with `scope` ∈ {runtime, build, dev, tool}, `adr` and `reason`. A missing or unknown field, or a value of the wrong TOML type, is internal error LIM-XTASK-0003 with its line (no model can be built); every other violation is an R030 finding (`spec_version` 8). |
| XTASK-R031 | MUST | Every workspace member is registered, with a `path` equal to its manifest directory (relative to the repository root, compared with `/` separators on every OS) and a matching `kind` (`bin` = has at least one binary target, and may also have a library target; see the `architecture.toml` header). |
| XTASK-R032 | MUST | For every workspace crate, every dependency on another workspace crate (normal, build or dev) targets a strictly lower layer, or a crate listed in the dependent's `same_layer_deps`. |
| XTASK-R033 | MUST | Every external normal or build dependency is listed in the crate's `external`, and its scope is allowed: layers 0–4 may use `runtime` and `build` (`build` only as a build-dependency or from a `proc-macro` crate); layer 5 may use `runtime`, `build` and `tool`. |
| XTASK-R034 | MUST | Every external dev-dependency is listed in the crate's `dev_external`, with scope `dev` (layer 5 crates may also use `tool`). |
| XTASK-R035 | MUST | Every crate with `unsafe = "forbid"` contains `#![forbid(unsafe_code)]` in its crate root file. |
| XTASK-R036 | MUST | No crate in layers 0–4 depends, with any dependency kind, on a layer 5 crate. |
| XTASK-R037 | MUST | Each violation is reported with the crate, the dependency (if any), the rule ID, and a `help` text saying how to fix it. |
| XTASK-R038 | SHOULD | From M2: `liminal_engine` and crates under `games/` depend only on crates whose spec maturity is `usable` or higher (ADR-0011). |
| XTASK-R039 | MUST | For every workspace crate registered with `deterministic = true`, the clippy configuration clippy applies to it (the nearest `.clippy.toml` or `clippy.toml`, see details) has a `disallowed-methods` list that contains every path of the banned list in `docs/process/CODE_STYLE.md` §Lints: `f32::sin`, `f32::cos`, `f32::tan`, `f32::powf`, `f32::exp`, `f32::ln`, the same six for `f64`, `std::collections::HashMap::new` and `std::time::Instant::now` (ADR-0009 item 7; `spec_version` 23). |

`check-arch` details (added in `spec_version` 8):

- Model. The loader runs `cargo metadata --format-version 1 --no-deps --offline` in the
  repository root (`$CARGO`, else `cargo` from `PATH`). The workspace crates are the packages
  listed in `workspace_members`. Every path is made relative to `workspace_root` after replacing
  `\` with `/`; a member manifest or target outside it is LIM-XTASK-0002. A dependency's name
  is its package name (the crates.io name, not a `rename`). Kind `null` is a normal dependency;
  target-specific dependencies count like unconditional ones. A dependency is a **workspace
  dependency** if it is a path dependency whose directory is a workspace crate's manifest
  directory; every other dependency is **external**. The crate root files (R035) are the root
  files of targets of kind `lib`, `rlib`, `dylib`, `cdylib`, `staticlib`, `proc-macro` and
  `bin`. Tests, examples, benches and build scripts are not crate roots.
- R030 values. `[layers]` has exactly the keys `"0"` … `"5"`. A crate registered twice is
  reported at its second entry. `same_layer_deps` entries in another layer, lower or higher,
  are findings. `[external.*]` `adr` matches `^ADR-\d{4}$` and `reason` is not empty or
  whitespace only. Findings about an entry are located at its line
  (`architecture.toml:<line of [[crate]] or [external.name]>`), others at `architecture.toml`.
- R031. The kind a crate must be registered with follows from its targets: `proc-macro` if it
  has a proc-macro target, otherwise `bin` if it has a binary target, otherwise `lib`. A
  registered kind that is not valid is reported by R030 only.
- Rules R032–R036 skip a workspace crate that is not registered (R031 reports it) or is
  registered with a layer outside 0–5 (R030 reports it): no finding of R032–R036 is reported for
  such a crate, and none for a dependency on it (wording clarified in `spec_version` 9).
- R032 and R036. A dependency of any kind from a layer 0–4 crate on a layer 5 crate is
  reported as R036 only, not also as R032. Any other workspace dependency is allowed if the
  target's layer is lower, or the target is listed in the dependent's `same_layer_deps`.
- R033 and R034. An external normal or build dependency missing from the crate's `external`
  list is an R033 finding; an external dev-dependency missing from `dev_external` is an R034
  finding. A listed crate without an `[external.*]` entry, or with an invalid scope, is reported
  by R030 only. Otherwise the scope decides:

  | Crate | Dependency kind | `runtime` | `build` | `dev` | `tool` |
  |---|---|---|---|---|---|
  | layer 0–4, kind `lib` or `bin` | normal | yes | no | no | no |
  | layer 0–4, kind `proc-macro` | normal | yes | yes | no | no |
  | layer 0–4, any kind | build | yes | yes | no | no |
  | layer 5 | normal or build | yes | yes | no | yes |
  | layer 0–4 | dev | no | no | yes | no |
  | layer 5 | dev | no | no | yes | yes |
- R035. A crate root contains `#![forbid(unsafe_code)]` if one of its leading inner attributes
  (the `#![…]` attributes before the first item; whitespace, comments, doc comments, a byte
  order mark and a shebang line are skipped) is `forbid(…)` with `unsafe_code` as one of its
  comma-separated items. `#![cfg_attr(…, forbid(unsafe_code))]`, an inner attribute inside an
  inline module, and text in comments or string literals do not count.
- R039. The effective configuration of a crate is found as clippy finds it: in the crate's
  manifest directory, then in each parent directory up to and including the repository root,
  the first directory containing `.clippy.toml` or `clippy.toml`; `.clippy.toml` wins within
  a directory. Clippy does not merge files (verified with clippy 0.1.99, M1-T011 journal), so
  the root `clippy.toml` counts only for a crate without a nearer file. Directories above the
  repository root and `CLIPPY_CONF_DIR` are not considered. A `disallowed-methods` entry is a
  string or a table whose string `path` is the banned path; paths are compared as exact strings,
  and extra entries are allowed. A missing `disallowed-methods` key is an empty list. A file
  that is not valid TOML, a `disallowed-methods` that is not an array, or an entry of another
  shape is LIM-XTASK-0004. Only crates registered as deterministic (the first `[[crate]]` entry
  of the name) are judged; the layer does not matter. One finding per crate: at
  `<manifest directory>/Cargo.toml` if no file applies, else at the effective file, naming every
  missing path in the list's order. The help names `<manifest directory>/clippy.toml`, the
  crate's own file (CODE_STYLE.md §Lints).
- R037. Finding locations: `<manifest directory>/Cargo.toml` for R031–R034 and R036, the
  crate root file for R035, R039 as above. Messages name the crate and, for R032–R034 and R036, the
  dependency and its kind. Identical findings (a dependency declared under two `cfg` targets)
  are reported once.

### `trace` and `hygiene` (R040–R059)

| ID | Level | Requirement |
|---|---|---|
| XTASK-R040 | MUST | Requirements are extracted from tables in section 6 of each spec: a row whose first cell matches `^[A-Z]+-R\d{3}$`, whose second cell is MUST, SHOULD or MAY, and whose third cell is the text. A requirement is withdrawn if its text contains `(withdrawn` outside inline code spans (`spec_version` 10). The prefix MUST equal the spec's `req_prefix`. |
| XTASK-R041 | MUST | Test annotations are lines in `.rs` files of workspace crates matching `^\s*///\s*Verifies:\s*ID(\s*,\s*ID)*\s*$`. Each annotation is attached to the next `fn` item, and the function name is recorded. |
| XTASK-R042 | MUST | `trace` reports, per requirement, the verifying tests (`path:line fn_name`), and per crate the count of MUST/SHOULD/MAY requirements with and without tests. |
| XTASK-R043 | MUST | An annotation naming an unknown or withdrawn requirement ID is an error. |
| XTASK-R044 | MUST | For each spec whose `maturity` is `usable`, `stable` or `optimized`, every non-withdrawn MUST requirement without a verifying test is an error. For lower maturities it is reported as information only. |
| XTASK-R045 | MUST | Duplicate requirement IDs (within or across specs) are an error. |
| XTASK-R050 | MUST | `hygiene` reports, in `.rs` files of the workspace: `todo!(`, `unimplemented!(`, `dbg!(`; comments containing `TODO`, `FIXME`, `XXX` or `HACK` not immediately followed by `(M<n>-T<nnn>)`; and `#[ignore]` without a reason string containing a task ID. |
| XTASK-R051 | MUST | Task IDs referenced by TODO comments exist in `docs/tasks/` and are not `done` (a TODO pointing at a finished task is stale). Task IDs in `#[ignore]` reasons only need to exist: long-lived ignores (e.g. "needs network", "needs GPU") legitimately outlive the task that added them. |
| XTASK-R052 | MUST | Paths excluded from `hygiene` are listed in `ci/hygiene.toml`, each with a `reason` field. No other exclusion mechanism exists. |

`trace` and `hygiene` details (added in `spec_version` 10):

- Scope. Both read the `.rs` files below the manifest directory of each workspace member
  (`[workspace].members`, as for R026), minus the exclusions in `ci/hygiene.toml`:
  `[[hygiene.exclude]]` for `hygiene`, `[[trace.exclude]]` for `trace`. Each entry has exactly the
  fields `path` (repository-relative file or directory; a directory covers every file below it) and
  `reason`. A missing or unknown field, or a wrong TOML type, is LIM-XTASK-0004. `hygiene` reports
  as R052, for entries of both sections: a reason that is empty or whitespace only, a path that is
  absolute, contains `\`, `.` or `..` components or empty components, and a path that does not
  exist.
- Lexing. Files are split into tokens by a small Rust lexer (identifiers, punctuation, string
  literals of every form, other literals, line and nested block comments). It is not a parser.
  Comments, string contents and code are told apart exactly, so patterns inside string literals
  are never findings, and `trace` and `hygiene` need no exclusion for xtask's own source.
  A numeric literal (`spec_version` 22) is its digits, letters and `_`; a `+` or `-` directly
  after a decimal exponent `e`/`E` (before any other letter) and followed by a digit; and one `.`
  in a decimal literal before any letter, unless that `.` is followed by `.`, `_` or a letter, or
  the literal directly follows a `.` that is not part of `..` (a tuple field index). So `1.5e3`,
  `1_000u32`, `0x1F` and `1e-3` are one literal each; `0..n`, `1.max(2)`, `x.0.1` and `0x1E-3`
  split.
- Requirements (R040). Section 6 is the text from the level-2 heading that starts with `6.` or
  `6 ` up to the next level-1 or level-2 heading, outside fenced code. A row whose first cell is a
  requirement ID but whose second cell is not MUST, SHOULD or MAY, or whose third cell is empty,
  is an R040 finding. So is an ID whose prefix differs from the spec's `req_prefix`; that
  requirement still counts. Extraction does not need valid frontmatter: the crate is the spec's
  file name, and `req_prefix` and `maturity` are used only when readable. A malformed row's ID is
  a known ID for R028 and R043, so one mistake gives one finding. R045 reports a repeated ID at
  each later row and keeps the first.
- Annotations (R041). An annotation is a `///` line comment (not `////`) that is the first token
  on its line and whose text after `///` and whitespace starts with `Verifies:`. If the rest does
  not match `\s*ID(\s*,\s*ID)*\s*`, the line is an R041 finding (malformed). It is attached to the
  item that follows, after comments and outer attributes (`#[…]`, any number of lines): that item
  must start with qualifiers (`pub`, `pub(…)`, `const`, `async`, `unsafe`, `safe`, `extern` with an
  optional ABI string, `default`) and `fn <name>`; a string literal anywhere but directly after
  `extern` (comments between are allowed) ends the header (`spec_version` 22), otherwise the annotation is an R041 finding (not
  attached). This replaces the M0-T005 note's "within 10 lines" heuristic: the token rule is exact.
  A test is reported at the line of its function name. A function that names an ID twice is listed
  once.
- Matrix and gate (R042, R044). `trace` lists, per spec (sorted by path), each requirement with its
  tests (`path:line fn_name`, sorted by path and line), `untested`, or `withdrawn`, then the counts
  of tested requirements per level. Withdrawn requirements are not counted and never gated. For a
  spec below `usable` (or with unreadable `maturity`), an untested MUST is shown in the matrix and
  the counts only; it is not a finding. `--crate <name>` limits the matrix and the findings to the
  spec `docs/specs/<name>.md` (findings about a spec row belong to its spec's crate, findings about
  an annotation to the crate of its file); a name without a spec, a missing name or a repeated
  `--crate` is a usage error. `trace --json` prints `{ "version": 1, "ok", "findings",
  "requirements": [ { "id", "level", "crate", "spec": "path:line", "withdrawn", "tests": [
  { "location", "function" } ] } ], "crates": [ { "crate", "spec", "maturity", "gated", "levels":
  { "MUST" | "SHOULD" | "MAY": { "tested", "untested" } } } ] }`. The `ci` step reports the findings
  only.
- Forbidden macros (R050): an identifier `todo`, `unimplemented` or `dbg` in code, followed by `!`
  and `(`, `[` or `{` (whitespace and comments between are allowed, as in Rust).
- Task markers (R050, R051): in comments only (line, block and doc comments). A marker is `TODO`,
  `FIXME`, `XXX` or `HACK`, case-sensitive, as a whole word (the characters around it are not
  ASCII letters, digits or `_`); the plural with `s` is a marker too. It is tracked only if the
  singular marker is immediately followed by `(M<n>-T<nnn>)`. A task ID is `M\d+-T\d{3}` not
  inside a longer word or number. A task exists if a task file name starts with its ID; it is
  `done` if that file's frontmatter says `status: done`. A task file with unreadable frontmatter
  exists and is not `done`.
- Ignore attributes (R050, R051): `ignore` as a meta item inside an outer or inner attribute
  (directly after `[` or `,`, e.g. `#[ignore]`, `#[cfg_attr(miri, ignore)]`). `(` was removed in
  `spec_version` 22: no test-ignore attribute puts `ignore` directly after `(` (the first argument
  of `cfg_attr` is its predicate), so it matched only non-test attributes such as `#[foo(ignore)]`. Without
  `= "reason"`, or with a reason that contains no task ID, it is R050. Every task ID in the reason
  must exist (R051); its status does not matter.
- Locations: `path:line`. For R040, R044 and R045 the spec row; for R041 and R043 the annotation
  line; for R050 and R051 the line of the macro name, the marker, or `ignore`; for R052 the
  entry's `[[…]]` line in `ci/hygiene.toml`.

### `coverage` (R060–R069)

| ID | Level | Requirement |
|---|---|---|
| XTASK-R060 | MUST | `coverage` runs `cargo llvm-cov --workspace --all-features --locked --json --summary-only` and computes line coverage per workspace crate. |
| XTASK-R061 | MUST | Each crate's coverage is compared with its floor in `ci/coverage.toml`. Coverage below `floor − 0.5` percentage points is an error naming the crate, the floor and the actual value. |
| XTASK-R062 | MUST | A workspace crate without an entry in `ci/coverage.toml` is an error, with help text showing the line to add. |
| XTASK-R063 | MUST | `--update-floors` sets each floor to `max(current floor, measured value rounded down to 0.1)` and never lowers a floor. |
| XTASK-R064 | MUST | If `cargo-llvm-cov` is not installed, the step is `skipped` with an install hint (R006). |

`coverage` details (added in `spec_version` 11):

- Measuring (R060). `--locked` (added in `spec_version` 11) keeps `Cargo.lock` unchanged, as in
  every other cargo step (R010). The tool is installed if `cargo llvm-cov --version` succeeds;
  otherwise, or if `cargo` cannot be started, the result is `skipped` (R064). Doc tests are not
  measured. A crate's coverage is the sum of `summary.lines.covered` over the sum of
  `summary.lines.count` of the export's files below its manifest directory; a file below nested
  members belongs to the innermost one. Files outside the repository root or outside every
  member are ignored. `\` in paths reads as `/`. If the export lists files but none is inside the
  root, that is LIM-XTASK-0002, so a differently spelled root never makes every crate look
  unmeasured. A crate with no measured lines is shown as such and keeps its floor on update.
  Its coverage is unknown, not 100 %: if its floor is above 0.0 it is an R061 error (changed in
  `spec_version` 19; before, it always passed), whose help names the likely causes (member path,
  source files) and says a crate with no code to measure gets floor 0.0 with a journal entry.
  With floor 0.0 it passes. If `cargo llvm-cov` fails (for example a test fails), that is
  LIM-XTASK-0006.
- `ci/coverage.toml`. Exactly one top-level table, `[floors]`, mapping crate name to a TOML
  integer or float from 0 to 100 with at most one decimal. Anything else is LIM-XTASK-0004.
- Arithmetic. Floors are tenths of a point and measurements are line counts; comparisons are
  exact integer arithmetic. R061: a crate fails if `covered × 1000 < (floor_tenths − 5) × count`
  (a floor below 0.5 never fails). A drop of exactly 0.5 points passes. Messages show the
  measured value with two decimals, rounded down.
- R062 also reports an entry for a crate that is not a workspace crate (located at its line).
  The help line of a missing entry is `<crate> = <measured value rounded down to 0.1>`.
- R063. `--update-floors` replaces only the number of each raised entry, so comments and layout
  are kept. It never adds a missing entry (that stays an R062 finding), and it writes the file
  only if a floor rises. It then checks against the updated floors.
- Output. Human mode: one line per crate (`<crate>  <percent>% (<covered>/<count> lines),
  floor <floor>%`, or `no lines measured` / `no floor`), a `raised floor of `<crate>`: <old> ->
  <new>` line per update, then the findings and result line of every check command. `--json`
  prints `{ "version": 1, "ok", "findings", "crates": [ { "crate", "covered", "count",
  "percent", "floor" } ] }` (`percent` and `floor` are `null` when absent). If the tool is
  missing, the command prints `coverage: skipped: <reason>` and `help:` (in JSON a `skipped`
  object with `reason` and `help`, and `ok: false`) and exits 1. The `ci` step reports the
  findings only and never updates floors.

### `status` (R070–R079)

| ID | Level | Requirement |
|---|---|---|
| XTASK-R070 | MUST | `status` generates `docs/STATUS.md` with: the current milestone (lowest milestone with any task not `done`); a table of that milestone's tasks (id, title, status, depends_on); a table of every crate in `architecture.toml` with layer and maturity (`planned` if no spec); a list of `blocked` tasks; and a link to the hand-written `docs/STATUS_NOTES.md`. |
| XTASK-R071 | MUST | Output is deterministic for a given repository state: sorted tables, no timestamps. The file starts with a notice that it is generated and must not be edited by hand. |
| XTASK-R072 | MUST | `status --check` (the `status-check` step in `ci`) fails if `docs/STATUS.md` differs from the generated content. |

`status` details (added in `spec_version` 12):

- Sources: task files (R023 set) for `id`, `title`, `milestone`, `status`, `depends_on`; spec
  files (R022 set) for `crate` and `maturity`; `architecture.toml` for crate names and layers;
  milestone names from `docs/ROADMAP.md` headings of the exact form `## M<n> — <name>` (U+2014,
  spaces around it). A milestone without such a heading is shown by its ID alone.
- Unreadable input: a task file whose frontmatter does not parse (R020), or whose `id`, `title`,
  `milestone`, `status` or `depends_on` is missing or not of the R023 form, a task ID used twice,
  and a spec file whose frontmatter does not parse or whose `crate` or `maturity` is missing or
  not of the R022 form, and a spec file whose `crate` an earlier spec file (by path) already
  names (at the `crate` line; `spec_version` 20), each give one XTASK-R070 error finding at the
  file (with the line where known). Then `status` writes nothing and exits 1, and `status --check` fails with those
  findings. `check-docs` reports the same files in detail. A spec whose `crate` is not in
  `architecture.toml` is ignored here (R022 reports it).
- Order: milestones by number (`M2` before `M10`); tasks by milestone number, then task number;
  crates by layer, then name (byte order). Strings are written as read; `|` in a title is
  written `\|` so the table stays intact.
- Layout, in this order (line endings `\n`, one blank line between blocks, file ends with one
  `\n`):
  1. Line 1: ``<!-- Generated by `cargo xtask status` (XTASK-R071). Do not edit by hand. -->``
  2. `# Status`, then a block quote naming the sources, saying the file is regenerated with
     `cargo xtask status`, and linking `[STATUS_NOTES.md](STATUS_NOTES.md)` for hand-written
     notes (prerequisites, owner decisions, context for blockers).
  3. `## Current milestone`: `**<id>** (<name>): <d> of <n> tasks done.` (`**<id>**:` without
     a name) and a table
     `| Task | Title | Status | Depends on |`, the task ID linked to its file, `depends_on`
     joined with `, `, `—` if empty. If no task is open: `No milestone has open tasks.` and no
     table.
  4. `## Blocked tasks`: one line per `blocked` task in any milestone,
     `- [<id>](tasks/<file>): <title>`, or `None.`
  5. `## Milestones`: `| Milestone | Tasks | todo | in-progress | blocked | review | done |`, one
     row per milestone that has tasks, labelled `<id> (<name>)` or `<id>`.
  6. `## Crates`: `| Crate | Layer | Maturity | Spec |`, one row per `[[crate]]`; maturity from
     the spec whose `crate` is that name, else `planned`; spec cell `[<file>](specs/<file>)` if
     a spec exists, `<file>` being that spec's file name, else `—`. R022 makes `<file>` equal
     `<crate>.md`; for a spec that breaks that rule the link still names an existing file
     (`spec_version` 20).
- `status` writes `docs/STATUS.md` only if its content differs (an up-to-date file is not
  opened for writing, so its modification time stays; `spec_version` 20), then prints
  `status: wrote docs/STATUS.md` or `status: docs/STATUS.md is up to date` and exits 0.
- `status --check` never writes (R010). If the file is missing, or differs, it reports one
  XTASK-R072 error at `docs/STATUS.md:<first differing line>` (`docs/STATUS.md` if missing),
  help `Run cargo xtask status and commit docs/STATUS.md`. Output follows "Check command
  output" with the name `status`. `status` has no `--json` (it is not a check command of R009);
  in `ci`, `status-check` is a native check step.

### `doctor` and `deny` (R080–R089)

| ID | Level | Requirement |
|---|---|---|
| XTASK-R080 | MUST | `doctor` reports, for each tool (rustc, cargo, rustfmt, clippy, cargo-llvm-cov, cargo-deny, nightly toolchain with Miri), whether it is present, its version, and an install hint if missing. |
| XTASK-R081 | MUST | The `deny` step runs `cargo deny check` with the repository `deny.toml`. If `cargo-deny` is missing, the step is skipped (R006). |
| XTASK-R082 | MUST | `deny.toml` rejects every dependency whose licence is not in the allow list of `docs/process/DEPENDENCIES.md` ("Licence policy"), in particular every GPL, LGPL and AGPL licence. MPL-2.0 is accepted only through a per-crate `exceptions` entry. |
| XTASK-R083 | MUST | `deny.toml` rejects every dependency whose source is not crates.io: any git dependency (`allow-git` is empty) and any other registry. |
| XTASK-R084 | MUST | `deny.toml` rejects every dependency that has a RustSec vulnerability advisory or an `unmaintained` advisory, unless the advisory is in `[advisories] ignore`. |
| XTASK-R085 | MUST | `hygiene` reports every `[advisories] ignore` entry of `deny.toml` that has no `reason` ending with `expires YYYY-MM-DD (<task ID>)`, whose date is not a valid date or is today or earlier (UTC), or whose task does not exist or is `done`. |

`doctor` and `deny` details (added in `spec_version` 13):

- Tools, in output order. Each is probed by running a command in the repository root with stdin
  closed and `RUSTUP_AUTO_INSTALL=0`, so a probe never installs a toolchain (ADR-0022). `cargo`
  is `$CARGO`, else `cargo` from `PATH`, as in `ci`. A tool is present if its probe starts and
  exits with status 0. Its version is the first non-empty line of the probe's stdout, trimmed
  (`null` if there is none).
- Probe stdin and stderr (`spec_version` 25). "stdin closed" is tested: a probe of a program
  that reads stdin to its end returns at once, having read nothing, and its stderr is not in the
  probe's stdout (`a_probe_that_reads_stdin_sees_it_closed` in `xtask/src/doctor/tests.rs`,
  whose probe program is the test binary itself). The test fails (timeout) if stdin is inherited
  and the parent's stdin stays open. Removing `.stdin(Stdio::null())` from `RealProber::command`
  passes it: `Command::output` already gives the child a closed stdin, so that call states the
  rule rather than changes it. `.stderr(Stdio::null())` discards stderr instead of capturing it;
  nothing reads it, so removing it is not observable either.

  | Tool | Probe | Needed by `ci` steps |
  |---|---|---|
  | `rustc` | `rustc --version` | fmt, clippy, test, test-features-off, doc, coverage, deny |
  | `cargo` | `cargo --version` | fmt, clippy, test, test-features-off, doc, coverage, deny |
  | `rustfmt` | `cargo fmt --version` | fmt |
  | `clippy` | `cargo clippy --version` | clippy |
  | `cargo-llvm-cov` | `cargo llvm-cov --version` | coverage |
  | `cargo-deny` | `cargo deny --version` | deny |
  | `miri` | `rustup run nightly cargo miri --version` | none (Miri tests of `unsafe` crates from M1, ADR-0018) |

- Findings. Each missing tool gives one XTASK-R080 finding located at the tool name (no file is
  involved). It is an `error` if a `ci` step needs the tool (that step would be skipped) and a
  `warning` otherwise. Its message names the tool and the steps that need it; its help is the
  install hint, followed by `; ask the project owner before installing (ADR-0022)`.
- Output follows "Check command output" with the name `doctor`; human mode first prints one line
  per tool, `  <tool>  <version>` or `  <tool>  missing`, names padded to one width. `--json`
  prints `{ "version": 1, "ok", "findings", "tools": [ { "tool", "present", "version",
  "needed_by", "help" } ] }`: `version` is `null` when missing, `needed_by` is the list of step
  names (empty for `miri`), `help` is the install hint when missing and `null` otherwise. Exit 0
  without error findings (warnings do not fail), 1 otherwise. `doctor` writes nothing (R010).
- `deny` step (R081). Runs `cargo deny --locked --config deny.toml check` in the repository root
  (`--locked` keeps `Cargo.lock` unchanged, R010). If `cargo deny --version` fails, the step is
  `skipped` with reason `cargo-deny is not installed` and the `doctor` install hint as help. The
  advisory check fetches the RustSec database into `$CARGO_HOME/advisory-dbs` (outside the
  repository); without network access the step fails, it is not skipped.
- R082 is verified by running `cargo deny check licenses` with the repository `deny.toml` against
  the fixture `xtask/tests/fixtures/deny/`, whose root crate depends on a GPL-3.0-only path
  crate (rejected) and, after the fixture licence is changed to MIT, passes. The test needs
  `cargo-deny`, so it is ignored in the `test` step and run by the `deny` job of the CI
  workflow (`cargo test --test deny_policy -- --ignored`, `xtask/tests/deny_policy.rs`).
- R083 is verified by running `cargo deny check sources` with the repository `deny.toml` against
  a fixture crate whose only dependency is a git dependency on a local repository
  (`file://<temp>/dep`, created by the test with `git`), which is rejected; and against a
  fixture crate whose only dependency comes from a second registry, which is rejected with
  `error[source-not-allowed]` (added in `spec_version` 26). That registry is a git index the
  test creates at a `file://` URL (named through `CARGO_REGISTRIES_FIXTURE_INDEX`), whose `dl`
  is a `file://` directory holding the `.crate` file of a crate of the repository `Cargo.lock`,
  copied from the local crates.io cache, with the checksum from `Cargo.lock`; cargo runs with a
  temporary `CARGO_HOME`, so no network is needed and the user's cache is not changed.
- R084 is verified by running `cargo deny --offline check advisories` with a copy of the
  repository `deny.toml` that adds only `db-path` and `db-urls` to `[advisories]`, pointing at a
  fixture advisory database (a git repository the test creates) that has one advisory for a
  crates.io crate of the repository `Cargo.lock`. An `unmaintained` advisory and a vulnerability
  advisory are each rejected; the same database without either passes. The `unmaintained` case
  runs twice: once on a crate the fixture app reaches only through a path crate, once on a
  direct dependency of the app (added in `spec_version` 26), so `unmaintained = "all"` is
  distinguished from `"workspace"` and `"transitive"`. The crates come from the
  local cargo registry cache, so no network is needed once the repository has been built.
  cargo-deny reads the database from `<db-path>/advisory-db-<hash of the URL>`; the test takes
  that directory name from cargo-deny's own error output on an empty `db-path`, so it does not
  depend on the hash function. Both tests need `cargo-deny` (and R083 needs `git`), so they are
  ignored in the `test` step and run by the `deny` CI job like R082.
- `confidence-threshold` (R082, `spec_version` 25). It applies only to a crate that declares its
  licence through a `license-file`, not to an SPDX `license` expression. The policy value 0.93 is
  verified by `a_licence_file_below_the_confidence_threshold_is_rejected`: the fixture's
  dependency gets a `license-file` with the exact MIT text (accepted), then with the MIT text
  whose last two lines are replaced (cargo-deny 0.20.2 scores it 0.88: rejected). The test
  asserts the reported score lies strictly between cargo-deny's default threshold 0.8 and the
  policy's, so a threshold of 0.88 or less, or the key removed, fails it. A threshold between
  0.88 and 0.93 is not distinguished.
- Ignored advisories (R085, `spec_version` 25). `hygiene` reads `[advisories] ignore` of
  `deny.toml` (no file: nothing to check; invalid TOML or `ignore` not an array:
  LIM-XTASK-0004; other keys are not read). Each entry must be a table with a string `reason`
  (a bare advisory ID string has none). The reason, with trailing whitespace removed, must end
  with `expires <date> (<task ID>)`, with `expires` at the start or after whitespace and the
  task ID as in R050. `<date>` must be `YYYY-MM-DD` naming an existing day; the entry is expired
  on that date and after it (today is the UTC date of the system clock). The task must exist and
  must not be `done`, as for R051. Each violation is an `error` at `deny.toml:<line of the
  entry>`, named by its `id` (else `crate`). R085 depends on the date, so the same repository
  can pass today and fail on a later day; that is its purpose (INV-1 holds for one day).
- `doctor` in CI (`spec_version` 25). Each `checks` job runs `cargo xtask doctor --json` after
  `ci`, prints the report and uploads it with the CI summary. Exit 1 is accepted there (the job
  installs neither cargo-llvm-cov nor cargo-deny); exit 2 fails the job.

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
| LIM-XTASK-0003 | `architecture.toml` is not valid TOML, or a field is missing, unknown or of the wrong TOML type (R030) | Fix the syntax at the reported line |
| LIM-XTASK-0004 | Another TOML file (`Cargo.toml`, `ci/hygiene.toml`, `ci/coverage.toml`, the clippy configuration of a deterministic crate (R039), `deny.toml` for R085) is invalid, misses a field xtask needs, has an unknown field, or uses an unsupported workspace member pattern | Fix the file at the reported location |
| LIM-XTASK-0005 | `cargo metadata` could not be started or failed (its stderr is in the message) | Run `cargo metadata --no-deps` in the repository root to see the error, and fix the `Cargo.toml` it names |
| LIM-XTASK-0006 | `cargo llvm-cov` failed, for example because a test failed (the last 50 lines of its stderr are in the message when captured) | Run `cargo llvm-cov --workspace` in the repository root to see the error |
| LIM-XTASK-0007 | The repository root compiled into xtask does not exist or has no `architecture.toml` (R015) | The checkout was moved or renamed after xtask was built: run `cargo clean -p xtask`, then the command again |

An internal error names the file concerned relative to the repository root; when that is the root itself, it names the root's absolute path (never an empty path). A child process (`cargo`, a probe) that cannot be spawned because its working directory does not exist is reported as `working directory <path> does not exist`, not as `could not start <program>` (`spec_version` 27).

## 9. Performance budgets

None yet beyond XTASK-R013. Set at `stable` from measurements.

## 10. Test plan

| Area | Test kinds | Oracle |
|---|---|---|
| Frontmatter parser | unit, property (round-trip of generated valid frontmatter; random invalid lines are rejected, never accepted) | Subset definition in DOCUMENTATION.md |
| Each check rule | integration tests on fixture repositories in `xtask/tests/fixtures/<case>/` | One passing fixture plus one fixture per rule that must fail with exactly that rule ID |
| `check-arch` | unit tests on a hand-built repo model; integration on fixture `cargo metadata` JSON saved in fixtures | Rules R030–R037 and R039 (R038 from M2) |
| `ci` orchestration | unit tests with a fake step runner (steps are a trait; real runner spawns processes) | Order, continue-on-failure, skip semantics, exit codes |
| `test-features-off` (R014) | unit tests with a fake command function; the real step on a workspace written to a temporary directory | Expected command lines; tests that exist only with a feature on or off |
| `status` | snapshot test of generated STATUS.md for a fixture | Committed snapshot |
| `doctor` | unit tests with a fake probe runner; the real probes run on the developer machine | Tool table above |
| `deny.toml` (R082) | `cargo deny check licenses` on a GPL fixture, with a passing control | cargo-deny's licence check |
| `deny.toml` (R083, R084) | `cargo deny check sources` on a local git dependency and on a dependency from a local second registry; `check advisories` on a fixture advisory database with transitive and direct advised crates, with passing controls | cargo-deny's sources and advisories checks |
| `deny.toml` ignores (R085) | unit tests of the entry rules with a fixed date; `hygiene` fixture `r085_advisory_ignore` | Rules above |
| Every command (real binary) | integration tests running `CARGO_BIN_EXE_xtask` on this repository (`xtask/tests/cli.rs`, `ci.rs`, `commands.rs`): an `ok` run, and a usage error or a failing run with `$CARGO` naming a missing program; `ci` only with steps that do not run the test suite; a real `coverage` run is ignored in the `test` step and run by the `coverage` CI job | Exit codes (R007), output formats of §5 and "Check command output" |
| Moved checkout (R015) | unit tests of the command dispatcher with a root that does not exist, and one without `architecture.toml` (the binary's compiled-in root cannot be changed without rebuilding it, so no real-binary test) | R015 |
| Real repository | `cargo xtask ci` on the actual repository in CI | Must pass |

The fake step runner is a test double for process spawning only, and is used solely to test
orchestration logic. Every real step is also exercised in CI against the real repository.

## 11. Determinism

Not a deterministic crate in the ADR-0009 sense, but all output MUST be deterministic for a given
repository state (R071, INV-1).

## 12. Open questions

- None blocking. OQ-004 is resolved by ADR-0023; M1-T017 adds and specifies `cargo xtask bench`
  and the perf-baseline files.

## 13. Maturity

Current: `usable` (promoted from `planned` in one step, M0-T010, 2026-11-09). Next target:
`stable` (gates in MATURITY.md; not scheduled). Gate evidence from the M0 exit review (M0-T010,
2026-11-09; details in that task's "Review"):

- [x] Spec follows `docs/specs/_TEMPLATE.md` and passed a spec review (REVIEW.md §2), M0-T010.
- [x] Compiles, all lints pass, registered in `ci/coverage.toml` (`cargo xtask ci`).
- [x] All MUST requirements traced: `cargo xtask trace --crate xtask` with `maturity: usable`
      gives MUST 49/49, `trace: ok`.
- [x] Coverage ≥ 75 % (tool crate): 97.39 % (M0-T009), floor 97.6.
- [x] Every public item documented (`missing_docs = "deny"`, `doc` step with `-D warnings`).
- [x] Each check shown to catch a seeded violation (M0-T010 journal, 14 seeds).
- [x] Used in CI: `cargo xtask ci` runs in every job of `.github/workflows/ci.yml`.
- [x] Every command covered by a real-binary integration test checking exit code and output
      (tool-crate gate), M0-T011: `help` in `xtask/tests/cli.rs`; `ci` in `xtask/tests/ci.rs`;
      `check-arch`, `check-docs`, `trace`, `hygiene`, `status`, `doctor`, `coverage` in
      `xtask/tests/commands.rs` (the real `coverage` measurement runs in the `coverage` CI job).
- [x] Runnable doc examples for the main types: `run`, `COMMANDS`, `Command`, `Exit`
      (`xtask/src/cli.rs`), M0-T011.
- [x] Promotion review by a session that implemented neither the gates' evidence nor M0-T011
      (M0-T010, continued, 2026-11-09): trace MUST 49/49 at `usable`; coverage 98.05 %
      (5050/5150), floor 97.6; `cargo xtask ci` passes; CI run 42674386561 attempt 2: 5 jobs pass.

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
| 8 | 2026-11-08 | `check-arch` details (model from `cargo metadata --no-deps`, R030 values and locations, R031 kind rule, R032/R036 overlap, R033/R034 scope table, R035 attribute rule, locations); R030: shape errors are LIM-XTASK-0003; LIM-XTASK-0005; `cargo_metadata` approved but unused | M0-T004 |
| 9 | 2026-11-08 | `check-arch` details: R032–R036 skip a crate with an unregistered or invalid-layer entry entirely, and dependencies on it (clarified wording; review 1 of M0-T004) | M0-T004 |
| 10 | 2026-11-08 | `trace` and `hygiene` details (scope, `ci/hygiene.toml` format, lexing, R040 malformed rows, R041 attachment, matrix and `--json`, R050/R051 pattern rules, locations); R040: `(withdrawn` counts only outside inline code spans (R040's own text names the marker); LIM-XTASK-0004 covers `ci/hygiene.toml` | M0-T005 |
| 11 | 2026-11-08 | `coverage` details (measurement, file → crate mapping, `ci/coverage.toml` format, integer arithmetic, stale entries are R062, `--update-floors` keeps comments, output); R060: `--locked`; LIM-XTASK-0006 | M0-T006 |
| 12 | 2026-11-09 | `status` details (sources, unreadable input is R070, order, layout, write-if-changed, `--check` finding location) | M0-T008 |
| 13 | 2026-11-09 | `doctor` and `deny` details (tool table and probes, `RUSTUP_AUTO_INSTALL=0`, error vs. warning findings, output, `deny` command line); new R082 (licence policy in `deny.toml`) | M0-T009 |
| 14 | 2026-11-09 | New R083 (sources policy) and R084 (advisory policy) of `deny.toml`, with their test approach (review 1 of M0-T009) | M0-T009 |
| 15 | 2026-11-09 | §13 Maturity: `usable` gate evidence from the M0 exit review; two tool-crate gates open (M0-T011) | M0-T010 |
| 16 | 2026-11-09 | §3 and §10: real-binary integration tests per command; §13: the two tool-crate gates' evidence | M0-T011 |
| 17 | 2026-11-09 | Maturity `planned` → `usable` (promotion review, §13) | M0-T010 |
| 18 | 2026-11-09 | §6 `check-docs` details: table lines and cells, and closing fences, trim only space and tab (GFM); the line-1 error names a leading U+FEFF | M1-T002 |
| 19 | 2026-11-09 | `coverage`: a crate with no measured lines and a floor above 0.0 is an R061 error; workspace member entries are normalized (`./name`, `a/./b`) | M1-T005 |
| 20 | 2026-11-09 | §6 `check-docs` details: the file-name rule makes a spec's `crate` unique. `status` details: a second spec for one crate is R070 unreadable input; the spec cell links the spec's file name; an up-to-date file keeps its modification time | M1-T007 |
| 21 | 2026-11-10 | §2 and §12: OQ-004 is resolved by ADR-0023; benchmark commands come with M1-T017 (text only, no requirement change) | M1-T054 |
| 22 | 2026-11-09 | Lexing: numeric literals end at `..`, `.` + identifier and after a tuple-field `.`; R041: a string literal counts only directly after `extern`; R050: `ignore` counts only directly after `[` or `,` (not `(`) | M1-T004 |
| 23 | 2026-11-09 | New R039 (deterministic crates' clippy `disallowed-methods` configuration) with its details; LIM-XTASK-0004 covers that configuration; ADR-0009 in `adrs` | M1-T011 |
| 24 | 2026-11-10 | New R014: `ci` step `test-features-off` (each workspace package with Cargo features is tested with `--no-default-features`), its details and test approach; `doctor` lists the step for `rustc` and `cargo` | M1-T056 |
| 25 | 2026-11-10 | `doctor` and `deny` details: probe stdin/stderr test and why the `Stdio::null()` calls are not observable; `confidence-threshold` test; new R085 (expiry of ignored advisories, checked by `hygiene`); `doctor` in the `checks` CI jobs; LIM-XTASK-0004 covers `deny.toml` | M1-T008 |
| 26 | 2026-11-10 | R083 test approach: a second (local git index) registry is now tested; R084: the `unmaintained` case also covers a direct dependency (review 2 of M0-T009) | M1-T009 |
| 27 | 2026-11-10 | New R015 (moved checkout: the compiled-in root is checked first, one LIM-XTASK-0007 error); §8: LIM-XTASK-0007, the root is named by its absolute path, a missing working directory is not reported as "could not start" (written as R014 in branch version 20; renumbered to R015 and version 27 on merge because main's version 24 took R014 and M1-T009 took 26) | M1-T010 |
