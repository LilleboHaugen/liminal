#[test]
#[ignore]
fn a() {}
#[ignore = "needs GPU"]
#[ignore = "M0-T005: needs GPU"]
#[ignore = "M0-T001: a done task is fine for ignores"]
#[ignore = "M9-T999: unknown task"]
#[cfg_attr(miri, ignore)]
#[cfg_attr(not(feature = "gpu"), ignore = "M0-T005: needs a GPU")]
#[
    ignore
]
// #[ignore] in a comment
const S: &str = "#[ignore]";
#[doc = "ignore"]
use ignore::Walk;
#[serde(rename = "ignore")]
#[ignore = "M0-T0050: not a task ID"]
#![ignore]
#[tool::ignore]
