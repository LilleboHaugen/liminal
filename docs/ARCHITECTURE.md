# Architecture

This document explains how Liminal is structured. The normative, machine-checked version of the crate
graph is `architecture.toml`; if the two disagree, `architecture.toml` wins and this file must be fixed.

## 1. Layers

Crates are arranged in layers. A crate MAY depend only on crates in strictly lower layers, plus
same-layer crates explicitly listed in its `same_layer_deps` entry in `architecture.toml`. Cargo forbids
cycles; the layer rule also forbids "upward" knowledge (ADR-0004).

```
Layer 5  Tools (never linked into shipped games)
         liminal_cli · liminal_mcp · liminal_cook · xtask
Layer 4  Agent interface & integration
         liminal_probe · liminal_testkit · liminal_engine (facade, lib name `liminal`)
Layer 3  Subsystems
         liminal_render · liminal_shader · liminal_voxel · liminal_voxel_render · liminal_physics
         liminal_audio · liminal_text · liminal_ui · liminal_anim
Layer 2  Platform & GPU boundary (FFI, OS and driver code)
         liminal_platform · liminal_gpu
Layer 1  Runtime
         liminal_jobs · liminal_ecs · liminal_app · liminal_input · liminal_assets · liminal_scene
         liminal_serial · liminal_procgen
Layer 0  Foundation (pure Rust, no OS or driver specifics)
         liminal_core · liminal_math · liminal_memory · liminal_telemetry · liminal_reflect
         liminal_reflect_derive (proc-macro)
```

Design consequences:

- Everything in layers 0–1 runs headless and is testable on any CI machine without a GPU or window.
- Layer 2 is the only place that talks to the OS windowing system or the GPU driver.
- Subsystems (layer 3) communicate through ECS data (components, resources, events) rather than by
  calling each other, so one can be removed without touching the others.
- Tools (layer 5) may use more external crates (importers, CLI parsing) because they never ship inside a
  game (ADR-0003).

## 2. Module map

Maturity of each crate is tracked in its spec frontmatter and summarized in `docs/STATUS.md`.

| Crate | Layer | Purpose | First milestone |
|---|---|---|---|
| `liminal_core` | 0 | Error model with stable codes, generational IDs/handles, deterministic hashing and RNG, time types | M1 |
| `liminal_math` | 0 | Scalar + SIMD linear algebra, geometry primitives, intersection tests, deterministic transcendental functions (`det`) | M1 |
| `liminal_memory` | 0 | Arena, frame (linear), pool and aligned allocators; allocation tracking | M1 |
| `liminal_telemetry` | 0 | Structured logs, spans, counters, frame stats; JSON and Chrome-trace export | M1 |
| `liminal_reflect` | 0 | Runtime type information, field access by name, stable type IDs | M1 |
| `liminal_reflect_derive` | 0 | `#[derive(Reflect)]` | M1 |
| `liminal_jobs` | 1 | Work-stealing job system, job graphs, parallel-for, scoped tasks | M1 |
| `liminal_ecs` | 1 | Archetype ECS: entities, components, resources, events, queries, commands, change detection, schedules with parallel execution | M1 |
| `liminal_app` | 1 | App builder, module registration, main loop (fixed-step simulation + interpolated rendering), runners | M1 |
| `liminal_serial` | 1 | Text (JSON) and binary serialization driven by reflection; JSON Schema generation | M2 |
| `liminal_input` | 1 | Device-agnostic input state, action maps, input recording and replay | M3 |
| `liminal_assets` | 1 | Asset handles, registry, async loading, cooked format reader, hot reload | M4 |
| `liminal_scene` | 1 | Transforms, hierarchy, transform propagation, scenes and prefabs from data | M4 |
| `liminal_procgen` | 1 | Noise (value, gradient, cellular, fBm), procedural mesh builders, palettes | M5 |
| `liminal_platform` | 2 | Windows, events, display info, OS paths; windowed runner (wraps `winit`, ADR-0007) | M3 |
| `liminal_gpu` | 2 | Liminal's own RHI: Vulkan 1.3 and Metal backends, bindless-first (ADR-0006) | M3 |
| `liminal_shader` | 3 | Shader sources, build-time compilation to SPIR-V and MSL, reflection (language: OQ-001) | M3 |
| `liminal_render` | 3 | Render graph, cameras, meshes, materials, stylized lighting, shadows, post-processing, debug draw | M4 |
| `liminal_voxel` | 3 | Chunk storage (palette-compressed), meshing, voxel lighting, streaming. CPU-only, no GPU code | M5 |
| `liminal_voxel_render` | 3 | Voxel-specific GPU paths: packed vertices, chunk culling, upload scheduling | M5 |
| `liminal_physics` | 3 | Collision detection, queries, character controller, rigid bodies, voxel collision | M6 |
| `liminal_audio` | 3 | Mixer, spatial audio, streaming playback; device output via `cpal` | M6 |
| `liminal_text` | 3 | Font parsing, glyph rasterization (MSDF atlas), simple shaping | M4 |
| `liminal_ui` | 3 | Game UI: layout, widgets, HUD | M6 |
| `liminal_anim` | 3 | Skeletal animation, blending, IK | M8 |
| `liminal_probe` | 4 | In-process control server (JSON-RPC 2.0): introspection, frame stepping, input injection, capture, telemetry | M2 |
| `liminal_testkit` | 4 | Headless app harness, deterministic stepping, replay, state-hash assertions, golden-image comparison | M1 |
| `liminal_engine` | 4 | Facade: curated re-exports, `prelude`, default module bundle, feature flags per subsystem | M2 |
| `liminal_cli` | 5 | `liminal` binary: new, run, step, query, capture, replay, profile, validate, schema, package | M2 |
| `liminal_mcp` | 5 | MCP server adapting the probe protocol for agent clients | M2 |
| `liminal_cook` | 5 | Asset cooker: importers (glTF, PNG, audio, fonts) → engine-native binary formats | M4 |
| `xtask` | 5 | Repository automation and quality gates | M0 |

`liminal_net` (networking) and a 2D module set are planned after M8 and are not yet registered.

## 3. How a game is built

A game is a Rust binary crate that depends on `liminal_engine` (imported as `liminal`):

```rust
use liminal::prelude::*;

fn main() -> Result<(), liminal::Error> {
    App::new()
        .add_modules(DefaultModules)          // or pick modules individually
        .add_module(MyGameModule)             // game code: components, systems, resources
        .run()
}
```

- **Logic** is Rust: components (plain data structs), systems (functions over queries), resources and
  events, grouped into modules. Modules are the unit of composition for both engine and game code.
- **Data** is JSON validated by schemas generated from reflected types (ADR-0013): scenes, prefabs,
  tuning values, input maps, materials. Data can hot-reload without recompiling.
- **Assets** are cooked by `liminal_cook` into engine-native binary formats. The runtime never parses
  source formats such as PNG or glTF.
- Subsystems are turned on and off with `liminal_engine` Cargo features (`voxel`, `physics`, `audio`,
  `ui`, …) and by which modules are added to the `App`.

## 4. Frame lifecycle

The simulation runs on a fixed timestep (default 60 Hz, configurable). Rendering runs as fast as the
display allows and interpolates between the last two simulation states.

```
loop {
    platform events → input state                      (liminal_platform → liminal_input)
    probe commands applied (pause/step/inject)         (liminal_probe; when compiled in)
    while accumulator >= dt:                           (liminal_app)
        schedule FixedUpdate:  PreUpdate → Update → PostUpdate   (game + physics + voxel edits)
        apply deferred commands (deterministic order)
        state hash recorded if requested               (liminal_testkit / probe)
    schedule Frame: extract → prepare → render graph   (liminal_render; interpolation alpha)
    telemetry frame record closed                      (liminal_telemetry)
}
```

Headless runs (tests, CI, agents) skip the platform and render stages, or render offscreen, and step
the simulation explicitly: `app.step(n)`.

## 5. Cross-cutting design

### 5.1 Determinism (ADR-0009)
Fixed timestep; RNG streams from `liminal_core::rng`, seeded from the world seed; no randomly seeded
hash maps in simulation state; parallel systems may not race on shared state (the scheduler forbids
conflicting access); deferred commands apply in a defined order; transcendental functions come from
`liminal_math::det`. `World::state_hash()` hashes all reflected simulation state and is the basis of
replay tests. Guarantee: bit-identical results for the same binary on the same platform. Cross-platform
bit-identity is a goal, verified per crate as it matures.

### 5.2 Errors (ADR-0017)
Each crate defines error enums. Every variant has a stable code (`LIM-ECS-0007`), a message, structured
context, and where possible a `help:` hint telling the caller how to fix it. Errors serialize to JSON for
agent tooling. Panics are reserved for violated invariants (bugs), never for bad input or I/O.

### 5.3 Telemetry (ADR-0016)
`liminal_telemetry` provides spans, counters and logs with near-zero cost when disabled. Every frame
produces a record (timings per schedule and system, entity counts, memory, GPU timings). It exports as
Chrome-trace JSON (viewable in Perfetto) and as compact JSON summaries for agents.

### 5.4 Agent interface (ADR-0019)
`liminal_probe` runs inside development builds and exposes JSON-RPC 2.0 over stdio or a localhost
socket. Agents can pause, resume and step the simulation, query and edit entities, inject input, read
state hashes, record and replay sessions, read telemetry and logs, and capture screenshots. The
authoritative method list is in ADR-0019. `liminal_cli` and `liminal_mcp` are clients of this protocol.
Packaged games do not include the probe unless they explicitly opt in (ADR-0019 build gating).

### 5.5 Rendering (ADR-0006, ADR-0014)
`liminal_gpu` is a thin, explicit RHI with compile-time backend selection (Metal on macOS, Vulkan on
Windows and Linux, with Vulkan through MoltenVK on macOS for testing only). Backend types are concrete,
with no trait objects on hot paths. The renderer above it uses a render graph that derives barriers
and transient resource lifetimes. The feature set targets stylized looks: cel/toon ramps, flat
shading, outlines, fog, cascaded shadow maps, stylized ambient occlusion, bloom, color grading, and
GPU-driven instancing.

### 5.6 Testing and verification (ADR-0010)
See `docs/process/TESTING.md`. Each crate's spec lists which kinds of tests apply: unit, property,
differential (against a reference implementation), golden image, replay/determinism, benchmark, fuzz,
Miri, loom.

## 6. Repository layout

```
AGENTS.md  CLAUDE.md  README.md  architecture.toml  Cargo.toml (M0)  rust-toolchain.toml (M0)
crates/liminal_<name>/      engine crates: src/, tests/; README.md not required (spec lives in docs/specs)
benches/                    liminal_benches: harness + engine benchmarks (ADR-0023)
benches/competitive/        separate workspace: comparisons with bevy_ecs, hecs, glam, rayon
xtask/                      repository automation
games/<name>/               original demo games (from M6)
examples/                   minimal single-feature examples (from M3)
ci/coverage.toml            per-crate coverage floors (ratchet)
ci/perf/                    performance baselines per machine
docs/                       vision, architecture, roadmap, status, ADRs, specs, tasks, journal, process
```
