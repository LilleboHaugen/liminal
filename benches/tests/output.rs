//! Integration tests of the result file and the bench target layout through the public API
//! (spec §10). The result file is parsed with a minimal JSON reader and compared with the
//! schema of spec §5 "Result file".

use std::collections::BTreeMap;
use std::hint::black_box;
use std::num::NonZeroU64;
use std::path::Path;

use liminal_benches::{Bench, Case, Config};

/// A parsed JSON value. Objects keep their key order.
#[derive(Debug, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Number(f64),
    Text(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

/// A recursive-descent reader for the JSON subset the harness writes (RFC 8259 without
/// surrogate-pair escapes).
struct Reader<'a> {
    rest: &'a str,
}

impl Reader<'_> {
    fn skip_space(&mut self) {
        self.rest = self.rest.trim_start_matches([' ', '\n', '\r', '\t']);
    }

    fn eat(&mut self, token: &str) -> bool {
        self.skip_space();
        if let Some(rest) = self.rest.strip_prefix(token) {
            self.rest = rest;
            true
        } else {
            false
        }
    }

    fn value(&mut self) -> Json {
        self.skip_space();
        if self.eat("null") {
            Json::Null
        } else if self.eat("true") {
            Json::Bool(true)
        } else if self.eat("false") {
            Json::Bool(false)
        } else if self.rest.starts_with('"') {
            Json::Text(self.string())
        } else if self.eat("[") {
            let mut items = Vec::new();
            if !self.eat("]") {
                loop {
                    items.push(self.value());
                    if self.eat("]") {
                        break;
                    }
                    assert!(self.eat(","), "expected , in array at {:?}", self.rest);
                }
            }
            Json::Array(items)
        } else if self.eat("{") {
            let mut fields = Vec::new();
            if !self.eat("}") {
                loop {
                    self.skip_space();
                    let key = self.string();
                    assert!(self.eat(":"), "expected : at {:?}", self.rest);
                    fields.push((key, self.value()));
                    if self.eat("}") {
                        break;
                    }
                    assert!(self.eat(","), "expected , in object at {:?}", self.rest);
                }
            }
            Json::Object(fields)
        } else {
            let end = self
                .rest
                .find(|c: char| !(c.is_ascii_digit() || "-+.eE".contains(c)))
                .unwrap_or(self.rest.len());
            let number = self.rest[..end].parse().expect("a JSON number");
            self.rest = &self.rest[end..];
            Json::Number(number)
        }
    }

    fn string(&mut self) -> String {
        let mut chars = self
            .rest
            .strip_prefix('"')
            .expect("a string")
            .char_indices();
        let mut out = String::new();
        while let Some((i, c)) = chars.next() {
            match c {
                '"' => {
                    self.rest = &self.rest[i + 2..];
                    return out;
                }
                '\\' => match chars.next().expect("escape").1 {
                    'u' => {
                        let hex: String = (0..4).map(|_| chars.next().expect("hex").1).collect();
                        let code = u32::from_str_radix(&hex, 16).expect("hex digits");
                        out.push(char::from_u32(code).expect("scalar value"));
                    }
                    other => out.push(other),
                },
                c => out.push(c),
            }
        }
        panic!("unterminated string");
    }
}

fn parse(text: &str) -> Json {
    let mut reader = Reader { rest: text };
    let value = reader.value();
    reader.skip_space();
    assert_eq!(reader.rest, "", "trailing text");
    value
}

fn object(value: &Json) -> &[(String, Json)] {
    match value {
        Json::Object(fields) => fields,
        other => panic!("expected an object, got {other:?}"),
    }
}

fn keys(fields: &[(String, Json)]) -> Vec<&str> {
    fields.iter().map(|(k, _)| k.as_str()).collect()
}

fn get<'a>(fields: &'a [(String, Json)], key: &str) -> &'a Json {
    &fields
        .iter()
        .find(|(k, _)| k == key)
        .expect("key present")
        .1
}

/// Verifies: BENCH-R030, BENCH-R031, BENCH-R032, BENCH-R035
#[test]
fn a_bench_run_with_fixed_iterations_writes_the_specified_json() {
    let out_dir = std::env::temp_dir().join(format!("liminal-bench-output-{}", std::process::id()));
    let config = Config {
        iterations: NonZeroU64::new(1000),
        samples: 5,
        out_dir: out_dir.clone(),
        ..Config::default()
    };
    let mut bench = Bench::new("demo", config).expect("valid target");
    let mut total = 0_u64;
    let sum = Case::new("demo/arith/sum")
        .expect("valid")
        .with_elements(NonZeroU64::new(4).expect("non-zero"));
    bench
        .run(&sum, || {
            total = total.wrapping_add(black_box(1) + black_box(2) + black_box(3) + black_box(4));
            total
        })
        .expect("runs");
    let spin = Case::new("demo/arith/spin")
        .expect("valid")
        .multi_threaded();
    bench.run(&spin, || black_box(7_u32) * 3).expect("runs");
    let path = bench.finish().expect("written");
    assert_eq!(path, out_dir.join("demo.json"));

    let text = std::fs::read_to_string(&path).expect("readable");
    assert!(text.ends_with("}\n") && !text.contains('\r'));
    let root = parse(&text);
    let root = object(&root);
    assert_eq!(keys(root), ["version", "target", "machine", "cases"]);
    assert_eq!(get(root, "version"), &Json::Number(1.0));
    assert_eq!(get(root, "target"), &Json::Text("demo".into()));

    let machine = object(get(root, "machine"));
    assert_eq!(
        keys(machine),
        [
            "id",
            "os",
            "arch",
            "os_version",
            "toolchain",
            "commit",
            "power"
        ]
    );
    for (_, value) in machine {
        assert!(matches!(value, Json::Text(t) if !t.is_empty()), "{value:?}");
    }
    assert_eq!(get(machine, "os"), &Json::Text(std::env::consts::OS.into()));

    let Json::Array(cases) = get(root, "cases") else {
        panic!("cases is not an array")
    };
    let ids: Vec<&Json> = cases.iter().map(|c| get(object(c), "id")).collect();
    assert_eq!(
        ids,
        [
            &Json::Text("demo/arith/sum".into()),
            &Json::Text("demo/arith/spin".into())
        ]
    );
    let mut threaded = BTreeMap::new();
    for case in cases {
        let case = object(case);
        assert_eq!(
            keys(case),
            [
                "id",
                "single_threaded",
                "elements",
                "iterations_per_sample",
                "samples",
                "median_ns",
                "mad_ns",
                "min_ns",
                "max_ns",
                "throughput_per_s"
            ]
        );
        assert_eq!(get(case, "iterations_per_sample"), &Json::Number(1000.0));
        assert_eq!(get(case, "samples"), &Json::Number(5.0));
        let ns = |key| match get(case, key) {
            Json::Number(n) => *n,
            other => panic!("{key} is {other:?}"),
        };
        assert!(ns("min_ns") <= ns("median_ns") && ns("median_ns") <= ns("max_ns"));
        assert!(ns("mad_ns") >= 0.0);
        if let Json::Text(id) = get(case, "id") {
            threaded.insert(id.clone(), get(case, "single_threaded"));
        }
    }
    assert_eq!(threaded["demo/arith/sum"], &Json::Bool(true));
    assert_eq!(threaded["demo/arith/spin"], &Json::Bool(false));
    let spin = object(&cases[1]);
    assert_eq!(get(spin, "elements"), &Json::Null);
    assert_eq!(get(spin, "throughput_per_s"), &Json::Null);
    assert_eq!(get(object(&cases[0]), "elements"), &Json::Number(4.0));

    std::fs::remove_dir_all(&out_dir).expect("cleanup");
}

/// Verifies: BENCH-R060
#[test]
fn bench_targets_are_named_after_a_crate_and_use_the_own_harness() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).expect("manifest");
    let blocks: Vec<&str> = manifest.split("[[bench]]").skip(1).collect();
    assert!(!blocks.is_empty(), "no [[bench]] target");
    for block in blocks {
        let block = block.split("\n[").next().expect("block text");
        let name = block
            .lines()
            .find_map(|l| l.trim().strip_prefix("name = "))
            .expect("bench name")
            .trim_matches('"');
        assert!(
            block.lines().any(|l| l.trim() == "harness = false"),
            "{name}: harness = false missing"
        );
        assert!(
            root.join("benches").join(format!("{name}.rs")).is_file(),
            "{name}.rs"
        );
        assert!(
            root.join("..")
                .join("crates")
                .join(name)
                .join("Cargo.toml")
                .is_file(),
            "{name} is not a crate under crates/"
        );
    }
    // Every file in benches/benches/ is a declared target.
    for entry in std::fs::read_dir(root.join("benches")).expect("benches dir") {
        let file = entry.expect("entry").file_name();
        let stem = file.to_string_lossy().trim_end_matches(".rs").to_string();
        assert!(
            manifest.contains(&format!("name = \"{stem}\"")),
            "benches/{stem}.rs has no [[bench]] entry"
        );
    }
}
