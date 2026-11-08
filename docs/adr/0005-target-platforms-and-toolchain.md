---
id: ADR-0005
title: Target platforms and toolchain
status: accepted
date: 2026-11-08
supersedes: []
superseded_by: []
---

# ADR-0005: Target platforms and toolchain

## Context

Games must run natively on macOS, Windows and Linux, including SteamOS on the Steam Deck. The project
owner develops on an Apple Silicon Mac (M5 Pro, macOS 27.2). Building and testing for Windows and Linux
from a Mac is unreliable, especially anything touching the GPU.

## Decision

Tier 1 platforms (built and tested in CI on every change, releases ship):

| Target triple | OS | GPU API |
|---|---|---|
| `aarch64-apple-darwin` | macOS 13+ (Apple Silicon) | Metal 3 baseline |
| `x86_64-pc-windows-msvc` | Windows 10/11 | Vulkan 1.3 |
| `x86_64-unknown-linux-gnu` | Linux, SteamOS 3 | Vulkan 1.3 |

- Intel Macs are not supported.
- ARM Linux and ARM Windows are planned later. Web, mobile and consoles are not planned.
- Toolchain: Rust **stable**, pinned to an exact version in `rust-toolchain.toml` (the pin is chosen at
  bootstrap, task M0-T001, as the current stable release), edition 2024, resolver 3.
- Each Tier 1 platform builds natively on its own CI runner (GitHub Actions matrix). Cross-compilation
  is not part of the release path.
- Linux release builds target the Steam Linux Runtime for compatibility with SteamOS (details: OQ-006).

## Alternatives considered

- **Cross-compiling everything from macOS:** fragile for Windows (MSVC) and cannot test GPU code.
- **Include Intel macOS:** adds a target that cannot use Apple Silicon-only GPU features, and the
  installed base is shrinking.
- **DX12 on Windows:** Vulkan 1.3 is well supported by NVIDIA, AMD and Intel drivers on Windows, so one
  API serves Windows and Linux. DX12 can be added later if needed (ADR-0006).

## Consequences

CI needs three OS runners. Some verification (real GPUs on Windows and Linux) needs real hardware and
is recorded manually in the journal until self-hosted runners exist.

## Verification

CI matrix (task M0-T007). `rust-toolchain.toml` pin.
