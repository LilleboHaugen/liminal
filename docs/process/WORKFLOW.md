# Workflow

How a session works, from start to handoff. `AGENTS.md` has the summary; this is the full procedure.

## 1. Orient (≤ 5 minutes of reading)

1. `AGENTS.md` (loaded automatically in Claude Code through `CLAUDE.md`).
2. `docs/STATUS.md`: current milestone, task states, blockers.
3. The three newest files in `docs/journal/`.

## 2. Choose a task

Priority order:
1. A task in `review` that you did **not** work on: review it (`docs/process/REVIEW.md`).
2. A task `in-progress` with a handoff note and no active session (continue it).
3. The lowest-numbered `todo` task in the current milestone whose `depends_on` tasks are all `done`.

Set `status: in-progress` in the task frontmatter. If two agents work in parallel, each works on its own
branch `task/<ID>-<slug>`, and they must not take tasks that touch the same crate.

**Session boundary (ADR-0020).** Only the project owner starts sessions; parallel agents are parallel
sessions the owner started. One session does at most:
1. one review (priority 1 above). If it sets the task to `done`, merge the branch to `main` and push
   (§8). Then:
2. one implementation task (priority 2 or 3).

Never start sub-agents, background agents, forks or new sessions to review, implement or change the
status of a task. Read-only lookups that change no files and decide nothing may use helpers. Never
review a task you worked on in this session.

If no task is available, do not invent work. Write a journal entry saying so. If the milestone's
remaining work is not covered by tasks, the correct next task is to write those tasks (kickoff rule,
`docs/ROADMAP.md`).

## 3. Load context

Read the task file, then every spec, ADR and open question it references, then the code you will
change and its tests. If a referenced document contradicts the task, stop and resolve it (§6) before
writing code.

## 4. Implement

- **Test-first where feasible.** Write tests that carry `/// Verifies: <REQ-ID>` for the requirements
  the task covers. Confirm they fail for the right reason, then implement.
- **Small, verified steps.** Keep the build green between steps. Commit at each green state with a
  Conventional Commit message that includes the task ID.
- **Reference first, then optimize.** For algorithms that will be optimized, write the simple reference
  implementation and its tests first. The optimized version comes later (possibly in another task),
  with a differential test against the reference.
- **Keep files small.** Aim for source files under ~600 lines and functions under ~60 lines. Split
  modules by responsibility before they grow beyond that.
- During development use fast loops (`cargo check -p <crate>`, `cargo test -p <crate>`). Before
  handoff, run the full `cargo xtask ci`.

## 5. Verify

`cargo xtask ci` must pass. If a step cannot run in your environment (no GPU, missing tool), the task
cannot be marked `review`. Say exactly which step did not run and why, in the handoff note.

Also check by hand:
- Did you add a public item without docs or an example? (lint catches docs; examples are your job)
- Did any doc become stale? Search the docs for the names you changed.
- Did you add a dependency? It needs an ADR and an `architecture.toml` entry.

## 6. When reality disagrees with the plan

- **Spec is wrong or incomplete:** edit the spec in its own commit. Bump `spec_version`, never reuse or
  renumber requirement IDs, and mark removed requirements `(withdrawn: <reason>)` rather than deleting
  them. Explain the change in the journal.
- **A decision is needed that no ADR covers:** write an ADR with `status: proposed`. If it blocks you,
  set the task to `blocked` with a note and move to other work. The project owner or a review session
  accepts or rejects the ADR.
- **Task is too large:** split it. Create new task files for the remainder, link them with
  `depends_on`, and record the split in the handoff note. Never shrink scope silently.
- **You found a bug outside your task:** create a task file for it (`status: todo`, milestone = current)
  instead of fixing it on the side, unless it blocks you.

## 7. Hand off (always, even if unfinished)

1. Task file: update `status` (`in-progress`, `blocked` or `review`), tick acceptance criteria, and write
   the **Handoff** section: what is done, what is not, exact next step, traps you found.
2. Journal: new entry `docs/journal/YYYY-MM-DD-<task-id-or-topic>.md` (format in `docs/journal/README.md`).
3. Regenerate status (`cargo xtask status`, once it exists).
4. Commit everything. A session must never end with uncommitted changes and no note explaining them.
5. Stop task work once your implementation task is in `review`. End your final message to the owner
   with the next step as an instruction for the owner. The same answer applies whenever the owner asks
   what is next or for the status while a task written in this session is in `review`:
   > Start a new session. It will review `<task>`, then continue with `<next task>`.

   Do not start that work yourself, in this session or through another agent (ADR-0020).

## 8. Branching and commits

- One branch per task: `task/M1-T014-archetype-moves`. Merge to `main` only with CI green.
- Conventional Commits: `feat|fix|perf|refactor|test|docs|build|ci|chore(<crate or area>): <summary> (<task-id>)`.
- ADRs: `docs(adr): ADR-0019 <title>`.
- Do not rewrite published history on `main`.
