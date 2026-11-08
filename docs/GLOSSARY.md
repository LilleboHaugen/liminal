# Glossary

Terms with fixed meanings in this project. Use them exactly.

| Term | Meaning |
|---|---|
| **Agent** | An AI coding agent working on Liminal or building a game with it. |
| **ADR** | Architecture Decision Record in `docs/adr/`. Immutable once `accepted`. |
| **Archetype** | The set of component types an entity has. Entities with the same archetype are stored together in one table. |
| **Baseline** | A recorded performance measurement in `ci/perf/` that later runs are compared against. |
| **Cooking** | Offline conversion of source assets (glTF, PNG, …) into engine-native binary formats by `liminal_cook`. |
| **Coverage floor** | Minimum line coverage recorded per crate in `ci/coverage.toml`. It only goes up (ratchet). |
| **Deterministic crate** | A crate marked `deterministic = true` in `architecture.toml`. Simulation determinism rules (ADR-0009) apply to it. |
| **Differential test** | A test that runs an optimized implementation and a simple reference implementation on the same inputs and asserts identical results. |
| **Facade** | `liminal_engine` (imported as `liminal`): the only crate games depend on. |
| **Golden image** | An approved reference image. Rendering tests compare their output against it with defined tolerances. |
| **Handoff note** | The section in a task file describing exactly what is done, what is not, and what the next session must know. |
| **Kickoff** | First task of a milestone: resolves the open questions due then, and writes the milestone's detailed task files (including spec-writing tasks). |
| **Layer** | Position of a crate in the dependency order (0 = foundation … 5 = tools). See `architecture.toml`. |
| **Maturity** | A crate's level: `planned`, `experimental`, `usable`, `stable`, `optimized`. See `docs/process/MATURITY.md`. |
| **Module** | Unit of composition at runtime: a type implementing `liminal_app::Module` that registers components, systems and resources. Not the same as a Rust `mod`. |
| **Oracle** | Anything that independently defines correct behaviour: a reference implementation, golden image, recorded replay, spec requirement, or baseline. |
| **Probe** | `liminal_probe`: the in-process control server agents use to inspect and drive a running app. |
| **Reference machine** | Apple M5 Pro, 24 GB, macOS 27.2. The machine performance budgets are defined on. |
| **Requirement ID** | Stable identifier of a spec requirement, `<PREFIX>-R<NNN>`, e.g. `ECS-R012`. Never reused. |
| **RHI** | Render Hardware Interface: `liminal_gpu`, Liminal's own abstraction over Vulkan and Metal. |
| **State hash** | A 64-bit hash of all reflected simulation state at a given step. Used for determinism and replay tests. |
| **System** | A function run by the ECS scheduler over queries and resources. |
| **Tier 1 platform** | A platform where CI builds and tests on every change and releases ship. |
