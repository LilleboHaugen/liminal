---
id: ADR-0001
title: Record architecture decisions
status: accepted
date: 2026-11-08
supersedes: []
superseded_by: []
---

# ADR-0001: Record architecture decisions

## Context

Liminal is developed by many AI agent sessions over a long time. No session remembers earlier ones.
Without written decisions, later sessions re-decide settled questions, undo good choices they do not
understand, or make inconsistent ones.

## Decision

- Every significant decision MUST be recorded as an ADR in `docs/adr/` using `0000-template.md`.
  Significant means: affects more than one crate, adds an external dependency, changes a rule in
  `AGENTS.md` or the process docs, or would be costly to reverse.
- Accepted ADRs are immutable apart from the `status` and `superseded_by` fields. Changes happen
  through a superseding ADR.
- Decisions deliberately deferred are tracked in `docs/OPEN_QUESTIONS.md`.
- Specs reference the ADRs they implement in their `adrs` frontmatter field.

## Alternatives considered

- **Decisions in commit messages only:** not discoverable, and agents do not read history.
- **One long design document:** grows stale and hides when and why things changed.

## Consequences

Some overhead per decision. In exchange, agents can find the reasoning behind any structural choice in
one place.

## Verification

`cargo xtask check-docs` validates ADR frontmatter and supersession links. Review checks that
significant changes come with an ADR.
