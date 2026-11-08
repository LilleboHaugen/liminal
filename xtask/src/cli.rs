//! Argument parsing, the command table and the `help` command (XTASK-R002).
//!
//! Parsing is hand-written (no CLI-parsing crate, `docs/specs/xtask.md` §3). The first argument
//! selects a command from [`COMMANDS`]; the remaining arguments are passed to that command, which
//! parses its own options and reports unknown ones as [`CommandError::Usage`].

use std::fmt;
use std::io::{self, Write};

/// Process exit status of an xtask invocation (XTASK-R007).
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
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::Usage(message) => write!(f, "{message}"),
            CommandError::Io(error) => write!(f, "could not write output: {error}"),
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
pub const COMMANDS: &[Command] = &[Command {
    name: "help",
    summary: "Print this list of commands.",
    run: help,
}];

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
    let result = match args.split_first() {
        None => help(&[], stdout),
        Some((name, rest)) => match find_command(name) {
            Ok(command) => (command.run)(rest, stdout),
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
        Err(error @ CommandError::Io(_)) => {
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
fn unexpected_argument(command: &str, argument: &str) -> CommandError {
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
                .find(|line| line.trim_start().starts_with(command.name))
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
