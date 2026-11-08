//! Locating the repository. The shared repo-model loader (`docs/specs/xtask.md` §3) grows here.

use std::path::{Path, PathBuf};

/// The repository root: the parent of the `xtask/` manifest directory.
///
/// Fixed at compile time, so `cargo xtask` works the same from any directory (XTASK-R001).
pub(crate) fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("invariant: xtask/ is a direct child of the repository root")
        .to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_contains_the_workspace_manifest_and_xtask() {
        let root = root();
        assert!(root.join("Cargo.toml").is_file());
        assert!(root.join("xtask/Cargo.toml").is_file());
        assert!(root.join("architecture.toml").is_file());
    }
}
