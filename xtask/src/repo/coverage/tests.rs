use super::*;
use crate::test_support::TempRoot;

fn member(name: &str, path: &str) -> Member {
    Member {
        path: path.to_string(),
        name: name.to_string(),
    }
}

fn lines(covered: u64, count: u64) -> Lines {
    Lines { covered, count }
}

/// An llvm-cov export with one file per `(filename, covered, count)`.
fn export(files: &[(&str, u64, u64)]) -> String {
    let files: Vec<serde_json::Value> = files
        .iter()
        .map(|(filename, covered, count)| {
            serde_json::json!({ "filename": filename, "summary": {
                "lines": { "count": count, "covered": covered, "percent": 0.0 },
                "functions": { "count": 0, "covered": 0, "percent": 0.0 } } })
        })
        .collect();
    serde_json::json!({ "type": "llvm.coverage.json.export", "version": "3.1.0",
        "data": [ { "files": files, "totals": {} } ] })
    .to_string()
}

fn error_code(result: Result<Measurement, LoadError>) -> &'static str {
    match result {
        Ok(measurement) => panic!("expected an error, got {measurement:?}"),
        Err(error) => error.code,
    }
}

/// Verifies: XTASK-R060
#[test]
fn a_saved_llvm_cov_export_is_summed_per_workspace_crate() {
    let json = include_str!("../../../tests/fixtures/coverage/llvm-cov-summary.json");
    let members = [
        member("xtask", "xtask"),
        member("liminal_core", "crates/liminal_core"),
    ];
    let measurement = parse_export(json, "/work/liminal", &members).unwrap();
    // The fixture's 26 files are all below `xtask/`; its `totals.lines` is 4220 of 4290.
    assert_eq!(
        measurement,
        Measurement::from([
            ("liminal_core".to_string(), lines(0, 0)),
            ("xtask".to_string(), lines(4220, 4290)),
        ])
    );
}

/// Verifies: XTASK-R060
#[test]
fn files_map_to_the_innermost_member_and_others_are_ignored() {
    let json = export(&[
        ("/work/liminal/outer/src/lib.rs", 1, 2),
        ("/work/liminal/outer/inner/src/lib.rs", 3, 4),
        ("/work/liminal/outer/inner/src/more.rs", 5, 5),
        // Not below a member, outside the root, and a sibling directory whose name starts with
        // the root's name: all ignored.
        ("/work/liminal/build.rs", 7, 9),
        ("/rustc/abc/library/core/src/lib.rs", 10, 20),
        ("/work/liminal-old/outer/src/lib.rs", 10, 20),
        // A member-like prefix that is not a directory boundary.
        ("/work/liminal/outer2/src/lib.rs", 10, 20),
    ]);
    // `inner` is listed first: the length of the directory decides, not the order.
    let members = [member("inner", "outer/inner"), member("outer", "outer")];
    let measurement = parse_export(&json, "/work/liminal/", &members).unwrap();
    assert_eq!(
        measurement,
        Measurement::from([
            ("inner".to_string(), lines(8, 9)),
            ("outer".to_string(), lines(1, 2)),
        ])
    );
}

/// Verifies: XTASK-R060
#[test]
fn windows_backslash_paths_are_read_as_slashes() {
    let json = export(&[(r"C:\work\liminal\xtask\src\lib.rs", 3, 4)]);
    let measurement = parse_export(&json, r"C:\work\liminal", &[member("xtask", "xtask")]).unwrap();
    assert_eq!(measurement["xtask"], lines(3, 4));
}

/// Verifies: XTASK-R060
#[test]
fn a_member_spelled_dot_slash_owns_its_files() {
    // `members = ["./xtask"]` is valid for cargo; its files must still count for `xtask`.
    let root = TempRoot::new("coverage-dot-member");
    std::fs::write(
        root.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"./xtask\"]\n",
    )
    .unwrap();
    std::fs::create_dir_all(root.path().join("xtask")).unwrap();
    std::fs::write(
        root.path().join("xtask/Cargo.toml"),
        "[package]\nname = \"xtask\"\n",
    )
    .unwrap();
    let members = workspace::load(root.path()).unwrap();
    let json = export(&[("/work/liminal/xtask/src/lib.rs", 3, 4)]);
    let measurement = parse_export(&json, "/work/liminal", &members).unwrap();
    assert_eq!(measurement["xtask"], lines(3, 4));
}

#[test]
fn an_export_without_files_measures_nothing() {
    let measurement =
        parse_export(&export(&[]), "/work/liminal", &[member("xtask", "xtask")]).unwrap();
    assert_eq!(measurement["xtask"], lines(0, 0));
}

#[test]
fn files_none_of_which_is_inside_the_root_are_an_error() {
    // A root spelled differently from the tool's paths must not make every crate "unmeasured".
    let json = export(&[("/private/work/liminal/xtask/src/lib.rs", 3, 4)]);
    let result = parse_export(&json, "/work/liminal", &[member("xtask", "xtask")]);
    let Err(error) = result else {
        panic!("expected an error, got {result:?}");
    };
    assert_eq!(error.code, "LIM-XTASK-0002");
    assert!(
        error.detail.contains("none of its 1 files"),
        "{}",
        error.detail
    );
}

#[test]
fn malformed_exports_are_unparseable_tool_output() {
    let members = [member("xtask", "xtask")];
    let wrong_type = export(&[]).replace("llvm.coverage.json.export", "something.else");
    let more_covered_than_lines = export(&[("/work/liminal/xtask/src/lib.rs", 5, 4)]);
    for json in [
        "not json",
        "{}",
        "{\"type\": \"llvm.coverage.json.export\", \"data\": [{\"files\": [{\"filename\": \"a\"}]}]}",
        &wrong_type,
        &more_covered_than_lines,
    ] {
        assert_eq!(
            error_code(parse_export(json, "/work/liminal", &members)),
            "LIM-XTASK-0002",
            "{json}"
        );
    }
}

#[test]
fn floors_are_read_in_tenths_with_their_line_and_value_span() {
    let text = "# header\n[floors]\nxtask = 78.4\nliminal_core = 85\nfull = 100.0\nzero = 0\n\
                int_full = 100\n";
    let floors = parse_floors(text).unwrap();
    let tenths: Vec<(&str, u32, usize)> = floors
        .iter()
        .map(|(name, floor)| (name.as_str(), floor.tenths, floor.line))
        .collect();
    assert_eq!(
        tenths,
        [
            ("full", 1000, 5),
            ("int_full", 1000, 7),
            ("liminal_core", 850, 4),
            ("xtask", 784, 3),
            ("zero", 0, 6)
        ]
    );
    let (start, end) = floors["xtask"].span;
    assert_eq!(&text[start..end], "78.4");
    let (start, end) = floors["liminal_core"].span;
    assert_eq!(&text[start..end], "85");
}

#[test]
fn an_empty_floors_table_is_valid() {
    assert_eq!(parse_floors("[floors]\n").unwrap(), Floors::new());
}

#[test]
fn invalid_floor_files_are_lim_xtask_0004() {
    for text in [
        "",
        "[floors]\nxtask = 78.45\n",
        "[floors]\nxtask = 78.44\n",
        "[floors]\nxtask = 100.1\n",
        "[floors]\nxtask = 101\n",
        "[floors]\nxtask = -1\n",
        "[floors]\nxtask = -0.1\n",
        "[floors]\nxtask = \"78.4\"\n",
        "[floors]\nxtask = nan\n",
        "[floors]\nxtask = inf\n",
        "[floors]\nxtask = 78.4\n[extra]\n",
        "floors = 1\n",
        "[floors\n",
    ] {
        let result = parse_floors(text);
        let Err(error) = result else {
            panic!("expected an error for {text:?}, got {result:?}");
        };
        assert_eq!(error.code, "LIM-XTASK-0004", "{text:?}");
        assert_eq!(error.path, CONFIG_PATH);
    }
    let Err(error) = parse_floors("[floors]\n\nxtask = 78.45\n") else {
        panic!("expected an error");
    };
    assert!(
        error.detail.starts_with("line 3: the floor of `xtask`"),
        "{}",
        error.detail
    );
}

/// Verifies: XTASK-R060
#[test]
fn the_measuring_command_is_the_one_the_spec_names() {
    assert_eq!(
        LLVM_COV_ARGS,
        [
            "llvm-cov",
            "--workspace",
            "--all-features",
            "--locked",
            "--json",
            "--summary-only",
            "--no-default-ignore-filename-regex",
            "--ignore-filename-regex",
            IGNORE_FILENAME_REGEX
        ]
    );
    assert_eq!(LLVM_COV_PROBE, ["llvm-cov", "--version"]);
}

fn cargo() -> std::ffi::OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into())
}

/// Verifies: XTASK-R064
#[test]
fn a_failing_tool_probe_is_a_skip_with_an_install_hint() {
    let root = TempRoot::new("coverage-probe");
    let result = measure(
        root.path(),
        &cargo(),
        &["xtask-test-no-such-subcommand"],
        &[],
        Stderr::Capture,
    );
    assert_eq!(
        result,
        Err(MeasureError::Skipped {
            reason: MISSING_REASON.to_string(),
            help: MISSING_HELP.to_string(),
        })
    );
    assert!(MISSING_HELP.starts_with("cargo install cargo-llvm-cov"));
}

/// Verifies: XTASK-R064
#[test]
fn a_cargo_that_cannot_start_is_a_skip() {
    let root = TempRoot::new("coverage-no-cargo");
    let result = measure(
        root.path(),
        OsStr::new("xtask-test-no-such-program"),
        LLVM_COV_PROBE,
        &[],
        Stderr::Capture,
    );
    let Err(MeasureError::Skipped { reason, help }) = result else {
        panic!("expected a skip, got {result:?}");
    };
    assert!(
        reason.starts_with("could not start `xtask-test-no-such-program`"),
        "{reason}"
    );
    assert_eq!(help, crate::process::CARGO_HELP);
}

#[test]
fn a_failing_llvm_cov_run_is_lim_xtask_0006_with_its_stderr_tail() {
    // The probe `--version` passes. `cargo llvm-cov` then fails in a directory without a
    // `Cargo.toml` above it (the system temp directory): "could not find `Cargo.toml`" if
    // cargo-llvm-cov is installed, "no such command" otherwise. Both are failures.
    let root = TempRoot::new("coverage-fails");
    let result = measure(root.path(), &cargo(), &["--version"], &[], Stderr::Capture);
    let Err(MeasureError::Failed(error)) = result else {
        panic!("expected a failure, got {result:?}");
    };
    assert_eq!(error.code, "LIM-XTASK-0006");
    assert_eq!(
        error.path,
        format!(
            "cargo llvm-cov --workspace --all-features --locked --json --summary-only \
             --no-default-ignore-filename-regex --ignore-filename-regex {IGNORE_FILENAME_REGEX}"
        )
    );
    assert!(error.detail.starts_with("failed ("), "{}", error.detail);
    assert!(
        error.detail.contains("Cargo.toml") || error.detail.contains("no such command"),
        "{}",
        error.detail
    );
}
