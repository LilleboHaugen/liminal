# `check-docs` fixtures

`valid/` is a complete repository that passes every `check-docs` rule. Every other directory is
an overlay: the test copies `valid/`, copies the overlay on top (replacing files with the same
path), and runs `check-docs`. Each overlay must fail with exactly the rule in its name and no
other rule (`xtask/src/check_docs/fixture_tests.rs`).

| Case | Change against `valid/` | Rule |
|---|---|---|
| `r020_frontmatter_syntax` | `status:todo` (no space) in a task | XTASK-R020 |
| `r021_adr_fields` | ADR status `approved` | XTASK-R021 |
| `r022_spec_fields` | spec `layer` disagrees with `architecture.toml` | XTASK-R022 |
| `r023_task_fields` | dependency cycle M0-T001 → M0-T002 → M0-T001 | XTASK-R023 |
| `r024_broken_link` | link to a missing file | XTASK-R024 |
| `r025_adr_index` | ADR-0002 missing from the ADR index | XTASK-R025 |
| `r026_member_spec` | workspace member without a spec | XTASK-R026 |
| `r027_spec_index` | spec index maturity disagrees with the spec | XTASK-R027 |
| `r028_task_requirements` | task `requirements` names `DEMO-R002`, which no spec defines | XTASK-R028 |
