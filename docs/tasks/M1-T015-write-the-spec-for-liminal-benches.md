---
id: M1-T015
title: Write the spec for liminal_benches
milestone: M1
status: done
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

2026-11-10, review sub-agent (ADR-0026, ADR-0029 Decision 3, ADR-0030; journal
`docs/journal/2026-11-10-M1-T015-M1-T016-review.md`). Spec review (REVIEW.md §2) and task
review of the criteria above. Outcome: **done**, no blocking finding against this task.

Checked: all template sections present, `maturity: planned`, `spec_version: 1`, `adrs`
ADR-0003/0017/0018/0023; purpose, scope and non-goals; every requirement has an ID, a level and a
testable statement (no vague words); §5 API consistent with ADR-0023 Decisions 1, 2 and 4
(10 ms samples, 30 samples, median/MAD/min/max, throughput, `version: 1`, machine metadata,
single-threaded marker, fixed iterations for `k`/`2k`) and with `liminal_core::error`;
error codes LIM-BENCH-0001 … 0006 with help; test plan with oracles and the statement that no
optimized path exists; budgets: none, with the measuring task named; MUST assignment to M1-T016
matches its `requirements`; four open questions; `docs/specs/README.md` row and prefix.

The code/spec disagreement on BENCH-R024 is a required change of M1-T016 (item 5 there); if it is
resolved by a spec change, the re-reviewer checks that change. This status reaches `main` with the
merge of the shared branch after M1-T016's re-review (ADR-0029 Decision 3).

Backlog: shared with M1-T016 (`docs/BACKLOG.md`, "M1-T015, M1-T016 — review 2026-11-10").
Follow-up task: none.
