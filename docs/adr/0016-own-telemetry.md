---
id: ADR-0016
title: Own telemetry instead of log/tracing
status: accepted
date: 2026-11-08
supersedes: []
superseded_by: []
---

# ADR-0016: Own telemetry instead of `log`/`tracing`

## Context

Agents learn what a game is doing from telemetry: per-frame timings per system, counters, structured
logs, memory and GPU timings. The ecosystem crates `log` and `tracing` are general-purpose. `tracing` in
particular has dynamic subscriber dispatch and a large feature surface. Liminal needs frame-based,
low-overhead, machine-readable telemetry designed around its own frame loop and probe.

## Decision

- `liminal_telemetry` (layer 0) provides:
  - spans (scoped timing, nested), counters and gauges, and structured log events with levels,
    target and key-value fields;
  - per-frame records that aggregate spans and counters for each simulation step and rendered frame;
  - lock-free per-thread buffers, drained once per frame;
  - export to Chrome-trace JSON (viewable in Perfetto), to compact JSON summaries for agents
    (e.g. top-N spans by time), and to the probe (`telemetry.frames`, `log.tail`).
- With the `telemetry` feature off, all macros compile to nothing.
- Engine crates MUST use `liminal_telemetry` and MUST NOT print directly (lint `print_stdout`/`print_stderr`).

## Alternatives considered

- **`tracing` with a custom subscriber:** mature and flexible, but dynamic dispatch on hot paths, no
  frame model, and our hot paths would depend on an external design. It contradicts ADR-0003, since
  telemetry runs in the hot loop.
- **`log`:** no spans and no structure.

## Consequences

We own a crate that needs careful benchmarking (overhead per span is a budget in its spec). External
crates that log through `log` (e.g. `winit`) are bridged in `liminal_platform` only if needed, through an
ADR.

## Verification

Telemetry benchmarks (overhead budget). Lints. Snapshot tests of export formats.
