/// SR integration tests — today, review, stats, and SR query patterns.
///
/// Fixtures (tests/fixtures/sr/):
///   sr_overdue.md  — next_review=2026-01-01, reps=2, interval=6, ease=2.50  (always past)
///   sr_future.md   — next_review=2099-12-31, reps=10, interval=30, ease=2.80 (always future)
///   sr_no_sr.md    — no [sr] block, should be ignored by all SR commands
use std::fs;
use std::path::Path;
use std::process::Command;

fn fme() -> Command {
    Command::new(env!("CARGO_BIN_EXE_md-fme"))
}

const SR_DIR: &str = "tests/fixtures/sr";

fn with_overdue_temp(test_name: &str, f: impl FnOnce(&Path)) {
    let dir = std::env::temp_dir().join(format!("fme_sr_{}_{}", std::process::id(), test_name));
    fs::create_dir_all(&dir).unwrap();
    let src = Path::new(SR_DIR).join("sr_overdue.md");
    let tmp = dir.join("sr_overdue.md");
    fs::copy(&src, &tmp).unwrap();
    f(&tmp);
    let _ = fs::remove_dir_all(&dir);
}

// ── today ─────────────────────────────────────────────────────────────────────

#[test]
fn today_shows_overdue_file() {
    let out = fme().args(["today", "--folder", SR_DIR]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("sr_overdue.md"), "today: {stdout}");
}

#[test]
fn today_hides_future_file() {
    let out = fme().args(["today", "--folder", SR_DIR]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.contains("sr_future.md"), "today should hide future: {stdout}");
}

#[test]
fn today_skips_file_without_sr() {
    let out = fme().args(["today", "--folder", SR_DIR]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.contains("sr_no_sr.md"), "today should skip no-sr: {stdout}");
}

#[test]
fn today_shows_item_count() {
    let out = fme().args(["today", "--folder", SR_DIR]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("1 item(s) due"), "today count: {stdout}");
}

#[test]
fn today_labels_item_as_overdue() {
    let out = fme().args(["today", "--folder", SR_DIR]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("overdue"), "today overdue label: {stdout}");
}

// ── stats ─────────────────────────────────────────────────────────────────────

#[test]
fn stats_counts_two_sr_items() {
    // sr_overdue + sr_future have [sr] blocks; sr_no_sr is ignored
    let out = fme().args(["stats", "--folder", SR_DIR]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let total_line = stdout.lines().find(|l| l.contains("Total SR items:")).unwrap_or("");
    assert!(total_line.contains("2"), "stats total: {stdout}");
}

#[test]
fn stats_shows_one_overdue() {
    let out = fme().args(["stats", "--folder", SR_DIR]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let overdue_line = stdout.lines().find(|l| l.contains("Overdue:")).unwrap_or("");
    assert!(overdue_line.contains("1"), "stats overdue: {stdout}");
}

#[test]
fn stats_shows_seven_day_forecast() {
    let out = fme().args(["stats", "--folder", SR_DIR]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("7-day load:"), "stats forecast: {stdout}");
}

#[test]
fn stats_shows_weakest_section() {
    let out = fme().args(["stats", "--folder", SR_DIR]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Weakest"), "stats weakest: {stdout}");
}

// ── review ────────────────────────────────────────────────────────────────────
//
// SM-2 math for sr_overdue (reps=2, interval=6, ease=2.50):
//   quality=4 → new_ease=2.50, new_reps=3, new_interval=15  (6 * 2.50)
//   quality=1 → new_ease=1.96, new_reps=0, new_interval=1   (failure reset)

#[test]
fn review_quality4_increases_interval_and_reps() {
    with_overdue_temp("q4", |tmp| {
        let out = fme()
            .args(["review", "--file", tmp.to_str().unwrap(), "--quality", "4"])
            .output()
            .unwrap();
        assert!(out.status.success(), "review q=4 failed: {}", String::from_utf8_lossy(&out.stderr));
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains("q=4"), "stdout: {stdout}");
        assert!(stdout.contains("interval=15"), "interval: {stdout}");
        assert!(stdout.contains("reps=3"), "reps: {stdout}");

        let after = fs::read_to_string(tmp).unwrap();
        assert!(after.contains("reps = 3"), "file reps: {after}");
        assert!(after.contains("interval = 15"), "file interval: {after}");
        assert!(!after.contains("next_review = \"2026-01-01\""), "next_review updated: {after}");
    });
}

#[test]
fn review_quality1_resets_reps_and_interval() {
    with_overdue_temp("q1", |tmp| {
        let out = fme()
            .args(["review", "--file", tmp.to_str().unwrap(), "--quality", "1"])
            .output()
            .unwrap();
        assert!(out.status.success(), "review q=1 failed: {}", String::from_utf8_lossy(&out.stderr));
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains("q=1"), "stdout: {stdout}");
        assert!(stdout.contains("interval=1"), "interval reset: {stdout}");
        assert!(stdout.contains("reps=0"), "reps reset: {stdout}");

        let after = fs::read_to_string(tmp).unwrap();
        assert!(after.contains("reps = 0"), "file reps reset: {after}");
        assert!(after.contains("interval = 1"), "file interval reset: {after}");
    });
}

#[test]
fn review_updates_last_reviewed() {
    with_overdue_temp("lr", |tmp| {
        fme()
            .args(["review", "--file", tmp.to_str().unwrap(), "--quality", "4"])
            .output()
            .unwrap();
        let after = fs::read_to_string(tmp).unwrap();
        // was "2025-12-26", should now be today
        assert!(!after.contains("last_reviewed = \"2025-12-26\""), "last_reviewed updated: {after}");
        assert!(after.contains("last_reviewed"), "last_reviewed present: {after}");
    });
}

#[test]
fn review_rejects_quality_above_5() {
    with_overdue_temp("q6err", |tmp| {
        let out = fme()
            .args(["review", "--file", tmp.to_str().unwrap(), "--quality", "6"])
            .output()
            .unwrap();
        assert!(!out.status.success(), "quality=6 should fail");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(stderr.contains("Quality must be 0-5"), "error: {stderr}");
    });
}

// ── SR query patterns ─────────────────────────────────────────────────────────

#[test]
fn query_sr_due_matches_overdue_only() {
    let out = fme()
        .args(["query", "sr.next_review <= today", "--folder", SR_DIR])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("sr_overdue.md"), "due query: {stdout}");
    assert!(!stdout.contains("sr_future.md"), "due query excludes future: {stdout}");
}

#[test]
fn query_sr_future_excludes_overdue() {
    let out = fme()
        .args(["query", "sr.next_review > today", "--folder", SR_DIR])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("sr_future.md"), "future query: {stdout}");
    assert!(!stdout.contains("sr_overdue.md"), "future query excludes overdue: {stdout}");
}

#[test]
fn query_sr_ease_filter() {
    // sr_future ease=2.80, sr_overdue ease=2.50
    let out = fme()
        .args(["query", "sr.ease > 2.7", "--folder", SR_DIR])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("sr_future.md"), "ease > 2.7: {stdout}");
    assert!(!stdout.contains("sr_overdue.md"), "ease > 2.7 excludes overdue: {stdout}");
}

#[test]
fn query_sr_reps_gt_filter() {
    // sr_future reps=10, sr_overdue reps=2
    let out = fme()
        .args(["query", "sr.reps > 5", "--folder", SR_DIR])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("sr_future.md"), "reps > 5: {stdout}");
    assert!(!stdout.contains("sr_overdue.md"), "reps > 5 excludes overdue: {stdout}");
}

#[test]
fn query_sr_exists_excludes_no_sr_file() {
    let out = fme()
        .args(["query", "sr.next_review exists", "--folder", SR_DIR])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.contains("sr_no_sr.md"), "exists excludes no-sr: {stdout}");
    assert!(stdout.contains("sr_overdue.md"), "exists matches sr_overdue: {stdout}");
    assert!(stdout.contains("sr_future.md"), "exists matches sr_future: {stdout}");
}

#[test]
fn query_sr_missing_matches_no_sr_file() {
    let out = fme()
        .args(["query", "sr.next_review missing", "--folder", SR_DIR])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("sr_no_sr.md"), "missing matches no-sr: {stdout}");
    assert!(!stdout.contains("sr_overdue.md"), "missing excludes sr_overdue: {stdout}");
    assert!(!stdout.contains("sr_future.md"), "missing excludes sr_future: {stdout}");
}

#[test]
fn query_combined_type_and_due() {
    let out = fme()
        .args(["query", "type = sr-test AND sr.next_review <= today", "--folder", SR_DIR])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("sr_overdue.md"), "combined: {stdout}");
    assert!(!stdout.contains("sr_future.md"), "combined excludes future: {stdout}");
}

#[test]
fn query_review_type_filter() {
    // sr_overdue review_type=recall, sr_future review_type=solve
    let out = fme()
        .args(["query", "review_type = solve", "--folder", SR_DIR])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("sr_future.md"), "review_type=solve: {stdout}");
    assert!(!stdout.contains("sr_overdue.md"), "review_type=solve excludes recall: {stdout}");
}
