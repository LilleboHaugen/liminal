---
id: M1-T034
title: Write the spec for liminal_jobs
milestone: M1
status: done
crates: [liminal_jobs]
depends_on: [M1-T027]
requirements: []
---

# M1-T034: Write the spec for `liminal_jobs`

## Goal

`docs/specs/liminal_jobs.md` is a complete, reviewable contract for `liminal_jobs`, so the implementation tasks can be done without guessing.

## Context

- `docs/ARCHITECTURE.md` §2 (module map: work-stealing job system, job graphs, parallel-for, scoped tasks).
- `docs/specs/_TEMPLATE.md`; `docs/specs/xtask.md` as the quality bar.
- `architecture.toml` entry for `liminal_jobs` (layer, `unsafe`, `deterministic`, allowed deps).
- ADR-0009 item 4; ADR-0018; `docs/specs/liminal_telemetry.md`.
- PERFORMANCE.md "Reference implementations reachable from benchmarks" (M1-T054): the spec's API section lists the crate's `reference` items for each optimized path.
- Written by the M1 kickoff (M1-T001, 2026-11-09).

## Scope

**In:** Spec for: thread pool (thread count, start and shutdown), work-stealing deques and the simple locked reference they are checked against, scoped tasks and `join`, parallel-for with chunking, job graphs with dependencies, panic behaviour, how results exposed to simulation stay deterministic (ADR-0009 item 4), telemetry spans emitted.

**Out:** Code. The implementation tasks (M1-T035, M1-T036) write it.

## Acceptance criteria

- [x] `docs/specs/liminal_jobs.md` exists, follows `docs/specs/_TEMPLATE.md` (all sections), with
      `maturity: planned`, `spec_version: 1` and `adrs` listing every ADR it implements.
- [x] Every requirement has an ID `JOBS-Rnnn`, an RFC 2119 level and a testable statement with
      units, ranges and failure behaviour. No vague words ("fast", "robust", "efficient").
- [x] §5 Public API covers the scope below and is consistent with `docs/ARCHITECTURE.md` and the
      specs of the crates it depends on.
- [x] Error codes `LIM-JOBS-NNNN` are listed with message and `help` (ADR-0017), or the spec says
      why the crate has no recoverable errors.
- [x] The test plan names the test kinds that apply (TESTING.md) and, for every optimized path, the
      reference implementation it is checked against (AGENTS.md rule 4).
- [x] Performance budgets name scenarios and machines; numbers that need a measurement say which
      task measures them (PERFORMANCE.md: budgets are never invented).
- [x] Every MUST requirement is assigned to exactly one implementation task (M1-T035, M1-T036): the spec
      lists the assignment, and this session writes the IDs into those tasks' `requirements`
      frontmatter (`check-docs` R028 validates them).
- [x] Open questions are listed in the spec instead of being decided silently.
- [x] `docs/specs/README.md` lists the spec (XTASK-R027); `cargo xtask ci` passes.

## Test requirements

None (documentation task). The review is a spec review (REVIEW.md §2).

## Notes

Requirement IDs leave gaps between groups (template §6). Keep the API small: every public item has to be documented, tested and later snapshotted.

## Handoff

**Done (2026-11-10, sub-agent session).**
- `docs/specs/liminal_jobs.md`, `spec_version` 1, `maturity: planned`, prefix `JOBS`: 50 MUST +
  2 SHOULD requirements (JOBS-R001 … JOBS-R141), five error codes (LIM-JOBS-0001 to 0003, 0010,
  0011), 12 open questions, §9 scenarios without numbers (M1-T036 measures, M1-T051 compares
  with `rayon`).
- `requirements` frontmatter and Notes of M1-T035 (27 IDs) and M1-T036 (23 IDs). Every MUST is
  in exactly one task (checked by script against the spec's requirement tables).
- `docs/specs/README.md` index row; `docs/STATUS.md` regenerated.

**Design decisions to check in review** (no ADR: no dependency, layer or unsafe-policy change).
- No global pool. `ThreadPool` is a value; `threads` counts the caller plus `threads − 1`
  workers; a waiting caller runs queued jobs ("helping", spec §4).
- `threads == 1` is the *sequential mode*: no thread, every closure on the caller in a specified
  order. It is the reference executor for all scheduling (spec §10).
- No detached (`'static`) jobs: every job ends before its pool call returns (INV-1). Asset
  loading will need more (open question 1).
- The deque is public and fixed-capacity, and stores `&'a T` in one atomic pointer per slot, so
  a lost steal race is not a data race (INV-6). A full deque makes the worker run the job
  inline. Its oracle is the public module `liminal_jobs::reference` (PERFORMANCE.md rule from
  M1-T054), with a benchmark pair for the ADR-0018 clause.
- Chunk sizes are always given by the caller (`NonZeroUsize`); boundaries never depend on the
  thread count. `parallel_map_fold` folds per-chunk results on the caller in chunk order, so
  results are bit-identical to the sequential fold (ADR-0009 item 4).
- Job graphs reject a cycle-closing edge in `add_dependency`, so `run_graph` has no error
  result; graphs are reusable (`FnMut` nodes, `&mut JobGraph`).
- Panics: no cancellation (except graph dependents); the resumed payload is fixed for `join`,
  loop functions and graphs.
- Spans: one per `scope`, loop call and `run_graph`, plus one per graph node named by the node;
  none for `join`, spawned jobs or chunks (buffer size).

**Scope changes written into other tasks (with reasons, in their Notes).** The panic rules of
`scope` and `join` are assigned to M1-T035 although M1-T036's text lists "panic propagation";
M1-T036 adds the `jobs.scope` span to M1-T035's `scope`.

**Not verified.** Nothing in the spec is implemented, so: that the signatures of §5 compile as
written (`scope` follows the shape of `rayon`'s; assumption), that JOBS-R064 (no allocation
after warm-up) is achievable with `std` parking on all three platforms, and every performance
expectation. The default 2 MiB worker stack size is unverified on Windows (open question 5).

**Next step:** a different session reviews M1-T034 (spec review, REVIEW.md §2).

## Review

2026-11-10, review sub-agent (ADR-0026, ADR-0029) of parent session
`session/2026-11-10-m1-reviews` (journal `2026-11-10-session-m1-reviews.md`), fresh context,
prompt: the ADR-0026 review template with worktree and task ID filled in. The author was an
implementation sub-agent of the same parent; ADR-0029 Decision 1.2 (on `main` since `c75a54e`,
merged into this branch before the review commit) allows that. This agent did no implementation
work on M1-T034. macOS 27.2 (Darwin, arm64), worktree `liminal-worktrees/M1-T034-review`,
reviewed commit `1a06db9`. Task review (REVIEW.md §1) plus spec review (REVIEW.md §2).

Outcome: **done**, no blocking finding (B1–B5). No contract-changing follow-up, so no follow-up
task. Backlog: 9 entries in `docs/BACKLOG.md` under "M1-T034 — review 2026-11-10".

Checked:
- AC 1: 14 template sections in order; `maturity: planned`, `spec_version: 1`,
  `adrs: [ADR-0009, ADR-0010, ADR-0016, ADR-0017, ADR-0018, ADR-0023]` (ADR-0024: backlog).
- AC 2: 52 requirement rows, IDs unique, 50 MUST and 2 SHOULD, no reference to an undefined ID
  (script). A word search for "fast", "robust", "efficient" and similar finds only "faster"
  (§3, a condition on a measurement) and "not the fast path" (§5.2). JOBS-R011 is checked in
  review only (backlog).
- AC 3: §5 covers every item of the task scope. Consistent with `docs/ARCHITECTURE.md` §2,
  `architecture.toml` (layer 1, `same_layer_deps = []`, `external = []`, dev `proptest` and
  `loom`, `unsafe = "allow"`, `deterministic = false`), `docs/specs/liminal_core.md`
  `spec_version` 3 (`EngineError`, `write_display`, `validate_error`, `ContextValue::UInt` and
  `Text`, context keys match `^[a-z][a-z0-9_]*$`, `help()` returns `Option<String>` so help
  text may contain context values, `hash::StateHasher`, `rng`) and
  `docs/specs/liminal_telemetry.md` `spec_version` 1 (`span!` takes any `&'static str`
  expression, so node names work; inactive macros do not allocate, so JOBS-R064 holds with the
  `telemetry` feature on and no `Collector`; `SpanGuard` is not `Send` and JOBS-R120 keeps each
  span on one thread). None of the ten items of M1-T056 (`todo`) changes what this spec uses.
- AC 3, signatures: a scratch `rustc` program (not committed) with the items of §5.1 to §5.4 as
  written and stub bodies compiles. In it: the `split_at_mut` example of JOBS-R042 and the
  nested spawn of JOBS-R041 compile and run; a job that borrows a local of `f` is rejected
  (E0373) when `Scope<'scope>` is invariant in `'scope`; the three loop functions written on
  `join` in safe code give the chunks of §4, including `usize::MAX − 5 .. usize::MAX` with
  chunk sizes 4 and `usize::MAX`; `run_graph` written on `scope` in safe code runs `FnMut`
  nodes that borrow locals three times. This shows that the signatures type-check and that
  `parallel` and `graph` can be written without `unsafe` (§3). It shows nothing about the
  real pool.
- AC 4: 5 codes with variant, message, help and context; groups and free numbers stated.
- AC 5: test kinds per area use the names of TESTING.md; the three optimized paths name their
  reference (`liminal_jobs::reference`, sequential mode, the sequential fold). PERFORMANCE.md
  "Reference implementations reachable from benchmarks" items 1 to 4 hold for `reference`.
- AC 6: §9 has scenarios on the reference machine (PERFORMANCE.md: Apple M5 Pro, 6P + 12E), no
  numbers, measuring tasks M1-T036 and M1-T051 (both exist); the `deque` / `reference` pair
  answers the ADR-0018 benchmark clause; ADR-0023 Decision 4 excludes multithreaded cases
  from the instruction gate as the spec says.
- AC 7: the 50 MUST IDs equal the union of the `requirements` frontmatter of M1-T035 (27) and
  M1-T036 (23), no ID twice (script); the §6 table expands to the same sets. The scope changes
  are in both tasks' "Notes".
- AC 8: 12 open questions. Four name "the `stable` promotion" as resolver (backlog).
- AC 9: `docs/specs/README.md` row; `cargo xtask ci` on `1a06db9`: pass, 11 of 11 steps, none
  skipped; branch CI run 42864759822 on `1a06db9` green. `cargo xtask ci` on the review
  commit's tree (after the merge of `main`): pass, 11 of 11 steps, none skipped.
- By derivation: the sums 2,147,450,880 (JOBS-R007, R044) and 11,000 (JOBS-R041); the order
  2, 1, 3, 0 of JOBS-R085; the quoted clauses of ADR-0009 item 4 and ADR-0018; `steal` reads
  the slot before its compare-exchange on `top`, and a slot is overwritten only after `top`
  has moved past it, so INV-6 and the order clause of JOBS-R023 hold for a fixed ring with
  position counters that do not wrap in practice; JOBS-R103 follows from JOBS-R102 when the
  left half of a split is `a`; `'scope` outlives the `scope` call, so INV-1 is enough for the
  borrows of JOBS-R042.
- No fake progress, oracle changes: none (documentation only). Docs: STATUS.md regenerated.
- Session boundaries: the journal entry names its parent; that parent's journal lists M1-T034
  among the tasks its implementation sub-agents wrote, but its sub-agent table is not written
  yet ("in progress" on the session branch), so the listing is not verifiable here. The task
  file and the journals show no earlier review of M1-T034 and no install or spawn by the
  author. The owner's instruction behind ADR-0029 is known to this review only from the ADR
  text on `main` and the parent's journal.
- ADR-0028: no workflow change in the task's commit; no credentials or personal data.

Not verified: anything about code that does not exist yet, in particular that JOBS-R064 can be
met on Linux and Windows, and every performance expectation.
