# Specs

One spec per crate: `docs/specs/<crate_name>.md`, written from `_TEMPLATE.md`. The spec is the contract
for the crate. Code, tests and reviews are checked against it.

Rules:
- A crate gets a spec before it gets code. The spec is reviewed (`docs/process/REVIEW.md` §2) before
  the crate can become `experimental`.
- Requirement IDs (`<PREFIX>-R<NNN>`) are permanent. They are never renumbered or reused. Withdrawn
  requirements stay in the list, marked `(withdrawn: <reason>)`.
- Number requirements in groups with gaps between them (e.g. R001–R019 for one area, R020–R029 for the
  next), so related requirements can be added later without renumbering.
- Every behavioural change to the spec bumps `spec_version` and is explained in the journal.
- Performance budgets are numbers with units, scenario and machine, set from measurements.

## Index

| Spec | Crate | Prefix | Maturity |
|---|---|---|---|
| [liminal_telemetry.md](liminal_telemetry.md) | `liminal_telemetry` | TELE | planned |
| [xtask.md](xtask.md) | `xtask` | XTASK | usable |

Specs for M1 crates are written by dedicated M1 tasks, which the M1 kickoff task (M1-T001) creates.

## Requirement prefixes (reserved)

| Prefix | Crate | Prefix | Crate |
|---|---|---|---|
| CORE | liminal_core | GPU | liminal_gpu |
| MATH | liminal_math | SHDR | liminal_shader |
| MEM | liminal_memory | REND | liminal_render |
| TELE | liminal_telemetry | VOX | liminal_voxel |
| REFL | liminal_reflect (+derive) | VOXR | liminal_voxel_render |
| JOBS | liminal_jobs | PHYS | liminal_physics |
| ECS | liminal_ecs | AUD | liminal_audio |
| APP | liminal_app | TEXT | liminal_text |
| SER | liminal_serial | UI | liminal_ui |
| INPT | liminal_input | ANIM | liminal_anim |
| ASST | liminal_assets | PRB | liminal_probe |
| SCN | liminal_scene | TKIT | liminal_testkit |
| PGEN | liminal_procgen | ENG | liminal_engine |
| PLAT | liminal_platform | CLI | liminal_cli |
| MCP | liminal_mcp | COOK | liminal_cook |
| XTASK | xtask | | |

These prefixes are also the `AREA` part of error codes (`LIM-ECS-0001`, ADR-0017).
