---
id: M1-T054
title: M1 kickoff follow-ups from the M1-T001 review
milestone: M1
status: review
crates: []
depends_on: [M1-T001]
requirements: []
---

# M1-T054: M1 kickoff follow-ups from the M1-T001 review

## Goal

The non-blocking findings of the M1-T001 review are fixed before the first spec with an optimized
path (M1-T019) is written.

## Context

- Review note in `docs/tasks/M1-T001-m1-kickoff.md` ("Review"), 2026-11-09.
- ADR-0023 (benchmarks see only public API; engine crates have no `benches/`); AGENTS.md rule 4
  (optimized code has a reference implementation); M1-T022 AC "Benchmarks in `liminal_benches`
  show each kept SIMD path faster than scalar".
- `docs/process/DEPENDENCIES.md` "Adding a dependency" step 2.
- M1-T019 (math spec) and M1-T053 (exit review) depend on this task.

## Scope

**In:**
1. **Reachable reference implementations.** Decide how a crate exposes the reference
   implementation of an optimized path so `liminal_benches` can benchmark both (examples: a
   documented public `reference` module, a cargo feature, `#[doc(hidden)]` items). The choice
   must keep the public API small (the API snapshot of ADR-0025 sees every public item) and must
   not need a change to ADR-0023; if it does, write a superseding ADR instead. Record the rule in
   `docs/process/TESTING.md` or `docs/process/PERFORMANCE.md` and name it in the Context of the
   spec tasks for crates with optimized paths (M1-T019, M1-T024, M1-T034, M1-T037) if they are
   still `todo`.
2. **Comparison-crate data.** Add an AC to M1-T051 and M1-T052 (if still `todo`): record, for each
   comparison crate, licence, maintenance status (last release date), transitive dependency count
   at the pinned version and whether it uses `unsafe`, in the competitive crate's documentation
   (DEPENDENCIES.md step 2; ADR-0023 is accepted and immutable).
3. **Stale text.**
   - `docs/ROADMAP.md` line 24: "Only M0 (and the M1 kickoff task) have detailed task files
     today" — M1 now has them.
   - `docs/ROADMAP.md` §M1 work items: "ADR required per OQ-004" → ADR-0023.
   - `docs/process/TESTING.md` "Concurrency" row: add `liminal_telemetry` (ROADMAP exit
     criterion 4, M1-T028).
   - `docs/specs/xtask.md` §2 and §12 describe OQ-004 as open: update with a `spec_version` bump
     and history row, unless M1-T017 has already done so.

**Out:** Implementing reference modules in any crate (each crate's tasks do that).

## Acceptance criteria

- [x] The reference-exposure rule is written in TESTING.md or PERFORMANCE.md, with the reason,
      and the `todo` spec tasks listed in Scope item 1 reference it.
- [x] M1-T051 and M1-T052 (if `todo`) have the comparison-crate data AC.
- [x] The four stale passages are corrected (or the journal says which other task already did).
- [x] `cargo xtask ci` passes.

## Test requirements

None (documentation task), unless the xtask spec change adds a requirement.

## Notes

Written by the review of M1-T001 (2026-11-09).

## Handoff

2026-11-10, implementation sub-agent (ADR-0026), branch `task/M1-T054-m1-kickoff-follow-ups`.
All four ACs met; status `review`.

1. **Reference exposure.** Rule written in `docs/process/PERFORMANCE.md` "Reference
   implementations reachable from benchmarks": a public, always-compiled, documented
   `reference` module at the crate root, items named like their optimized counterpart, only for
   optimized paths that exist or are specified; reasons against a cargo feature (resolver-3
   feature unification gives two API surfaces) and `#[doc(hidden)]` (hidden from docs and,
   assumption, from the `cargo public-api` snapshot) in a table. Needs no ADR-0023 change: the
   items are public API. Pointers added to PERFORMANCE.md "Rules for optimization work" item 2
   and the TESTING.md "Differential" row. The Context of M1-T019, M1-T024, M1-T034 and M1-T037
   (all `todo`) names the rule.
2. **Comparison-crate data.** New AC in M1-T051 (`glam`, `rayon`) and M1-T052 (`bevy_ecs`,
   `hecs`), both `todo`.
3. **Stale text.** `docs/ROADMAP.md` kickoff-rule sentence and §M1 "ADR required per OQ-004" →
   ADR-0023; `docs/process/TESTING.md` Concurrency row adds `liminal_telemetry`;
   `docs/specs/xtask.md` §2 and §12 (spec_version 20 → 21, history row; M1-T017 had not done it).
4. `cargo xtask ci`: pass, 11 steps, macOS 27.2, Rust 1.99.1.

Trap for the reviewer and for merges: `docs/specs/xtask.md` `spec_version` 21 may collide with
another branch that also bumps it; renumber on merge, keeping both history rows.

## Review

_Empty._
