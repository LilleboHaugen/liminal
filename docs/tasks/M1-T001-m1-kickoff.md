---
id: M1-T001
title: M1 kickoff
milestone: M1
status: review
crates: []
depends_on: [M0-T010]
requirements: []
---

# M1-T001: M1 kickoff

## Goal

Milestone M1 (headless core) is broken into detailed, session-sized tasks, and the decisions it needs
are made, so implementation agents can start without guessing.

## Context

- `docs/ROADMAP.md` §M1 (work items and exit criteria), milestone kickoff rule.
- `docs/OPEN_QUESTIONS.md` OQ-004 (benchmark harness), due at M1 kickoff.
- ADR-0008 (ECS), ADR-0009 (determinism), ADR-0016 (telemetry), ADR-0017 (errors), ADR-0018 (unsafe).
- `docs/specs/_TEMPLATE.md`, and `docs/specs/xtask.md` as the quality bar for specs.

## Scope

**In:**
1. Resolve OQ-004 with an ADR (benchmark harness, CI regression method, how comparison crates are
   brought in). Add any approved crates to `architecture.toml`. Resolve OQ-010 (location of
   determinism tests for layer 0–1 crates) with an ADR.
2. Choose the public-API snapshot tool for the `stable` gate (MATURITY.md). ADR if it adds a tool.
3. Write task files for all M1 work items. Suggested order and granularity:
   - Per crate: one task "write spec for `<crate>`" followed by implementation tasks of roughly one
     session each, ordered reference-implementation-first.
   - Crate order: `liminal_core` → `liminal_math` → `liminal_memory` → `liminal_telemetry` →
     `liminal_reflect`(+derive) → `liminal_jobs` → `liminal_ecs` → `liminal_app` → `liminal_testkit`
     (headless subset) → `headless_boids` example → competitive benchmarks → Miri CI job → M1 exit
     review.
4. Update `docs/tasks/README.md` and STATUS.

**Out:** writing the specs themselves (each is its own task, so each gets a focused session and an
independent review).

## Acceptance criteria

- [ ] OQ-004 and OQ-010 resolved by accepted ADRs. Their OPEN_QUESTIONS entries are marked resolved.
      (ADR-0023, ADR-0024 written as `proposed`, entries marked resolved. Acceptance is the ADR
      review, REVIEW.md §4, by the session that reviews this task; see Handoff.)
- [x] Every M1 work item and exit criterion in ROADMAP.md is covered by at least one task. Each task has
      checkable acceptance criteria and `depends_on` set.
- [x] `cargo xtask check-docs` passes (task graph valid, no cycles).
- [x] The project owner is told about the M1 plan, along with any decision that needs their input.

## Handoff

2026-11-09, one session (it reviewed M0-T010 first, journal `2026-11-09-M0-T010-review.md`).
Journal: `2026-11-09-M1-T001.md`. Status: `review`.

**Done:**
1. ADR-0023 (OQ-004: own `std`-only harness in `liminal_benches`, wall-clock baselines on the
   reference machine via `cargo xtask bench`, instruction-count CI gate under Valgrind on Linux,
   competitive crates in a separate workspace `benches/competitive/`), ADR-0024 (OQ-010:
   crate-local `tests/determinism.rs` on the `liminal_core` state hasher; testkit for apps),
   ADR-0025 (API snapshot tool: `cargo-public-api`, from the first `stable` promotion; nothing
   installed in M1). All three `proposed`. OPEN_QUESTIONS OQ-004 and OQ-010 marked resolved.
2. `architecture.toml`: planned crates `liminal_benches` and `liminal_benches_competitive`;
   `[external.*]` entries `bevy_ecs`, `hecs`, `glam`, `rayon` (scope `dev`, ADR-0023), listed only
   by the competitive crate.
3. Process docs aligned with the ADRs: TESTING.md, PERFORMANCE.md, MATURITY.md, ARCHITECTURE.md
   §6, DEPENDENCIES.md.
4. Task files M1-T011 … M1-T053, `docs/tasks/README.md` (also the missing M0-T011 row), ROADMAP
   §M1 task line, STATUS.md regenerated.

**Order.** IDs are in intended order; the pick rule (first `todo` with dependencies `done`) then
gives: follow-ups M1-T002 … T010 first (all small, all harden gates engine crates meet; the first
engine implementation task M1-T013 depends on T003–T005), then per crate spec → implementation,
core → benches → Miri/loom CI → math → memory → telemetry → reflect → instruction gate → jobs →
ECS → app → testkit → `headless_boids` → competitive → exit review. Two deviations from the
suggested order in Scope item 3, with reasons: benchmark tooling (T015–T017) and the Miri/loom CI
job (T018) come right after `liminal_core`, because math SIMD (T022) must benchmark against the
scalar path and the first `unsafe = "allow"` crates need Miri from their first commit.

**Coverage of ROADMAP §M1:**

| Work item / exit criterion | Tasks |
|---|---|
| Specs + implementation to `usable` of core, math, memory, telemetry, reflect(+derive), jobs, ecs, app | T012–T014, T019–T023, T024–T026, T027–T029, T030–T032, T034–T036, T037–T043, T044–T046 |
| `liminal_testkit` headless subset | T047–T049 |
| Benchmark harness decision and first baselines | ADR-0023; T015–T017; T033 (CI gate) |
| Competitive benchmarks ECS, math, jobs | T051, T052 |
| `headless_boids`, identical hash 10 × 10,000 steps, multithreaded | T050 |
| Miri on every `unsafe = "allow"` M1 crate; loom for jobs, telemetry, ecs | T018 (CI), T022, T025, T028, T035, T042 (tests), usable ACs |
| Determinism lints (ADR-0009 item 7), needed by every deterministic crate | T011 |
| Exit review | T053 |

**For the reviewer:**
- Do the ADR review (REVIEW.md §4) of ADR-0023, ADR-0024 and ADR-0025 and set them `accepted`
  (or require changes). None adds a runtime dependency or changes a layer rule or AGENTS.md
  rule, so owner approval is not required by §4; the owner was told about all three.
- `liminal_benches` is a tool crate needed by M1 but not an M1 work item: its maturity is not an
  M1 exit gate. Say so in the review if you disagree.
- Each spec task fills the `requirements` of its implementation tasks; until then they are `[]`.

**Owner decisions coming up (not blocking now):** M1-T018 asks the owner whether to install a
pinned nightly toolchain with Miri on the Mac (ADR-0022); M1-T051/T052 report any `cargo deny`
issue of the comparison crates to the owner.

## Review

_Empty._
