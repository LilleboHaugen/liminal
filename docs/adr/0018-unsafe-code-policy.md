---
id: ADR-0018
title: Unsafe code policy
status: accepted
date: 2026-11-08
supersedes: []
superseded_by: []
---

# ADR-0018: Unsafe code policy

## Context

A high-performance engine needs `unsafe` in specific places: custom allocators, ECS column storage,
lock-free job queues, SIMD intrinsics, and FFI to GPU and OS APIs. Unsafe code written by agents
without strict rules is the most likely source of hard-to-find bugs (undefined behaviour that passes
tests).

## Decision

- `unsafe` is allowed only in crates with `unsafe = "allow"` in `architecture.toml`. All other crates
  MUST declare `#![forbid(unsafe_code)]` (checked by `check-arch`).
- Every `unsafe` block MUST have a `// SAFETY:` comment naming the invariants relied on and why they
  hold at that point (lint `clippy::undocumented_unsafe_blocks = deny`).
- Every `unsafe fn` MUST document its caller obligations under `# Safety`.
  `unsafe_op_in_unsafe_fn = deny`.
- Unsafe code MUST be wrapped in safe abstractions with minimal unsafe surface, and MUST be covered by
  tests run under **Miri** in CI where Miri supports the operations (not FFI). Exceptions are listed in
  the crate spec with reasons.
- Lock-free data structures MUST also have **loom** tests.
- Unsafe for performance requires a benchmark showing the safe version is insufficient, referenced in
  the commit or spec.
- Reviewers MUST check every new `SAFETY:` comment (`docs/process/REVIEW.md` §1.6).

## Alternatives considered

- **No unsafe anywhere:** not achievable for FFI, and costly for allocators and ECS storage.
- **Unsafe allowed anywhere with comments:** spreads risk across the codebase. Rejected.

## Consequences

Unsafe code stays concentrated in a few crates where review and Miri effort is focused. Changing a
crate's unsafe policy is an architecture change (ADR).

## Verification

`check-arch` (forbid attribute present). Clippy lints. Miri and loom jobs in CI.
