---
id: M0-T010
title: M0 exit review
milestone: M0
status: todo
crates: [xtask]
depends_on: [M0-T001, M0-T002, M0-T003, M0-T004, M0-T005, M0-T006, M0-T007, M0-T008, M0-T009]
requirements: []
---

# M0-T010: M0 exit review

## Goal

An independent session confirms that every M0 exit criterion holds and that `xtask` meets the `usable`
gates. After this, M1 can start on a trustworthy foundation.

## Context

`docs/ROADMAP.md` §M0 exit criteria. `docs/process/MATURITY.md` (`usable` gates).
`docs/process/REVIEW.md` §3. MUST be done by a session that implemented none of M0-T002 … M0-T009,
or if that is impossible, one that starts from a fresh context and states this in the review.

## Scope

**In:** verify each M0 exit criterion with evidence; seed one violation per check on a scratch branch
and confirm each is caught (then discard the branch); promote `xtask` to `usable` if every gate holds;
update the ROADMAP checkboxes.

**Out:** writing M1 tasks (M1-T001).

## Acceptance criteria

- [ ] Every M0 exit criterion in `docs/ROADMAP.md` checked off, with evidence links in this file's
      Review section.
- [ ] Seeded-violation run: fmt, clippy, test failure, doc warning, layer violation, unapproved
      dependency, missing `forbid(unsafe_code)`, broken doc link, invalid frontmatter, untested MUST in
      a `usable` spec, untracked TODO, coverage drop, stale STATUS.md, GPL dependency. Each caught by
      the expected step. Results in a table in the journal.
- [ ] `docs/specs/xtask.md` reviewed as a spec (REVIEW.md §2), then its maturity set to `usable` in
      one promotion (allowed by MATURITY.md when all cumulative gates hold; tool-crate gates apply).
      If any gate fails, list the failing gates and turn them into tasks.
- [ ] XTASK-R013 (SHOULD): measure the native checks' total time on the reference machine and record
      it in the journal. This is not a test assertion (TESTING.md forbids wall-clock assertions).

## Handoff

_Empty._

## Review

_Empty._
