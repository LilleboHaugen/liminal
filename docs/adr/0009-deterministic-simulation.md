---
id: ADR-0009
title: Deterministic simulation
status: accepted
date: 2026-11-08
supersedes: []
superseded_by: []
---

# ADR-0009: Deterministic simulation

## Context

Agents verify behaviour by running games headless and comparing results. That works only if the same
inputs produce the same state. Determinism also enables replay-based regression tests, reproducible bug
reports, and (later) lockstep networking. Retrofitting determinism into an engine is very hard, so it
has to be designed in from the start.

## Decision

1. **Fixed timestep.** Simulation runs at a fixed `dt` (default 1/60 s). Rendering interpolates.
   Simulation code MUST NOT read wall-clock time.
2. **Seeded RNG.** All randomness in simulation comes from `liminal_core::rng`, with streams derived
   from the world seed and a stable stream key. No other RNG is used in deterministic crates.
3. **Deterministic collections.** Simulation state MUST NOT use randomly seeded hash maps. Use
   `liminal_core` collections or hashers with fixed seeds. Iteration order of any collection that
   affects simulation results MUST be defined.
4. **Deterministic parallelism.** The ECS scheduler forbids unordered conflicting access (ADR-0008).
   Command buffers apply in a defined order. Parallel reductions MUST be order-independent or performed
   in a fixed order.
5. **Floating point.** Same binary + same platform ⇒ bit-identical results (guaranteed). Deterministic
   crates MUST use `liminal_math::det` for transcendental functions (sin, cos, exp, pow, …) instead of
   platform `libm`, so results do not depend on the OS math library. Cross-platform bit-identity is a
   goal: each deterministic crate records in its spec whether it has been verified across platforms.
6. **State hash.** `World::state_hash()` hashes all reflected simulation state in a canonical order.
   Testkit and probe use it for determinism and replay assertions.
7. Crates marked `deterministic = true` in `architecture.toml` follow these rules, enforced by lints
   (`clippy.toml` `disallowed-methods` / `disallowed-types`) and by tests.

## Alternatives considered

- **Best-effort determinism:** unverifiable, and it rots. Rejected.
- **Fixed-point arithmetic for all simulation:** cross-platform determinism by construction, but slower
  and awkward to work with. It remains an option for specific systems (e.g. lockstep networking),
  decided in their specs.

## Consequences

Some extra care in every simulation crate, and some performance cost for `det` functions (measured and
optimized). In return there is a whole class of automated tests that are not otherwise possible.

## Verification

Determinism tests in `liminal_testkit` (`assert_deterministic`). Replay tests. Lints in deterministic
crates. Single-threaded vs multithreaded scheduler equivalence tests.
