# Tasks

A task is one unit of work, sized so a single agent session can usually finish it. Not finishing is
normal: leave a precise handoff note instead of rushing (AGENTS.md rule 2).

- File name: `<ID>-<kebab-slug>.md`, ID `M<milestone>-T<NNN>`, e.g. `M1-T014-archetype-moves.md`.
- Template: `_TEMPLATE.md`. Frontmatter fields: `docs/process/DOCUMENTATION.md`.
- Status flow: `todo → in-progress → review → done`, with `blocked` when waiting on a decision or
  another task. A task is set to `done` only by a reviewing session (`docs/process/REVIEW.md` §1).
- Tasks are never deleted. Obsolete tasks are set to `done` with a note "Obsolete: <reason>".
- Detailed task files exist only for the current and next milestone. Later work lives in
  `docs/ROADMAP.md` until that milestone's kickoff.

## Current task files

| ID | Title |
|---|---|
| [M0-T001](M0-T001-workspace-bootstrap.md) | Bootstrap the Cargo workspace |
| [M0-T002](M0-T002-xtask-ci-command.md) | xtask `ci` command |
| [M0-T003](M0-T003-check-docs.md) | Frontmatter parser and `check-docs` |
| [M0-T004](M0-T004-check-arch.md) | `check-arch` |
| [M0-T005](M0-T005-trace-and-hygiene.md) | `trace` and `hygiene` |
| [M0-T006](M0-T006-coverage-floors.md) | Coverage with per-crate floors |
| [M0-T007](M0-T007-github-actions-ci.md) | GitHub Actions CI |
| [M0-T008](M0-T008-status-generator.md) | `status` generator |
| [M0-T009](M0-T009-deny-and-doctor.md) | `cargo deny` policy and `doctor` |
| [M0-T010](M0-T010-m0-exit-review.md) | M0 exit review |
| [M1-T001](M1-T001-m1-kickoff.md) | M1 kickoff |
