---
id: ADR-0017
title: Error model with stable codes
status: accepted
date: 2026-11-08
supersedes: []
superseded_by: []
---

# ADR-0017: Error model with stable codes

## Context

Agents act on errors: they read the message, search the docs, and try a fix. Free-text errors change
wording over time, break tests that match on text, and give agents nothing stable to look up. Panics
in an engine end the whole game and lose context.

## Decision

- Each crate defines its own error enum(s). Every variant has:
  - a **stable code** `LIM-<AREA>-<NNNN>` (AREA = the crate's spec `req_prefix`, e.g. `ECS`, `GPU`),
    never reused or renumbered;
  - a human- and agent-readable message;
  - structured context fields;
  - an optional `help` hint describing how to fix the problem when that is knowable.
- `liminal_core` provides the shared `ErrorCode` type and the trait every engine error implements
  (code, message, help, JSON serialization).
- Each crate's spec lists its error codes in an **Error codes** table. Removed codes are marked
  `(retired)`, not deleted.
- Recoverable conditions (bad data, missing assets, I/O, device loss) MUST return errors. Panics are
  only for violated internal invariants. `unwrap()` is denied in library code (CODE_STYLE.md).
- The probe and CLI return errors as JSON objects with `code`, `message`, `help`, `context`.

## Alternatives considered

- **`thiserror` / `anyhow`:** `anyhow` loses type information. `thiserror` is a convenient derive
  but adds nothing the codes require, and we would still need our own trait. A Liminal derive macro
  may be added later if boilerplate becomes a problem.
- **Plain strings:** not stable, not machine-actionable.

## Consequences

Some boilerplate per error. Tests can assert on codes instead of message text, and agents can look up
codes in the specs.

## Verification

Review. `cargo xtask check-docs` (from M1) checks that every code used in a crate is listed in its spec
and vice versa.
