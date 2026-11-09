# Review

Reviews are done by a session that did **not** write the work being reviewed, starting from a fresh
context. That session is started by the project owner, never by the author's session or any other
agent (ADR-0020). The reviewer's job is to find what is wrong, not to confirm what was claimed. A review that
finds nothing should say what it checked.

## 1. Task review (status `review` → `done`)

Check, in order:
1. **Acceptance criteria:** each one is met. Verify by running or reading, not by trusting the checkbox.
2. **`cargo xtask ci` passes** on your machine. Note your OS.
3. **Tests prove behaviour:**
   - Do the tests fail if the implementation is broken? Mentally mutate one line. Would a test catch it?
   - Are there assertion-free tests, tests that only check "no panic", or tests that mirror the
     implementation instead of the spec?
   - Do `Verifies:` tags point at requirements the test actually verifies?
4. **No fake progress:** no placeholders, no silently shrunk scope, no weakened or deleted tests,
   no oracle (golden, baseline, floor) changed without a justified journal entry.
5. **Spec agreement:** code and spec say the same thing. Public API matches the spec's API section.
6. **Unsafe:** every `SAFETY:` comment is correct and sufficient. Miri was run.
7. **Determinism:** no banned functions or collections in deterministic crates. New simulation state
   is included in the state hash.
8. **Docs:** public items documented; examples run; no stale docs elsewhere.
9. **Architecture:** no new dependency without ADR; layer rules hold.
10. **Session boundaries (ADR-0020):** the journal entries for this task show that the work was done in
    owner-started sessions, with no sub-agent implementing or reviewing it.

### Findings and outcome (ADR-0021)

Classify every finding. It is **blocking** only if it is one of:
- **B1.** An acceptance criterion is not met, or `cargo xtask ci` fails.
- **B2.** A non-negotiable rule in `AGENTS.md` is broken.
- **B3.** Wrong behaviour on realistic input: a file now in the repository, or input an author or
  agent on this project could plausibly write. Give the concrete input and the wrong result.
  Input that only probes the edges of a standard is not realistic.
- **B4.** A behaviour the task or handoff claims is untested: breaking it changes an outcome
  (accepted/rejected, a value, a finding) and no test fails. Message wording alone is not B4.
- **B5.** Code and spec disagree on behaviour.

Everything else is a **follow-up**. Write all follow-ups of the review into one new task file
(`status: todo`, next free ID, current milestone only if an exit criterion needs it) and name it
in the review note. Follow-ups never block `done`.

**Re-reviews** (the task had a review before) check only: each previous blocking finding is fixed,
the diff since the previous review, and `cargo xtask ci`. A finding in unchanged code blocks only
under B1, B2 or B3.

Outcome, recorded in a "Review" note in the task (reviewer, date, what was checked, follow-up task):
- No blocking findings: set `status: done`.
- Blocking findings, and the task has had fewer than two "changes required" outcomes: set
  `status: in-progress` with a numbered list of required changes, each with its class (B1–B5).
- Blocking findings, and the task already had two "changes required" outcomes: set
  `status: blocked`, state each finding, its class and the options, and add it under "Blockers
  needing an owner decision" in `docs/STATUS_NOTES.md` (the generated `docs/STATUS.md` lists
  every `blocked` task by itself). The project owner decides (accept as `done` with a follow-up task, or allow
  one more round), and the decision is recorded in the task file.

## 2. Spec review (before a crate becomes `experimental`)

- Purpose and scope are clear. Non-goals are listed.
- Every requirement is testable. Vague words ("fast", "robust", "efficient") are replaced by numbers or
  removed.
- Requirements have IDs, RFC 2119 keywords, and no duplicates.
- The API sketch is consistent with ARCHITECTURE.md and with the specs of crates it depends on.
- Error codes are listed. Invariants are stated so they can be asserted.
- Test plan names test kinds and the oracles for optimized paths.
- Open questions are listed rather than hidden.

## 3. Maturity promotion review

Verify every gate in `docs/process/MATURITY.md` for the target level against the evidence in the spec's
Maturity checklist. Re-run coverage and trace yourself. Promotion is all or nothing.

## 4. ADR review (`proposed` → `accepted`)

- The context states the problem and its constraints. Alternatives were really considered,
  including building it ourselves.
- Consequences include costs and risks, not only benefits.
- It does not silently contradict an accepted ADR. If it changes one, it supersedes it explicitly.
- Architecture-changing ADRs (new layer rules, new runtime dependencies, rule changes in AGENTS.md)
  also need the project owner's approval, recorded in the ADR.
