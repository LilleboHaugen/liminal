# Performance

Liminal claims to be fast only when there are numbers behind the claim. This document defines how
performance is measured, recorded and defended.

## Machines

| Role | Machine | Notes |
|---|---|---|
| Reference | Apple M5 Pro (6P + 12E cores), 20-core GPU, 24 GB, macOS 27.2, Metal 4 | Budgets are defined here |
| Low-end target | Steam Deck (AMD Van Gogh APU, 16 GB shared, 1280×800) | Budgets for demo games, when hardware is available |
| CI | GitHub-hosted runners | Shared and noisy. Used only for instruction-count gates (ADR-0023) |

Each baseline file records the machine ID, OS version, toolchain version, commit and power state
(plugged in, no thermal throttling).

## Harness and baselines (ADR-0023)

- Engine benchmarks live in the `liminal_benches` crate (`benches/`), on Liminal's own `std`-only
  harness. Engine crates have no `benches/` directories.
- Wall-clock baselines: reference machine only, `ci/perf/<machine-id>/<crate>.json`, compared by
  `cargo xtask bench` (not part of `cargo xtask ci`).
- CI gate: instruction counts of single-threaded cases under Valgrind on Linux,
  `ci/perf/ci-linux-x86_64/instructions.json`; more than 1 % increase fails.

## Reference implementations reachable from benchmarks

Benchmarks see only public API (ADR-0023), yet an optimized path must show it beats its simple
reference (AGENTS.md rule 4, "Rules for optimization work" below). A crate makes the reference
reachable as follows (rule set by M1-T054):

1. **A public `reference` module.** A reference implementation that `liminal_benches` (or a test
   outside the crate) needs is a `pub` item in a module named `reference` at the crate root, for
   example `liminal_math::reference::mat4_mul`. It has the same name and signature as its
   optimized counterpart, so the differential test and the benchmark compare `crate::x` with
   `crate::reference::x`.
2. **Always compiled, never hidden.** The module has no cargo feature gate and no
   `#[doc(hidden)]`. Its items are documented like every public item; their docs name the
   optimized counterpart and state that the item is the correctness oracle, not the fast path.
3. **Only what is needed.** An item enters `reference` only when an optimized counterpart exists
   or is specified. A reference used only by the crate's own unit tests stays private
   (`pub(crate)` or `#[cfg(test)]`) until a benchmark or an outside test needs it.
4. **Spec and rules.** The crate's spec lists the `reference` items in its API section and names
   the differential tests that compare them. Reference items follow the crate's other rules
   (determinism per ADR-0009, `det` functions in simulation crates). A reference SHOULD contain no
   `unsafe`; where the operation is itself unsafe (for example raw allocation), the spec says why.
5. **Several kept paths.** When a crate keeps more than one optimized path for one operation (for
   example two SIMD widths chosen by `cfg(target_feature)`), the benchmark measures the path the
   public item selects on the measuring machine. Other paths are not made public to be
   benchmarked; the per-target differential tests and the runs on each machine cover them.

Reasons, against the alternatives the M1-T001 review named:

| Option | Outcome and reason |
|---|---|
| Cargo feature (`bench-reference`) | Rejected. Resolver 3 unifies features across the workspace members of one build, so every `--workspace` build in `cargo xtask ci` would compile the feature on (because `liminal_benches` enables it), while games built alone would not: two API surfaces, one missing from an API snapshot taken without the feature, plus one more feature that must build alone (CODE_STYLE.md "Formatting and structure"). |
| `#[doc(hidden)]` public items | Rejected. Rustdoc omits hidden items unless run with `--document-hidden-items`, so the items would be undocumented and (assumption, to be confirmed by the ADR-0025 task) absent from the `cargo public-api` snapshot: changes to them would not show in review. |
| Public `reference` module | Chosen. Visible in docs and in the ADR-0025 snapshot like any other item, one build configuration, no change to ADR-0023. Cost: a few public items per optimized crate, kept small by item 3. |

## Budgets

- Each spec's **Performance budgets** section lists budgets as concrete numbers with units, a scenario
  and a machine. Example: "spawn 1,000,000 entities with 3 components: ≤ 40 ms, reference machine,
  single thread".
- Budgets are set from **measured baselines and competitive benchmarks**, never invented. The first
  measurement sets the starting point. Budgets tighten as the crate matures.
- Frame budgets for demo games: 60 Hz at 1080p on the reference machine, 60 Hz at 1280×800 on the
  low-end target (goal, verified when hardware is available).

## Competitive benchmarks

For each performance-critical crate, the same workload runs on Liminal and on the strongest
open-source equivalent:

| Liminal crate | Compared against |
|---|---|
| `liminal_ecs` | `bevy_ecs`, `hecs` (workloads modelled on ecs_bench_suite: simple insert/iter, fragmented iter, schedule, add/remove) |
| `liminal_math` | `glam` |
| `liminal_jobs` | `rayon` (parallel-for, join trees) |
| `liminal_physics` | `rapier` |
| `liminal_voxel` meshing | Published binary greedy meshing implementations (chosen at M5 kickoff) |
| `liminal_serial` | `serde_json`, `bincode` / `postcard` |

Comparison crates are dev-dependencies of the competitive benchmark crate only, a separate Cargo
workspace at `benches/competitive/`, never of engine crates. ADR-0023 approves `bevy_ecs`, `hecs`,
`glam` and `rayon`; later comparison crates need an ADR each. Results are written to
`ci/perf/competitive/<crate>-<date>.json` and summarized in the spec.

## Iteration-latency budgets

Agents spend much of their time waiting for the compiler. These are tracked from M2 onwards:

| Metric | How measured |
|---|---|
| Clean build of the facade (dev profile) | `cargo build -p liminal_engine` from an empty target directory |
| Incremental rebuild after a one-line change in game code | Template game, change to one system body |
| Incremental rebuild after a one-line change in `liminal_ecs` | — |
| `cargo xtask ci` wall time | — |

Regressions here are treated like runtime regressions. Tools: crate splitting, avoiding heavy
generic instantiation in public APIs, dev-profile tuning, and optional dynamic linking of the engine in
dev builds (evaluated at M2).

## Rules for optimization work

1. Measure first. Profile (`liminal_telemetry` spans, Instruments on macOS, `perf` on Linux) and
   record what dominates.
2. Keep or write the simple reference implementation. Differential test before and after. If a
   benchmark needs it, it lives in the crate's `reference` module (see above).
3. Benchmark before and after on the reference machine. Commit both numbers in the commit message or
   perf report.
4. Do not trade correctness or determinism for speed. A faster wrong answer is a regression.
5. Keep the result: update the baseline in `ci/perf/`, and the spec budget if it tightened.

## Telemetry in shipped games

Telemetry spans compile to nothing when the `telemetry` feature is off. With it on, the overhead
budget per span is defined in the `liminal_telemetry` spec and measured in its benchmarks.
