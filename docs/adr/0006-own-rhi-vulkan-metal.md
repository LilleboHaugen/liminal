---
id: ADR-0006
title: Own GPU abstraction over Vulkan 1.3 and Metal
status: accepted
date: 2026-11-08
supersedes: []
superseded_by: []
---

# ADR-0006: Own GPU abstraction over Vulkan 1.3 and Metal

## Context

The renderer needs Metal on macOS and a native API on Windows and Linux. Existing portable abstractions
(`wgpu`, and older ones such as `gfx-hal`) add validation and tracking overhead and expose a lowest
common denominator. That limits bindless resources, explicit synchronization, subgroup operations and
newer pipeline types. Recent agent-driven projects have rebuilt console graphics layers on Metal in
days, which suggests our own thin RHI is within reach and worth building.

## Decision

- `liminal_gpu` is Liminal's own RHI (Render Hardware Interface) with two backends:
  - **Vulkan 1.3** (Windows, Linux/SteamOS), on raw bindings from `ash`. Required features: dynamic
    rendering, synchronization2, descriptor indexing (bindless), timeline semaphores, buffer device
    address.
  - **Metal** (macOS), on raw bindings from `objc2-metal`, `objc2-foundation` and `objc2-quartz-core`.
    Baseline: Metal 3 on Apple Silicon (argument buffers tier 2 for bindless). Metal 4 features are
    optional.
- Backend selection is at **compile time** (`cfg(target_os)` plus features). Public types are concrete,
  and hot paths do not use trait objects or dynamic dispatch. Vulkan through MoltenVK on macOS MAY be
  supported as a test configuration only.
- The RHI is explicit and close to the hardware: resource lifetime, memory allocation (our own
  allocator) and synchronization are visible to the renderer. A render graph in `liminal_render` derives
  barriers automatically, so game code never handles barriers.
- The RHI MUST support headless operation (no window, offscreen targets, readback) for golden-image
  tests and agent screenshots.
- DX12 is not planned. It may be added as a third backend later through a new ADR.
- External crates approved by this ADR (`runtime` scope): `ash`, `objc2`, `objc2-foundation`,
  `objc2-metal`, `objc2-quartz-core`, `raw-window-handle`. These are bindings only, not abstractions.

## Alternatives considered

- **`wgpu`:** fastest to a working renderer. Rejected as the long-term base for the overhead and
  feature-ceiling reasons above. It is a useful reference implementation to compare against when
  debugging.
- **Vulkan only, with MoltenVK on macOS:** one backend, but a translation layer on our primary
  development platform, and Metal-specific features would be out of reach.
- **Writing our own bindings instead of `ash`/`objc2-*`:** pure transcription work, no performance gain.

## Consequences

- Two backends to write and keep in sync. Every RHI feature needs tests on both.
- Validation layers (Vulkan) and Metal API validation are mandatory in tests.
- Shader compilation must target both SPIR-V and Metal (OQ-001).

## Verification

Golden-image tests per backend. Validation-layer-clean test runs (M3 exit criteria). `check-arch`
restricts these external crates to `liminal_gpu` (and `raw-window-handle` to `liminal_platform` too).
