---
id: ADR-0011
title: Maturity levels and promotion gates
status: accepted
date: 2026-11-08
supersedes: []
superseded_by: []
---

# ADR-0011: Maturity levels and promotion gates

## Context

The engine is meant to be feature-complete eventually, and no session is expected to finish a feature.
Without explicit levels, partially built crates look finished, and games come to rely on unproven
code.

## Decision

- Each crate has one maturity level: `planned`, `experimental`, `usable`, `stable`, `optimized`, stored
  in its spec frontmatter.
- Gates for each level are defined in `docs/process/MATURITY.md`. Promotion requires every gate, with
  evidence recorded in the spec, and a review by a different session.
- Demo games and the facade may depend only on crates at `usable` or higher. A `stable` crate may not
  depend on an `experimental` one.
- Demotion is allowed and expected when a gate stops holding.

## Alternatives considered

- **Semantic versions as the maturity signal:** versions describe API compatibility, not quality.
- **Binary "done / not done":** encourages declaring things done early.

## Consequences

Progress is visible per crate in `docs/STATUS.md`. Reaching `stable` and `optimized` takes deliberate
work, which is intended.

## Verification

`cargo xtask status` shows levels. `cargo xtask check-arch` (from M2) enforces the maturity dependency
rule.
