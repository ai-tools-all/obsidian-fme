/// Integration tests for `format = "datetime"` in enforce and the `now` keyword /
/// date-vs-datetime comparisons in query.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn fme() -> Command {
    Command::new(env!("CARGO_BIN_EXE_md-fme"))
}

const SCHEMA: &str = "[fields.start]\nmandatory = true\nformat = \"datetime\"\n";

/// Writes `files` (name, content) plus a datetime schema.toml into a fresh temp dir.
fn make_dir(test_name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("fme_dt_{test_name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("schema.toml"), SCHEMA).unwrap();
    for (name, content) in files {
        fs::write(dir.join(name), content).unwrap();
    }
    dir
}

fn yaml(start: &str) -> String {
    format!("---\ntitle: t\nstart: {start}\n---\n# Body\n")
}

fn enforce(dir: &Path) -> (bool, String) {
    let out = fme()
        .args(["enforce", "--folder", dir.to_str().unwrap()])
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}

fn query(dir: &Path, expr: &str) -> String {
    let out = fme()
        .args(["query", expr, "--folder", dir.to_str().unwrap()])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).to_string()
}

// ── enforce: format = "datetime" ─────────────────────────────────────────────

#[test]
fn enforce_datetime_accepts_iso8601_variants() {
    let valid = [
        ("minutes.md", yaml("2026-10-14T09:00")),
        ("quoted.md", yaml("\"2026-10-14T09:00\"")),
        ("space.md", yaml("2026-10-14 09:00")),
        ("seconds.md", yaml("2026-10-14T09:00:30")),
        ("fraction.md", yaml("2026-10-14T09:00:30.125")),
        ("zulu.md", yaml("2026-10-14T09:00:30Z")),
        ("offset.md", yaml("2026-10-14T09:00+05:30")),
        ("neg_offset.md", yaml("2026-10-14T09:00:30.5-08:00")),
        // TOML frontmatter with a native TOML datetime value
        (
            "toml_native.md",
            "+++\ntitle = \"t\"\nstart = 2026-10-14T09:00:00\n+++\n".to_string(),
        ),
    ];
    let files: Vec<(&str, &str)> = valid.iter().map(|(n, c)| (*n, c.as_str())).collect();
    let dir = make_dir("valid", &files);
    let (ok, out) = enforce(&dir);
    assert!(ok, "all valid datetimes should pass: {out}");
    assert!(out.contains("9 passed, 0 failed"), "9 passed: {out}");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn enforce_datetime_rejects_invalid_values() {
    let invalid = [
        ("garbage.md", "garbage"),
        ("bare_date.md", "2026-10-14"),
        ("one_digit_hour.md", "2026-10-14T9:00"),
        ("bad_hour.md", "2026-10-14T25:00"),
        ("bad_day.md", "2026-02-30T09:00"),
        ("compact_offset.md", "2026-10-14T09:00+0530"),
        ("slashes.md", "2026/10/14T09:00"),
        ("number.md", "20261014"),
    ];
    let contents: Vec<(&str, String)> = invalid.iter().map(|(n, v)| (*n, yaml(v))).collect();
    let files: Vec<(&str, &str)> = contents.iter().map(|(n, c)| (*n, c.as_str())).collect();
    let dir = make_dir("invalid", &files);
    let (ok, out) = enforce(&dir);
    assert!(!ok, "invalid datetimes should fail: {out}");
    assert!(out.contains("0 passed, 8 failed"), "8 failed: {out}");
    for (_, v) in invalid {
        assert!(
            out.contains(&format!("`start` invalid datetime: \"{v}\"")),
            "should report {v:?}: {out}"
        );
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn enforce_datetime_single_file_fails() {
    let content = yaml("garbage");
    let dir = make_dir("single", &[("task.md", content.as_str())]);
    let out = fme()
        .args(["enforce", "--file", dir.join("task.md").to_str().unwrap()])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!out.status.success(), "single file should fail: {stdout}");
    assert!(
        stdout.contains("invalid datetime"),
        "error message: {stdout}"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn enforce_date_format_still_rejects_datetime_t() {
    // format = "date" is unchanged: an ISO datetime with T is not a date.
    let dir = make_dir(
        "date_unchanged",
        &[("a.md", "---\nd: 2026-10-14T09:00\n---\n")],
    );
    fs::write(
        dir.join("schema.toml"),
        "[fields.d]\nmandatory = true\nformat = \"date\"\n",
    )
    .unwrap();
    let (ok, out) = enforce(&dir);
    assert!(!ok, "date format should reject datetime: {out}");
    assert!(out.contains("`d` invalid date"), "date error: {out}");
    let _ = fs::remove_dir_all(&dir);
}

// ── query: now and date/datetime comparisons ─────────────────────────────────

fn query_dir(test_name: &str) -> PathBuf {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let later_today = format!("{today}T23:59");
    let earlier_today = format!("{today} 00:00:00");
    make_dir(
        test_name,
        &[
            ("past.md", yaml("2000-01-01T00:00").as_str()),
            ("future.md", yaml("2999-01-01T00:00").as_str()),
            ("later_today.md", yaml(&later_today).as_str()),
            ("earlier_today.md", yaml(&earlier_today).as_str()),
        ],
    )
}

#[test]
fn query_now_past_and_future() {
    let dir = query_dir("now");
    let le = query(&dir, "start <= now");
    assert!(le.contains("past.md"), "past <= now: {le}");
    assert!(!le.contains("future.md"), "future not <= now: {le}");
    assert!(
        le.contains("earlier_today.md"),
        "today 00:00:00 <= now: {le}"
    );
    let gt = query(&dir, "start > now");
    assert!(gt.contains("future.md"), "future > now: {gt}");
    assert!(!gt.contains("past.md"), "past not > now: {gt}");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn query_today_compares_datetimes_by_day() {
    let dir = query_dir("today");
    let le = query(&dir, "start <= today");
    assert!(
        le.contains("later_today.md"),
        "later today <= today (by day): {le}"
    );
    assert!(
        le.contains("earlier_today.md"),
        "earlier today <= today: {le}"
    );
    assert!(le.contains("past.md"), "past <= today: {le}");
    assert!(!le.contains("future.md"), "future not <= today: {le}");
    let gt = query(&dir, "start > today");
    assert!(
        !gt.contains("later_today.md"),
        "same day is not > today: {gt}"
    );
    assert!(gt.contains("future.md"), "future > today: {gt}");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn query_datetime_literal_mixed_formats() {
    let dir = make_dir(
        "mixed",
        &[
            ("nine.md", yaml("2026-10-14 09:00:00").as_str()),
            ("ten.md", yaml("2026-10-14T10:00").as_str()),
        ],
    );
    // Space vs T and with/without seconds compare chronologically, not as text.
    let lt = query(&dir, "start < 2026-10-14T10:00");
    assert!(lt.contains("nine.md"), "09:00:00 < 10:00: {lt}");
    assert!(!lt.contains("ten.md"), "10:00 not < 10:00: {lt}");
    let eq = query(&dir, "start >= \"2026-10-14 10:00:00\"");
    assert!(eq.contains("ten.md"), "10:00 >= 10:00:00: {eq}");
    let _ = fs::remove_dir_all(&dir);
}
