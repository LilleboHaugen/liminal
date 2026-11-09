---
id: ADR-0023
title: Own benchmark harness, instruction-count regression gate, competitive crates outside the workspace
status: proposed
date: 2026-11-09
supersedes: []
superseded_by: []
---

# ADR-0023: Own benchmark harness, instruction-count regression gate, competitive crates outside the workspace

## Context

Resolves OQ-004 (`docs/OPEN_QUESTIONS.md`). M1 needs benchmarks for three reasons: budgets in specs
are set from measured baselines (`docs/process/PERFORMANCE.md`), optimized code must show it is
faster than the safe reference before `unsafe` is kept (`docs/process/CODE_STYLE.md`), and the M1
exit criteria require competitive results for ECS, math and jobs against `bevy_ecs`, `hecs`, `glam`
and `rayon`.

Facts and constraints:
- Wall-clock timing on GitHub-hosted runners is noisy (shared VMs, unknown neighbours, CPU model
  varies between runs). `PERFORMANCE.md` already restricts CI to instruction-count gates.
- Valgrind does not run on macOS arm64, so the reference machine (Apple M5 Pro) cannot measure
  instruction counts with Valgrind. It runs on the `ubuntu-latest` x86_64 runners.
- Layer rules (XTASK-R032) forbid a layer 0–4 crate from depending on a layer 5 crate, also as a
  dev-dependency. A shared benchmark harness therefore cannot be a dev-dependency of the engine
  crates unless it sits in layer 0.
- `bevy_ecs` alone brings several dozen transitive crates. If it were a workspace member's
  dependency, every `cargo xtask ci` run (clippy, test, doc, coverage) would compile it, and
  `Cargo.lock`, `cargo deny` and the iteration-latency budget (`cargo xtask ci` wall time,
  PERFORMANCE.md) would all carry it.
- What a harness has to do is small: warm up, choose an iteration count, take samples with
  `std::time::Instant`, keep results alive with `std::hint::black_box`, compute median and spread,
  and write JSON that `xtask` reads.

## Decision

1. **One benchmark crate.** All engine benchmarks live in one workspace crate `liminal_benches` at
   `benches/` (layer 5, `kind = "lib"`, `publish = false`, `unsafe = "forbid"`). Its library holds
   the harness. Its `[[bench]]` targets (`harness = false`) benchmark the engine crates through
   their public API. Engine crates MUST NOT have their own `benches/` directories.
2. **Own harness.** The harness uses only `std`. It MUST: run a warm-up phase; calibrate iterations
   per sample so a sample takes at least 10 ms on the measuring machine; take a configurable number
   of samples (default 30); report median, median absolute deviation (MAD), minimum and maximum
   time per iteration, and throughput where the case declares an element count; and write one JSON
   file per bench target (`version: 1`) with machine ID, OS, toolchain, commit and power state
   (PERFORMANCE.md "Machines"). Case IDs are stable strings `<crate>/<group>/<case>`. Its statistics
   functions MUST have unit tests with hand-computed values.
3. **Wall-clock baselines.** Wall-clock baselines are recorded on the reference machine only, at
   `ci/perf/<machine-id>/<crate>.json`. A new `xtask` command (`cargo xtask bench`, specified in
   `docs/specs/xtask.md` by the task that adds it) runs the benchmarks and compares them with the
   baseline for the current machine. A case is a regression when its median is slower than the
   baseline median by more than `max(5 %, 3 × baseline MAD / baseline median)`. This command is
   **not** a step of `cargo xtask ci`. It is run by optimization work (PERFORMANCE.md "Rules") and
   at `stable` and `optimized` promotions.
4. **CI regression gate: instruction counts.** A Linux job in `.github/workflows/ci.yml` installs
   Valgrind on the runner (not on the owner's machine) and measures each single-threaded case with
   `valgrind --tool=cachegrind --cache-sim=no`. To remove set-up cost, each case runs once with `k`
   iterations and once with `2k`, and the per-iteration count is `(I(2k) − I(k)) / k`. Counts are
   compared with `ci/perf/ci-linux-x86_64/instructions.json`. An increase of more than 1 % for a
   case fails the job. Multithreaded cases (for example `liminal_jobs` scaling) are excluded from
   this gate and are marked so in the case definition. The baseline file is produced by the CI job
   (uploaded as an artifact) and committed. Toolchain updates and intended changes regenerate it
   with a journal entry (AGENTS.md rule 5).
5. **Competitive crates outside the workspace.** Comparison benchmarks live in a separate Cargo
   workspace at `benches/competitive/` (crate `liminal_benches_competitive`, its own `Cargo.lock`,
   path dependencies on the engine crates and on `liminal_benches` for the harness). It is listed in
   the root `Cargo.toml` `[workspace] exclude`. This ADR approves `bevy_ecs`, `hecs`, `glam` and
   `rayon` in scope `dev` for that crate only. Each is registered in `architecture.toml`. The task
   that adds them pins exact versions, runs `cargo deny` on the separate lock file and extends
   `check-arch` (or records why it cannot) so the competitive crate is checked like a workspace
   member. Each competitive workload is defined once in prose (what is created, how many, what is
   measured) and implemented for each engine in the same bench target. Results go to
   `ci/perf/competitive/<crate>-<date>.json` (PERFORMANCE.md).

## Alternatives considered

- **`criterion`.** The established Rust harness, with good statistics and HTML reports. Rejected:
  it pulls a large dependency tree (plotting, `rayon`, `serde`, `regex`, `clap`) into the
  workspace for features we do not use, and its JSON is not the format `xtask` reads. It can be
  reconsidered if the own harness's statistics prove insufficient.
- **`divan`.** Smaller than `criterion`, attribute-based. Rejected for the same reason in a smaller
  form: an external crate for about 300 lines of `std` code, against ADR-0003's default.
- **`iai-callgrind` (now `gungraun`) for instruction counts.** Uses Callgrind client requests to
  exclude set-up exactly. Rejected for now: it needs a matching runner binary installed with
  `cargo install` and a library dependency. The `k`/`2k` difference gives the same per-iteration
  count without either. Revisit if the difference method proves noisy (for example allocator
  behaviour that differs between `k` and `2k`).
- **Wall-clock gate in CI.** Rejected: noise on shared runners exceeds the regressions we want to
  catch.
- **Harness in layer 0 with benches in each crate.** Allows benchmarking private items. Rejected:
  an extra layer-0 crate for test tooling, and private items can be benchmarked through their
  public callers. Benchmarks that need internals are a sign the internal belongs behind a public
  API.
- **Comparison crates as dev-dependencies inside the workspace.** Rejected: their compile time
  lands on every `cargo xtask ci` run, and they enter the engine's `Cargo.lock` and `deny` scope.
- **Do nothing until `stable`.** Rejected: M1 exit criteria need competitive numbers, and budgets
  must come from measurements, not guesses.

## Consequences

- No new external crate in the main workspace for benchmarking. The harness is ours to maintain,
  and its statistics are only as good as its tests.
- Instruction counts are a proxy: they miss cache and memory-bandwidth effects and say nothing
  about multithreaded scaling. Wall-clock numbers on the reference machine remain the measure for
  budgets.
- Benchmarks see only public API.
- The competitive workspace needs its own maintenance (lock file updates, `deny`). It is never
  built by `cargo xtask ci`, so it can rot unnoticed; `cargo xtask bench --competitive` and the
  M-exit reviews are where it is exercised.
- Revisit when: the instruction-count gate produces false failures more than once per milestone,
  or a crate needs benchmarks that the public API cannot express.

## Verification

- `check-arch`: engine crates contain no `benches/` targets (added to `check-arch` by the
  `liminal_benches` task) and depend on no comparison crate.
- Unit tests for the harness statistics; integration test of the JSON output format.
- The `bench-instructions` CI job fails on a seeded 2 % instruction increase (shown once by the
  task that adds the job).
- Review checks that budgets cite a baseline file.
