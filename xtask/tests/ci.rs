//! Runs the real `xtask` binary for `ci` usage errors (XTASK-R007, XTASK-R008).
//!
//! The full `ci` run is not started from tests: its `test` step would run this test again. The real
//! steps run when `cargo xtask ci` runs locally and in CI (`docs/specs/xtask.md` §10).

use std::process::{Command, Output};

fn xtask(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .output()
        .expect("test setup: the xtask binary must start")
}

/// Verifies: XTASK-R007, XTASK-R008
#[test]
fn unknown_step_names_exit_2_without_running_anything() {
    for args in [
        &["ci", "--only", "nonsense"][..],
        &["ci", "--skip", "fmt,nonsense"][..],
        &["ci", "--allow-skip=nonsense"][..],
        &["ci", "--bogus"][..],
    ] {
        let output = xtask(args);
        assert_eq!(output.status.code(), Some(2), "args: {args:?}");
        assert!(output.stdout.is_empty(), "args: {args:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.starts_with("error: "), "stderr: {stderr}");
        assert!(stderr.contains("usage: cargo xtask ci"), "stderr: {stderr}");
    }
}
