---
id: M1-T034
title: Write the spec for liminal_jobs
milestone: M1
status: review
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

_Empty._
