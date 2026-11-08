# Liminal

A from-scratch game engine in Rust, built by AI agents, for AI agents.

Liminal's purpose is to let an AI agent turn a description of a game into a fast, stylized game that
compiles to a native binary for macOS, Windows and Linux (including SteamOS). 3D comes first, beginning
with voxel worlds; 2D follows.

**Status:** planning complete, implementation starting at milestone M0. See `docs/STATUS.md`.

## Start here

- Agents: read `AGENTS.md` first. It is binding.
- Why this exists and what it is for: `docs/VISION.md`
- How it is built: `docs/ARCHITECTURE.md`
- What happens next: `docs/ROADMAP.md`, `docs/STATUS.md`

## Requirements (planned)

- Rust stable toolchain, pinned in `rust-toolchain.toml` (added in task M0-T001)
- macOS on Apple Silicon (Metal 3+), Windows 10/11 x64 or Linux x64 with a Vulkan 1.3 driver

## License

Dual-licensed under either of

- Apache License, Version 2.0 (`LICENSE-APACHE`)
- MIT license (`LICENSE-MIT`)

at your option. Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above,
without any additional terms or conditions.
