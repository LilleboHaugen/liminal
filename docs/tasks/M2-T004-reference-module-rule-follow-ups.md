---
id: M2-T004
title: Reference-module rule follow-ups from the M1-T054 review
milestone: M2
status: todo
crates: []
depends_on: [M1-T054]
requirements: []
---

# M2-T004: Reference-module rule follow-ups from the M1-T054 review

## Goal

The reference-module rule in `docs/process/PERFORMANCE.md` covers every shape of optimized path
the M1 specs contain, and the small text gaps the M1-T054 review found are closed.

## Context

- Follow-up task under ADR-0021, written by the review of M1-T054 (see that task's "Review").
- `docs/process/PERFORMANCE.md` "Reference implementations reachable from benchmarks" (rule set
  by M1-T054), items 1, 3 and 4, and the reasons table.
- No M1 exit criterion needs this, so it is in M2 (ADR-0021). Until it is done, a spec task that
  meets one of the gaps below decides the case in its spec and says so in its journal.

## Scope

**In:**
1. **Counterparts that are not free functions.** Item 1 says a reference item "has the same name
   and signature as its optimized counterpart" and gives `liminal_math::reference::mat4_mul`.
   It does not say what the reference is when the optimized path is a method or operator
   (`Mat4 * Mat4`, `Vec3::dot`), a trait implementation, or a type (an allocator, a lock-free
   queue, an ECS storage). Read the M1 specs that exist by then (`liminal_math`,
   `liminal_memory`, `liminal_jobs`, `liminal_ecs`), state the naming and signature rule for
   each shape they use, and bring PERFORMANCE.md item 1 and those specs into agreement.
2. **Context line stronger than the rule.** M1-T019, M1-T024, M1-T034 and M1-T037 Context says
   "the spec's API section lists the crate's `reference` items for each optimized path".
   Item 3 makes a reference public only when a benchmark or an outside test needs it. Where one
   of those tasks is still `todo`, reword the line to match item 3; where it is not, check that
   the spec made no reference public that item 3 keeps private (`liminal_memory` keeps its
   reference models in `tests/common/reference.rs`, which item 3 allows).
3. **Unverified assumption.** The reasons table says `#[doc(hidden)]` items are "(assumption, to
   be confirmed by the ADR-0025 task) absent from the `cargo public-api` snapshot". No task is
   named. When `cargo-public-api` is installed for the first `stable` promotion (ADR-0025,
   ADR-0022), run it on a crate with a `#[doc(hidden)]` public item and replace the assumption
   with the observed result, or name the task that will.
4. **Kickoff-rule sentence.** `docs/ROADMAP.md` "Milestone kickoff rule" now starts "M0 and M1
   have detailed task files today". M2 already has follow-up task files (M2-T001 onward), and the
   sentence goes stale at every kickoff. Replace it with a statement that needs no update per
   milestone (for example a pointer to `docs/STATUS.md` "Milestone progress"), and re-wrap the
   over-long line that follows it.
5. **Parent journal file name.** `docs/journal/2026-11-10-M1-T054.md` names its parent as "the
   2026-11-10 parent session's journal entry", not as a file (`docs/journal/README.md`,
   ADR-0026 Decision 8.2). The ADR-0026 prompt templates do not carry the file name. Add to
   `docs/journal/README.md` or `docs/process/WORKFLOW.md` how a sub-agent finds it (the parent
   commits its entry `docs/journal/YYYY-MM-DD-session-<topic>.md` on
   `session/YYYY-MM-DD-<topic>` before its first spawn), without changing an accepted ADR.
   Journal entries are append-only: do not edit the old entry.

**Out:** Implementing or moving any reference implementation (each crate's tasks do that).
Changing ADR-0023, ADR-0025 or ADR-0026.

## Acceptance criteria

- [ ] PERFORMANCE.md item 1 states the rule for every shape of optimized path that an existing
      M1 spec names, and those specs' API sections agree with it.
- [ ] Scope items 2 to 5 are done, or the journal says which other task already did each.
- [ ] `cargo xtask ci` passes.

## Test requirements

None (documentation task).

## Notes

Written by the review of M1-T054 (2026-11-10).

## Handoff

_Empty._

## Review

_Empty._
