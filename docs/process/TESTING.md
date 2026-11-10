# Testing

Testing in Liminal answers one question: **how does a machine know this is correct?** Every kind of test
below is a different kind of oracle. Each spec states which kinds apply to its crate. Decision record:
ADR-0010.

## Test kinds

| Kind | Where | Oracle | Required for |
|---|---|---|---|
| Unit | `#[cfg(test)] mod tests` next to the code | Hand-written expected values | All crates |
| Integration | `crates/<crate>/tests/*.rs` | Public-API behaviour per spec | All library crates |
| Doc tests | `# Examples` in rustdoc | Example compiles and runs | Primary entry points of each crate |
| Property | Unit or integration, using `proptest` | Invariants over generated inputs | Math, ECS, serialization, allocators, meshing, anything with invariants |
| Differential | Unit or integration | Simple reference implementation (public `reference` module when benchmarks need it, PERFORMANCE.md) | Every optimized algorithm (AGENTS.md rule 4) |
| Golden image | Integration, rendering crates | Approved image in `tests/golden/<backend>/` | Every rendering feature |
| Snapshot | Integration | Approved text/JSON output in `tests/snapshots/` | Schemas, protocol messages, CLI output, error formats |
| Determinism / replay | Layer 0–1: the crate's `tests/determinism.rs`; apps: `liminal_testkit` (ADR-0024) | State hashes from `liminal_core`, compared across runs; recorded literals where cross-platform identity is claimed | Deterministic crates at `usable`+, demo games |
| Concurrency | `loom` tests behind `cfg(loom)` | Exhaustive interleavings | Lock-free code in `liminal_jobs`, `liminal_telemetry`, `liminal_ecs` |
| Miri | `cargo +nightly miri test -p <crate>` | Undefined-behaviour detection | Every crate with `unsafe = "allow"` (where Miri can run) |
| Fuzz | `fuzz/` with `cargo fuzz` (nightly) | No panic, round-trip invariants | Parsers: cooked formats, JSON, protocol, importers (at `stable`) |
| Benchmark | `benches/` crate `liminal_benches`; comparisons in `benches/competitive/` (ADR-0023) | Baselines in `ci/perf/` | Performance-critical crates at `stable`+ |

## Requirement traceability

Specs list requirements with IDs (`ECS-R012`). A test proves a requirement by carrying a doc-comment line:

```rust
/// Verifies: ECS-R012, ECS-R013
#[test]
fn despawned_entity_ids_are_not_reused_until_generation_wraps() { … }
```

`cargo xtask trace` builds the requirement → test matrix. Gate: for crates at maturity `usable` or
higher, every `MUST` requirement needs at least one verifying test. `SHOULD` requirements are reported
but do not fail the gate. Tests that name unknown IDs fail the gate.

The annotation belongs in the doc comment directly above the function; attributes and comments may
come between. A `/// Verifies:` line that is malformed or not followed by a function fails `trace`,
as does an annotation naming a withdrawn requirement. Exact rules: `docs/specs/xtask.md`, "`trace`
and `hygiene` details". Paths that `trace` and `hygiene` skip (fixture repositories) are listed in
`ci/hygiene.toml`, each with a reason.

## Coverage

Measured with `cargo llvm-cov` (line coverage) per crate: `cargo xtask coverage`, and the `coverage`
step of `cargo xtask ci`. Install the tool with `cargo install cargo-llvm-cov --locked`; without it
the step is `skipped`, and a task cannot go to `review`. Doc tests are not measured. Floors live in
`ci/coverage.toml` (format in its header and in `docs/specs/xtask.md` "`coverage` details"). Every
workspace crate needs a floor: a new crate adds its line in the same change.

- **Ratchet:** CI fails if a crate's coverage drops more than 0.5 percentage points below its floor.
  `cargo xtask coverage --update-floors` raises floors to the current values (never lowers them). Run it
  when coverage improves and commit the result.
- **Lowering a floor** requires a journal entry explaining why (e.g. code moved to another crate).
- **Maturity targets** (gates for promotion, not for every commit), in percent line coverage:

  | Crate type | `usable` | `stable` |
  |---|---|---|
  | Pure logic (layers 0–1, `liminal_voxel`, `liminal_physics`, `liminal_text`, `liminal_procgen`) | 85 | 90 |
  | GPU/platform/render (`liminal_gpu`, `liminal_platform`, `liminal_render`, `liminal_voxel_render`) | 60 | 70 |
  | Tools (`xtask`, `liminal_cli`, `liminal_mcp`, `liminal_cook`) | 75 | 85 |

  GPU and platform code has lower line-coverage targets because much of it only runs on specific
  hardware. Golden-image tests on each backend make up for this.

Coverage is a floor, not a goal. A covered line with no meaningful assertion is worse than an uncovered
line, because it hides the gap. Reviewers check for assertion-free tests.

## Golden images

- Rendered offscreen at a fixed resolution (default 256×256) with fixed seed, time and camera.
- Stored as PNG at `crates/<crate>/tests/golden/<backend>/<test_name>.png`. Backends: `vulkan-lavapipe`
  (the canonical CI reference), `vulkan-hw`, `metal`.
- Comparison: per-pixel absolute difference per channel. A test fails if more than `max_bad_pixels`
  (default 0.1%) exceed `channel_tolerance` (default 2/255). Tolerances are set per test and must be
  justified in a code comment when loosened.
- On failure the test writes `<name>.actual.png` and `<name>.diff.png` next to the golden file (both
  git-ignored), so an agent can inspect them with an image-capable tool.
- Updating a golden: `LIMINAL_UPDATE_GOLDEN=1 cargo test …`, then the journal entry MUST state why
  the image changed and that the new image was inspected. (AGENTS.md rule 5.)

## Determinism tests

Layer 0–1 deterministic crates test determinism in their own `tests/determinism.rs`, with the
`liminal_core` state hasher: a seeded workload runs at least twice and the hashes are compared;
`liminal_ecs` and `liminal_app` also compare single- and multithreaded execution. Where a spec
claims cross-platform bit-identity, the test also compares against a recorded hash literal
(ADR-0024).

For whole apps, `liminal_testkit` provides:
- `assert_deterministic(app_builder, seed, steps, runs)`: runs N times, compares state hashes at every
  step, and reports the first diverging step and the components that differ.
- Replay files: recorded input streams plus expected state hashes at checkpoints. A replay is a
  regression test for both behaviour and determinism.
- Multithreaded and single-threaded schedulers must produce identical hashes.

## Test hygiene rules

- Tests are deterministic. No sleeps, no wall-clock timing assertions in unit or integration tests,
  no network, and no dependence on test execution order. Randomized tests use fixed or logged seeds.
- `#[ignore]` requires a reason that names a task: `#[ignore = "M3-T012: needs GPU"]`.
- GPU tests are gated behind the `gpu-tests` Cargo feature and skip with a clear message when no
  adapter exists. A skip is never reported as a pass in CI summaries.
- Test names describe behaviour: `despawn_invalidates_handle`, not `test_despawn_2`.
- Test helpers live in the crate's `tests/common/` or in `liminal_testkit`. Do not copy them between
  crates.

## What "fully tested, without halting progress" means here

- Every change brings tests for the behaviour it adds or changes. This is always required.
- Coverage targets and full traceability gate **maturity promotion**, not each commit. An
  `experimental` crate can grow quickly as long as each change is tested and floors do not drop.
- The ratchet stops slow decay without demanding perfection on day one.
