//! XTASK-R090: engine crates (layers 0–4) have no benchmark targets; engine benchmarks live in
//! `liminal_benches` (ADR-0023 Decision 1 and "Verification").

use super::rules::{judged, manifest};
use crate::finding::Finding;
use crate::repo::ArchModel;

/// The lowest layer whose crates may have benchmark targets (the tools layer, ADR-0004).
const TOOLS_LAYER: i64 = 5;

/// XTASK-R090: one finding per target of a judged layer 0–4 crate that has kind `bench` or
/// whose root source file is under `<manifest directory>/benches/`.
pub(super) fn engine_benches(model: &ArchModel, findings: &mut Vec<Finding>) {
    for package in &model.packages {
        let Some(entry) = judged(&model.architecture, &package.name) else {
            continue;
        };
        if entry.layer >= TOOLS_LAYER {
            continue;
        }
        let benches_dir = if package.manifest_dir.is_empty() {
            "benches/".to_string()
        } else {
            format!("{}/benches/", package.manifest_dir)
        };
        for target in &package.targets {
            let is_bench = target.kinds.iter().any(|kind| kind == "bench");
            if !is_bench && !target.src_path.starts_with(&benches_dir) {
                continue;
            }
            findings.push(Finding::error(
                "XTASK-R090",
                manifest(package),
                format!(
                    "engine crate `{}` (layer {}) has a benchmark target `{}`; engine crates \
                     have no `benches/` targets",
                    package.name, entry.layer, target.src_path
                ),
                "Move the benchmark to a bench target of `liminal_benches` \
                 (`benches/benches/<crate>.rs`) that measures the crate through its public API \
                 (ADR-0023)",
            ));
        }
    }
}
