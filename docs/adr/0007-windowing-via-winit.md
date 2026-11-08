---
id: ADR-0007
title: Windowing and OS events via winit behind liminal_platform
status: accepted
date: 2026-11-08
supersedes: []
superseded_by: []
---

# ADR-0007: Windowing and OS events via winit behind liminal_platform

## Context

Creating windows and receiving input events needs three separate OS integrations (AppKit, Win32,
Wayland/X11). This code is not performance-critical: events arrive at human speed. Writing it ourselves
would mean a lot of platform-specific code that is hard to test, with no runtime gain.

## Decision

- `liminal_platform` uses `winit` for windows, the OS event loop, keyboard/mouse/gamepad events (where
  `winit` supports them) and display information. `raw-window-handle` passes native handles to
  `liminal_gpu`.
- `winit` types MUST NOT appear in `liminal_platform`'s public API. All events are translated into
  Liminal's own types, so `winit` can be replaced without touching other crates.
- Gamepad support is evaluated at M3 kickoff. If `winit` coverage is insufficient, a dedicated input
  backend is designed in an ADR (our own per-OS implementation is acceptable).

## Alternatives considered

- **Our own per-OS windowing:** maximum control and no dependency. Deferred under ADR-0003 rule 3:
  replace when `winit` is shown to be a bottleneck or limitation.
- **SDL3:** mature and includes gamepads and audio, but it is a C dependency with its own event model,
  and it overlaps with systems we build ourselves.

## Consequences

We inherit `winit`'s event-loop model (the main thread owns the loop on macOS). The windowed runner in
`liminal_platform` must adapt `liminal_app`'s loop to it.

## Verification

`check-arch` restricts `winit` to `liminal_platform`. Review checks that the public API contains no
`winit` types.
