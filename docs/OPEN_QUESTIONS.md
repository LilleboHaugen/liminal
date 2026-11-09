# Open questions

Decisions deliberately **not** made yet, because making them now would mean guessing. Each one has a
deadline milestone, candidate options and evaluation criteria. An open question is closed by an ADR,
and the entry here is then marked `resolved` with the ADR number. Do not make a decision listed here
without going through that process.

| ID | Question | Resolve by | Status |
|---|---|---|---|
| OQ-001 | Shader language and compilation toolchain | M3 kickoff (spike) | open |
| OQ-002 | Serialization: own reflection-driven serializer or `serde` | M2 kickoff | open |
| OQ-003 | Authored text format: strict JSON, JSON5/JSONC or other | M2 kickoff | open |
| OQ-004 | Benchmark harness and CI regression method | M1 kickoff | resolved (ADR-0023) |
| OQ-005 | Audio decoding formats and decoder source | M6 kickoff | open |
| OQ-006 | Linux packaging target (Steam Linux Runtime version, glibc floor) | M7 kickoff | open |
| OQ-007 | GPU test coverage on hosted CI runners (macOS/Windows) | M3 | open |
| OQ-008 | Physics architecture (solver type, determinism strategy across platforms) | M6 kickoff | open |
| OQ-009 | Dev-time cooking vs. layer rules (runtime cannot depend on `liminal_cook`) | M4 kickoff | open |
| OQ-010 | Where determinism/replay tests for layer 0–1 crates live (`liminal_testkit` is layer 4) | M1 kickoff | resolved (ADR-0024) |

---

## OQ-001 — Shader language and compilation toolchain

Shaders must reach SPIR-V (Vulkan) and MSL or Metal IR (Metal). Compilation happens at build/cook
time, never at game runtime.

Candidates:
- **Slang** → SPIR-V and Metal. Modern language with modules, generics and automatic differentiation.
  The compiler is native C++ (a build-time dependency).
- **WGSL with `naga`** → SPIR-V and MSL. Pure Rust, but WGSL is less expressive (limited
  bindless support, no subgroup operations in some versions).
- **HLSL with DXC → SPIR-V, then SPIRV-Cross → MSL.** The most widely used industry path. Two native
  tools.
- **Rust (rust-gpu) → SPIR-V, then translation to MSL.** One language for the whole engine, but the
  toolchain needs nightly Rust and is less mature.
- **Own shading language** compiled to SPIR-V and MSL. Maximum control, very large cost.

Criteria: bindless and descriptor-indexing support; compute, subgroup and mesh-shader reach;
debuggability; quality of error messages for agents; build-time cost; determinism of output; licence.
Method: implement the same three shaders (textured mesh, compute particles, toon lighting) with ≥2
candidates on both backends and record the results.

## OQ-002 — Serialization

Options: an own serializer driven by `liminal_reflect` (one type system, schema generation built in,
tuned to our formats), or `serde`, the ecosystem standard (proven, but a second trait system next to
reflection). Criteria: schema generation, speed of binary encode/decode, compile-time cost, error
quality, and whether games need to interoperate with other serde-based crates.

## OQ-003 — Authored text format

Strict JSON is what agents generate most reliably, and JSON Schema tooling is everywhere, but it has
no comments. JSON5/JSONC allows comments and trailing commas. Criteria: agent generation error rate
(measure it), schema tooling, human diffs.

## OQ-004 — Benchmark harness and CI regression method

**Resolved by ADR-0023** (M1-T001, 2026-11-09). The text below is the original question.

Wall-clock timing on shared CI runners is noisy. Candidates: `criterion` for local wall-clock
benchmarks plus instruction-count benchmarks (`iai-callgrind`, Linux/Valgrind) for CI gating; or an
own harness that writes the JSON `xtask` consumes. Also decide how competitive comparison crates
(`bevy_ecs`, `hecs`, `glam`, `rayon`) are brought in: they must not become dependencies of engine
crates, only of a separate `benches` crate.

## OQ-005 — Audio decoding

Candidates: own decoders for simple formats (WAV, QOA) at first, an own Vorbis/Opus decoder later, or
an existing decoder crate behind `liminal_audio`. Cooking can transcode everything into one runtime
format.

## OQ-006 — Linux packaging

Which Steam Linux Runtime (e.g. "sniper" or newer) to build against, the minimum glibc version, and
whether to also produce AppImage or Flatpak builds.

## OQ-007 — GPU tests on hosted CI

Linux can run Vulkan through lavapipe. It is unverified whether GitHub-hosted macOS runners expose
working Metal and whether Windows runners can load a software Vulkan ICD (lavapipe or SwiftShader). If
they cannot, GPU tests run on Linux in CI and locally on macOS. A self-hosted runner on the reference
Mac is an option. Related: XTASK-R012 runs everything with `--all-features`. Once backend-selection and
`gpu-tests` features exist, that must be revised so CI never enables a GPU feature on a machine without
a GPU, and a GPU test that cannot run is reported as skipped, never as passed.

## OQ-009 — Dev-time cooking vs. layer rules

ADR-0013 lets dev builds cook assets on demand when a source file changes. `liminal_cook` is layer 5,
and no layer 0–4 crate may depend on it (XTASK-R036). Candidates: the cooker runs as a separate
process (`liminal run` watches files, cooks, and tells the game to reload through the probe); or a
cooking core moves to a lower layer with importers kept in tools. Decide before `liminal_assets` hot
reload is specified.

## OQ-010 — Location of determinism tests for low-layer crates

**Resolved by ADR-0024** (M1-T001, 2026-11-09). The text below is the original question.

TESTING.md and MATURITY.md require determinism tests through `liminal_testkit` for deterministic crates,
but layer 0–1 crates cannot depend on `liminal_testkit` (layer 4), not even as a dev-dependency
(XTASK-R032). Candidates: a workspace-level test crate at layer 5 (e.g. `tests/determinism`) that
depends on everything; a small `liminal_testkit_core` at layer 1 with the hashing/replay primitives; or
crate-local determinism tests that do not need testkit. Decide before the first deterministic crate
reaches `usable`.

## OQ-008 — Physics architecture

Solver type (sequential impulse, TGS, XPBD), broad phase (BVH, SAP, grid), and how to keep rigid-body
simulation deterministic across platforms. Benchmark baseline: `rapier`.
