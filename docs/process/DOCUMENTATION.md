# Documentation

Docs in Liminal are part of the product. An agent with no memory of this project must be able to act
correctly from the docs alone. A wrong doc is worse than a missing one, so any change that makes a doc
false updates that doc in the same commit.

## Audience and style

- Audience: an expert agent with zero project memory and a limited context window.
- Normative words: MUST, MUST NOT, SHOULD, SHOULD NOT, MAY (RFC 2119), in specs and ADRs.
- Concrete: numbers with units, exact names, file paths, error codes. No adjectives where a number fits.
- No filler, no marketing words, no emojis, no restating what the code obviously does.
- Explicit uncertainty: "unverified", "assumption:", "measured on reference machine only".
- One fact, one place. Link instead of copying. Generated docs (STATUS.md) are never edited by hand.

## Document types

| Type | Location | Template | Mutable? |
|---|---|---|---|
| ADR | `docs/adr/NNNN-slug.md` | `docs/adr/0000-template.md` | Only while `proposed`. Superseded instead of edited |
| Spec | `docs/specs/<crate>.md` | `docs/specs/_TEMPLATE.md` | Yes, with `spec_version` bump |
| Task | `docs/tasks/<ID>-slug.md` | `docs/tasks/_TEMPLATE.md` | Yes, until `done` |
| Journal | `docs/journal/YYYY-MM-DD-<task-id-or-topic>.md` | `docs/journal/README.md` | No (append-only, new entries only) |
| Guide | `docs/guides/<slug>.md` | — | Yes (from M2) |
| Rustdoc | In code | — | With the code |

## Frontmatter

Specs, tasks and ADRs start with frontmatter between `---` lines, in a **restricted YAML subset** so it
can be parsed without a YAML library (xtask parser, task M0-T003):

- One `key: value` per line. Keys are `snake_case`. Values are strings, integers or dates (`YYYY-MM-DD`).
- Lists are inline only: `key: [a, b, c]`. An empty list is `[]`.
- No nesting, no multi-line values, no anchors. `#` starts a comment only after whitespace.
- No quoting or escaping exists. Therefore scalar values MUST NOT contain ` #` (space + hash), and list
  items MUST NOT contain `,`, `[`, `]`, `{` or `}`. Leading and trailing spaces and tabs are trimmed;
  other whitespace (U+00A0, U+3000) is part of the value. Control characters, U+2028, U+2029 and
  U+FEFF (byte order mark, also at the start of the file) are rejected.
- `cargo xtask check-docs` enforces the subset (XTASK-R020). The exact grammar, including what it
  rejects (blank and comment-only lines inside frontmatter, values containing `: `, starting with a
  YAML indicator such as `"`, `|` or `- `, or ending with `:`), is in `docs/specs/xtask.md` ("`check-docs` details").

### ADR fields
```
---
id: ADR-0006
title: Own GPU abstraction over Vulkan 1.3 and Metal
status: accepted            # proposed | accepted | superseded | rejected
date: 2026-11-08
supersedes: []
superseded_by: []
---
```

### Spec fields
```
---
crate: liminal_ecs
req_prefix: ECS
layer: 1
maturity: planned           # planned | experimental | usable | stable | optimized
spec_version: 1
last_reviewed: 2026-11-08
adrs: [ADR-0008, ADR-0009]
---
```

### Task fields
```
---
id: M1-T014
title: Archetype moves on component add/remove
milestone: M1
status: todo                # todo | in-progress | blocked | review | done
crates: [liminal_ecs]
depends_on: [M1-T012]
requirements: [ECS-R020, ECS-R021]
---
```

## Rustdoc

- Crate root docs: one-paragraph purpose, link to the spec, feature flags, and a minimal example.
- Every public item: what it is, invariants, errors (with codes), panics (if any, only for invariant
  violations), and complexity where relevant (`O(1)` amortized, etc.).
- `# Examples` on primary entry points. Examples are compiled and run as doc tests.
- `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS="-D warnings"` must pass, so broken intra-doc
  links fail CI.
