//! Tests of XTASK-R085 (expiry of ignored advisories in `deny.toml`).

use std::collections::BTreeMap;

use super::*;

const TODAY: Date = Date {
    year: 2026,
    month: 11,
    day: 9,
};

fn tasks() -> BTreeMap<String, Option<String>> {
    BTreeMap::from([
        ("M1-T008".to_string(), Some("in-progress".to_string())),
        ("M0-T001".to_string(), Some("done".to_string())),
        ("M1-T050".to_string(), None),
    ])
}

/// `(location, message)` of every finding for the `[advisories]` table `table`.
fn found(table: &str) -> Vec<(String, String)> {
    let entries = parse(table).unwrap();
    check(&entries, TODAY, &tasks())
        .into_iter()
        .map(|f| {
            assert_eq!(f.rule, "XTASK-R085");
            assert_eq!(f.severity, crate::finding::Severity::Error);
            (f.location, f.message)
        })
        .collect()
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(a, b)| ((*a).to_string(), (*b).to_string()))
        .collect()
}

/// Verifies: XTASK-R085
#[test]
fn entries_with_a_future_expiry_and_an_open_task_pass() {
    let text = "[advisories]\nignore = [\n\
        { id = \"RUSTSEC-2026-0001\", reason = \"not reachable; expires 2026-11-10 (M1-T008)\" },\n\
        { crate = \"demo@1.0.0\", reason = \"dev only;  expires 2027-02-28 (M1-T050)  \" },\n\
        { id = \"RUSTSEC-2026-0002\", reason = \"expires 2028-02-29 (M1-T008)\" },\n\
        ]\n";
    assert_eq!(found(text), Vec::new());
}

/// Verifies: XTASK-R085
#[test]
fn an_entry_expires_on_its_date() {
    let text = "[advisories]\nignore = [\n\
        { id = \"RUSTSEC-2026-0001\", reason = \"r; expires 2026-11-09 (M1-T008)\" },\n\
        { id = \"RUSTSEC-2026-0002\", reason = \"r; expires 2025-12-31 (M1-T008)\" },\n\
        ]\n";
    assert_eq!(
        found(text),
        pairs(&[
            (
                "deny.toml:3",
                "ignored advisory `RUSTSEC-2026-0001` expired on 2026-11-09 (today is 2026-11-09)"
            ),
            (
                "deny.toml:4",
                "ignored advisory `RUSTSEC-2026-0002` expired on 2025-12-31 (today is 2026-11-09)"
            ),
        ])
    );
}

/// Verifies: XTASK-R085
#[test]
fn entries_without_a_reason_or_expiry_fail() {
    let text = "[advisories]\nignore = [\n\
        \"RUSTSEC-2026-0001\",\n\
        { id = \"RUSTSEC-2026-0002\" },\n\
        { id = \"RUSTSEC-2026-0003\", reason = \"no expiry\" },\n\
        { id = \"RUSTSEC-2026-0004\", reason = \"r; expires 2027-01-01\" },\n\
        { id = \"RUSTSEC-2026-0005\", reason = \"r; expires 2027-01-01 (task)\" },\n\
        { id = \"RUSTSEC-2026-0006\", reason = \"r; expires 2027-01-01 (M1-T008) later\" },\n\
        { id = \"RUSTSEC-2026-0007\", reason = \"r; reexpires 2027-01-01 (M1-T008)\" },\n\
        { reason = 3 },\n\
        ]\n";
    let none = |id: &str| format!("ignored advisory `{id}` has no `reason`");
    let late =
        |id: &str| format!("the reason of ignored advisory `{id}` does not end with an expiry");
    let expected = [
        ("deny.toml:3", none("RUSTSEC-2026-0001")),
        ("deny.toml:4", none("RUSTSEC-2026-0002")),
        ("deny.toml:5", late("RUSTSEC-2026-0003")),
        ("deny.toml:6", late("RUSTSEC-2026-0004")),
        ("deny.toml:7", late("RUSTSEC-2026-0005")),
        ("deny.toml:8", late("RUSTSEC-2026-0006")),
        ("deny.toml:9", late("RUSTSEC-2026-0007")),
        (
            "deny.toml:10",
            "ignored advisory entry has no `reason`".to_string(),
        ),
    ];
    let expected: Vec<(String, String)> = expected
        .into_iter()
        .map(|(location, message)| (location.to_string(), message))
        .collect();
    assert_eq!(found(text), expected);
}

/// Verifies: XTASK-R085
#[test]
fn invalid_dates_fail() {
    for date in [
        "2027-02-29",
        "2027-13-01",
        "2027-04-31",
        "2027-1-01",
        "27-01-01",
        "2027-01-00",
    ] {
        let text = format!(
            "[advisories]\nignore = [{{ id = \"RUSTSEC-2026-0001\", reason = \"r; expires {date} \
             (M1-T008)\" }}]\n"
        );
        assert_eq!(
            found(&text),
            pairs(&[(
                "deny.toml:2",
                &format!(
                    "ignored advisory `RUSTSEC-2026-0001` expires on `{date}`, which is not a date"
                )
            )]),
            "{date}"
        );
    }
}

/// Verifies: XTASK-R085
#[test]
fn the_task_must_exist_and_not_be_done() {
    let text = "[advisories]\nignore = [\n\
        { id = \"RUSTSEC-2026-0001\", reason = \"r; expires 2027-01-01 (M9-T999)\" },\n\
        { id = \"RUSTSEC-2026-0002\", reason = \"r; expires 2027-01-01 (M0-T001)\" },\n\
        { id = \"RUSTSEC-2026-0003\", reason = \"r; expires 2020-01-01 (M0-T001)\" },\n\
        ]\n";
    let findings = found(text);
    let locations: Vec<&str> = findings.iter().map(|(l, _)| l.as_str()).collect();
    assert_eq!(
        locations,
        ["deny.toml:3", "deny.toml:4", "deny.toml:5", "deny.toml:5"]
    );
    assert!(findings[0].1.contains("task M9-T999, which does not exist"));
    assert!(findings[1].1.contains("task M0-T001, which is done"));
    assert!(findings[2].1.contains("expired on 2020-01-01"));
    assert!(findings[3].1.contains("task M0-T001, which is done"));
}

/// Verifies: XTASK-R085
#[test]
fn a_policy_without_ignores_has_no_entries_and_invalid_toml_is_lim_xtask_0004() {
    assert_eq!(parse("").unwrap(), Vec::new());
    assert_eq!(
        parse("[advisories]\nyanked = \"warn\"\n").unwrap(),
        Vec::new()
    );
    assert_eq!(
        parse("[licenses]\nallow = [\"MIT\"]\n").unwrap(),
        Vec::new()
    );
    for text in [
        "[advisories",
        "[advisories]\nignore = \"RUSTSEC-2026-0001\"\n",
    ] {
        let error = parse(text).unwrap_err();
        assert_eq!(error.code, "LIM-XTASK-0004", "{text}");
        assert_eq!(error.path, "deny.toml", "{text}");
    }
    let missing = crate::test_support::TempRoot::new("advisories-no-deny");
    assert_eq!(load(missing.path()).unwrap(), Vec::new());
}

#[test]
fn unix_days_convert_to_calendar_dates() {
    let cases = [
        (0, "1970-01-01"),
        (-1, "1969-12-31"),
        (11_016, "2000-02-29"),
        (20_766, "2026-11-09"),
        (2_932_896, "9999-12-31"),
    ];
    for (days, date) in cases {
        assert_eq!(Date::from_unix_days(days).to_string(), date, "{days}");
        assert_eq!(
            Date::parse(date),
            Some(Date::from_unix_days(days)),
            "{date}"
        );
    }
    // Every day of four centuries round-trips through the text form and increases.
    let mut previous = Date::from_unix_days(-1);
    for days in 0..146_097 {
        let date = Date::from_unix_days(days);
        assert!(date > previous, "{date}");
        assert_eq!(Date::parse(&date.to_string()), Some(date));
        previous = date;
    }
}

#[test]
fn today_is_after_the_date_this_test_was_written() {
    assert!(today() >= TODAY, "{}", today());
}
