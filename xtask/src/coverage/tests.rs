use super::*;
use crate::repo::coverage::{Floor, MISSING_HELP, MISSING_REASON};
use crate::test_support::TempRoot;

fn lines(covered: u64, count: u64) -> Lines {
    Lines { covered, count }
}

fn floors(entries: &[(&str, u32)]) -> Floors {
    entries
        .iter()
        .enumerate()
        .map(|(index, (name, tenths))| {
            let floor = Floor {
                tenths: *tenths,
                line: index + 3,
                span: (0, 0),
            };
            ((*name).to_string(), floor)
        })
        .collect()
}

fn measurement(entries: &[(&str, Lines)]) -> Measurement {
    entries
        .iter()
        .map(|(name, lines)| ((*name).to_string(), *lines))
        .collect()
}

fn rules(findings: &[Finding]) -> Vec<(&str, &str)> {
    findings
        .iter()
        .map(|f| (f.rule.as_str(), f.location.as_str()))
        .collect()
}

/// Verifies: XTASK-R061
#[test]
fn a_drop_of_0_4_points_passes_and_a_drop_of_0_6_fails() {
    let floors = floors(&[("xtask", 784)]);
    // 78.0 % is 0.4 below 78.4; 77.9 % is exactly 0.5 below (not "below floor − 0.5").
    for covered in [785, 784, 780, 779] {
        let measured = measurement(&[("xtask", lines(covered, 1000))]);
        assert_eq!(evaluate(&measured, &floors), Vec::new(), "{covered}/1000");
    }
    // 77.8 % is 0.6 below; 77.89 % is 0.51 below.
    for (covered, count) in [(778, 1000), (7789, 10_000)] {
        let measured = measurement(&[("xtask", lines(covered, count))]);
        let findings = evaluate(&measured, &floors);
        assert_eq!(rules(&findings), [("XTASK-R061", "ci/coverage.toml:3")]);
    }
}

/// Verifies: XTASK-R061
#[test]
fn a_coverage_finding_names_the_crate_the_floor_and_the_measured_value() {
    let measured = measurement(&[("xtask", lines(7789, 10_000))]);
    let findings = evaluate(&measured, &floors(&[("xtask", 784)]));
    assert_eq!(
        findings[0].message,
        "`xtask` line coverage 77.89% (7789 of 10000 lines) is more than 0.5 points below its \
         floor 78.4%"
    );
}

/// Verifies: XTASK-R061
#[test]
fn floors_below_the_tolerance_and_crates_without_lines_never_fail() {
    let measured = measurement(&[("low", lines(0, 100)), ("empty", lines(0, 0))]);
    let floors = floors(&[("low", 5), ("empty", 1000)]);
    assert_eq!(evaluate(&measured, &floors), Vec::new());
    // 0.6 is more than 0.5 above 0 %.
    let measured = measurement(&[("low", lines(0, 100))]);
    let findings = evaluate(&measured, &self::floors(&[("low", 6)]));
    assert_eq!(rules(&findings), [("XTASK-R061", "ci/coverage.toml:3")]);
}

/// Verifies: XTASK-R062
#[test]
fn a_crate_without_a_floor_fails_with_the_line_to_add() {
    let measured = measurement(&[("xtask", lines(9836, 10_000)), ("empty", lines(0, 0))]);
    let findings = evaluate(&measured, &Floors::new());
    assert_eq!(
        rules(&findings),
        [
            ("XTASK-R062", "ci/coverage.toml"),
            ("XTASK-R062", "ci/coverage.toml")
        ]
    );
    let xtask = findings
        .iter()
        .find(|f| f.message.contains("`xtask`"))
        .unwrap();
    assert_eq!(
        xtask.message,
        "workspace crate `xtask` has no coverage floor"
    );
    assert!(
        xtask
            .help
            .starts_with("add `xtask = 98.3` under `[floors]`"),
        "{}",
        xtask.help
    );
    let empty = findings
        .iter()
        .find(|f| f.message.contains("`empty`"))
        .unwrap();
    assert!(
        empty.help.starts_with("add `empty = 0.0`"),
        "{}",
        empty.help
    );
}

/// Verifies: XTASK-R062
#[test]
fn a_floor_for_a_crate_that_is_not_in_the_workspace_fails() {
    let measured = measurement(&[("xtask", lines(1, 1))]);
    let findings = evaluate(&measured, &floors(&[("xtask", 0), ("gone", 500)]));
    assert_eq!(rules(&findings), [("XTASK-R062", "ci/coverage.toml:4")]);
    assert_eq!(
        findings[0].message,
        "coverage floor for `gone`, which is not a workspace crate"
    );
}

/// Verifies: XTASK-R063
#[test]
fn update_raises_floors_to_the_measured_value_rounded_down_and_never_lowers_them() {
    let measured = measurement(&[
        ("up", lines(98_369, 100_000)), // 98.369 → 98.3, floor 97.0
        ("same", lines(500, 1000)),     // 50.0, floor 50.0
        ("down", lines(100, 1000)),     // 10.0, floor 60.0: kept
        ("empty", lines(0, 0)),         // no lines: kept
        ("missing", lines(1000, 1000)), // no floor: not added
        ("full", lines(1000, 1000)),    // 100.0, floor 99.9
    ]);
    let floors = floors(&[
        ("up", 970),
        ("same", 500),
        ("down", 600),
        ("empty", 100),
        ("full", 999),
    ]);
    assert_eq!(
        raised_floors(&measured, &floors),
        [
            Raise {
                name: "full".to_string(),
                from: 999,
                to: 1000
            },
            Raise {
                name: "up".to_string(),
                from: 970,
                to: 983
            },
        ]
    );
}

#[test]
fn applying_raises_replaces_only_the_values() {
    // The first value grows (`97` → `98.3`), so the later spans must be replaced first.
    let text =
        "# Floors.\n[floors]\nxtask = 97 # keep this comment\nliminal_core=85\nother = 1.0\n";
    let floors = loader::parse_floors(text).unwrap();
    let raises = [
        Raise {
            name: "xtask".to_string(),
            from: 970,
            to: 983,
        },
        Raise {
            name: "liminal_core".to_string(),
            from: 850,
            to: 1000,
        },
    ];
    assert_eq!(
        apply_raises(text, &floors, &raises),
        "# Floors.\n[floors]\nxtask = 98.3 # keep this comment\nliminal_core=100.0\nother = 1.0\n"
    );
}

#[test]
fn tenths_and_percentages_are_formatted_exactly() {
    assert_eq!(tenths_text(0), "0.0");
    assert_eq!(tenths_text(784), "78.4");
    assert_eq!(tenths_text(1000), "100.0");
    assert_eq!(percent_text(lines(4220, 4290)), "98.36");
    assert_eq!(percent_text(lines(1, 3)), "33.33");
    assert_eq!(percent_text(lines(2, 3)), "66.66");
    assert_eq!(percent_text(lines(5, 5)), "100.00");
    assert_eq!(measured_tenths(lines(2, 3)), Some(666));
    assert_eq!(measured_tenths(lines(0, 0)), None);
}

/// A repository with workspace members `xtask` and `crates/liminal_a`, and `ci/coverage.toml`.
fn repository(name: &str, config: &str) -> TempRoot {
    let root = TempRoot::new(name);
    let write = |path: &str, text: &str| {
        let path = root.path().join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write(
        "Cargo.toml",
        "[workspace]\nmembers = [\"xtask\", \"crates/liminal_a\"]\n",
    );
    write("xtask/Cargo.toml", "[package]\nname = \"xtask\"\n");
    write(
        "crates/liminal_a/Cargo.toml",
        "[package]\nname = \"liminal_a\"\n",
    );
    write("ci/coverage.toml", config);
    root
}

const CONFIG: &str = "# Floors.\n[floors]\nliminal_a = 50.0\nxtask = 97.0 # comment\n";

fn fixed(xtask: Lines) -> impl FnOnce(&[repo::Member]) -> Result<Measurement, MeasureError> {
    move |members| {
        let names: Vec<&str> = members.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(names, ["xtask", "liminal_a"]);
        Ok(measurement(&[
            ("liminal_a", lines(60, 100)),
            ("xtask", xtask),
        ]))
    }
}

fn run(
    args: &[&str],
    root: &Path,
    measure: impl FnOnce(&[repo::Member]) -> Result<Measurement, MeasureError>,
) -> (Result<Exit, CommandError>, String) {
    let args: Vec<String> = args.iter().map(ToString::to_string).collect();
    let mut out = Vec::new();
    let result =
        parse_options(&args).and_then(|options| execute(&options, &mut out, root, measure));
    (result, String::from_utf8(out).unwrap())
}

/// Verifies: XTASK-R009
#[test]
fn json_reports_findings_and_per_crate_values() {
    let root = repository("coverage-json", CONFIG);
    let (result, out) = run(&["--json"], root.path(), fixed(lines(900, 1000)));
    assert!(matches!(result, Ok(Exit::Failure)), "{result:?}");
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(value["version"], 1);
    assert_eq!(value["ok"], false);
    assert_eq!(value.get("skipped"), None);
    assert_eq!(value["findings"][0]["rule"], "XTASK-R061");
    assert_eq!(value["findings"][0]["location"], "ci/coverage.toml:4");
    assert_eq!(
        value["crates"],
        serde_json::json!([
            { "crate": "liminal_a", "covered": 60, "count": 100, "percent": 60.0, "floor": 50.0 },
            { "crate": "xtask", "covered": 900, "count": 1000, "percent": 90.0, "floor": 97.0 }
        ])
    );
}

/// Verifies: XTASK-R010
#[test]
fn without_update_floors_the_command_writes_nothing() {
    let root = repository("coverage-human", CONFIG);
    let (result, out) = run(&[], root.path(), fixed(lines(9836, 10_000)));
    assert!(matches!(result, Ok(Exit::Success)), "{result:?}");
    assert_eq!(
        out,
        "liminal_a  60.00% (60/100 lines), floor 50.0%\n\
         xtask      98.36% (9836/10000 lines), floor 97.0%\ncoverage: ok\n"
    );
    let config = std::fs::read_to_string(root.path().join(CONFIG_PATH)).unwrap();
    assert_eq!(config, CONFIG);
}

/// Verifies: XTASK-R063
#[test]
fn update_floors_writes_the_raised_floors_and_checks_against_them() {
    let root = repository("coverage-update", CONFIG);
    let (result, out) = run(
        &["--update-floors"],
        root.path(),
        fixed(lines(9836, 10_000)),
    );
    assert!(matches!(result, Ok(Exit::Success)), "{result:?}");
    assert_eq!(
        std::fs::read_to_string(root.path().join(CONFIG_PATH)).unwrap(),
        "# Floors.\n[floors]\nliminal_a = 60.0\nxtask = 98.3 # comment\n"
    );
    assert!(
        out.contains("raised floor of `liminal_a`: 50.0 -> 60.0\n"),
        "{out}"
    );
    assert!(
        out.contains("raised floor of `xtask`: 97.0 -> 98.3\n"),
        "{out}"
    );
    assert!(
        out.contains("xtask      98.36% (9836/10000 lines), floor 98.3%\n"),
        "{out}"
    );
    // A second run with lower coverage lowers nothing and fails against the raised floor.
    let (result, out) = run(
        &["--update-floors"],
        root.path(),
        fixed(lines(9700, 10_000)),
    );
    assert!(matches!(result, Ok(Exit::Failure)), "{result:?}");
    assert!(!out.contains("raised floor"), "{out}");
    assert!(out.contains("error XTASK-R061 ci/coverage.toml:4"), "{out}");
    assert_eq!(
        std::fs::read_to_string(root.path().join(CONFIG_PATH)).unwrap(),
        "# Floors.\n[floors]\nliminal_a = 60.0\nxtask = 98.3 # comment\n"
    );
}

fn missing_tool(_: &[repo::Member]) -> Result<Measurement, MeasureError> {
    Err(MeasureError::Skipped {
        reason: MISSING_REASON.to_string(),
        help: MISSING_HELP.to_string(),
    })
}

/// Verifies: XTASK-R064
#[test]
fn the_command_reports_a_missing_tool_as_skipped_and_fails() {
    let root = repository("coverage-skip", CONFIG);
    let (result, out) = run(&[], root.path(), missing_tool);
    assert!(matches!(result, Ok(Exit::Failure)), "{result:?}");
    assert_eq!(
        out,
        format!("coverage: skipped: {MISSING_REASON}\n  help: {MISSING_HELP}\n")
    );
    let (result, out) = run(&["--json", "--update-floors"], root.path(), missing_tool);
    assert!(matches!(result, Ok(Exit::Failure)), "{result:?}");
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(value["ok"], false);
    assert_eq!(value["skipped"]["reason"], MISSING_REASON);
    assert_eq!(value["skipped"]["help"], MISSING_HELP);
    assert_eq!(
        std::fs::read_to_string(root.path().join(CONFIG_PATH)).unwrap(),
        CONFIG
    );
}

#[test]
fn load_and_measure_errors_are_internal_errors() {
    let root = repository("coverage-errors", "[floors]\nxtask = 78.45\n");
    let (result, _) = run(&[], root.path(), fixed(lines(1, 1)));
    let Err(CommandError::Internal { code, .. }) = result else {
        panic!("expected an internal error, got {result:?}");
    };
    assert_eq!(code, "LIM-XTASK-0004");
    let root = repository("coverage-errors-2", CONFIG);
    let (result, _) = run(&[], root.path(), |_: &[repo::Member]| {
        Err(MeasureError::Failed(LoadError::read(
            "x",
            &std::io::Error::other("e"),
        )))
    });
    assert!(
        matches!(
            result,
            Err(CommandError::Internal {
                code: "LIM-XTASK-0001",
                ..
            })
        ),
        "{result:?}"
    );
}

#[test]
fn unknown_arguments_are_usage_errors() {
    for args in [
        &["--verbose"][..],
        &["extra"][..],
        &["--update-floors=1"][..],
    ] {
        let args: Vec<String> = args.iter().map(ToString::to_string).collect();
        assert!(matches!(parse_options(&args), Err(CommandError::Usage(_))));
    }
}

/// Verifies: XTASK-R064
#[test]
fn the_ci_step_maps_pass_findings_and_skip_and_passes_the_output_mode_on() {
    let root = repository("coverage-step", CONFIG);
    for (mode, stderr) in [
        (OutputMode::Capture, Stderr::Capture),
        (OutputMode::Inherit, Stderr::Inherit),
    ] {
        let measured = |xtask: Lines| {
            move |members: &[repo::Member], given: Stderr| {
                assert_eq!(given, stderr);
                fixed(xtask)(members)
            }
        };
        assert_eq!(
            run_step(root.path(), mode, measured(lines(970, 1000))),
            Outcome::Pass
        );
        let Outcome::Fail {
            findings,
            output_tail: None,
        } = run_step(root.path(), mode, measured(lines(964, 1000)))
        else {
            panic!("expected a failure with findings");
        };
        assert_eq!(rules(&findings), [("XTASK-R061", "ci/coverage.toml:4")]);
        assert_eq!(
            run_step(root.path(), mode, |members: &[repo::Member], _| {
                missing_tool(members)
            }),
            Outcome::Skipped {
                reason: MISSING_REASON.to_string(),
                help: MISSING_HELP.to_string(),
            }
        );
    }
    // The ci step never updates floors.
    assert_eq!(
        std::fs::read_to_string(root.path().join(CONFIG_PATH)).unwrap(),
        CONFIG
    );
}

/// Verifies: XTASK-R064
#[test]
fn the_real_ci_step_is_skipped_when_cargo_cannot_start() {
    let root = repository("coverage-step-no-cargo", CONFIG);
    let outcome = run_real_step(
        root.path(),
        OsStr::new("xtask-test-no-such-program"),
        OutputMode::Capture,
    );
    let Outcome::Skipped { reason, .. } = outcome else {
        panic!("expected a skip, got {outcome:?}");
    };
    assert!(reason.starts_with("could not start"), "{reason}");
}

#[test]
fn the_ci_step_fails_with_internal_errors_as_output_tail() {
    let empty = TempRoot::new("coverage-step-error");
    let invalid = repository("coverage-step-invalid", "[floors]\nxtask = 78.45\n");
    let measuring_failed = repository("coverage-step-failed", CONFIG);
    let cases: [(&Path, &str, bool); 3] = [
        (empty.path(), "LIM-XTASK-0001: Cargo.toml", false),
        (invalid.path(), "LIM-XTASK-0004: ci/coverage.toml", false),
        (measuring_failed.path(), "LIM-XTASK-0006", true),
    ];
    for (root, prefix, measure_fails) in cases {
        let outcome = run_step(root, OutputMode::Capture, |_: &[repo::Member], _| {
            assert!(measure_fails, "measured although loading failed");
            Err(MeasureError::Failed(LoadError {
                code: "LIM-XTASK-0006",
                path: "cargo llvm-cov".to_string(),
                detail: "failed".to_string(),
                help: "h",
            }))
        });
        let Outcome::Fail {
            findings,
            output_tail: Some(tail),
        } = outcome
        else {
            panic!("expected a failure with output_tail, got {outcome:?}");
        };
        assert_eq!(findings, Vec::new());
        assert!(tail.starts_with(prefix), "{tail}");
    }
}

/// Verifies: XTASK-R062
#[test]
fn human_mode_shows_crates_without_lines_or_floor_and_the_finding() {
    let root = repository("coverage-human-gaps", "[floors]\nxtask = 97.0\n");
    let (result, out) = run(&[], root.path(), |_: &[repo::Member]| {
        Ok(measurement(&[
            ("liminal_a", lines(0, 0)),
            ("xtask", lines(980, 1000)),
        ]))
    });
    assert!(matches!(result, Ok(Exit::Failure)), "{result:?}");
    assert_eq!(
        out,
        "liminal_a  no lines measured, no floor\n\
         xtask      98.00% (980/1000 lines), floor 97.0%\n\
         error XTASK-R062 ci/coverage.toml: workspace crate `liminal_a` has no coverage floor\n  \
         help: add `liminal_a = 0.0` under `[floors]` (its measured coverage, rounded down to \
         0.1)\ncoverage: 1 finding\n"
    );
}
