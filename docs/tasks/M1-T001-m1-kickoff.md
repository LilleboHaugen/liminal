---
id: M1-T001
title: M1 kickoff
milestone: M1
status: todo
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
- [ ] Every M1 work item and exit criterion in ROADMAP.md is covered by at least one task. Each task has
      checkable acceptance criteria and `depends_on` set.
- [ ] `cargo xtask check-docs` passes (task graph valid, no cycles).
- [ ] The project owner is told about the M1 plan, along with any decision that needs their input.

## Handoff

_Empty._

## Review

_Empty._
