# `hygiene` fixtures

`valid/` is a complete repository that passes `hygiene`: tracked task markers, `ignore` attributes
whose reasons name tasks, forbidden patterns inside string literals, a violation in a directory
excluded by `ci/hygiene.toml`, and a violation in a `.rs` file outside every workspace member
(`scripts/`). Every other directory except `cases/` is an overlay: the test copies `valid/`, copies
the overlay on top, and runs `hygiene`. Each overlay must give exactly the findings its test lists
(`xtask/src/hygiene/fixture_tests.rs`).

| Case | Change against `valid/` | Rule |
|---|---|---|
| `r050_untracked_marker` | a task marker without a task | XTASK-R050 |
| `r050_forbidden_macro` | a forbidden debugging macro | XTASK-R050 |
| `r050_bare_ignore` | an `ignore` attribute without a reason | XTASK-R050 |
| `r051_unknown_task` | a marker naming `M9-T999`, which does not exist | XTASK-R051 |
| `r051_done_task` | a marker naming `M0-T001`, which is `done` | XTASK-R051 |
| `r052_exclusion` | an exclusion with a blank reason, and one whose path does not exist | XTASK-R052 |

`cases/` holds single `.rs` files that `xtask/src/hygiene/tests.rs` scans directly
(`include_str!`). Each test lists the expected rule and line for every finding, so a line added
to or removed from a case file changes the expected list.

These files contain the patterns that `hygiene` forbids. They are excluded from the real
repository's scan by `ci/hygiene.toml` (`xtask/tests/fixtures`), and no `Cargo.toml` here is a
workspace member. Test inputs with forbidden patterns live here, never in string literals of
`.rs` files (M0-T005 notes).
