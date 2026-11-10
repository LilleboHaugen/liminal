//! The crate's operation set (spec §10 "Operation set"): MATH-R004. Reads the crate's clippy
//! configuration and fails if a path that MATH-R004 bans is missing from `disallowed-methods`.

/// The paths MATH-R004 requires in `crates/liminal_math/clippy.toml`, written from the
/// requirement text.
fn required_paths() -> Vec<String> {
    let float_functions = [
        // CODE_STYLE.md list.
        "sin", "cos", "tan", "powf", "exp", "ln",
        // Added by MATH-R004.
        "mul_add", "sin_cos", "asin", "acos", "atan", "atan2", "sinh", "cosh", "tanh", "asinh",
        "acosh", "atanh", "powi", "exp2", "exp_m1", "ln_1p", "log", "log2", "log10", "hypot",
        "cbrt",
    ];
    let x86 = ["_mm_rsqrt_ps", "_mm_rsqrt_ss", "_mm_rcp_ps", "_mm_rcp_ss", "_mm_fmadd_ps"];
    let arm = [
        "vrsqrteq_f32",
        "vrecpeq_f32",
        "vfmaq_f32",
        "vfmsq_f32",
        "vmlaq_f32",
        "vmlsq_f32",
    ];
    let mut paths = Vec::new();
    for float in ["f32", "f64"] {
        for function in float_functions {
            paths.push(format!("{float}::{function}"));
        }
    }
    paths.extend(x86.iter().map(|name| format!("core::arch::x86_64::{name}")));
    paths.extend(arm.iter().map(|name| format!("core::arch::aarch64::{name}")));
    paths
}

/// The `path` value of every `disallowed-methods` entry of a clippy.toml, read line by line:
/// the file writes one `{ path = "…", … }` table per line.
fn configured_paths(text: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut in_list = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("disallowed-methods") {
            in_list = true;
            continue;
        }
        if in_list && line.starts_with(']') {
            break;
        }
        if in_list && let Some(rest) = line.strip_prefix("{ path = \"") {
            let end = rest.find('"').expect("a closing quote after the path");
            paths.push(rest[..end].to_string());
        }
    }
    paths
}

/// Verifies: MATH-R004
#[test]
fn clippy_configuration_bans_every_path_of_math_r004() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/clippy.toml");
    let text = std::fs::read_to_string(path).unwrap();
    let configured = configured_paths(&text);
    let missing: Vec<String> = required_paths()
        .into_iter()
        .filter(|required| !configured.contains(required))
        .collect();
    assert!(missing.is_empty(), "missing in {path}: {missing:?}");
    // The XTASK-R039 entries are there too.
    for required in ["std::collections::HashMap::new", "std::time::Instant::now"] {
        assert!(configured.iter().any(|p| p == required), "{required}");
    }
}

/// Verifies: MATH-R004
#[test]
fn the_line_reader_finds_entries_and_stops_at_the_list_end() {
    let text = "a = 1\ndisallowed-methods = [\n  # comment\n  { path = \"f32::sin\", reason = \"x\" },\n]\n{ path = \"after\" }\n";
    assert_eq!(configured_paths(text), ["f32::sin"]);
}
