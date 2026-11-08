# `trace` fixtures

`valid/` is a complete repository that passes `trace`: the spec `demo` is at maturity `usable`,
every non-withdrawn MUST has a test, a SHOULD is untested (information only), a withdrawn MUST is
untested, and a file excluded by `[[trace.exclude]]` in `ci/hygiene.toml` names an unknown ID.
Every other directory except `cases/` is an overlay: the test copies `valid/`, copies the overlay
on top, and runs the `trace` step. Each overlay must give exactly one finding, of the rule in its
name (`xtask/src/trace/fixture_tests.rs`).

| Case | Change against `valid/` | Rule |
|---|---|---|
| `r040_bad_row` | requirement row `DEMO-R005` with level `Must` | XTASK-R040 |
| `r041_unattached` | an annotation above a `struct` | XTASK-R041 |
| `r043_unknown_id` | an annotation naming `DEMO-R009`, which no spec defines | XTASK-R043 |
| `r043_withdrawn_id` | an annotation naming the withdrawn `DEMO-R003` | XTASK-R043 |
| `r044_untested_must` | an untested MUST `DEMO-R005` in the `usable` spec | XTASK-R044 |
| `r045_duplicate_id` | `DEMO-R001` defined twice | XTASK-R045 |

`xtask/src/trace/tests.rs` also runs `r044_untested_must` at every maturity: below `usable` it
passes, from `usable` on it fails.

`cases/annotations.rs` is scanned directly (`include_str!`); the test lists every annotation with
its line, IDs and attached function.

The annotations here name fixture-only and invalid requirement IDs. They are excluded from the
real repository's `trace` and `hygiene` scans by `ci/hygiene.toml` (`xtask/tests/fixtures`).
