---
id: M1-T054
title: M1 kickoff follow-ups from the M1-T001 review
milestone: M1
status: done
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

Reviewer: review sub-agent (ADR-0026), 2026-11-10, parent
`docs/journal/2026-11-10-session-m1-reviews.md`; did not write M1-T054 (its author was a
sub-agent of `2026-11-09-session-m1-subagents.md`). Prompt: the ADR-0026 review template
verbatim. Outcome: **done**, no blocking findings. Follow-up task:
`docs/tasks/M2-T004-reference-module-rule-follow-ups.md`.

Checked, macOS 27.2, Rust 1.99.1, on `97e89b2`:
1. Acceptance criteria (by reading the diff `main...97e89b2` and the documents it cites):
   - Reference-exposure rule: `docs/process/PERFORMANCE.md` "Reference implementations reachable
     from benchmarks" states the rule (5 items) and the reasons against a cargo feature and
     `#[doc(hidden)]`. Checked against the sources: ADR-0023 Decision 1 and Consequences
     ("benchmarks see only public API") are kept, so no superseding ADR is needed; the root
     `Cargo.toml` has `resolver = "3"`; CODE_STYLE.md "Formatting and structure" has "every
     feature must build alone"; ADR-0025 names `cargo public-api`. Item 5 agrees with the
     M1-T022 AC ("each kept SIMD path faster than scalar on the reference machine": only the
     path selected there can run there). M1-T019, M1-T024, M1-T034 and M1-T037 are `todo` on
     this branch and on `main`, and their Context names the rule. Met.
   - M1-T051 (`glam`, `rayon`) and M1-T052 (`bevy_ecs`, `hecs`) are `todo` and carry the AC with
     the four data points of DEPENDENCIES.md "Adding a dependency" step 2. Met.
   - Stale text: ROADMAP.md kickoff-rule sentence and §M1 "ADR-0023"; TESTING.md Concurrency row
     has `liminal_telemetry` (agrees with ROADMAP §M1 exit criterion 4 and M1-T028);
     `docs/specs/xtask.md` §2 and §12, `spec_version` 21 with history row, and M1-T017 (`todo`)
     does specify `cargo xtask bench`. No other document outside journals, tasks and ADRs
     describes OQ-004 as open (grep). Met.
   - `cargo xtask ci`: pass, 11 steps, none skipped. Met.
2. Tests: documentation task, no code or test changed; no requirement added (history row says
   so, and the requirement tables are unchanged in the diff).
3. No fake progress: no oracle, floor, baseline or test touched. Scope not reduced.
4. Spec agreement, unsafe, determinism, architecture: no code, no dependency, no
   `architecture.toml` change.
5. Session boundaries (REVIEW.md item 10): the implementation sub-agent is no. 17 in
   `docs/journal/2026-11-09-session-m1-subagents.md` (owner-started parent, "impl template");
   it ran alone as a `crates: []` task. This review belongs to a different owner-started session.
   No install, no spawned agent in either.
6. Public repository (REVIEW.md item 11): no workflow change; no issue, pull request or outside
   code acted on.
7. Branch CI: run 42856758108 on `97e89b2` is red only because the `checks (macos-latest)` job
   was never acquired by a hosted runner (15 min, no step ran); Linux, Windows, coverage and
   deny jobs are green. Not a finding against the task. The run on this review commit is the one
   the merge waits for (see the review journal entry).

Non-blocking findings (follow-up M2-T004):
- PERFORMANCE.md item 1 ("same name and signature") does not say what the reference is for an
  optimized method, operator, trait implementation or type.
- The Context line added to the four spec tasks ("for each optimized path") is stronger than
  item 3 (public only when a benchmark or outside test needs it).
- The `#[doc(hidden)]` / `cargo public-api` assumption names "the ADR-0025 task"; no such task
  is identified.
- ROADMAP.md "M0 and M1 have detailed task files today": M2 has task files too, and the
  sentence goes stale at each kickoff; the next line is not re-wrapped.
- The task's journal entry names its parent by description, not by file name.
