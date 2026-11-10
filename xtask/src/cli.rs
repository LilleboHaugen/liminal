//! Argument parsing, the command table and the `help` command (XTASK-R002).
//!
//! Parsing is hand-written (no CLI-parsing crate, `docs/specs/xtask.md` §3). The first argument
//! selects a command from [`COMMANDS`]; the remaining arguments are passed to that command, which
//! parses its own options and reports unknown ones as [`CommandError::Usage`].

use std::fmt;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Process exit status of an xtask invocation (XTASK-R007).
///
/// # Examples
///
/// ```
/// use xtask::Exit;
///
/// assert_eq!(Exit::Success.code(), 0);
/// assert_eq!(Exit::Failure.code(), 1);
/// assert_eq!(Exit::Usage.code(), 2);
///
/// // A binary turns the result of `run` into its process exit code.
/// let exit = xtask::run(&["help".to_string(), "--bogus".to_string()], &mut Vec::new(), &mut Vec::new());
/// assert_eq!(exit, Exit::Usage);
/// let _code = std::process::ExitCode::from(exit.code());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exit {
    /// The command ran and every check passed. Exit code 0.
    Success,
    /// The command ran and at least one check failed, or output could not be written. Exit code 1.
    Failure,
    /// The command line was invalid. Exit code 2.
    Usage,
}

impl Exit {
    /// The numeric process exit code: 0, 1 or 2.
    #[must_use]
    pub fn code(self) -> u8 {
        match self {
            Exit::Success => 0,
            Exit::Failure => 1,
            Exit::Usage => 2,
        }
    }
}

/// Why a command did not complete.
#[derive(Debug)]
pub enum CommandError {
    /// The arguments were invalid. The message names the offending argument; [`run`] prints it
    /// with the usage text to stderr and exits with [`Exit::Usage`].
    Usage(String),
    /// Writing output failed. [`run`] reports it on stderr and exits with [`Exit::Failure`].
    Io(io::Error),
    /// A file the command must write (e.g. the `ci` summary) could not be written. [`run`] reports
    /// it on stderr and exits with [`Exit::Failure`].
    File {
        /// The file that could not be written.
        path: PathBuf,
        /// The underlying error.
        error: io::Error,
    },
    /// The repository could not be read into a model (`docs/specs/xtask.md` §8). [`run`] reports
    /// it on stderr and exits with [`Exit::Failure`].
    Internal {
        /// The error code, `LIM-XTASK-nnnn`.
        code: &'static str,
        /// The file concerned, what went wrong and how to fix it.
        message: String,
    },
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::Usage(message) => write!(f, "{message}"),
            CommandError::Io(error) => write!(f, "could not write output: {error}"),
            CommandError::File { path, error } => {
                write!(f, "could not write {}: {error}", path.display())
            }
            CommandError::Internal { code, message } => write!(f, "{code}: {message}"),
        }
    }
}

impl From<io::Error> for CommandError {
    fn from(error: io::Error) -> Self {
        CommandError::Io(error)
    }
}

/// Signature of a command implementation: its arguments (without the command name) and the
/// writer for its normal output. Returns [`Exit::Success`] or [`Exit::Failure`] when it ran.
pub type CommandFn = fn(args: &[String], out: &mut dyn Write) -> Result<Exit, CommandError>;

/// One entry of the command table.
///
/// # Examples
///
/// ```
/// let help = xtask::COMMANDS
///     .iter()
///     .find(|command| command.name == "help")
///     .expect("`help` is registered");
///
/// // `run` takes the arguments after the command name and writes normal output to `out`.
/// let mut out = Vec::new();
/// let exit = (help.run)(&[], &mut out).expect("`help` takes no arguments");
/// assert_eq!(exit, xtask::Exit::Success);
/// assert!(String::from_utf8(out).unwrap().contains(help.summary));
///
/// // Invalid arguments are a usage error, not output.
/// assert!(matches!(
///     (help.run)(&["--bogus".to_string()], &mut Vec::new()),
///     Err(xtask::CommandError::Usage(_))
/// ));
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Command {
    /// Name typed on the command line, e.g. `help`.
    pub name: &'static str,
    /// One-line description shown by `help`.
    pub summary: &'static str,
    /// The implementation.
    pub run: CommandFn,
}

/// Every implemented command, in the order `help` lists them. Later commands are added here by the
/// task that implements them (`docs/specs/xtask.md` §5); a command is never listed before it works.
///
/// # Examples
///
/// ```
/// let names: Vec<&str> = xtask::COMMANDS.iter().map(|command| command.name).collect();
/// assert_eq!(
///     names,
///     ["help", "ci", "check-arch", "check-docs", "trace", "hygiene", "coverage", "status", "doctor"]
/// );
/// ```
pub const COMMANDS: &[Command] = &[
    Command {
        name: "help",
        summary: "Print this list of commands.",
        run: help,
    },
    Command {
        name: "ci",
        summary: "Run every CI step in order; write target/xtask/ci-summary.json.",
        run: crate::ci::command,
    },
    Command {
        name: "check-arch",
        summary: "Check the crate graph against architecture.toml (XTASK-R030 … R039).",
        run: crate::check_arch::command,
    },
    Command {
        name: "check-docs",
        summary: "Check doc frontmatter, links and indexes (XTASK-R020 … R028).",
        run: crate::check_docs::command,
    },
    Command {
        name: "trace",
        summary: "Requirement -> test matrix; gate untested MUSTs by maturity (XTASK-R040 … R045).",
        run: crate::trace::command,
    },
    Command {
        name: "hygiene",
        summary: "Find forbidden macros, untracked task markers, unreasoned ignores (XTASK-R050 … R052).",
        run: crate::hygiene::command,
    },
    Command {
        name: "coverage",
        summary: "Line coverage per crate (cargo llvm-cov) against ci/coverage.toml floors (XTASK-R060 … R064).",
        run: crate::coverage::command,
    },
    Command {
        name: "status",
        summary: "Generate docs/STATUS.md from task and spec frontmatter; --check fails if stale (XTASK-R070 … R072).",
        run: crate::status::command,
    },
    Command {
        name: "doctor",
        summary: "List the tools the checks need: present or missing, version, install hint (XTASK-R080).",
        run: crate::doctor::command,
    },
];

/// Runs xtask with the given arguments (the process arguments without the program name).
///
/// No arguments behaves like `help`. Normal output goes to `stdout`. Usage errors print a message
/// and the usage text to `stderr` and return [`Exit::Usage`] (XTASK-R002).
///
/// # Examples
///
/// ```
/// let mut stdout = Vec::new();
/// let mut stderr = Vec::new();
///
/// let exit = xtask::run(&["help".to_string()], &mut stdout, &mut stderr);
/// assert_eq!(exit, xtask::Exit::Success);
/// assert!(String::from_utf8(stdout).unwrap().contains("help"));
///
/// let exit = xtask::run(&["nonsense".to_string()], &mut Vec::new(), &mut stderr);
/// assert_eq!(exit.code(), 2);
/// ```
pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> Exit {
    run_in(&crate::repo::root(), args, stdout, stderr)
}

/// [`run`] with the repository root `root` checked before any command except `help` runs
/// (XTASK-R015). Commands themselves use the compiled-in root; tests pass a root that does not
/// exist to exercise the check.
pub(crate) fn run_in(
    root: &Path,
    args: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Exit {
    let result = match args.split_first() {
        None => help(&[], stdout),
        Some((name, rest)) => match find_command(name) {
            Ok(command) if command.name == "help" => (command.run)(rest, stdout),
            Ok(command) => crate::repo::check_root(root)
                .map_err(CommandError::from)
                .and_then(|()| (command.run)(rest, stdout)),
            Err(error) => Err(error),
        },
    };
    match result {
        Ok(exit) => exit,
        Err(CommandError::Usage(message)) => {
            // Nothing more can be reported if stderr itself fails; the exit code still says usage.
            let _ = write!(stderr, "error: {message}\n\n{}", usage());
            Exit::Usage
        }
        Err(
            error @ (CommandError::Io(_)
            | CommandError::File { .. }
            | CommandError::Internal { .. }),
        ) => {
            let _ = writeln!(stderr, "error: {error}");
            Exit::Failure
        }
    }
}

/// Looks up a command by name. Arguments starting with `-` are options, which are not valid in
/// command position.
fn find_command(name: &str) -> Result<&'static Command, CommandError> {
    if name.starts_with('-') {
        return Err(CommandError::Usage(format!(
            "unknown option `{name}`; a command must come first"
        )));
    }
    COMMANDS
        .iter()
        .find(|command| command.name == name)
        .ok_or_else(|| CommandError::Usage(format!("unknown command `{name}`")))
}

/// The usage text: synopsis and the command table.
fn usage() -> String {
    let width = COMMANDS
        .iter()
        .map(|command| command.name.len())
        .max()
        .unwrap_or(0);
    let mut text = String::from("Usage: cargo xtask <command> [options]\n\nCommands:\n");
    for command in COMMANDS {
        let line = format!("  {:width$}  {}\n", command.name, command.summary);
        text.push_str(&line);
    }
    text
}

/// The `help` command. Takes no arguments.
fn help(args: &[String], out: &mut dyn Write) -> Result<Exit, CommandError> {
    if let Some(argument) = args.first() {
        return Err(unexpected_argument("help", argument));
    }
    out.write_all(usage().as_bytes())?;
    Ok(Exit::Success)
}

/// The usage error for an argument a command does not accept.
pub(crate) fn unexpected_argument(command: &str, argument: &str) -> CommandError {
    if argument.starts_with('-') {
        CommandError::Usage(format!("unknown option `{argument}` for `{command}`"))
    } else {
        CommandError::Usage(format!("unexpected argument `{argument}` for `{command}`"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs xtask on `args` and returns (exit, stdout, stderr).
    fn invoke(args: &[&str]) -> (Exit, String, String) {
        let args: Vec<String> = args.iter().map(ToString::to_string).collect();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = run(&args, &mut stdout, &mut stderr);
        (
            exit,
            String::from_utf8(stdout).unwrap(),
            String::from_utf8(stderr).unwrap(),
        )
    }

    /// A writer that always fails, to exercise the I/O error path.
    struct BrokenWriter;

    impl Write for BrokenWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    /// Verifies: XTASK-R002
    #[test]
    fn help_lists_every_command_with_its_summary() {
        let (exit, stdout, stderr) = invoke(&["help"]);
        assert_eq!(exit, Exit::Success);
        assert_eq!(stderr, "");
        for command in COMMANDS {
            let line = stdout
                .lines()
                .find(|line| line.split_whitespace().next() == Some(command.name))
                .unwrap();
            assert!(line.ends_with(command.summary), "line: {line}");
        }
    }

    /// Verifies: XTASK-R002
    #[test]
    fn no_arguments_prints_help() {
        assert_eq!(invoke(&[]), invoke(&["help"]));
    }

    /// Verifies: XTASK-R002
    #[test]
    fn unknown_command_is_a_usage_error() {
        let (exit, stdout, stderr) = invoke(&["nonsense"]);
        assert_eq!(exit, Exit::Usage);
        assert_eq!(exit.code(), 2);
        assert_eq!(stdout, "");
        assert!(stderr.starts_with("error: unknown command `nonsense`"));
        assert!(stderr.contains(&usage()));
    }

    /// Verifies: XTASK-R002
    #[test]
    fn unknown_option_in_command_position_is_a_usage_error() {
        let (exit, stdout, stderr) = invoke(&["--verbose"]);
        assert_eq!(exit, Exit::Usage);
        assert_eq!(stdout, "");
        assert!(stderr.starts_with("error: unknown option `--verbose`"));
        assert!(stderr.contains(&usage()));
    }

    /// Verifies: XTASK-R002
    #[test]
    fn unknown_option_after_command_is_a_usage_error() {
        let (exit, stdout, stderr) = invoke(&["help", "--json"]);
        assert_eq!(exit, Exit::Usage);
        assert_eq!(stdout, "");
        assert!(stderr.starts_with("error: unknown option `--json` for `help`"));
        assert!(stderr.contains(&usage()));
    }

    /// Verifies: XTASK-R002
    #[test]
    fn unexpected_positional_argument_is_a_usage_error() {
        let (exit, _, stderr) = invoke(&["help", "ci"]);
        assert_eq!(exit, Exit::Usage);
        assert!(stderr.starts_with("error: unexpected argument `ci` for `help`"));
    }

    #[test]
    fn failed_output_write_is_a_failure_reported_on_stderr() {
        let mut stderr = Vec::new();
        let exit = run(&["help".to_string()], &mut BrokenWriter, &mut stderr);
        assert_eq!(exit, Exit::Failure);
        assert!(
            String::from_utf8(stderr)
                .unwrap()
                .starts_with("error: could not write output: broken pipe")
        );
    }

    /// Verifies: XTASK-R015
    #[test]
    fn every_command_but_help_reports_one_error_for_a_missing_root() {
        let missing = crate::repo::root().join("target/xtask-test-no-such-root");
        // An option no command accepts: if the root check did not come first, each command would
        // fail fast with a usage error instead of running (and possibly writing) anything.
        for command in COMMANDS {
            let args = vec![command.name.to_string(), "--xtask-test-bogus".to_string()];
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let exit = run_in(&missing, &args, &mut stdout, &mut stderr);
            let stderr = String::from_utf8(stderr).unwrap();
            if command.name == "help" {
                // `help` reads no repository; its usage error is unchanged.
                assert_eq!(exit, Exit::Usage);
                continue;
            }
            assert_eq!(exit, Exit::Failure, "{}: {stderr}", command.name);
            assert_eq!(stdout, b"", "{}", command.name);
            assert_eq!(
                stderr,
                format!(
                    "error: LIM-XTASK-0007: {}: the repository root compiled into xtask does not \
                     exist (help: The checkout was moved or renamed after xtask was built: run \
                     `cargo clean -p xtask`, then the command again)\n",
                    missing.display()
                ),
                "{}",
                command.name
            );
        }
    }

    /// Verifies: XTASK-R015
    #[test]
    fn a_root_without_architecture_toml_is_rejected_and_help_still_works() {
        let not_a_root = crate::repo::root().join("xtask");
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        let args = ["status".to_string(), "--xtask-test-bogus".to_string()];
        let exit = run_in(&not_a_root, &args, &mut stdout, &mut stderr);
        assert_eq!(exit, Exit::Failure);
        assert_eq!(stdout, b"");
        let stderr = String::from_utf8(stderr).unwrap();
        assert!(stderr.starts_with("error: LIM-XTASK-0007: "), "{stderr}");
        assert!(stderr.contains("has no architecture.toml"), "{stderr}");
        assert_eq!(stderr.lines().count(), 1, "{stderr}");

        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        let exit = run_in(&not_a_root, &[], &mut stdout, &mut stderr);
        assert_eq!(exit, Exit::Success);
        assert_eq!(String::from_utf8(stdout).unwrap(), usage());
    }

    #[test]
    fn exit_codes_match_xtask_r007() {
        assert_eq!(Exit::Success.code(), 0);
        assert_eq!(Exit::Failure.code(), 1);
        assert_eq!(Exit::Usage.code(), 2);
    }

    #[test]
    fn command_names_are_unique_and_not_options() {
        for (index, command) in COMMANDS.iter().enumerate() {
            assert!(!command.name.starts_with('-'));
            assert!(COMMANDS[index + 1..].iter().all(|c| c.name != command.name));
        }
    }
}
