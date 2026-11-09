# `status` fixture

`valid/` is a repository for `cargo xtask status` (`xtask/src/status/tests.rs`). Its generated
output is the approved snapshot `xtask/tests/snapshots/STATUS.md`. Change the snapshot only for
an intended format change, and say why in the journal (AGENTS.md rule 5).

What the fixture exercises:

| Content | Checks |
|---|---|
| Milestones M1 (all done), M2, M10 | current milestone is the lowest with an open task; `M2` sorts before `M10` |
| `M2-T010` after `M2-T003` | tasks sort by task number |
| Title `First \| piped title` | `\|` is escaped in table cells |
| `## M10 Tenth without a dash` in ROADMAP | a milestone without a `## M<n> — <name>` heading shows its ID alone |
| `alpha_lib`, `planned_lib` (layer 1), `demo` (layer 5) | crates sort by layer, then name; a crate without a spec is `planned` |
| `docs/specs/ghost.md` | a spec of a crate not in `architecture.toml` is ignored |
| `_TEMPLATE.md`, `docs/specs/README.md` | not task or spec files |
