---
id: M1-T049
title: Implement liminal_testkit replay; reach usable
milestone: M1
status: todo
crates: [liminal_testkit]
depends_on: [M1-T048]
requirements: []
---

# M1-T049: Implement `liminal_testkit` replay; reach `usable`

## Goal

Recorded inputs replay with hash checkpoints, and `liminal_testkit` is ready for the `usable` promotion review.

## Context

- `docs/specs/liminal_testkit.md`.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Replay recording and playback, checkpoint comparison, the `usable` evidence for the headless subset.

**Out:** Golden images (M3).

## Acceptance criteria

- [ ] Every requirement in this task's `requirements` frontmatter has at least one test with a
      `/// Verifies:` line that checks the behaviour (REVIEW.md §1 item 3).
- [ ] New public items are documented; primary entry points have runnable `# Examples`.
- [ ] `cargo xtask ci` passes with no failed or skipped step.
- [ ] A replay recorded once passes on replay; a changed system fails it at the first checkpoint after the change.
- [ ] The spec's §Maturity checklist holds the evidence for every `usable` gate in
      `docs/process/MATURITY.md` (trace with `maturity: usable` applied on a scratch copy, coverage
      against the target for its crate type (TESTING.md has no row for layer 4 yet: this task
      proposes one in TESTING.md, with the reason), examples, integration tests).
- [ ] The task is set to `review` with `maturity` still unchanged. The reviewing session performs
      the promotion review (REVIEW.md §3) and, if every gate holds, sets `maturity: usable` in the
      spec and `docs/specs/README.md` and regenerates STATUS.md (MATURITY.md "Promotion
      procedure": a different session changes `maturity`).

## Test requirements

Requirements in frontmatter.

## Notes

_None._

## Handoff

_Empty._

## Review

_Empty._
