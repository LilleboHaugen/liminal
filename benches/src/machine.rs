//! Machine metadata recorded in every result file (spec §5 "Machine metadata", BENCH-R032;
//! PERFORMANCE.md "Machines").

use std::path::Path;
use std::process::Command;

/// The value of a field that could not be determined.
pub const UNKNOWN: &str = "unknown";

/// Where and how a result was measured. Every field is a non-empty string; a field that cannot
/// be determined is [`UNKNOWN`].
///
/// # Examples
///
/// ```
/// use liminal_benches::Metadata;
///
/// let metadata = Metadata::detect();
/// assert_eq!(metadata.os, std::env::consts::OS);
/// assert_eq!(metadata.arch, std::env::consts::ARCH);
/// assert!(!metadata.toolchain.is_empty());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Metadata {
    /// Machine ID, the directory name under `ci/perf/`: `$LIMINAL_BENCH_MACHINE_ID`.
    pub machine_id: String,
    /// `std::env::consts::OS`, for example `macos`.
    pub os: String,
    /// `std::env::consts::ARCH`, for example `aarch64`.
    pub arch: String,
    /// OS version: `$LIMINAL_BENCH_OS_VERSION`, else `sw_vers -productVersion` (macOS) or
    /// `uname -r` (other Unix).
    pub os_version: String,
    /// `$LIMINAL_BENCH_TOOLCHAIN`, else the first line of `$RUSTC -V` (`rustc` if unset).
    pub toolchain: String,
    /// `$LIMINAL_BENCH_COMMIT`, else `git rev-parse HEAD`, with `-dirty` appended when
    /// `git status --porcelain` lists changes.
    pub commit: String,
    /// `ac`, `battery` or `unknown`: `$LIMINAL_BENCH_POWER`, else `pmset -g ps` (macOS).
    pub power: String,
}

impl Metadata {
    /// Detects the metadata of this machine and process (BENCH-R032). Spawns `git`, `rustc` and
    /// the OS tools named on the fields; a tool that is missing or fails gives [`UNKNOWN`].
    #[must_use]
    pub fn detect() -> Metadata {
        detect_with(&|name| std::env::var(name).ok(), &run)
    }
}

/// Reads an environment variable.
type Env<'a> = &'a dyn Fn(&str) -> Option<String>;
/// Runs a program with arguments and returns its standard output if it exits with status 0.
type Run<'a> = &'a dyn Fn(&str, &[&str]) -> Option<String>;

/// [`Metadata::detect`] with the environment and process runner given.
pub(crate) fn detect_with(env: Env<'_>, run: Run<'_>) -> Metadata {
    let pick = |var: &str, detect: &dyn Fn() -> Option<String>| {
        env(var)
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .or_else(detect)
            .unwrap_or_else(|| UNKNOWN.to_string())
    };
    let first_line = |out: String| out.lines().next().map(|l| l.trim().to_string());
    let rustc = env("RUSTC").unwrap_or_else(|| "rustc".to_string());
    Metadata {
        machine_id: pick("LIMINAL_BENCH_MACHINE_ID", &|| None),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        os_version: pick("LIMINAL_BENCH_OS_VERSION", &|| {
            let out = if cfg!(target_os = "macos") {
                run("sw_vers", &["-productVersion"])
            } else if cfg!(unix) {
                run("uname", &["-r"])
            } else {
                None
            };
            out.and_then(first_line).filter(|l| !l.is_empty())
        }),
        toolchain: pick("LIMINAL_BENCH_TOOLCHAIN", &|| {
            run(&rustc, &["-V"])
                .and_then(first_line)
                .filter(|l| !l.is_empty())
        }),
        commit: pick("LIMINAL_BENCH_COMMIT", &|| commit(run)),
        power: pick("LIMINAL_BENCH_POWER", &|| {
            if cfg!(target_os = "macos") {
                run("pmset", &["-g", "ps"]).and_then(|out| power_from_pmset(&out))
            } else {
                None
            }
        }),
    }
}

/// `git rev-parse HEAD`, plus `-dirty` if the work tree has changes.
fn commit(run: Run<'_>) -> Option<String> {
    let head = run("git", &["rev-parse", "HEAD"])?.trim().to_string();
    if head.is_empty() {
        return None;
    }
    match run("git", &["status", "--porcelain"]) {
        Some(status) if !status.trim().is_empty() => Some(format!("{head}-dirty")),
        _ => Some(head),
    }
}

/// The power source from `pmset -g ps` output: its first line is
/// `Now drawing from 'AC Power'` or `Now drawing from 'Battery Power'`.
pub(crate) fn power_from_pmset(output: &str) -> Option<String> {
    let line = output.lines().next()?;
    if line.contains("'AC Power'") {
        Some("ac".to_string())
    } else if line.contains("'Battery Power'") {
        Some("battery".to_string())
    } else {
        None
    }
}

/// Runs `program` with `args` in the workspace root; standard output if the exit status is 0.
fn run(program: &str, args: &[&str]) -> Option<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    if output.status.success() {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_env(_: &str) -> Option<String> {
        None
    }

    fn fail(_: &str, _: &[&str]) -> Option<String> {
        None
    }

    /// Verifies: BENCH-R032
    #[test]
    fn environment_overrides_win_and_missing_values_are_unknown() {
        let env = |name: &str| match name {
            "LIMINAL_BENCH_MACHINE_ID" => Some("m5-pro".to_string()),
            "LIMINAL_BENCH_OS_VERSION" => Some(" 27.2 ".to_string()),
            "LIMINAL_BENCH_TOOLCHAIN" => Some("rustc 1.99.1".to_string()),
            "LIMINAL_BENCH_COMMIT" => Some("abc".to_string()),
            "LIMINAL_BENCH_POWER" => Some("ac".to_string()),
            _ => None,
        };
        let given = detect_with(&env, &fail);
        assert_eq!(
            given,
            Metadata {
                machine_id: "m5-pro".into(),
                os: std::env::consts::OS.into(),
                arch: std::env::consts::ARCH.into(),
                os_version: "27.2".into(),
                toolchain: "rustc 1.99.1".into(),
                commit: "abc".into(),
                power: "ac".into(),
            }
        );
        let unknown = detect_with(&no_env, &fail);
        for field in [
            &unknown.machine_id,
            &unknown.os_version,
            &unknown.toolchain,
            &unknown.commit,
            &unknown.power,
        ] {
            assert_eq!(field, UNKNOWN);
        }
    }

    /// Verifies: BENCH-R032
    #[test]
    fn tools_fill_fields_without_overrides() {
        let run = |program: &str, args: &[&str]| -> Option<String> {
            match (program, args) {
                ("rustc", ["-V"]) => Some("rustc 1.99.1 (abc 2026-10-01)\n".into()),
                ("git", ["rev-parse", "HEAD"]) => Some("0123abcd\n".into()),
                ("git", ["status", "--porcelain"]) => Some(" M benches/src/lib.rs\n".into()),
                ("sw_vers", _) => Some("27.2\n".into()),
                ("uname", _) => Some("6.8.0\n".into()),
                ("pmset", _) => {
                    Some("Now drawing from 'Battery Power'\n -InternalBattery-0\n".into())
                }
                _ => None,
            }
        };
        let detected = detect_with(&no_env, &run);
        assert_eq!(detected.toolchain, "rustc 1.99.1 (abc 2026-10-01)");
        assert_eq!(detected.commit, "0123abcd-dirty");
        if cfg!(target_os = "macos") {
            assert_eq!(detected.os_version, "27.2");
            assert_eq!(detected.power, "battery");
        } else if cfg!(unix) {
            assert_eq!(detected.os_version, "6.8.0");
            assert_eq!(detected.power, UNKNOWN);
        }
        let clean = |program: &str, args: &[&str]| match args {
            ["status", "--porcelain"] => Some(String::new()),
            _ => run(program, args),
        };
        assert_eq!(detect_with(&no_env, &clean).commit, "0123abcd");
    }

    #[test]
    fn pmset_output_names_the_power_source() {
        assert_eq!(
            power_from_pmset("Now drawing from 'AC Power'\n"),
            Some("ac".into())
        );
        assert_eq!(power_from_pmset("Now drawing from 'UPS Power'"), None);
        assert_eq!(power_from_pmset(""), None);
    }

    #[test]
    fn real_detection_reports_this_platform() {
        let detected = Metadata::detect();
        assert_eq!(detected.os, std::env::consts::OS);
        assert_ne!(detected.commit, "");
    }
}
