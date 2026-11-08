---
id: ADR-0010
title: Testing strategy — traceability, oracles, coverage ratchet
status: accepted
date: 2026-11-08
supersedes: []
superseded_by: []
---

# ADR-0010: Testing strategy — traceability, oracles, coverage ratchet

## Context

The project owner requires full test coverage, but not at the cost of halting progress. Agents can
produce tests that raise coverage without proving anything. The project needs tests that act as
oracles: independent definitions of correct behaviour that a machine can check.

## Decision

- Test kinds, locations and requirements are defined in `docs/process/TESTING.md`.
- **Traceability:** spec requirements have stable IDs. Tests declare which requirements they verify with
  `/// Verifies: <ID>`. `cargo xtask trace` enforces that every MUST requirement of a crate at maturity
  `usable` or above has a test.
- **Oracles for optimized code:** every non-trivial optimized algorithm has a reference implementation
  and differential tests.
- **Coverage ratchet:** per-crate floors in `ci/coverage.toml`. CI fails if coverage drops more than
  0.5 percentage points below the floor. Floors only rise, except through a justified journal entry.
  Absolute coverage targets gate maturity promotion, not individual commits.
- **Golden images** for rendering, with per-backend references and explicit tolerances.
- **Determinism and replay tests** for deterministic crates (ADR-0009).
- Approved dev-scope crates: `proptest` (property testing) and `loom` (concurrency permutation testing).
  Tools (not crates): `cargo-llvm-cov`, Miri, `cargo-fuzz`, `cargo-deny`.

## Alternatives considered

- **Hard global coverage threshold on every commit (e.g. 90%):** blocks early work and encourages
  assertion-free tests. Rejected.
- **No coverage measurement:** decay is invisible. Rejected.
- **Mutation testing as a gate:** strong signal but slow. It may be added for `stable` crates later
  (`cargo-mutants`), through a new ADR.

## Consequences

Writing specs with IDs takes effort up front, and in return gives an automated answer to "is this
crate actually finished?". The ratchet allows uneven progress across crates.

## Verification

`cargo xtask trace`, `cargo xtask coverage`, review checklist (`docs/process/REVIEW.md` §1).
