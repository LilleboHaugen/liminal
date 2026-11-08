---
crate: liminal_<name>
req_prefix: <PREFIX>
layer: <0-5>
maturity: planned
spec_version: 1
last_reviewed: YYYY-MM-DD
adrs: []
---

# Spec: `liminal_<name>`

## 1. Purpose

One paragraph: what this crate is for and why it exists as a separate crate.

## 2. Scope

**In scope:** bullet list.

**Out of scope / non-goals:** bullet list. Name the crate responsible for each excluded area, if any.

## 3. Dependencies

- Engine crates: must match `architecture.toml`.
- External crates: with ADR references.

## 4. Concepts

Terms and the data model this crate introduces, defined precisely. Diagrams as ASCII if useful.

## 5. Public API

The authoritative sketch of the public surface: types, traits, key functions with signatures and
semantics. Code blocks in Rust. Anything not listed here is not public without a spec change.

## 6. Requirements

RFC 2119 keywords. Every requirement must be testable. Group by area. Leave numbering gaps between groups.

| ID | Level | Requirement |
|---|---|---|
| PREFIX-R001 | MUST | … |
| PREFIX-R002 | SHOULD | … |

## 7. Invariants

Properties that always hold. Each is asserted (`debug_assert!` or type system) and covered by property
tests where possible.

- INV-1: …

## 8. Error codes

| Code | Meaning | Help |
|---|---|---|
| LIM-PREFIX-0001 | … | … |

## 9. Performance budgets

| Scenario | Budget | Machine | Source of number |
|---|---|---|---|
| … | ≤ X ms | Reference | Baseline `ci/perf/…` / competitive benchmark |

Write "None yet: set at `stable` from baselines" if not yet measured. Never invent numbers.

## 10. Test plan

Which test kinds apply (`docs/process/TESTING.md`), and the oracle for each optimized path.

| Area | Test kinds | Oracle |
|---|---|---|
| … | unit, property, differential | reference implementation `…` |

## 11. Determinism

If the crate is `deterministic = true`: how it meets ADR-0009, and whether cross-platform bit-identity
has been verified.

## 12. Open questions

Unresolved points, each with the task or milestone that resolves it.

## 13. Maturity

Current level and the promotion checklist for the next level, with evidence links.

- [ ] Gate … (evidence: …)

## 14. History

| spec_version | Date | Change | By task |
|---|---|---|---|
| 1 | YYYY-MM-DD | Initial spec | Mx-Txxx |
