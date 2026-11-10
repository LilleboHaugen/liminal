# Backlog

Minor review findings that are not worth a task of their own (ADR-0029 Decision 2). A reviewer
appends its non-blocking, non-contract-changing findings here. Findings that change a contract a
pending task implements go into a follow-up task instead (`docs/process/REVIEW.md`).

Rules:
- One heading per review: `## <task ID> — review YYYY-MM-DD`. One list item per finding: what is
  wrong and the file it concerns, in one to three lines.
- An implementation task MAY fix entries in files it changes anyway. It deletes the entry and
  says so in its handoff.
- Milestone kickoff and exit reviews work through the file: each entry becomes a task or is
  deleted, with a one-line reason in that session's journal entry.
- On merge, keep both sides' entries.

Follow-up tasks written before 2026-11-10 (M1-T055 … M1-T058, M2-T001 … M2-T007) stay tasks.

## M1-T013 — review 2026-11-10

- `crates/liminal_core/tests/handles.rs`: no test fails when `HandleMap::new()` uses a slot
  limit other than `u32::MAX` (mutant `u32::MAX - 1` survives). Compare its `Debug` form with
  `with_slot_limit(u32::MAX)`, as the allocator test does.
- `crates/liminal_core/src/error/code.rs`: the `compile_fail` doc test for CORE-R003 passes on
  any compile error, not only the const panic. No stable way to match the message; say so in
  the spec's test plan or add a UI test when a harness for one exists.
- `crates/liminal_core/tests/error_model.proptest-regressions`, `handles.proptest-regressions`:
  four committed seeds whose causes `docs/journal/2026-11-10-M1-T013.md` does not explain (it
  says no regression file was kept). Record what each seed found, or delete seeds that came
  from test bugs.
- `docs/specs/liminal_core.md` §7 INV-1 says `debug_assert!` in `allocate` and `free`;
  `crates/liminal_core/src/handle/allocator.rs` asserts the slot state only in `allocate`
  (`free` has the INV-2 count check). Add the assertion or correct the spec sentence.
- `docs/specs/liminal_core.md` CORE-R012 "the first repeated key" has two readings for keys
  `a, b, b, a`. Code and tests use "the first field whose key occurred earlier" (`b`). State
  that reading in the requirement.

## M1-T055 — review 2026-11-10

- `docs/specs/liminal_core.md` §5.1, §5.6: `ErrorContext`, `Tick` and `TickDuration` derive
  `Default`, and no requirement states the value (empty context; count 0). The derived values
  are the only reasonable ones and `ErrorContext::default()` is already tested as empty; add
  the statement to CORE-R007 / CORE-R111 when those rows are next edited, or leave it.
- `docs/process/DOCUMENTATION.md`, `docs/specs/_TEMPLATE.md`: the spec frontmatter field
  `last_reviewed` has no stated meaning (date of the last spec review, or of the last change?).
  `docs/specs/liminal_core.md` v2 keeps 2026-11-09, the review date of v1. Define the field.
