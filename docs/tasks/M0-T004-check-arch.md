---
id: M0-T004
title: check-arch
milestone: M0
status: done
crates: [xtask]
depends_on: [M0-T002]
requirements: [XTASK-R009, XTASK-R030, XTASK-R031, XTASK-R032, XTASK-R033, XTASK-R034, XTASK-R035, XTASK-R036, XTASK-R037]
---

# M0-T004: `check-arch`

## Goal

The workspace's real dependency graph is checked against `architecture.toml` on every CI run, so
layering, dependency approval and the unsafe policy hold without relying on review.

## Context

- ADR-0003 (dependency policy), ADR-0004 (layers), ADR-0018 (unsafe policy).
- `architecture.toml` (header comment lists the rules). `docs/specs/xtask.md` R030–R037.
- Approved `tool` crates for xtask: `cargo_metadata`, `toml`, `serde`, `serde_json`.

## Scope

**In:** full `architecture.toml` validation (R030); extending the shared repo-model loader (xtask spec
§3; created by M0-T003 if that task landed first, otherwise create it here, and M0-T003 reuses it) with
`cargo metadata` data (dependencies with kinds, manifest paths, targets); rules R031–R037;
`check-arch` command with `--json` (R009) and `ci` step; fixtures.

**Out:** R038 (maturity dependency rule). That starts in M2, and its task is created at M2 kickoff.

## Acceptance criteria

- [x] `cargo xtask check-arch` passes on the real repository (only `xtask` exists, plus the planned
      registry).
- [x] Rule logic runs on a repo model built from a saved `cargo metadata` JSON fixture plus an
      `architecture.toml` fixture, without spawning cargo in tests.
- [x] One failing fixture per rule (R030–R036), each reporting exactly its rule ID with `help` (R037).
- [x] Scope rules tested for every combination in R033/R034: runtime crate using `tool` scope fails;
      proc-macro using `build` passes; normal dependency on a `build`-scope crate from a non-proc-macro
      crate fails; dev-dependency not in `dev_external` fails.
- [x] `check-arch` is a step in `ci`.

## Test requirements

`Verifies:` tests for R030–R037. Fixtures under `xtask/tests/fixtures/check_arch/<case>/`.

## Notes

- Dependency kinds from `cargo metadata`: `normal`, `build`, `dev` (`null` kind means normal).
  Target-specific dependencies (`cfg(...)`) count the same as unconditional ones.
- R035: detect `#![forbid(unsafe_code)]` by parsing the crate root file's inner attributes. Ignore
  occurrences in comments and strings (a simple token-level scan is enough; no `syn` in xtask).
- External crate names in `architecture.toml` use the crates.io name (`objc2-metal`, with hyphen),
  which `cargo metadata` also reports. Do not normalize hyphens/underscores.
- `cargo metadata` reports Windows paths with backslashes. Normalize to `/` before comparing with
  `path` (R031).

## Handoff

2026-11-08, implementation session (after review 5 of M0-T003 in the same session). Status →
`review`.

**Done (all acceptance criteria met):**
- `xtask/src/repo/metadata.rs`: `run_cargo` runs `cargo metadata --format-version 1 --no-deps
  --offline` (the only process spawn); `parse` reads the JSON into `Package`/`Target`/`Dependency`
  with paths relative to `workspace_root`, `\` → `/`. Errors: LIM-XTASK-0002 (unexpected JSON),
  new LIM-XTASK-0005 (cargo cannot start or fails; stderr in the message).
- `xtask/src/repo.rs`: `ArchModel` (architecture, packages, crate root sources), `load_arch(root,
  metadata_json)`, `crate_roots`. `repo/architecture.rs`: each `[[crate]]` and `[external.*]`
  entry carries its line (via `toml::Spanned`); the `expect(dead_code)` attributes are gone except
  on `deterministic`, which no rule reads yet (R030 checks only its type).
- `xtask/src/check_arch.rs` (+ `rules.rs`, `attributes.rs`): R030–R036 as pure functions of the
  model; `attributes.rs` is a token-level lexer that finds `#![forbid(unsafe_code)]` among the
  leading inner attributes (comments, strings, raw strings, char literals, shebang, BOM handled).
- `check-arch` command (with `--json`, shared `check.rs` runner) and `ci` step before `check-docs`.
- Fixtures `xtask/tests/fixtures/check_arch/`: `valid/` is a real Cargo workspace;
  `metadata.json` files are real `cargo metadata` output (cargo 1.99.1, path replaced by
  `/fixture`), one per overlay that changes a manifest. README says how to regenerate them.
- Spec `spec_version` 8: "`check-arch` details", R030 wording (shape errors are LIM-XTASK-0003),
  LIM-XTASK-0005, §3 (`cargo_metadata` approved but unused). Root `Cargo.toml`: unused
  `cargo_metadata` workspace dependency removed (it was never in `Cargo.lock`).

**Spec decisions (v8), beyond the literal R030–R037 text:** `[layers]` must have exactly keys
`"0"`–`"5"`; `adr` must be an ADR ID and `reason` non-empty; a layer 0–4 → layer 5 dependency is
R036 only (it always also breaks R032, so without this no R036 fixture could fail with exactly
R036); unregistered crates and invalid layers are skipped by R032–R036 (no cascades); a
`proc-macro` target decides `kind` before a `bin` target; every lib/proc-macro/bin target root is
a crate root for R035 (xtask has two: `lib.rs`, `main.rs`), build scripts and tests are not.

**Tests:** 139 lib tests (was 104). R033/R034: a 47-row table written from the spec text covers
every layer group × crate kind × dependency kind × scope, each row run at layers 0 and 4 (or 5).
R030: one case per value rule, each asserting exactly one finding at its line.

**Verified (macOS 27.2, Apple M5 Pro, Rust 1.99.1):** `cargo xtask ci` passes (fmt, clippy, test,
doc, check-arch, check-docs); `check-arch` takes 11 ms. 29 single-line mutants of the new code:
all killed (3 survived the first run; tests added for each: lower-layer `same_layer_deps`,
proc-macro-before-bin, string escapes). Faults injected into the real repository, each reverted:
`main.rs` without the attribute (R035), xtask `kind = "lib"` (R031), an unapproved dev-dependency
`loom` (R034; `Cargo.lock` unchanged by `--no-deps`), xtask `path = "tools/xtask"` via
`ci --json --only check-arch` (step `fail` with the finding), a broken `xtask/Cargo.toml`
(LIM-XTASK-0005 with cargo's error). Not verified: Windows and Linux (M0-T007). Windows paths are
unit-tested with a `C:\w\…` metadata sample, not on Windows.

**Traps:**
- `xtask/tests/fixtures/check_arch/` holds `Cargo.toml` and `.rs` files. They are not workspace
  members. M0-T005 (`hygiene`, "`.rs` files of the workspace") must not scan fixture `.rs` files
  as engine code, or must exclude them in `ci/hygiene.toml`.
- Changing a fixture `Cargo.toml` without regenerating `metadata.json` makes the fixture test
  pass or fail for the wrong reason (`valid_fixture_metadata_is_what_the_readme_says` catches
  some of it).
- `cargo metadata` lists a package's targets in its own order (`lib`, `test`, `custom-build` for
  `peer`), not the manifest order.

**Next:** review (fresh session). Then the first `todo` in M0 with all dependencies done:
M0-T005 (`trace` and `hygiene`).

**Review 1 changes (2026-11-08, same session that wrote Review 1 below).** Status → `review`.
The next review must be done by a new session: this session wrote both Review 1 and these
changes.
1. B5: `rules::judged` returns a crate's entry only if it is registered and its layer is 0–5.
   `dependencies` (R032–R034, R036), `internal` (dependency target) and `forbid_unsafe` (R035)
   use it, so a crate with an invalid layer gets the R030 finding only. Spec `spec_version` 9
   states it without the ambiguity ("no finding of R032–R036 is reported for such a crate, and
   none for a dependency on it"). Behaviour as specified in v8; only the wording changed.
2. B4: tests `a_crate_with_an_invalid_layer_gets_only_the_r030_finding` (layer-7 crate with a
   `tool`-scope, an unlisted normal and an unlisted dev dependency, dependencies on a layer 0 and
   a layer 5 crate, no attribute; dependencies on it from layer 0 and layer 5) and
   `a_dependency_on_an_unregistered_workspace_crate_is_reported_by_r031_only` (normal, dev and
   build dependencies on `ghost` from layers 0 and 5). Each asserts the exact finding list.
- Verified (macOS 27.2, Rust 1.99.1): `cargo xtask ci` passes; 141 lib tests. Mutants, each
  killed: `judged` without the layer filter; `internal` target via `entry` instead of `judged`;
  the target guard replaced by `panic!`; `dependencies` and `forbid_unsafe` via `entry` instead
  of `judged`. Fault injection (reverted): xtask `layer = 7` in `architecture.toml` gives exactly
  one finding, R030 at line 379. Before the fix it would also have given R033 findings for the
  `tool`-scope `serde`, `serde_json`, `toml`. Not verified: Windows, Linux (M0-T007).

## Review

2026-11-08, new owner-started session (fresh context; did not write M0-T004). Outcome:
**changes required** (first "changes required" outcome of this task). Two blocking findings.

Checked, macOS 27.2, Apple M5 Pro, Rust 1.99.1:
1. `cargo xtask ci`: pass (fmt, clippy, test, doc, check-arch 10 ms, check-docs).
2. Acceptance criteria, each by reading or running: `check-arch --json` on the repository gives
   `ok: true`, no findings; fixture tests build the model from saved `metadata.json`
   (`fixture_tests.rs`, no cargo spawned); seven overlays, each asserting exactly its rule with
   `help`; the four R033/R034 combinations named in the criteria are rows of `SCOPE_TABLE`;
   `check-arch` is a `ci` step before `check-docs`.
3. Code read in full: `check_arch.rs`, `rules.rs`, `attributes.rs`, `repo/metadata.rs`,
   `repo.rs` (`load_arch`, `crate_roots`), the `architecture.rs` diff, `ci.rs`/`cli.rs` diffs.
   `rules.rs` checked against the spec's scope table row by row: agrees.
4. Own probes: a temporary test with a layer-7 crate (finding 1), and two mutants of the
   skip guards in `rules::internal` (finding 2). All reverted.
5. No placeholders, no weakened tests, no oracle changes. No `unwrap`/`expect` in non-test code
   except the existing `expect("invariant: …")` in `repo::root`. No unsafe. Determinism: n/a.
6. Architecture: no new dependency; the unused `cargo_metadata` workspace dependency was removed
   and stays approved in `architecture.toml` (spec §3). Session boundaries: the M0-T004 journal
   shows an owner-started session (after review 5 of M0-T003), no sub-agents.

Required changes:
1. **B5: R033 (and R034, R035) judge a crate whose layer is outside 0–5.** Spec "`check-arch`
   details": "Rules R032–R036 skip … a layer outside 0–5 (R030 reports it)". `rules::dependencies`
   and `rules::forbid_unsafe` check only that the crate is registered. Observed: a crate
   registered with `layer = 7` that lists `tl` (scope `tool`) in `external` and uses it as a
   normal dependency gets the R030 finding and also an R033 finding "`tool` scope is for layer 5
   crates only", a judgment from the invalid layer. The same crate with an unlisted external
   dependency gets R033/R034 findings as well. Fix: skip crates with an invalid layer in
   `dependencies` and `forbid_unsafe`, as the spec says. Or, if only the layer-dependent
   judgments should be skipped, say so in the spec (bump `spec_version`) and skip the scope
   judgment. Either way, add a test.
2. **B4: the "no cascades" skips in `rules::internal` are untested.** The handoff claims
   "unregistered crates and invalid layers are skipped by R032–R036 (no cascades)".
   (a) Mutant `if !valid_layer(from) || !valid_layer(to)` → `if false`: all 139 tests pass, and a
   layer-0 crate depending on a layer-7 crate then gets an extra R032 finding.
   (b) Replacing `return; // R031 reports the unregistered target.` with `panic!()`: all tests
   pass, so no test has a registered crate depending on an unregistered workspace crate. Add
   tests for both, each asserting the exact finding list.

Follow-ups (non-blocking) → [M1-T003](M1-T003-check-arch-follow-ups.md).

### Review 2

2026-11-08, new owner-started session (fresh context; wrote neither M0-T004 nor review 1).
Re-review (ADR-0021): the two previous blocking findings, the diff since review 1 (`05c5196`),
and `cargo xtask ci`. Outcome: **no blocking findings**, status → `done`. No new follow-ups;
the follow-ups of review 1 stay in [M1-T003](M1-T003-check-arch-follow-ups.md).

Checked, macOS 27.2, Apple M5 Pro, Rust 1.99.1:
1. `cargo xtask ci`: pass (fmt, clippy, test, doc, check-arch 10 ms, check-docs); 141 lib tests.
2. Finding 1 (B5) fixed: `rules::judged` (registered and layer 0–5) is used by `dependencies`
   for the dependent crate, by `internal` for the target, and by `forbid_unsafe`. Spec v9
   wording matches: no R032–R036 finding for such a crate or a dependency on it.
3. Finding 2 (B4) fixed: the two new tests assert the exact finding list (`one`: exactly one
   finding, rule, message, location). The layer-0 → layer-7 case of 2(a) and the registered →
   unregistered workspace crate case of 2(b) are both present.
4. Own mutants, each run and reverted, each killed by at least one test: `judged` without the
   layer filter; the `internal` target guard replaced by `panic!()`; `forbid_unsafe`,
   `dependencies` and the `internal` target via `entry` instead of `judged`.
5. Diff since review 1: only `rules.rs`, `tests.rs`, the spec, the task file, `STATUS.md` and a
   journal entry. No weakened or deleted tests, no oracle changes, no new dependency.
6. Session boundaries: review 1 and its changes were made in one owner-started session
   (`2026-11-08-M0-T004-review.md`, `2026-11-08-M0-T004-2.md`); this re-review is a different
   session. No sub-agents.
