---
id: ADR-0024
title: Determinism tests for layer 0–1 crates are crate-local, built on liminal_core hashing
status: accepted
date: 2026-11-09
supersedes: []
superseded_by: []
---

# ADR-0024: Determinism tests for layer 0–1 crates are crate-local, built on liminal_core hashing

## Context

Resolves OQ-010 (`docs/OPEN_QUESTIONS.md`). `docs/process/TESTING.md` puts determinism and replay
tests in `liminal_testkit`, and `docs/process/MATURITY.md` requires determinism tests before a
deterministic crate reaches `usable`. `liminal_testkit` is layer 4. Layer 0–1 crates cannot depend
on it, not even as a dev-dependency (XTASK-R032). In M1 six deterministic crates
(`architecture.toml` `deterministic = true`) at layers 0–1 must reach `usable`: `liminal_core`,
`liminal_math`, `liminal_memory`, `liminal_reflect`, `liminal_ecs`, `liminal_app`.

ADR-0009 already places the building blocks low in the graph: seeded RNG streams and fixed-seed
hashing in `liminal_core`, and `World::state_hash()` in `liminal_ecs`.

## Decision

1. **Primitives in `liminal_core`.** `liminal_core` provides the state-hashing primitive: a hasher
   with a fixed algorithm, fixed seed and defined byte order, so the same values give the same hash
   on every platform and in every process. Its exact API and algorithm are specified in
   `docs/specs/liminal_core.md`. No other crate defines its own state hasher.
2. **Crate-local determinism tests.** Each deterministic crate at layer 0–1 keeps its determinism
   tests in its own `tests/determinism.rs`, using only its own dependencies and the `liminal_core`
   primitive. A determinism test runs a seeded workload at least twice in the same process and
   compares state hashes after every step (or after the whole workload for crates without steps).
   `liminal_ecs` and `liminal_app` also compare the single-threaded and multithreaded executors.
   Tests carry `/// Verifies:` lines for the crate's determinism requirements.
3. **Recorded hashes where cross-platform identity is claimed.** If a crate's spec claims
   cross-platform bit-identity for a workload (ADR-0009 item 5), its determinism test MUST also
   compare against a hash literal recorded in the test. CI runs it on macOS, Windows and Linux. A
   crate that does not claim it does not record literals.
4. **`liminal_testkit` for apps.** `liminal_testkit` (layer 4) provides `assert_deterministic` and
   replay for whole apps, built on `World::state_hash()` and the `liminal_core` primitive. It is used
   by examples, demo games and its own tests, not by layer 0–1 crates.
5. `TESTING.md` is updated to say this ("Test kinds" table, "Determinism tests").

## Alternatives considered

- **A workspace-level test crate at layer 5 (`tests/determinism`) that depends on everything.**
  Rejected: a crate's determinism evidence would live away from the crate, and its reviewers would
  have to look in two places. It would also be a crate every engine change rebuilds.
- **A `liminal_testkit_core` crate at layer 1.** Rejected: its content would be the hasher, which
  ADR-0009 already puts in `liminal_core`, plus a run-twice loop of a few lines. A new crate for
  that adds a spec, a coverage floor and a review for no new capability.
- **Change the layer rules so testkit may be a dev-dependency of low layers.** Rejected: it creates a
  dependency cycle in the development graph (testkit depends on ECS and app) and weakens
  XTASK-R032.
- **Do nothing.** Not possible: MATURITY.md's `usable` gate cannot be met by layer 0–1 crates as
  TESTING.md is written today.

## Consequences

- Layer 0–1 determinism tests are simpler (no app harness). They cannot record inputs or replay;
  that stays in testkit, where apps exist.
- The `liminal_core` hasher becomes a compatibility point: changing its algorithm changes every
  recorded hash. Such a change is an oracle change (AGENTS.md rule 5) and needs a journal entry.
- Revisit if several crates start duplicating the same multi-step test scaffolding.

## Verification

- Review checks that each deterministic layer 0–1 crate has `tests/determinism.rs` with `Verifies:`
  lines before it is promoted to `usable`.
- `cargo xtask trace` gates the determinism requirements once the crate is `usable`.
