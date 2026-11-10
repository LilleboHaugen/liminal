//! The `test-features-off` step of `ci` (`docs/specs/xtask.md` XTASK-R014): every workspace
//! package that has Cargo features is tested once more with all of them off.
//!
//! The `test` step uses `--all-features` (XTASK-R012), so code and tests under
//! `cfg(not(feature = "…"))` are never compiled there. This step compiles and runs them.
//!
//! [`run`] is the step's logic over a list of packages and a function that runs one cargo
//! command; [`crate::process::RealRunner`] supplies `cargo metadata` output and real processes.

use crate::ci::Outcome;
use crate::repo::metadata::Package;

/// Step name in output and in `--only`, `--skip` and `--allow-skip`.
pub(crate) const NAME: &str = "test-features-off";

/// The step's command as shown in human mode after `cargo `.
pub(crate) const DESCRIPTION: &str = "test --package <crate> --no-default-features --locked \
                                      (once per workspace crate with features)";

/// Arguments after `cargo` that test `package` with every feature off. `--package` selects one
/// package, so no other workspace member can turn a feature on through feature unification.
/// `--locked`: like every cargo step, it must not update `Cargo.lock` (XTASK-R010).
pub(crate) fn test_args(package: &str) -> [&str; 5] {
    [
        "test",
        "--package",
        package,
        "--no-default-features",
        "--locked",
    ]
}

/// Names of the packages that have at least one feature, in the order of `packages`.
pub(crate) fn packages_with_features(packages: &[Package]) -> Vec<&str> {
    packages
        .iter()
        .filter(|package| !package.features.is_empty())
        .map(|package| package.name.as_str())
        .collect()
}

/// Runs [`test_args`] through `test` for each package with features, in the order of `packages`.
///
/// Stops at the first command that does not pass and returns its outcome; a failure's
/// `output_tail`, when present, gets the failed command as its first line. Passes without
/// calling `test` if no package has features.
pub(crate) fn run(packages: &[Package], mut test: impl FnMut(&[&str]) -> Outcome) -> Outcome {
    for package in packages_with_features(packages) {
        let args = test_args(package);
        match test(&args) {
            Outcome::Pass => {}
            Outcome::Fail {
                findings,
                output_tail,
            } => {
                return Outcome::Fail {
                    findings,
                    output_tail: output_tail
                        .map(|tail| format!("cargo {}\n{tail}", args.join(" "))),
                };
            }
            skipped @ Outcome::Skipped { .. } => return skipped,
        }
    }
    Outcome::Pass
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::Path;

    use super::*;
    use crate::ci::{Action, OutputMode, Step, StepRunner};
    use crate::process::RealRunner;
    use crate::test_support::TempRoot;

    fn package(name: &str, features: &[&str]) -> Package {
        Package {
            name: name.to_string(),
            manifest_dir: format!("crates/{name}"),
            targets: Vec::new(),
            dependencies: Vec::new(),
            features: features.iter().map(ToString::to_string).collect(),
        }
    }

    fn fail(output_tail: Option<&str>) -> Outcome {
        Outcome::Fail {
            findings: Vec::new(),
            output_tail: output_tail.map(ToString::to_string),
        }
    }

    /// Runs the step over `packages` with scripted outcomes per package (default: pass) and
    /// returns the outcome and the commands that ran.
    fn scripted(packages: &[Package], outcomes: &[(&str, Outcome)]) -> (Outcome, Vec<String>) {
        let mut commands = Vec::new();
        let outcome = run(packages, |args| {
            commands.push(args.join(" "));
            outcomes
                .iter()
                .find(|(name, _)| *name == args[2])
                .map_or(Outcome::Pass, |(_, outcome)| outcome.clone())
        });
        (outcome, commands)
    }

    /// Verifies: XTASK-R014
    #[test]
    fn each_package_with_features_is_tested_alone_with_default_features_off() {
        let packages = [
            package("a_plain", &[]),
            package("b_gated", &["telemetry"]),
            package("c_default_only", &["default"]),
            package("d_plain", &[]),
            package("e_optional_dep", &["serde"]),
        ];
        let (outcome, commands) = scripted(&packages, &[]);
        assert_eq!(outcome, Outcome::Pass);
        assert_eq!(
            commands,
            [
                "test --package b_gated --no-default-features --locked",
                "test --package c_default_only --no-default-features --locked",
                "test --package e_optional_dep --no-default-features --locked",
            ]
        );
    }

    /// Verifies: XTASK-R014
    #[test]
    fn no_package_with_features_passes_without_running_anything() {
        let (outcome, commands) = scripted(&[package("a", &[]), package("b", &[])], &[]);
        assert_eq!(outcome, Outcome::Pass);
        assert_eq!(commands, Vec::<String>::new());
        assert_eq!(scripted(&[], &[]), (Outcome::Pass, Vec::new()));
    }

    /// Verifies: XTASK-R014
    #[test]
    fn the_first_failure_ends_the_step_and_names_its_command() {
        let packages = [
            package("a", &["x"]),
            package("b", &["x"]),
            package("c", &["x"]),
        ];
        let (outcome, commands) = scripted(&packages, &[("b", fail(Some("test t ... FAILED")))]);
        assert_eq!(
            outcome,
            fail(Some(
                "cargo test --package b --no-default-features --locked\ntest t ... FAILED"
            ))
        );
        assert_eq!(
            commands,
            [
                "test --package a --no-default-features --locked",
                "test --package b --no-default-features --locked",
            ]
        );
        // Human mode: the child wrote to the terminal, so there is no tail to prefix.
        let (outcome, _) = scripted(&packages, &[("a", fail(None))]);
        assert_eq!(outcome, fail(None));
    }

    /// Verifies: XTASK-R014, XTASK-R006
    #[test]
    fn a_command_that_cannot_run_skips_the_step() {
        let skipped = Outcome::Skipped {
            reason: "could not start `cargo`".to_string(),
            help: "install it".to_string(),
        };
        let packages = [package("a", &["x"]), package("b", &["x"])];
        let (outcome, commands) = scripted(&packages, &[("a", skipped.clone())]);
        assert_eq!(outcome, skipped);
        assert_eq!(commands.len(), 1);
    }

    const STEP: Step = Step {
        name: NAME,
        action: Action::FeaturesOff,
    };

    /// Writes a workspace of two packages to `root`. `plain` has no feature and a test that
    /// always fails. `gated` has the feature `on`, on by default, a test that exists only with
    /// `on` and always fails, and a test that exists only without `on` with the body `off_body`.
    fn write_workspace(root: &Path, off_body: &str) {
        let write = |path: &str, text: &str| {
            let path = root.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        };
        let manifest = |name: &str, features: &str| {
            format!(
                "[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n{features}"
            )
        };
        write(
            "Cargo.toml",
            "[workspace]\nmembers = [\"gated\", \"plain\"]\nresolver = \"3\"\n",
        );
        // `--locked` needs an up-to-date lock file.
        write(
            "Cargo.lock",
            "version = 4\n\n[[package]]\nname = \"gated\"\nversion = \"0.0.0\"\n\n\
             [[package]]\nname = \"plain\"\nversion = \"0.0.0\"\n",
        );
        write("plain/Cargo.toml", &manifest("plain", ""));
        write(
            "plain/src/lib.rs",
            "#[test]\nfn plain_must_not_run() {\n    panic!(\"no features\");\n}\n",
        );
        write(
            "gated/Cargo.toml",
            &manifest("gated", "\n[features]\ndefault = [\"on\"]\non = []\n"),
        );
        write(
            "gated/src/lib.rs",
            &format!(
                "#[cfg(feature = \"on\")]\n#[test]\nfn only_with_on() {{\n    panic!(\"on\");\n}}\n\n\
                 #[cfg(not(feature = \"on\"))]\n#[test]\nfn only_with_on_off() {{\n    {off_body}\n}}\n"
            ),
        );
    }

    /// The real step on a real workspace: real `cargo metadata`, real `cargo test`.
    ///
    /// Verifies: XTASK-R014
    #[test]
    fn real_step_runs_the_tests_that_exist_only_with_features_off() {
        // Passing: `plain` (no features) is not tested, and `gated` is built without its default
        // feature, or one of the two always-failing tests would run.
        let root = TempRoot::new("features-off-pass");
        write_workspace(root.path(), "assert_eq!(1 + 1, 2);");
        let mut runner = RealRunner::new(root.path().to_path_buf());
        for mode in [OutputMode::Capture, OutputMode::Inherit] {
            assert_eq!(runner.run(&STEP, mode), Outcome::Pass, "{mode:?}");
        }

        // Failing: the test compiled only with the feature off does run.
        let root = TempRoot::new("features-off-fail");
        write_workspace(root.path(), "panic!(\"the features-off test ran\");");
        let mut runner = RealRunner::new(root.path().to_path_buf());
        let outcome = runner.run(&STEP, OutputMode::Capture);
        let Outcome::Fail {
            findings,
            output_tail: Some(tail),
        } = outcome
        else {
            panic!("expected a failure with output, got {outcome:?}");
        };
        assert_eq!(findings, Vec::new());
        assert!(
            tail.starts_with("cargo test --package gated --no-default-features --locked\n"),
            "{tail}"
        );
        assert!(
            tail.contains("only_with_on_off") && tail.contains("the features-off test ran"),
            "{tail}"
        );
        assert!(!tail.contains("plain_must_not_run"), "{tail}");
        // Human mode on a failing run is not run here: the child would write its failure
        // report into this test's output. `run` maps that failure in
        // `the_first_failure_ends_the_step_and_names_its_command`.
    }

    /// Verifies: XTASK-R014, XTASK-R006
    #[test]
    fn real_step_without_cargo_is_skipped_and_a_metadata_error_fails_it() {
        let mut runner = RealRunner::with_cargo(
            crate::repo::root(),
            OsString::from("xtask-test-no-such-program"),
        );
        let outcome = runner.run(&STEP, OutputMode::Capture);
        let Outcome::Skipped { reason, help } = outcome else {
            panic!("expected a skip, got {outcome:?}");
        };
        assert!(
            reason.starts_with("could not start `xtask-test-no-such-program`"),
            "{reason}"
        );
        assert_eq!(help, crate::process::CARGO_HELP);

        // No manifest: `cargo metadata` fails. Its message is reported in both output modes.
        let root = TempRoot::new("features-off-no-manifest");
        let mut runner = RealRunner::new(root.path().to_path_buf());
        for mode in [OutputMode::Capture, OutputMode::Inherit] {
            let outcome = runner.run(&STEP, mode);
            let Outcome::Fail {
                findings,
                output_tail: Some(tail),
            } = outcome
            else {
                panic!("expected a failure with output, got {outcome:?}");
            };
            assert_eq!(findings, Vec::new());
            assert!(
                tail.starts_with("LIM-XTASK-0005: cargo metadata: failed")
                    && tail.contains("Cargo.toml"),
                "{tail}"
            );
        }
    }
}
