//! Runs the real `xtask` binary and checks its exit codes and output streams (XTASK-R002).

use std::process::{Command, Output};

fn xtask(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .output()
        .expect("test setup: the xtask binary must start")
}

/// Verifies: XTASK-R002
#[test]
fn help_and_no_arguments_print_the_command_list_and_exit_0() {
    for args in [&[][..], &["help"][..]] {
        let output = xtask(args);
        assert_eq!(output.status.code(), Some(0), "args: {args:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Commands:"), "stdout: {stdout}");
        for command in xtask::COMMANDS {
            assert!(stdout.contains(command.name), "stdout: {stdout}");
        }
        assert_eq!(String::from_utf8_lossy(&output.stderr), "");
    }
}

/// Verifies: XTASK-R002
#[test]
fn unknown_command_and_unknown_option_print_usage_to_stderr_and_exit_2() {
    for args in [
        &["nonsense"][..],
        &["--nonsense"][..],
        &["help", "--nonsense"][..],
    ] {
        let output = xtask(args);
        assert_eq!(output.status.code(), Some(2), "args: {args:?}");
        assert!(output.stdout.is_empty(), "args: {args:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.starts_with("error: "), "stderr: {stderr}");
        assert!(stderr.contains("Usage: cargo xtask"), "stderr: {stderr}");
    }
}

#[cfg(unix)]
#[test]
fn non_utf8_argument_is_a_usage_error() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg(OsStr::from_bytes(b"\xff"))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("not valid UTF-8"));
}
