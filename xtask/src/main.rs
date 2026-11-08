//! `cargo xtask` entry point. All logic lives in the library; see `xtask::run`.
#![forbid(unsafe_code)]
#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "CLI output is xtask's job"
)]

use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = Vec::new();
    for arg in std::env::args_os().skip(1) {
        match arg.into_string() {
            Ok(arg) => args.push(arg),
            Err(arg) => {
                eprintln!("error: argument `{}` is not valid UTF-8", arg.display());
                return ExitCode::from(xtask::Exit::Usage.code());
            }
        }
    }
    let exit = xtask::run(
        &args,
        &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock(),
    );
    ExitCode::from(exit.code())
}
