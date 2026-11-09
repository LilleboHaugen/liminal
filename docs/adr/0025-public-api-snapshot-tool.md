---
id: ADR-0025
title: Public-API snapshots with cargo-public-api, from the first stable promotion
status: proposed
date: 2026-11-09
supersedes: []
superseded_by: []
---

# ADR-0025: Public-API snapshots with cargo-public-api, from the first stable promotion

## Context

`docs/process/MATURITY.md` (`stable` gate): "Public API snapshot committed (tool chosen at M1
kickoff). From now on, API changes are deliberate, visible in the snapshot diff, and noted in the
journal." M1-T001 chooses the tool. No crate reaches `stable` in M1 (the M1 exit criteria ask for
`usable`), so nothing is installed or run in M1.

Facts:
- The complete public surface of a crate, including re-exports, `impl` blocks, blanket and auto
  trait impls, is only known after name resolution. Rustdoc's JSON output has it. Rustdoc JSON
  needs a nightly toolchain, and its format version changes between nightlies.
- `cargo-public-api` (an external `cargo` subcommand, licence MIT, assumption to be confirmed by the
  task that installs it) reads rustdoc JSON and prints one line per public item, sorted, stable
  enough to commit and diff. It tracks the rustdoc JSON format and documents which nightly it
  needs.

## Decision

1. The public-API snapshot for the `stable` gate is the output of `cargo public-api` for the crate,
   committed at `ci/api/<crate>.txt`. The options used (for example whether blanket and auto-trait
   impls are listed) are fixed by the task that adds the step and recorded in the xtask spec.
2. A new `xtask` step (specified in `docs/specs/xtask.md` by the task that adds it) regenerates the
   snapshot for every crate at maturity `stable` or above and fails when it differs from the
   committed file. Updating a snapshot is an intended API change and needs a journal entry.
3. The nightly toolchain is pinned by date in one place (chosen by that task) and used only for
   this step, Miri and fuzzing (CODE_STYLE.md).
4. The tool and the nightly toolchain are installed on the owner's machine only after the owner
   approves (ADR-0022). The task is created at the kickoff of the first milestone in which a crate
   targets `stable`.

## Alternatives considered

- **Own snapshot generator in `xtask` from rustdoc JSON.** Possible with `serde_json`, which `xtask`
  already uses. Rejected for now: the rustdoc JSON format changes several times a year and the
  parser would need to follow it, which is maintenance with no engine benefit. Reconsider if
  `cargo-public-api` stops being maintained.
- **Own generator from source with a Rust parser.** Rejected: it misses re-exports, glob imports
  and macro-generated items, which are where accidental API changes hide.
- **`cargo-semver-checks`.** Answers "is this change semver-compatible", not "what is the API". It
  is a complement, not a snapshot. Not adopted now.
- **Do nothing.** Not possible: the `stable` gate requires a snapshot.

## Consequences

- One more external tool and a nightly toolchain on agent machines, from the first `stable`
  promotion on. `cargo xtask doctor` lists both once the step exists.
- Snapshot diffs make API changes visible in review without reading every file.
- Revisit if the tool falls behind the nightly that Miri needs, or stops being maintained.

## Verification

The `xtask` step fails on a seeded API change in a fixture crate (test in the task that adds it).
Review checks that snapshot changes have a journal entry.
