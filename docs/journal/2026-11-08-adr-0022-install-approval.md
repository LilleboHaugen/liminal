# 2026-11-08 — ADR-0022: Owner approval before installing software

**Session type:** planning
**Tasks touched:** none.

## Done
- The owner asked: "Make sure agents always asks me before installing stuff on my mac, and their
  reasonings." Wrote ADR-0022 (accepted, owner-approved).
- `AGENTS.md`: new non-negotiable rule 16; "Definition of done" no longer tells agents to install
  missing tools on their own.
- `docs/process/DEPENDENCIES.md` (external tools), `docs/adr/README.md` (index row).
- New `.claude/settings.json` with a `permissions.ask` list (cargo install, rustup changes, brew,
  pip, global npm, installers, `sudo`, …) so Claude Code prompts the owner before these run.

## Verified
- Docs and harness config only; no code changed. `cargo xtask check-docs`: ok.
- The `permissions.ask` list was not exercised in a live prompt in this session. `curl … | sh`
  cannot be matched by a prefix rule; the written rule covers it.

## Decisions
- ADR-0022. Committed on `main` from a separate worktree, because a session had uncommitted work
  on `task/M0-T006-coverage`. M0-T006 picks it up when it merges `main`.

## Oracle changes
- None.

## Problems and traps
- M0-T006 needs `cargo-llvm-cov`. Under ADR-0022 the session must ask the owner (with reasons)
  before `cargo install cargo-llvm-cov`.

## Next
- Unchanged: continue M0-T006.
