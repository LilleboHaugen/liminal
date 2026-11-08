---
id: ADR-0021
title: Review findings — blocking criteria, follow-up tasks and a round limit
status: accepted
date: 2026-11-08
supersedes: []
superseded_by: []
---

# ADR-0021: Review findings — blocking criteria, follow-up tasks and a round limit

## Context

`docs/process/REVIEW.md` allows two outcomes: `done`, or `in-progress` with required changes. It
does not say which findings justify "changes required". ADR-0020 means a session can never review
its own fix, so every round of changes needs another owner-started review session.

M0-T003 (frontmatter parser, `check-docs`) went through four reviews on 2026-11-08, each with
changes required:

| Review | Required changes | Kind |
|---|---|---|
| 1 | 4 | Values YAML misreads (`title: - x`), untested branches |
| 2 | 3 | YAML indicators `,` `]` `{` `}` |
| 3 | 1 | Non-ASCII whitespace, control characters |
| 4 | 2 | U+FEFF inside a value; tab indentation untested |

The findings shrank each round. Review 4's U+FEFF finding was correct by the spec, but it was not
a real problem: no author types U+FEFF into a title, and libyaml reads it the same way the parser
does. A strict reviewer can always find one more edge case in a parser, so with no bar for what
blocks, review rounds have no natural end. The project owner called this "getting stuck in loops"
and asked for a rule change (approval below).

## Decision

1. **Every review finding is either blocking or a follow-up.** A finding is **blocking** only if
   it is one of:
   - **B1.** An acceptance criterion of the task is not met, or `cargo xtask ci` fails.
   - **B2.** A non-negotiable rule in `AGENTS.md` is broken (fake progress, an oracle changed
     without justification, an architecture or dependency violation, `unsafe` without
     `// SAFETY:`, a covered requirement without a `Verifies:` test, …).
   - **B3.** Wrong behaviour on **realistic input**: a file now in the repository, or input that
     an author or agent working on this project could plausibly write. The reviewer MUST give the
     concrete input and the wrong result. Input that only exists to probe the edges of a
     standard (invisible code points, grammar corners nobody writes) is not realistic.
   - **B4.** A behaviour the task or its handoff claims is untested, so that breaking it changes
     an outcome (accepted or rejected, a computed value, a reported finding) and no test fails.
     A difference in message wording only is a follow-up.
   - **B5.** Code and spec disagree on behaviour.

   Everything else is a **follow-up**: theoretical edge cases, extra hardening, better messages,
   style, refactors, test improvements beyond B4.
2. **Follow-ups never block `done`.** The reviewer writes all follow-ups of a review into **one**
   new task file (`status: todo`, next free ID; milestone: the current one if a milestone exit
   criterion needs it, otherwise the reviewer picks a later one) and lists its ID in the review
   note. With no blocking findings, the reviewer sets the task to `done` in the same review.
   Follow-up tasks are not added to a milestone exit task's `depends_on` unless an exit criterion
   needs them.
3. **A re-review is scoped.** A review of a task that already had a review checks: each blocking
   finding of the previous review is fixed; the diff since the previous review; `cargo xtask ci`.
   It does not re-audit unchanged code. A finding in unchanged code blocks only under B1, B2 or B3;
   otherwise it is a follow-up.
4. **At most two "changes required" outcomes per task.** From the third review of a task, the
   reviewer MUST NOT set it back to `in-progress`. It sets the task to `done` (with a follow-up
   task if needed), or, if a blocking finding remains, to `blocked`. The review note then states
   the finding, why it is blocking (B1–B5) and the options. `docs/STATUS.md` lists it under
   Blockers. The project owner decides: accept as `done` with a follow-up task, or allow one
   more round of changes, recorded in the task file.
5. **Transition.** Reviews done before this ADR count. M0-T003 has had four "changes required"
   outcomes, so its next review falls under item 4.

## Alternatives considered

- **Do nothing.** Rejected: M0-T003 shows the loop. Each round costs the owner a session.
- **Only a round limit, no criteria.** Rejected: it stops the loop, but each review would still
  spend its rounds on edge cases, and the owner would decide without a stated bar.
- **Only criteria, no round limit.** Rejected: B3 and B4 still need judgement, and a reviewer
  can argue a borderline finding into them. The limit gives a guaranteed end and puts the
  judgement with the owner.
- **Let the reviewing session fix small findings and set the task `done` itself.** Rejected: the
  reviewer would then approve its own code, which ADR-0020 forbids.
- **Make all findings follow-ups ("approve and fix later").** Rejected: real defects would reach
  `main`, against the "every claim is checkable" rule in `AGENTS.md`.

## Consequences

- Reviews end. A task needs at most three reviews before it is `done` or goes to the owner.
- Minor findings are not lost: they become `todo` tasks with acceptance criteria. The backlog
  grows. Kickoff and exit reviews must look at follow-up tasks, not only the planned ones.
- B3 depends on what counts as "realistic", which is a judgement. The reviewer must show the
  input, so the judgement is visible. The round limit caps the cost of judging wrong.
- Revisit if follow-up tasks pile up unworked across milestones, or if defects found later turn out
  to be ones a review classed as follow-ups.
- Owner approval: requested by the project owner on 2026-11-08 ("I need a way to not get stuck in
  loops all the time. I think we need to change this somehow"). This ADR changes rules in
  `AGENTS.md` ("Session protocol", "Definition of done").

## Verification

- Review notes name the class (B1–B5) of each blocking finding and the follow-up task ID. A
  "changes required" without a class, or a third "changes required" on one task, is a violation
  that the next reviewer reports to the owner.
- `cargo xtask check-docs` checks that a follow-up task file is valid and its `depends_on` exist.
