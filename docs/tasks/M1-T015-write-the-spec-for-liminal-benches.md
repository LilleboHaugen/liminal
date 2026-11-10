---
id: M1-T015
title: Write the spec for liminal_benches
milestone: M1
status: review
crates: [liminal_benches]
depends_on: [M1-T014]
requirements: []
---

# M1-T015: Write the spec for `liminal_benches`

## Goal

`docs/specs/liminal_benches.md` is a complete, reviewable contract for `liminal_benches`, so the implementation tasks can be done without guessing.

## Context

- `docs/ARCHITECTURE.md` §2 (module map: benchmark harness and engine benchmarks (ADR-0023)).
- `docs/specs/_TEMPLATE.md`; `docs/specs/xtask.md` as the quality bar.
- `architecture.toml` entry for `liminal_benches` (layer, `unsafe`, `deterministic`, allowed deps).
- ADR-0023 (all of it); `docs/process/PERFORMANCE.md`.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Spec for: the `std`-only harness (warm-up, calibration, samples, median/MAD/min/max, throughput), case IDs `<crate>/<group>/<case>`, the JSON result format (`version: 1`, machine metadata), the single-threaded marker for the instruction-count gate, and how bench targets are laid out. This is a tool crate (layer 5): tool-crate gates apply.

**Out:** Code. The implementation tasks (M1-T016, M1-T017) write it.

## Acceptance criteria

- [x] `docs/specs/liminal_benches.md` exists, follows `docs/specs/_TEMPLATE.md` (all sections), with
      `maturity: planned`, `spec_version: 1` and `adrs` listing every ADR it implements.
- [x] Every requirement has an ID `BENCH-Rnnn`, an RFC 2119 level and a testable statement with
      units, ranges and failure behaviour. No vague words ("fast", "robust", "efficient").
- [x] §5 Public API covers the scope below and is consistent with `docs/ARCHITECTURE.md` and the
      specs of the crates it depends on.
- [x] Error codes `LIM-BENCH-NNNN` are listed with message and `help` (ADR-0017), or the spec says
      why the crate has no recoverable errors.
- [x] The test plan names the test kinds that apply (TESTING.md) and, for every optimized path, the
      reference implementation it is checked against (AGENTS.md rule 4).
- [x] Performance budgets name scenarios and machines; numbers that need a measurement say which
      task measures them (PERFORMANCE.md: budgets are never invented).
- [x] Every MUST requirement is assigned to exactly one implementation task (M1-T016, M1-T017): the spec
      lists the assignment, and this session writes the IDs into those tasks' `requirements`
      frontmatter (`check-docs` R028 validates them).
- [x] Open questions are listed in the spec instead of being decided silently.
- [x] `docs/specs/README.md` lists the spec (XTASK-R027); `cargo xtask ci` passes.

## Test requirements

None (documentation task). The review is a spec review (REVIEW.md §2).

## Notes

Requirement IDs leave gaps between groups (template §6). Keep the API small: every public item has to be documented, tested and later snapshotted.

## Handoff

Done as one unit with M1-T016 (ADR-0029 Decision 3), branch `task/M1-T015-M1-T016-liminal-benches`,
2026-11-10. Status `review`; one review covers both tasks (task review and spec review, REVIEW.md
§1 and §2).

- `docs/specs/liminal_benches.md` `spec_version` 1, `maturity: planned`: 27 MUST requirements
  (BENCH-R001 … R005, R010 … R015, R020 … R024, R030 … R035, R040 … R042, R050, R060) and one
  SHOULD (R061, the `liminal_core` case list), all MUST assigned to M1-T016 (its `requirements`
  frontmatter). M1-T017 is assigned none of them: it adds XTASK requirements and reads the result
  file (spec §6 "Assignment").
- Error codes LIM-BENCH-0001 … 0006 (§8). Four open questions (§12): machine IDs (M1-T017),
  exact case selection for the instruction-count job, power state off macOS, raw samples.
- New prefix `BENCH` added to the reserved table and the index in `docs/specs/README.md`.
- Budgets: none (tool crate); §9 says M1-T017 records the first baselines.
- Choices a reviewer should check: MAD is unscaled; loop and `black_box` overhead stay in every
  sample; metadata comes from environment variables first, then `git`, `rustc -V`, `sw_vers`,
  `uname`, `pmset`; the default output directory is `<workspace>/target/liminal-bench/`.

## Review

_Empty._
