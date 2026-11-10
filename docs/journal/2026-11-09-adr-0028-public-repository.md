# 2026-11-09 — ADR-0028: Public repository (proposed)

**Session type:** planning
**Tasks touched:** none.

## Done
- The owner made the GitHub repository public and asked for the rules to tell agents what changes
  for CI and the rest.
- Read the repository settings with `gh api` (read only; table in ADR-0028 Context). Nothing was
  changed on GitHub.
- Wrote `docs/adr/0028-public-repository.md` (`proposed`): publication, reading CI, runner cost,
  workflow security, untrusted input from issues and pull requests, outward GitHub actions need
  the owner, outside contributions deferred. Edits on acceptance (AGENTS.md rule 17, REVIEW.md,
  `ci.yml` comment) are written out and not applied.
- `docs/adr/README.md` row 0028. `docs/OPEN_QUESTIONS.md`: new OQ-011 (outside contributions);
  OQ-007 notes that a self-hosted runner now needs its own ADR.
- `docs/STATUS_NOTES.md`: interim guidance until ADR-0028 is accepted; the "private repository"
  line about reading CI is corrected.

## Verified
- `gh repo view`: visibility `PUBLIC`. `gh api`: default token `read`, fork PR approval
  `first_time_contributors`, no secrets, no variables, no branch protection, secret scanning off.
- `git grep` over tracked files: no credentials, tokens or keys found.
- macOS 27.2, Rust 1.99.1: `cargo xtask ci` on `adr/0028-public-repository` passes (11 steps).
- Not verified: GitHub billing and concurrency limits for the owner's account (ADR-0028 states
  them as an assumption from GitHub documentation).

## Decisions
- AGENTS.md changes only through an ADR ("Changing the rules"), so the rule is a new ADR, not a
  direct edit. This session wrote it and does not review it (ADR-0026 Decision 5.4).
- `ci.yml` already meets every workflow rule; no workflow change is needed now.

## Oracle changes
- None.

## Problems and traps
- `main` does not contain ADR-0028 or the interim note in STATUS_NOTES until this branch is merged.

## Next
- Start a new owner-started session on branch `adr/0028-public-repository`. It reviews and accepts
  ADR-0028 (REVIEW.md §4), applies the edits on acceptance, merges, then continues with task work.
- Owner, optional: enable secret scanning and push protection in the repository settings.
