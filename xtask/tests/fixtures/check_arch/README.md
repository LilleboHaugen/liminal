# `check-arch` fixtures

`valid/` is a complete Cargo workspace with an `architecture.toml` that passes every `check-arch`
rule. Every other directory is an overlay: the test copies `valid/`, copies the overlay on top
(replacing files with the same path), and runs the `check-arch` rules on the result. Each overlay
must fail with exactly the rule in its name and no other rule
(`xtask/src/check_arch/fixture_tests.rs`).

The tests do not run cargo. They read `metadata.json` instead: the output of
`cargo metadata --format-version 1 --no-deps --offline`, run in a copy of `valid/` with the
overlay applied, with the copy's absolute path replaced by `/fixture` (cargo 1.99.1,
pretty-printed). An overlay that changes no `Cargo.toml` has no `metadata.json` of its own and
uses the one in `valid/`. When you change a `Cargo.toml` here, regenerate the `metadata.json`
files the same way. A stale `metadata.json` makes a fixture test pass or fail for the wrong
reason.

`valid/` covers: a proc-macro using a `build`-scope crate as a normal dependency, a
`cfg(windows)` target-specific dependency, a build-dependency and a build script, an integration
test (not a crate root), an `unsafe = "allow"` crate without the attribute, a same-layer
dependency listed in `same_layer_deps`, a layer 5 crate with a library and a binary target and a
`tool`-scope dev-dependency, and a registered crate that is not in the workspace (`planned`).

| Case | Change against `valid/` | Rule |
|---|---|---|
| `r030_registry` | `planned` has `kind = "library"` | XTASK-R030 |
| `r031_unregistered_member` | new workspace member `crates/extra`, not registered | XTASK-R031 |
| `r032_layer_violation` | `base` (layer 0) has a dev-dependency on `runtime` (layer 1) | XTASK-R032 |
| `r033_external_scope` | `runtime` (layer 1) lists and uses `serde`, a `tool`-scope crate | XTASK-R033 |
| `r034_dev_dependency` | `base` has a dev-dependency on `loom`, not in its `dev_external` | XTASK-R034 |
| `r035_missing_forbid` | `peer` (`unsafe = "forbid"`) has no `#![forbid(unsafe_code)]` | XTASK-R035 |
| `r036_tool_dependency` | `runtime` (layer 1) depends on `cli` (layer 5) | XTASK-R036 |
