# `deny` fixture (XTASK-R082)

`app/` (MIT OR Apache-2.0) depends on `copyleft/` (GPL-3.0-only) by path. The test in
`xtask/tests/deny_policy.rs` copies both to a temporary directory and runs
`cargo deny --offline --config <repository>/deny.toml --manifest-path app/Cargo.toml check licenses`:
it must fail on `fixture_copyleft`, and pass after the copy's licence is changed to `MIT`.
Nothing here is a workspace member of the Liminal repository.

The sources (XTASK-R083) and advisories (XTASK-R084) tests in the same file build their fixtures
in a temporary directory instead (a git repository and an advisory database have to be created
with `git` at test time), so they have no files here.
