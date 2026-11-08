//! Helpers shared by unit tests: temporary directories and fixture copies.

use std::path::{Path, PathBuf};

/// A fresh, empty directory under the system temp directory, removed on drop.
pub(crate) struct TempRoot(PathBuf);

impl TempRoot {
    /// Creates `<temp>/xtask-<pid>-<name>`, emptying it first. `name` must be unique per test.
    pub(crate) fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("xtask-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    /// The directory.
    pub(crate) fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Copies the contents of `from` into `to` recursively, overwriting files that exist in both.
pub(crate) fn copy_dir(from: &Path, to: &Path) {
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            std::fs::create_dir_all(&target).unwrap();
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}
