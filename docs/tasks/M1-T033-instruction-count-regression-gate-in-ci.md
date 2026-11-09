---
id: M1-T033
title: Instruction-count regression gate in CI
milestone: M1
status: todo
crates: [xtask, liminal_benches]
depends_on: [M1-T017, M1-T022, M1-T006]
requirements: []
---

# M1-T033: Instruction-count regression gate in CI

## Goal

A CI job fails when a single-threaded benchmark case executes more than 1 % more instructions than its baseline.

## Context

- ADR-0023 item 4; PERFORMANCE.md; `.github/workflows/ci.yml`.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Instruction mode of the harness or `cargo xtask bench` (`k`/`2k` difference under `valgrind --tool=cachegrind --cache-sim=no`), the Linux CI job installing Valgrind on the runner, `ci/perf/ci-linux-x86_64/instructions.json` produced by CI and committed.

**Out:** Valgrind on the owner's machine (not supported on macOS arm64; not installed).

## Acceptance criteria

- [ ] XTASK spec requirements for parsing cachegrind output and comparing counts, tested on fixture output files.
- [ ] The job runs on every push; a seeded 2 % increase fails it and an unchanged run passes (scratch branch, journal records both runs).
- [ ] Two runs of the same commit give identical counts for every gated case, or the spread is recorded and the threshold justified in the journal.
- [ ] Multithreaded cases are excluded and listed.
- [ ] `cargo xtask ci` passes.

## Test requirements

New XTASK requirements with fixture tests.

## Notes

_None._

## Handoff

_Empty._

## Review

_Empty._
