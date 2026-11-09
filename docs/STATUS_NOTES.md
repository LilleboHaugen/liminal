# Status notes

Hand-written notes that `cargo xtask status` cannot derive from task and spec frontmatter or
`architecture.toml`. The generated overview is `docs/STATUS.md` (do not edit it by hand). Keep
this file short: facts an agent needs at session start, not history (that is in `docs/journal/`).

## Next step for the project owner

M0-T011 is `done` (reviewed 2026-11-09). M0-T010 (promotion of `xtask` to `usable`, last M0
exit criterion) continues in the session that reviewed M0-T011; it did not implement M0-T011.
ADR-0020.

## Prerequisites the project owner handles (not agent tasks)

- ~~Install the Rust toolchain (`rustup`).~~ Done 2026-11-08: rustup via Homebrew
  (`/opt/homebrew/opt/rustup/bin` on PATH), stable 1.99.1 installed. M0-T001 pinned `1.99.1` in
  `rust-toolchain.toml`.
- ~~`git init` and create the GitHub repository.~~ Done 2026-11-08:
  `https://github.com/LilleboHaugen/rust-game-engine` (branch `main`).

## Tools every agent machine needs

Once the matching task lands; otherwise `cargo xtask ci` reports skipped steps and the task cannot
go to `review`. Installs need the owner's approval first (ADR-0022).

- `cargo-llvm-cov` (from M0-T006).
- `cargo-deny` 0.20.2 (from M0-T009; on the owner's Mac via Homebrew, 2026-11-09).
- `cargo xtask doctor` lists every tool, its version and the install hint.
- Later: nightly toolchain + Miri (M1).
- To read CI runs of the private repository: GitHub CLI `gh`, logged in (installed and logged in
  on the owner's Mac on 2026-11-08).

## Review follow-up tasks (ADR-0021)

Non-blocking review findings of M0 tasks are collected in M1 tasks; the M1 kickoff (M1-T001)
places them in the M1 order.

| Reviewed task | Follow-up task |
|---|---|
| M0-T003 (`check-docs`) | M1-T002 |
| M0-T004 (`check-arch`) | M1-T003 |
| M0-T005 (`trace`, `hygiene`) | M1-T004 |
| M0-T006 (`coverage`) | M1-T005 |
| M0-T007 (GitHub Actions CI) | M1-T006 |
| M0-T008 (`status`) | M1-T007 |
| M0-T009 (`deny`, `doctor`), review 1 | M1-T008 |
| M0-T009 (`deny`, `doctor`), review 2 | M1-T009 |
| M0-T011 (real-binary tests) | M1-T010 |

## Module notes

- Only `xtask` exists, and only `xtask` has a spec (`docs/specs/xtask.md`). Its line coverage is
  98.05 % (floor 97.6 in `ci/coverage.toml`; measured on macOS in M0-T011). `xtask` stays `planned` until a spec review exists.
- CI (GitHub Actions, M0-T007) runs `cargo xtask ci` on every push on macOS (arm64), Windows and
  Linux; `coverage` and `deny` run in their own Linux jobs.

## Blockers needing an owner decision

None.
