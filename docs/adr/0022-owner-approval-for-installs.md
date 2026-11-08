---
id: ADR-0022
title: Owner approval before installing software on the owner's machine
status: accepted
date: 2026-11-08
supersedes: []
superseded_by: []
---

# ADR-0022: Owner approval before installing software on the owner's machine

## Context

Agents run on the project owner's own Mac. `AGENTS.md` ("Definition of done") told agents to
"Install missing tools (`cargo xtask doctor` lists them)", and `docs/process/DEPENDENCIES.md` lists
tools "agents install to run checks" (`cargo-llvm-cov`, `cargo-deny`, `cargo-nextest`, Miri,
`cargo-fuzz`, Vulkan SDK, later a shader toolchain). That wording lets an agent change the owner's
system without asking: binaries in `~/.cargo/bin`, rustup toolchains and components, Homebrew
packages, SDKs with system-wide effects. Each of these takes disk space, can change the behaviour of
other projects on the machine, and some need `sudo`.

The owner asked on 2026-11-08: "Make sure agents always asks me before installing stuff on my mac,
and their reasonings."

## Decision

1. **An agent MUST ask the project owner and get an explicit yes before it installs, upgrades or
   removes any software on the owner's machine.** This covers, among others:
   - `cargo install`, `cargo binstall`, `cargo uninstall`;
   - `rustup toolchain install|uninstall`, `rustup component add|remove`, `rustup target add|remove`,
     `rustup update`, `rustup self …`;
   - `brew`, `port`, `pip`/`pip3`/`pipx install`, `npm|pnpm|yarn` global installs, `gem install`;
   - `curl … | sh` or any downloaded installer, `.pkg`/`.dmg` installers, `xcode-select --install`,
     `softwareupdate`;
   - any command run with `sudo`.
2. **The request states the reasoning.** Before running the command, the agent tells the owner:
   - **What:** the tool and version, and the exact command it will run.
   - **Why:** which task, check or requirement needs it (e.g. "M0-T006: `cargo xtask coverage`
     needs `cargo-llvm-cov`"), and what happens without it (which step stays skipped).
   - **Where and how big:** where it installs (e.g. `~/.cargo/bin`, Homebrew prefix), whether it
     needs `sudo` or changes system settings, and the approximate download size if known.
   - **Alternatives:** any way to do the work without installing (e.g. the check runs in CI).
   - **Undo:** the command that removes it again.
3. **Approval is per request.** A yes covers the tools and commands named in that request, in that
   session. It does not carry over to other tools or later sessions.
4. **If the owner says no, or is not there to answer,** the agent does not install. The step that
   needs the tool stays not run, and the rules that already apply to a check that cannot run take
   effect: the task stays `in-progress` and the handoff names the missing tool and the command the
   owner would run (`AGENTS.md`, "Definition of done", rule 15).
5. **Not covered:** cargo fetching and building crate dependencies declared in the workspace into
   `~/.cargo/registry` and `target/` (these are covered by the dependency policy, ADR-0003), and
   tools installed on CI runners by the CI workflow (they do not touch the owner's machine).
6. Where the agent's harness supports it, the repository configures it to require a confirmation
   for the install commands in item 1. For Claude Code this is the `permissions.ask` list in
   `.claude/settings.json`. This is a safety net; item 1 applies whether or not a command is on that
   list.

## Alternatives considered

- **Do nothing (agents install what `doctor` lists).** Rejected: the owner explicitly asked for
  control over their machine.
- **Pre-approve a fixed list of tools (`cargo-llvm-cov`, `cargo-deny`, …).** Rejected: the owner
  asked to be asked every time, with reasons. A list could be added later by a new ADR.
- **Only a harness permission rule, no written rule.** Rejected: it does not cover agents running in
  other harnesses, cannot match every install path (`curl | sh`), and does not require reasons.
- **Install into a project-local prefix (`--root target/tools`) without asking.** Rejected for now:
  it still downloads and runs third-party code on the owner's machine, which is what the owner wants
  to approve.

## Consequences

- The owner sees and approves every change to their system, with the reasons.
- Sessions can stall on a missing tool when the owner does not answer; those tasks stay
  `in-progress` instead of reaching `review`. `cargo xtask doctor` (M0-T009) makes the list of
  missing tools and their install commands easy to hand to the owner in one request.
- Revisit if the prompts become a burden (then consider a pre-approved list or a project-local
  tool prefix, by a new ADR).
- Owner approval: requested by the project owner on 2026-11-08 (quoted under Context). This ADR
  changes rules in `AGENTS.md` (new non-negotiable rule 16, "Definition of done").

## Verification

- `.claude/settings.json` lists the commands of item 1 under `permissions.ask`, so Claude Code
  prompts the owner before running them.
- Review item: a journal entry or handoff that mentions installing a tool also records the owner's
  approval. An install without recorded approval is a B2 finding (ADR-0021).
