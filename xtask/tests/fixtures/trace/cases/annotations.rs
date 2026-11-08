/// Verifies: DEMO-R001
#[test]
fn a() {}

/// Doc line.
/// Verifies: DEMO-R001, DEMO-R002
///   more doc
// plain comment
#[cfg_attr(
    miri,
    ignore = "M0-T005: slow"
)]
pub(crate) async unsafe fn b() {}

/// Verifies:DEMO-R001 ,DEMO-R002, DEMO-R001
fn c() {}

/// Verifies: demo-r001
/// Verifies: DEMO-R001,
/// Verifies: DEMO-R001 and more
/// Verifies:
/// Verifies that the loader works.
//// Verifies: DEMO-R001
fn d() {}

/// Verifies: DEMO-R001
struct S;

fn e() {} /// Verifies: DEMO-R001
const T: &str = "
/// Verifies: DEMO-R009
";
/// Verifies: DEMO-R003
pub extern "C" fn f() {}
/// Verifies: DEMO-R001
mod m {}
/// Verifies: DEMO-R001
