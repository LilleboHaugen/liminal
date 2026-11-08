# Maturity levels

Every crate has exactly one maturity level, stored in the `maturity` field of its spec frontmatter.
Levels let the engine grow wide without pretending that anything is finished. Decision record: ADR-0011.

## Levels and gates

A crate is promoted only when **every** gate for the new level is met. Gates are cumulative.

### `planned`
- A row in `architecture.toml`. A purpose line in `docs/ARCHITECTURE.md`.
- No code required.

### `experimental`
- A spec exists using `docs/specs/_TEMPLATE.md` and has been reviewed (spec review, REVIEW.md §2).
- The crate compiles. All lints pass. Every implemented behaviour has tests.
- The crate is registered in `ci/coverage.toml` (floor may start at the current value).
- API may change freely. Only examples and tests may use it, not demo games.

### `usable`
- Every `MUST` requirement is implemented and traced to tests (`cargo xtask trace` gate on).
- Coverage ≥ the `usable` target for its crate type (`docs/process/TESTING.md`).
- Every public item is documented. Primary entry points have runnable examples.
- Integration tests exercise the public API the way a dependent crate would.
- Crates with `unsafe = "allow"`: Miri passes; every `unsafe` block has a `SAFETY:` comment that a
  reviewer has checked.
- Deterministic crates: determinism tests in place.
- At least one example or dependent crate uses it.

### `stable`
- Coverage ≥ the `stable` target. Property tests cover the crate's invariants. Differential tests
  cover every optimized path.
- Public API snapshot committed (tool chosen at M1 kickoff). From now on, API changes are deliberate,
  visible in the snapshot diff, and noted in the journal.
- Benchmarks exist with baselines in `ci/perf/`. Competitive benchmark recorded where a comparable
  open-source crate exists.
- Parsers of external input are fuzzed: ≥ 1 CPU-hour without findings, corpus committed.
- No known open bugs of severity P0/P1.

### `optimized`
- Performance budgets in the spec are met on the reference machine (and on the low-end target where
  the spec defines one).
- Competitive benchmarks show parity or better against the named comparison crates, or the spec
  documents why the comparison does not apply.
- A profiling report in `docs/perf-reports/<crate>-<date>.md` shows where time goes and what was tried.
- Performance regression gates are active in CI.

## Tool crates (layer 5)

For tool crates (`xtask`, `liminal_cli`, `liminal_mcp`, `liminal_cook`), two `usable` gates are
replaced:
- "Primary entry points have runnable examples" → the crate's library target has runnable doc
  examples for its main types, **and** every command is covered by an integration test that runs the
  real binary and checks its exit code and output.
- "At least one example or dependent crate uses it" → the tool is used in CI or by the project's own
  workflow (e.g. `cargo xtask ci` runs in CI).

## Promotion procedure

A crate may move up several levels in one promotion if every cumulative gate up to the target level
holds (including the spec review required for `experimental`).


1. The implementing session fills in the promotion checklist in the spec's **Maturity** section, with
   evidence for each gate (links to tests, coverage numbers, benchmark files).
2. A **different** session reviews the evidence (`docs/process/REVIEW.md` §3) and changes `maturity`
   only if every gate holds. Disagreements go into the spec's checklist as open items.
3. Demotion: if a gate stops holding (e.g. a P0 bug is found in a `stable` crate), lower the level in
   the same change that records the problem. Demotion is normal and not a failure.

## Dependency rule

A crate's maturity limits what may depend on it:
- Demo games and the facade may depend only on crates at `usable` or higher.
- A `stable` crate may not depend on an `experimental` crate.
