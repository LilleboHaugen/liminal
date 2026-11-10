---
id: M1-T061
title: Reflect impls for liminal_math and liminal_core types
milestone: M1
status: todo
crates: [liminal_reflect]
depends_on: [M1-T032, M1-T020]
requirements: []
---

# M1-T061: `Reflect` impls for `liminal_math` and `liminal_core` types

## Goal

Components can contain `liminal_math` vectors, quaternions and matrices and the `liminal_core`
types that make sense as state, derive `Reflect`, and so take part in `World::state_hash()`.

## Context

- `docs/specs/liminal_reflect.md` §12 items 1 and 2; `docs/specs/liminal_math.md` §12 item 7;
  `docs/specs/liminal_ecs.md` §12 items 1 and 2 (the decision), ECS-R004 (how `Entity` is
  reflected).
- Decision taken by M1-T037 (the `liminal_ecs` spec), as `liminal_reflect.md` §12 item 1 asks:
  option (a). `liminal_reflect` gets `liminal_math` in its `same_layer_deps` in
  `architecture.toml` and implements `Reflect` and `Typed` for the math types by hand;
  `liminal_math` stays free of reflection. No ADR is needed: both crates are layer 0, the edge
  adds no external dependency, and `architecture.toml` asks for an ADR or a task reference.
- Not decided by M1-T037: the shape of `liminal_core` types (`Tick`, `TickDuration`, `RawHandle`,
  `Handle<T>`, `Rng`, `StreamKey`). One candidate is the form `Entity` uses: a primitive shape
  over the stored bits, written only through `downcast_mut` of the whole type. It fits types
  that store one integer. `Rng` (four words, all-zero state invalid) and `Handle<T>` need a
  decision of their own (an "opaque" shape is the other candidate named in the reflect spec).
- Written by M1-T037 (2026-11-10) because the change is outside `liminal_ecs` (WORKFLOW.md §6).
  `headless_boids` (M1-T050) depends on this task.

## Scope

**In:**
- `architecture.toml`: `liminal_math` added to `same_layer_deps` of `liminal_reflect` (commit
  message references this task).
- `docs/specs/liminal_reflect.md`: `spec_version` bump; the new impls in §2 and §5; requirements
  with IDs for their type paths, shapes and state encoding; §12 items 1 and 2 resolved.
  `docs/specs/liminal_math.md` §12 item 7 and `docs/specs/liminal_ecs.md` §12 items 1 and 2
  updated (no behaviour change in those two specs).
- The impls and their tests in `liminal_reflect`, including a determinism workload entry if the
  reflect spec's §11 needs one.

**Out:** Reflection of `Entity` (done in `liminal_ecs`, ECS-R004). Serialization (M2).

## Acceptance criteria

- [ ] `cargo xtask check-arch` passes with the new edge; no dependency cycle
      (`liminal_math` does not depend on `liminal_reflect`).
- [ ] The reflect spec lists every math type and every core type that gets an impl, with type
      path and shape, and states for each core type without an impl why it has none.
- [ ] For every new impl: `validate_reflect` returns `Ok`, the state encoding equals the type's
      `StateHash` encoding where the type has one (or the spec states the difference), and a
      struct that contains the type derives `Reflect` and compiles.
- [ ] Reflection cannot put a value into an invalid state (for example generation 0, an
      all-zero RNG state); a test shows the rejected write for each such type.
- [ ] Every new requirement has a test with a `/// Verifies:` line; `cargo xtask ci` passes with
      no failed or skipped step.

## Test requirements

The new requirement IDs of the reflect spec; unit, property and known-answer tests as in that
spec's §10.

## Notes

`liminal_reflect` may still be at `planned` or `experimental` when this task starts; the new
requirements join its trace gate at `usable`. If M1-T032's promotion review has already made the
crate `usable`, every new MUST needs its test in this task.

## Handoff

_Empty._

## Review

_Empty._
