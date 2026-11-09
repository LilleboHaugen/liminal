# Architecture Decision Records

An ADR records one decision: the context, the decision, the alternatives considered, and the
consequences. ADRs are how later agents know **why** things are the way they are, so they do not undo
good decisions or repeat rejected ones.

Rules (ADR-0001):
- Number sequentially: `NNNN-kebab-title.md`, using `0000-template.md`.
- `proposed` ADRs may be edited. `accepted` ADRs are immutable. To change a decision, write a new ADR
  with `supersedes: [ADR-XXXX]`, and set the old one's `status: superseded` and `superseded_by`.
  Those two fields are the only edits allowed to an accepted ADR.
- Review procedure: `docs/process/REVIEW.md` §4.
- Deferred decisions live in `docs/OPEN_QUESTIONS.md` until an ADR resolves them.

## Index

| ADR | Title | Status |
|---|---|---|
| [0001](0001-record-architecture-decisions.md) | Record architecture decisions | accepted |
| [0002](0002-rust-for-engine-and-games.md) | Rust for engine and game code; no scripting language | accepted |
| [0003](0003-build-from-scratch-dependency-policy.md) | Build from scratch; dependency policy | accepted |
| [0004](0004-layered-crates-machine-checked.md) | Layered crate architecture, machine-checked | accepted |
| [0005](0005-target-platforms-and-toolchain.md) | Target platforms and toolchain | accepted |
| [0006](0006-own-rhi-vulkan-metal.md) | Own GPU abstraction over Vulkan 1.3 and Metal | accepted |
| [0007](0007-windowing-via-winit.md) | Windowing and OS events via winit behind liminal_platform | accepted |
| [0008](0008-archetype-ecs.md) | Own archetype-based ECS with explicit schedules | accepted |
| [0009](0009-deterministic-simulation.md) | Deterministic simulation | accepted |
| [0010](0010-testing-strategy.md) | Testing strategy — traceability, oracles, coverage ratchet | accepted |
| [0011](0011-maturity-levels.md) | Maturity levels and promotion gates | accepted |
| [0012](0012-agent-interface.md) | Agent interface — probe protocol, CLI and MCP server | superseded by ADR-0019 |
| [0013](0013-data-formats.md) | Authored data as text, runtime data as cooked binary | accepted |
| [0014](0014-stylized-rendering-focus.md) | Stylized rendering focus | accepted |
| [0015](0015-naming-and-licensing.md) | Naming and licensing | accepted |
| [0016](0016-own-telemetry.md) | Own telemetry instead of log/tracing | accepted |
| [0017](0017-error-model.md) | Error model with stable codes | accepted |
| [0018](0018-unsafe-code-policy.md) | Unsafe code policy | accepted |
| [0019](0019-agent-interface-revised.md) | Agent interface (revised) — probe protocol, CLI, MCP server and build gating | accepted |
| [0020](0020-session-boundaries.md) | Session boundaries — owner-started sessions only, no sub-agents for task work | accepted |
| [0021](0021-review-findings-and-round-limit.md) | Review findings — blocking criteria, follow-up tasks and a round limit | accepted |
| [0022](0022-owner-approval-for-installs.md) | Owner approval before installing software on the owner's machine | accepted |
| [0023](0023-benchmark-harness-and-regression-method.md) | Own benchmark harness, instruction-count regression gate, competitive crates outside the workspace | accepted |
| [0024](0024-determinism-test-location.md) | Determinism tests for layer 0–1 crates are crate-local, built on liminal_core hashing | accepted |
| [0025](0025-public-api-snapshot-tool.md) | Public-API snapshots with cargo-public-api, from the first stable promotion | accepted |
| [0026](0026-sub-agents-for-task-work.md) | Sub-agents for task work under the same rules as owner-started sessions | proposed |
