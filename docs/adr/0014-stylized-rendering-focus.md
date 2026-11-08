---
id: ADR-0014
title: Stylized rendering focus
status: accepted
date: 2026-11-08
supersedes: []
superseded_by: []
---

# ADR-0014: Stylized rendering focus

## Context

Photorealistic rendering needs large asset pipelines (scanned materials, high-poly art), complex
lighting (path tracing, physically exact GI) and artistic judgement that is hard to verify by machine.
Stylized games such as voxel sandboxes and cel-shaded open worlds depend on consistent systems,
procedural content and strong art direction through parameters. Agents are good at all three.

## Decision

- Rendering features are prioritized for stylized looks: flat and low-poly shading, cel/toon lighting
  ramps, outlines (inverted hull and screen-space edge detection), fog and atmospheric scattering
  approximations, cascaded shadow maps, stylized ambient occlusion, bloom, color grading, palette-based
  materials, and GPU-driven instancing for large numbers of objects.
- Lighting uses a physically motivated but simplified model (energy-conserving diffuse and specular
  terms with stylization controls). Full PBR material workflows are not a goal.
- Procedural content support (geometry, palettes, textures, noise) is a core feature (see
  `liminal_procgen`).
- Every rendering feature MUST have golden-image tests (`docs/process/TESTING.md`).

## Alternatives considered

- **General-purpose PBR renderer:** broader appeal, but large effort in areas that do not serve the
  target games, and harder to verify.

## Consequences

Realistic-looking games are outside Liminal's intended use. Later features (GI-lite, volumetrics,
M10) are judged by whether they serve stylized looks.

## Verification

Roadmap review at each milestone kickoff. Golden tests per feature.
