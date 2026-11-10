//! The result file (spec §5 "Result file", BENCH-R030 … R035).

use std::fmt::Write as _;

use crate::{Metadata, Summary, Threading};

/// The result file format version (ADR-0023).
pub const FORMAT_VERSION: u32 = 1;

/// The measured result of one case.
#[derive(Clone, Debug, PartialEq)]
pub struct CaseResult {
    /// The case ID, `<crate>/<group>/<case>`.
    pub id: String,
    /// The threading marker of the case.
    pub threading: Threading,
    /// Elements per iteration, if the case declares them.
    pub elements: Option<u64>,
    /// Iterations in every sample.
    pub iterations_per_sample: u64,
    /// Statistics of the samples, ns per iteration.
    pub summary: Summary,
}

/// The results of one bench target: what [`crate::Bench::finish`] writes as JSON.
#[derive(Clone, Debug, PartialEq)]
pub struct Report {
    /// The bench target name, also the file stem.
    pub target: String,
    /// Where and how the results were measured.
    pub metadata: Metadata,
    /// Results in the order the cases ran.
    pub cases: Vec<CaseResult>,
}

impl Report {
    /// The result file contents (BENCH-R031): one JSON object, two-space indentation, keys in
    /// the order of spec §5, `\n` line ends and a final `\n`.
    ///
    /// # Examples
    ///
    /// ```
    /// use liminal_benches::{Metadata, Report};
    ///
    /// let report = Report { target: "demo".into(), metadata: Metadata::detect(), cases: vec![] };
    /// let json = report.to_json();
    /// assert!(json.starts_with("{\n  \"version\": 1,\n  \"target\": \"demo\",\n"));
    /// assert!(json.ends_with("  \"cases\": []\n}\n"));
    /// ```
    #[must_use]
    pub fn to_json(&self) -> String {
        let m = &self.metadata;
        let mut out = String::new();
        out.push_str("{\n");
        let _ = writeln!(out, "  \"version\": {FORMAT_VERSION},");
        let _ = writeln!(out, "  \"target\": {},", string(&self.target));
        out.push_str("  \"machine\": {\n");
        let fields = [
            ("id", &m.machine_id),
            ("os", &m.os),
            ("arch", &m.arch),
            ("os_version", &m.os_version),
            ("toolchain", &m.toolchain),
            ("commit", &m.commit),
            ("power", &m.power),
        ];
        for (i, (key, value)) in fields.iter().enumerate() {
            let comma = if i + 1 < fields.len() { "," } else { "" };
            let _ = writeln!(out, "    \"{key}\": {}{comma}", string(value));
        }
        out.push_str("  },\n");
        if self.cases.is_empty() {
            out.push_str("  \"cases\": []\n");
        } else {
            out.push_str("  \"cases\": [\n");
            for (i, case) in self.cases.iter().enumerate() {
                let comma = if i + 1 < self.cases.len() { "," } else { "" };
                case_json(&mut out, case);
                let _ = writeln!(out, "    }}{comma}");
            }
            out.push_str("  ]\n");
        }
        out.push_str("}\n");
        out
    }
}

/// One case object without its closing brace.
fn case_json(out: &mut String, case: &CaseResult) {
    let s = &case.summary;
    let elements = case
        .elements
        .map_or_else(|| "null".to_string(), |e| e.to_string());
    let lines = [
        ("id", string(&case.id)),
        (
            "single_threaded",
            (case.threading == Threading::Single).to_string(),
        ),
        ("elements", elements),
        (
            "iterations_per_sample",
            case.iterations_per_sample.to_string(),
        ),
        ("samples", s.samples.to_string()),
        ("median_ns", number(Some(s.median_ns))),
        ("mad_ns", number(Some(s.mad_ns))),
        ("min_ns", number(Some(s.min_ns))),
        ("max_ns", number(Some(s.max_ns))),
        (
            "throughput_per_s",
            number(s.throughput_per_s(case.elements)),
        ),
    ];
    out.push_str("    {\n");
    for (i, (key, value)) in lines.iter().enumerate() {
        let comma = if i + 1 < lines.len() { "," } else { "" };
        let _ = writeln!(out, "      \"{key}\": {value}{comma}");
    }
}

/// A JSON number: Rust's shortest round-trip decimal form; `null` for `None`, NaN and
/// infinities (BENCH-R034).
fn number(value: Option<f64>) -> String {
    match value {
        Some(v) if v.is_finite() => format!("{v}"),
        _ => "null".to_string(),
    }
}

/// A JSON string literal (RFC 8259 §7): `"` and `\` escaped, control characters as `\u00XX`
/// (BENCH-R034).
fn string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if u32::from(c) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata() -> Metadata {
        Metadata {
            machine_id: "m".into(),
            os: "macos".into(),
            arch: "aarch64".into(),
            os_version: "27.2".into(),
            toolchain: "rustc 1.99.1".into(),
            commit: "abc".into(),
            power: "ac".into(),
        }
    }

    /// Verifies: BENCH-R031, BENCH-R035
    #[test]
    fn report_json_has_the_spec_layout() {
        let summary = Summary::from_samples(&[2.0, 4.0]).expect("valid");
        let report = Report {
            target: "liminal_core".into(),
            metadata: metadata(),
            cases: vec![
                CaseResult {
                    id: "liminal_core/hash/b".into(),
                    threading: Threading::Single,
                    elements: Some(6),
                    iterations_per_sample: 8,
                    summary,
                },
                CaseResult {
                    id: "liminal_core/hash/a".into(),
                    threading: Threading::Multi,
                    elements: None,
                    iterations_per_sample: 1,
                    summary,
                },
            ],
        };
        let expected = r#"{
  "version": 1,
  "target": "liminal_core",
  "machine": {
    "id": "m",
    "os": "macos",
    "arch": "aarch64",
    "os_version": "27.2",
    "toolchain": "rustc 1.99.1",
    "commit": "abc",
    "power": "ac"
  },
  "cases": [
    {
      "id": "liminal_core/hash/b",
      "single_threaded": true,
      "elements": 6,
      "iterations_per_sample": 8,
      "samples": 2,
      "median_ns": 3,
      "mad_ns": 1,
      "min_ns": 2,
      "max_ns": 4,
      "throughput_per_s": 2000000000
    },
    {
      "id": "liminal_core/hash/a",
      "single_threaded": false,
      "elements": null,
      "iterations_per_sample": 1,
      "samples": 2,
      "median_ns": 3,
      "mad_ns": 1,
      "min_ns": 2,
      "max_ns": 4,
      "throughput_per_s": null
    }
  ]
}
"#;
        assert_eq!(report.to_json(), expected);
    }

    /// Verifies: BENCH-R034
    #[test]
    fn strings_are_escaped_and_non_finite_numbers_are_null() {
        assert_eq!(string("a\"b\\c\n\u{1}é"), r#""a\"b\\c\u000a\u0001é""#);
        assert_eq!(number(Some(f64::NAN)), "null");
        assert_eq!(number(Some(f64::INFINITY)), "null");
        assert_eq!(number(None), "null");
        assert_eq!(number(Some(0.125)), "0.125");
        assert_eq!(number(Some(1e-7)), "0.0000001");
    }
}
