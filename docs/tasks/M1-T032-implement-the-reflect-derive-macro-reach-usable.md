---
id: M1-T032
title: Implement the Reflect derive macro; reach usable for both reflect crates
milestone: M1
status: todo
crates: [liminal_reflect, liminal_reflect_derive]
depends_on: [M1-T031]
requirements: [REFL-R120, REFL-R130, REFL-R131, REFL-R132, REFLD-R001, REFLD-R002, REFLD-R003, REFLD-R004, REFLD-R005, REFLD-R020, REFLD-R021, REFLD-R030, REFLD-R031, REFLD-R032, REFLD-R033, REFLD-R034, REFLD-R035, REFLD-R050, REFLD-R051, REFLD-R052, REFLD-R053, REFLD-R070]
---

# M1-T032: Implement the `Reflect` derive macro; reach `usable` for both reflect crates

## Goal

`#[derive(Reflect)]` generates impls equal to hand-written ones, and both reflect crates are ready for the `usable` promotion review.

## Context

- `docs/specs/liminal_reflect.md` (and the derive's spec); ADR-0003.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Derive crate; tests comparing derived and hand-written impls; compile-fail tests (rustdoc `compile_fail` doc tests, since no compile-test crate is approved); `tests/determinism.rs`; the `usable` evidence for both crates.

**Out:** Serialization (M2, OQ-002).

## Acceptance criteria

- [ ] The crate exists at the path in `architecture.toml`, is a workspace member, has
      `#![forbid(unsafe_code)]` if `unsafe = "forbid"`, and has a floor in `ci/coverage.toml`.
- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.
- [ ] Differential tests: derived impl and hand-written reference impl give the same descriptors and field access for the same types.
- [ ] The spec's §Maturity checklist holds the evidence for every `usable` gate in
      `docs/process/MATURITY.md` (trace with `maturity: usable` applied on a scratch copy, coverage
      against the 85 % target for its crate type, examples, integration tests).
- [ ] The task is set to `review` with `maturity` still unchanged. The reviewing session performs
      the promotion review (REVIEW.md §3) and, if every gate holds, sets `maturity: usable` in the
      spec and `docs/specs/README.md` and regenerates STATUS.md (MATURITY.md "Promotion
      procedure": a different session changes `maturity`).

## Test requirements

Requirements in frontmatter; differential and compile-fail tests.

## Notes

_None._

## Handoff

_Empty._

## Review

_Empty._
