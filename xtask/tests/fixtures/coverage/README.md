# Coverage fixtures

Inputs for `xtask/src/repo/coverage.rs` and `xtask/src/coverage.rs` unit tests. No tool is needed
to run those tests (M0-T006).

| Fixture | What it contains | Tested behaviour |
|---|---|---|
| `llvm-cov-summary.json` | Output of `cargo llvm-cov --workspace --all-features --json --summary-only` (cargo-llvm-cov 0.9.1, LLVM export format 3.1.0) on this repository at M0-T006, with the repository root replaced by `/work/liminal`. Pretty-printed; otherwise unchanged. | Parsing a real export: the per-crate sum of `summary.lines` of every file below the crate's manifest directory |
