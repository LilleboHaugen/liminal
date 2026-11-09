---
id: M1-T001
title: M1 kickoff
milestone: M1
status: done
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

- [x] OQ-004 and OQ-010 resolved by accepted ADRs. Their OPEN_QUESTIONS entries are marked resolved.
      (ADR-0023, ADR-0024 written as `proposed`, entries marked resolved; accepted by the ADR
      review in this task's review, 2026-11-09.)
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

Reviewer: new session, 2026-11-09 (did not write M1-T001). Outcome: **done**, no blocking
findings. ADR-0023, ADR-0024 and ADR-0025 accepted (REVIEW.md §4). Follow-up task:
`docs/tasks/M1-T054-m1-kickoff-follow-ups.md`.

Checked, macOS 27.2, Rust 1.99.1, cargo-llvm-cov and cargo-deny as in M0-T011:
1. Acceptance criteria:
   - OQ-004/OQ-010: OPEN_QUESTIONS rows and sections say "resolved (ADR-0023/0024)". ADRs
     accepted by this review (below). Met.
   - ROADMAP §M1 coverage: compared every work item and exit criterion with the handoff table and
     the task files. Specs + `usable` for all eight crates (each crate's last task carries the
     MATURITY.md evidence AC; T030/T032 cover `liminal_reflect_derive` too); testkit T047–T049;
     baselines T017; competitive T051/T052; `headless_boids` 10 × 10,000 steps multithreaded
     T050; Miri for math, memory, telemetry, jobs, ecs (T018 job; Miri ACs in T022, T025, T026,
     T028, T029, T035, T036, T038, T039, T042, T043); loom for jobs, telemetry, ecs (T028, T035,
     T042). Read in full: T011, T012, T014, T015, T016, T018, T022, T033, T050, T051, T053; the
     others' ACs by grep. ACs are checkable (named files, commands, seeded failures, journal
     records). Met.
   - `check-docs` passes inside `cargo xtask ci` (11 steps pass). Branch CI run 42678170768
     (0711096) passed. Met.
   - Owner told: STATUS_NOTES "Next step" and the handoff's "Owner decisions coming up". Met.
2. ADR review (REVIEW.md §4):
   - ADR-0023: context states the constraints (runner noise, no Valgrind on macOS arm64,
     XTASK-R032 forbids layer 5 dev-dependencies of engine crates). Alternatives include building
     it ourselves (chosen), `criterion`, `divan`, `iai-callgrind`, wall-clock CI gate, layer-0
     harness, in-workspace comparison crates. Consequences name costs (own harness to maintain,
     instruction counts are a proxy, the competitive workspace can rot). Consistent with ADR-0003
     (dev scope, ADR given) and PERFORMANCE.md. Accepted.
   - ADR-0024: the six deterministic layer 0–1 crates match `architecture.toml`. Rejected
     alternatives are real; no change to XTASK-R032. Accepted.
   - ADR-0025: defers install to the first `stable` milestone and to ADR-0022; licence stated as
     an assumption. Accepted.
   - None adds a runtime dependency, changes a layer rule or an AGENTS.md rule: no owner approval
     needed under §4.
3. No fake progress: documentation-only change; no oracle (golden, baseline, floor) changed; no
   test changed. `architecture.toml` additions are `planned` crates and `dev`-scope externals
   referenced by ADR-0023.
4. Session boundaries: one owner-started session (journal `2026-11-09-M1-T001.md`), after one
   review (M0-T010). The throw-away generator script is not an agent.
5. I agree that `liminal_benches` maturity is not an M1 exit gate: ROADMAP §M1 lists the crates
   that must reach `usable`, and it is not among them.

Non-blocking findings (follow-up M1-T054; M1-T019 and M1-T053 now depend on it):
- ADR-0023 lets benchmarks see only public API, but M1-T022 requires `liminal_benches` to show
  each SIMD path faster than its scalar reference. No task says how a reference implementation
  is reachable from `liminal_benches`. Must be settled before the first spec with an optimized
  path (M1-T019).
- ADR-0023 does not give licence, maintenance status, transitive dependency count and `unsafe`
  use of `bevy_ecs`, `hecs`, `glam`, `rayon` (DEPENDENCIES.md "Adding a dependency" step 2).
  `cargo deny` in M1-T051/T052 covers licences only.
- Stale text: ROADMAP.md line 24 ("Only M0 (and the M1 kickoff task) have detailed task files
  today"), ROADMAP.md §M1 "ADR required per OQ-004"; TESTING.md Concurrency row omits
  `liminal_telemetry` (ROADMAP exit criterion 4, M1-T028); `docs/specs/xtask.md`
  (§2 Scope, §12 Open questions) still describe OQ-004 as open.
