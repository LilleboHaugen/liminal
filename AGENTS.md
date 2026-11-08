# AGENTS.md — Rules for every agent working on Liminal

Liminal is a from-scratch game engine in Rust, built and maintained by AI agents, for AI agents to
build fast, stylized, natively compiled games (3D first, 2D later) for macOS, Windows and Linux/SteamOS.

This file is binding. Read it fully at the start of every session. If a rule here conflicts with any
other document, this file wins; if you believe a rule is wrong, propose a change (see "Changing the
rules") instead of ignoring it.

## The one idea behind every rule

Agents write code faster than anyone can review it. The only defence against slow decay is that **every
claim is checkable by a machine**: requirements map to tests, optimized code is checked against a simple
reference implementation, rendering is checked against approved images, performance against recorded
baselines, architecture against `architecture.toml`. Your job is to add capability *and* the evidence that
it works. Code without evidence is not progress.

## Session protocol

1. **Orient.** Read `docs/STATUS.md` and the three most recent entries in `docs/journal/`.
2. **Pick work**, in this priority order:
   1. **Review** a task in status `review` that you did not work on (`docs/process/REVIEW.md`). Only a
      reviewer can set a task to `done`, so reviews unblock everything else.
   2. **Continue** an `in-progress` task that has a handoff note.
   3. **Start** the first `todo` task in the current milestone whose `depends_on` tasks are all `done`.
   Set the task's status accordingly. Never start a task whose dependencies are not `done`.
3. **Load context.** Read the task file, every spec and ADR it references, and the code you will touch.
   Do not rely on memory of earlier sessions; the repository is the only source of truth.
4. **Work test-first** where feasible: write tests for the requirement IDs you are covering, see them fail,
   then implement. Run `cargo xtask ci` before declaring anything finished (before `ci` exists, run
   the plain cargo commands listed under "Commands").
5. **Hand off.** Before the session ends — finished or not — update the task file (status + handoff notes),
   write a journal entry (`docs/journal/README.md`), and update any doc your change made stale.
6. **Stop at the session boundary (ADR-0020).** A session does at most one review, then one
   implementation task. When you set your task to `review`, stop task work. Never review your own
   work, and never start other agents (sub-agents, background agents, forks) to review or implement
   tasks. Only the project owner starts sessions. When the owner asks what is next or for the
   status, and a task written in this session is in `review`, answer: "Start a new session. It will
   review `<task>`, then continue with `<next task>`." Do not start that work yourself.

Full detail: `docs/process/WORKFLOW.md`.

## Non-negotiable rules

1. **No fake progress.** Never use `todo!()`, `unimplemented!()`, placeholder return values, disabled
   or weakened assertions, `#[ignore]` without a task reference, or mocks that make a test pass without
   exercising real behaviour. Unfinished work stays visibly unfinished: the task remains `in-progress`
   with a handoff note saying exactly what is missing.
2. **No session is expected to finish a feature.** Stop at a clean, tested intermediate state rather than
   rushing. A smaller verified step beats a larger unverified one. Never reduce a task's scope silently;
   scope changes are written into the task file with the reason.
3. **Every behaviour has a test; every requirement has a test.** Spec requirements have IDs
   (`ECS-R012`). Tests that verify them carry a doc-comment line `/// Verifies: ECS-R012`.
   `cargo xtask trace` fails if a MUST requirement of a crate at maturity `usable` or above has no test.
4. **Optimized code has an oracle.** Any non-trivial optimized algorithm (SIMD, parallel, greedy meshing,
   custom allocators, lock-free structures) ships with a simple reference implementation and a
   differential/property test proving they agree.
5. **Never edit an approved oracle to make a test pass.** Golden images, snapshots, coverage floors and
   performance baselines change only when the behaviour change is intended, and the journal entry must
   say why. Deleting or weakening a test needs the same justification.
6. **Respect the architecture.** Crates depend only on what `architecture.toml` allows. New crates are
   registered there first. `cargo xtask check-arch` enforces this.
7. **No new external dependency without an ADR** and an entry in `architecture.toml`
   (`docs/process/DEPENDENCIES.md`). Runtime dependencies face the strictest bar.
8. **Accepted ADRs are immutable.** To change a decision, write a new ADR that supersedes it.
9. **Specs are the contract.** If the implementation shows a spec is wrong, change the spec explicitly
   (bump `spec_version`, explain in the journal). Never let code and spec silently disagree.
10. **Simulation code is deterministic** (ADR-0009): fixed timestep, seeded RNG from `liminal_core`,
    deterministic hashing/iteration, `liminal_math::det` functions instead of `f32::sin` and friends.
11. **`unsafe` only where `architecture.toml` allows it**, every block with a `// SAFETY:` comment,
    every unsafe crate tested under Miri where Miri can run it (`docs/process/CODE_STYLE.md`).
12. **No `unwrap()` / `expect()` in library code** except `expect("invariant: …")` for states that are
    impossible by construction. Recoverable failures return errors with stable error codes (ADR-0017).
13. **Every `TODO` names a task:** `// TODO(M1-T014): …`. Untracked TODOs fail `cargo xtask hygiene`.
14. **Original content only.** Demo games, assets, names and levels are original. Liminal targets
    *genres* (voxel sandbox, open-world action-adventure, top-down) and never recreates a specific
    existing game, its characters, art, music or levels.
15. **Never claim something you did not verify.** Write "not verified on Windows" rather than implying it.
    If a check could not run (missing tool, no GPU), say so in the handoff.

## Definition of done (for a task)

- All acceptance criteria in the task file are met and checked off.
- `cargo xtask ci` passes locally with no failed or skipped steps. Install missing tools
  (`cargo xtask doctor` lists them). If a step truly cannot run on your machine (e.g. no GPU), the task
  stays `in-progress` and the handoff says which step and why. It is not moved to `review`.
- New public items are documented; primary entry points have a runnable `# Examples` doc test.
- Requirement IDs covered by the task have `Verifies:` tests.
- Spec, ADRs, task file, journal and any affected docs are updated in the same change.
- The task is set to `review`, not `done`. A new session, started by the project owner (ADR-0020),
  reviews it with `docs/process/REVIEW.md` and sets it to `done`.

## Where things live

| Path | What |
|---|---|
| `docs/VISION.md` | Why Liminal exists, goals, non-goals, target games |
| `docs/ARCHITECTURE.md` | Layers, module map, frame lifecycle, cross-cutting design |
| `architecture.toml` | Machine-checked registry: crates, layers, allowed deps, unsafe policy |
| `docs/ROADMAP.md` | Milestones with exit criteria |
| `docs/STATUS.md` | Current state: milestone, module maturity, active tasks (generated from M0-T008 on) |
| `docs/STATUS_NOTES.md` | Hand-written notes the generator cannot know: prerequisites, blockers (from M0-T008) |
| `docs/OPEN_QUESTIONS.md` | Decisions deliberately deferred, with the milestone that must resolve them |
| `docs/adr/` | Architecture Decision Records (immutable once accepted) |
| `docs/specs/` | One spec per crate: requirements with IDs, invariants, budgets |
| `docs/tasks/` | Task files: one unit of work each, with acceptance criteria |
| `docs/journal/` | Session logs and handoffs |
| `docs/process/` | How we work: workflow, testing, maturity, style, performance, deps, review, docs |
| `docs/GLOSSARY.md` | Terms with fixed meanings in this project |
| `crates/` | Engine crates (`liminal_*`) |
| `xtask/` | Repository automation (`cargo xtask …`) |
| `ci/` | Coverage floors, performance baselines, other CI data |
| `games/`, `examples/` | Original demo games and minimal examples (from M6 / M3 onward) |

## Commands (available once milestone M0 is complete)

| Command | Does |
|---|---|
| `cargo xtask ci` | Everything CI runs: fmt, clippy, tests, docs, and all checks below |
| `cargo xtask check-arch` | Workspace matches `architecture.toml` (layers, deps, unsafe policy) |
| `cargo xtask check-docs` | Frontmatter valid, links resolve, task/ADR references consistent |
| `cargo xtask trace` | Requirement → test traceability report and gate |
| `cargo xtask hygiene` | Forbidden patterns (untracked TODO, `todo!()`, `dbg!`, bare `#[ignore]`) |
| `cargo xtask coverage` | Per-crate coverage, enforced against floors in `ci/coverage.toml` |
| `cargo xtask status` | Regenerates `docs/STATUS.md` from spec and task frontmatter |
| `cargo xtask doctor` | Lists required external tools, their versions, and install hints |

Until M0 is done these do not exist; use plain `cargo fmt --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo test --workspace`, `cargo doc --workspace --no-deps`.

## Writing rules (docs, comments, commit messages)

- Write for an expert agent with zero memory of this project. Precise, complete, no filler.
- Use MUST / SHOULD / MAY (RFC 2119) in specs. State units, ranges and failure behaviour.
- No marketing words ("robust", "seamless", "powerful", "blazing fast"). Give numbers or say nothing.
- State uncertainty explicitly ("unverified", "assumption:"), never paper over it.
- Prefer tables and lists for facts. Reference code as `path/to/file.rs` and items as `crate::Type`.
- Commit messages: Conventional Commits with the task ID, e.g. `feat(ecs): archetype moves (M1-T021)`.

Full detail: `docs/process/DOCUMENTATION.md`.

## Changing the rules

These rules are changed only by an accepted ADR that names the rule and the reason. Edit this file in
the same change as the ADR.
