# Vision

## Why Liminal exists

AI agents can now take large, unfamiliar native codebases and rebuild them for new platforms, often with
better performance than the original. Recent statically recompiled console games are an example
(see `docs/journal/2026-11-08-planning-session.md` for the research behind this). Those efforts share one
property: **the agent always had an oracle**. The original game defined correct behaviour, so every step
could be checked by a machine, thousands of times, with no human in the loop.

Existing engines (Unity, Unreal, Godot, Bevy) are designed for humans. They have editors, implicit
conventions and large surfaces built around human workflows. An agent using them works with poor
feedback: it cannot easily run the game headless, inspect world state, compare frames or measure the
effect of a change.

Liminal is an engine whose primary user is an AI agent. Its two promises:

1. **Agents can build games with it quickly** by describing intent in Rust code and structured data.
2. **Agents can verify everything they build**: behaviour, visuals, determinism and performance, through
   machine-readable interfaces, without a human playtesting.

The engine itself is developed the same way. Every module has a written contract, tests traced to that
contract, reference implementations for optimized paths, and recorded performance baselines.

## The end-state workflow

1. A person describes a game in plain language.
2. An agent turns the description into a Game Design Spec (template planned for milestone M7).
3. The agent runs `liminal new <name> --template <genre>` and gets a compiling project with tests.
4. The agent builds the game in vertical slices. After each slice it runs the game headless, steps
   frames, queries world state, injects input, captures screenshots and reads frame timings through the
   `liminal` CLI or MCP server. It then fixes what does not match the spec.
5. Performance is checked against budgets on reference hardware, including a low-end target.
6. `liminal package` produces native builds for macOS, Windows and Linux/SteamOS.

## Goals

- **Native performance.** A game is a Rust crate compiled to a native binary. No VM, no scripting
  runtime, no garbage collector.
- **Measurably fast.** Every performance-critical module is benchmarked against the best open-source
  equivalent (for example `bevy_ecs` and `hecs` for the ECS, `glam` for math, `rayon` for jobs, `rapier`
  for physics). Target: parity at maturity `stable`, better at maturity `optimized`. See
  `docs/process/PERFORMANCE.md`.
- **Stylized 3D first.** Voxel, low-poly, flat-shaded and cel/toon-shaded looks with outlines, fog,
  shadows and stylized lighting. These looks reward systems and consistency more than hand-made assets.
- **Procedural-first content.** Agents are strong at code and weak at producing art assets. Procedural
  geometry, palettes, materials defined by parameters, noise, procedural animation and synthesized audio
  are first-class features, not afterthoughts. Imported assets (glTF, PNG, audio) are supported as well.
- **Deterministic simulation.** Same inputs + same seed → same world state, every run. This makes replay
  tests, bug reproduction and automated verification possible (ADR-0009).
- **Modular.** Subsystems are optional crates and modules. Adding, replacing or removing one cannot break
  unrelated parts, and this is enforced by machine-checked architecture rules (ADR-0004).
- **Fast iteration for agents.** Incremental rebuild time after a small game-code change is a tracked
  budget. Agents spend their time waiting on the compiler; Liminal treats that as a performance problem.

## Target games (genres, not copies)

Liminal is driven by original demo games in these genres, in this order:

1. **Voxel sandbox** (first target, milestones M5–M6): a block world with streamed chunks, editing,
   day/night, a first-person controller.
2. **Stylized open-world action-adventure** (later): heightfield/SDF terrain, foliage, water, skeletal
   animation, physics, large-world streaming.
3. **Top-down 2D** (later): sprites, tilemaps, 2D physics, 2D lighting.

Liminal never ships recreations of existing games. See `AGENTS.md` rule 14.

## Platforms

| Platform | Architecture | GPU API | Status |
|---|---|---|---|
| macOS 13+ on Apple Silicon | aarch64 | Metal 3 (Metal 4 features optional) | Tier 1 |
| Windows 10/11 | x86_64 (MSVC) | Vulkan 1.3 | Tier 1 |
| Linux incl. SteamOS / Steam Deck | x86_64 | Vulkan 1.3 | Tier 1 |
| Linux / Windows on ARM | aarch64 | Vulkan 1.3 | Later |
| Web, mobile, consoles | — | — | Not planned |

Reference machine for performance budgets: Apple M5 Pro, 6 performance + 12 efficiency cores, 20-core
GPU, 24 GB RAM, macOS 27.2. Low-end target: Steam Deck (1280×800). See `docs/process/PERFORMANCE.md`.

## Non-goals

- A visual editor for humans. Tooling is CLI-, protocol- and file-based. Visual debugging is screenshots
  and structured dumps an agent can read.
- Photorealism (path tracing, film-quality PBR pipelines, photogrammetry workflows).
- A scripting language. Game logic is Rust (ADR-0002).
- Wrapping another engine or graphics abstraction (ADR-0003, ADR-0006).
- Source compatibility with other engines.

## Readability

Liminal is optimized for agents, but that does not mean obscure. Agents work through limited context
windows and depend on clear names, small modules, written invariants and precise docs, the same things
that make code readable to humans. Documentation is written *for* agents: dense, normative and complete,
rather than tutorial-style.
