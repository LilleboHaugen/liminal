# 2026-11-08 — Planning session: project foundation

**Session type:** planning
**Tasks touched:** none (all task files created in this session)

## Done
- Agreed the project direction with the project owner. Key points:
  - Engine built from scratch in Rust, for AI agents to build fast, stylized, natively compiled
    games. 3D first (voxel), 2D later.
  - Tier 1 platforms: macOS (Apple Silicon), Windows, Linux/SteamOS.
  - No session is expected to finish a feature. Every module has minimum requirements and is tested
    properly.
  - Tests are central. Every decision is documented.
- Wrote the planning set: `AGENTS.md`, `CLAUDE.md` (imports AGENTS.md), `README.md`, licences,
  `architecture.toml`, `docs/VISION.md`, `docs/ARCHITECTURE.md`, `docs/ROADMAP.md`, `docs/STATUS.md`,
  `docs/OPEN_QUESTIONS.md`, `docs/GLOSSARY.md`, `docs/process/*` (8 files), ADR-0001 … ADR-0018,
  spec template and the full `xtask` spec, task template, tasks M0-T001 … M0-T010 and M1-T001.

## Verified
- Crate names on crates.io (2026-11-08): `liminal` taken (unrelated project, v0.1.4, last updated
  2024-05-06). `liminal_core`, `liminal_engine`, `liminal-engine` free. → ADR-0015.
- Reference machine read from the system: Apple M5 Pro, 6P + 12E CPU cores, 20-core GPU, Metal 4,
  24 GB, macOS 27.2.
- `LICENSE-APACHE` downloaded from apache.org (SHA-256
  `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`).
- Not verified: no Rust toolchain was installed on the machine during this session, so nothing was
  compiled. No code exists yet.

## Decisions
- ADR-0001 … ADR-0018 (see `docs/adr/README.md`). The most consequential:
  - Own RHI over Vulkan 1.3 and Metal instead of `wgpu` (ADR-0006).
  - `winit` behind `liminal_platform` (ADR-0007).
  - Own archetype ECS with explicit schedules (ADR-0008).
  - Determinism as a core guarantee (ADR-0009).
  - Requirement traceability + coverage ratchet (ADR-0010).
  - Agent interface (probe, CLI, MCP) built in M2, before graphics (ADR-0012, ROADMAP rationale).
- Deferred deliberately: OQ-001 … OQ-008 in `docs/OPEN_QUESTIONS.md`.

## Research behind the vision (as reported by secondary sources, 2026-11)
- AI-assisted static recompilation of Switch 2 games running natively on PC (Pokopia Recomp), and of
  the 2026 Ocarina of Time remake running natively on Apple Silicon Macs within days of release (early,
  performance reports vary).
- AI-assisted "merged games" (Minecraft running inside Skyrim through shared memory).
- Takeaway recorded in `docs/VISION.md`: those efforts succeeded because the original game was an
  oracle. Liminal's process builds its own oracles (specs with requirement IDs, reference
  implementations, golden images, replays, baselines).

## Independent review of the planning set (same session)
A fresh-context reviewer read every file and found 6 blocking and 16 misleading issues. All were fixed
in this session. The main changes:
- AGENTS.md pick rule now puts **reviewing** first. Without it, M0 would stall, because only reviewers
  set `done`.
- `xtask` gets `proptest` as an approved dev dependency. xtask is a lib + thin bin (fixture tests and
  doc tests need a library). Binaries allow the print lints at their crate root.
- XTASK-R051: `#[ignore]` task references only need to exist. Before this change, closing M0-T009 would
  have broken `hygiene`.
- XTASK-R008: deselected steps (`--only`/`--skip`) are omitted, not "skipped". R012 uses `--locked`.
- Tool crates have adapted `usable` gates. Multi-level promotion is allowed (MATURITY.md).
- ADR-0012 superseded by **ADR-0019**: Cargo features cannot be per-profile, so the probe gating was
  rewritten (feature + release opt-in), and the full method list is now authoritative there.
- `liminal_telemetry` set to `unsafe = "allow"` with loom (lock-free buffers). loom added for `liminal_ecs`.
- New open questions OQ-009 (dev cooking vs layers) and OQ-010 (where low-layer determinism tests
  live, resolved at M1 kickoff).
- `.gitattributes` (LF) created now. Spec/task wording fixes (shared repo-model loader, Windows path
  normalization, CI artifact naming, cargo-deny schema caution).
- Not changed: the reference machine description comes from `sysctl`/`system_profiler` output on this
  Mac. The recompilation claims are secondary-source reports and are labelled as such.

## Oracle changes
- None (no code yet).

## Problems and traps
- `liminal` is taken on crates.io. Use `liminal_engine` with `[lib] name = "liminal"` for the facade.
- No crate other than `xtask` should be created before its spec exists. M0 deliberately has only `xtask`.

## Next
- Project owner: install Rust (`rustup`), `git init`, create the GitHub repository.
- Agents: start with `docs/tasks/M0-T001-workspace-bootstrap.md`.
