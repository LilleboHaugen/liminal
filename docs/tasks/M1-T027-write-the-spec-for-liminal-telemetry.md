---
id: M1-T027
title: Write the spec for liminal_telemetry
milestone: M1
status: todo
crates: [liminal_telemetry]
depends_on: [M1-T001]
requirements: []
---

# M1-T027: Write the spec for `liminal_telemetry`

## Goal

`docs/specs/liminal_telemetry.md` is a complete, reviewable contract for `liminal_telemetry`, so the implementation tasks can be done without guessing.

## Context

- `docs/ARCHITECTURE.md` §2 (module map: structured logs, spans, counters, frame stats; JSON and Chrome-trace export).
- `docs/specs/_TEMPLATE.md`; `docs/specs/xtask.md` as the quality bar.
- `architecture.toml` entry for `liminal_telemetry` (layer, `unsafe`, `deterministic`, allowed deps).
- ADR-0016 (own telemetry); ADR-0018.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Spec for: log records, spans, counters and their macros or functions; the compile-out feature and what "near-zero cost when disabled" means in numbers; per-thread buffers and their lock-free hand-off; frame records; Chrome-trace JSON and compact JSON summary formats (written without `serde`, which is not approved for runtime crates, OQ-002); the overhead budget per span (measured by the second telemetry task).

**Out:** Code. The implementation tasks (M1-T028, M1-T029) write it.

## Acceptance criteria

- [ ] `docs/specs/liminal_telemetry.md` exists, follows `docs/specs/_TEMPLATE.md` (all sections), with
      `maturity: planned`, `spec_version: 1` and `adrs` listing every ADR it implements.
- [ ] Every requirement has an ID `TEL-Rnnn`, an RFC 2119 level and a testable statement with
      units, ranges and failure behaviour. No vague words ("fast", "robust", "efficient").
- [ ] §5 Public API covers the scope below and is consistent with `docs/ARCHITECTURE.md` and the
      specs of the crates it depends on.
- [ ] Error codes `LIM-TEL-NNNN` are listed with message and `help` (ADR-0017), or the spec says
      why the crate has no recoverable errors.
- [ ] The test plan names the test kinds that apply (TESTING.md) and, for every optimized path, the
      reference implementation it is checked against (AGENTS.md rule 4).
- [ ] Performance budgets name scenarios and machines; numbers that need a measurement say which
      task measures them (PERFORMANCE.md: budgets are never invented).
- [ ] Every MUST requirement is assigned to exactly one implementation task (M1-T028, M1-T029): the spec
      lists the assignment, and this session writes the IDs into those tasks' `requirements`
      frontmatter (`check-docs` R028 validates them).
- [ ] Open questions are listed in the spec instead of being decided silently.
- [ ] `docs/specs/README.md` lists the spec (XTASK-R027); `cargo xtask ci` passes.

## Test requirements

None (documentation task). The review is a spec review (REVIEW.md §2).

## Notes

Requirement IDs leave gaps between groups (template §6). Keep the API small: every public item has to be documented, tested and later snapshotted.

## Handoff

_Empty._

## Review

_Empty._
