# Performance

Liminal claims to be fast only when there are numbers behind the claim. This document defines how
performance is measured, recorded and defended.

## Machines

| Role | Machine | Notes |
|---|---|---|
| Reference | Apple M5 Pro (6P + 12E cores), 20-core GPU, 24 GB, macOS 27.2, Metal 4 | Budgets are defined here |
| Low-end target | Steam Deck (AMD Van Gogh APU, 16 GB shared, 1280×800) | Budgets for demo games, when hardware is available |
| CI | GitHub-hosted runners | Shared and noisy. Used only for instruction-count gates (OQ-004) |

Each baseline file records the machine ID, OS version, toolchain version, commit and power state
(plugged in, no thermal throttling).

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

Comparison crates are dev-dependencies of a dedicated benchmark crate only, never of engine crates.
Each needs an ADR (OQ-004). Results are written to `ci/perf/competitive/<crate>-<date>.json` and
summarized in the spec.

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
2. Keep or write the simple reference implementation. Differential test before and after.
3. Benchmark before and after on the reference machine. Commit both numbers in the commit message or
   perf report.
4. Do not trade correctness or determinism for speed. A faster wrong answer is a regression.
5. Keep the result: update the baseline in `ci/perf/`, and the spec budget if it tightened.

## Telemetry in shipped games

Telemetry spans compile to nothing when the `telemetry` feature is off. With it on, the overhead
budget per span is defined in the `liminal_telemetry` spec and measured in its benchmarks.
