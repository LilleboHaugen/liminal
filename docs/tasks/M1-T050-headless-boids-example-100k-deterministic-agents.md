---
id: M1-T050
title: headless_boids example — 100k deterministic agents
milestone: M1
status: todo
crates: []
depends_on: [M1-T049, M1-T023]
requirements: []
---

# M1-T050: `headless_boids` example: 100k deterministic agents

## Goal

An original boids simulation with 100,000 agents runs headless and gives the same state hash over 10,000 steps across 10 runs with the multithreaded scheduler (M1 exit criterion 2).

## Context

- ROADMAP.md §M1; ADR-0009; ADR-0024 item 4; AGENTS.md rule 14 (original content).
- `check-arch` and `check-docs` rules: every workspace member is registered in `architecture.toml` and has a spec (XTASK-R026).
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Where the example lives (an `[[example]]` target of an existing crate, or its own crate under `examples/` with a registry entry and a short spec) decided and recorded; the simulation; a determinism test with `liminal_testkit`; the 10 × 10,000-step run.

**Out:** CLI launch of the example (M2).

## Acceptance criteria

- [ ] Location decision recorded (journal and, if it is a crate, `architecture.toml` and a spec).
- [ ] A test (reduced size, runs in `cargo xtask ci`) asserts determinism with the multithreaded executor.
- [ ] The full run (100,000 agents, 10,000 steps, 10 runs, multithreaded) is executed on the reference machine with one identical hash; command, time and hash in the journal.
- [ ] `cargo xtask ci` passes.

## Test requirements

Determinism test via `liminal_testkit`.

## Notes

_None._

## Handoff

_Empty._

## Review

_Empty._
