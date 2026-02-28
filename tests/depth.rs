use std::fs;
use std::process::Command;

fn fme() -> Command {
    Command::new(env!("CARGO_BIN_EXE_md-fme"))
}

fn make_nested_tree(base: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("fme_depth_{}_{}", std::process::id(), base));
    let _ = fs::remove_dir_all(&dir);

    // level 1: root
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("root.md"),
        "---\ntitle: root\ntype: sr-test\nreview_type: recall\nsr:\n  next_review: 2026-01-01\n  interval: 1\n  ease: 2.5\n  reps: 0\n  last_reviewed: \"~\"\n---\n# Root\n",
    ).unwrap();

    // level 2: sub/
    let sub = dir.join("sub");
    fs::create_dir_all(&sub).unwrap();
    fs::write(
        sub.join("level2.md"),
        "---\ntitle: level2\ntype: sr-test\nreview_type: recall\nsr:\n  next_review: 2026-01-01\n  interval: 1\n  ease: 2.5\n  reps: 0\n  last_reviewed: \"~\"\n---\n# Level 2\n",
    ).unwrap();

    // level 3: sub/deep/
    let deep = sub.join("deep");
    fs::create_dir_all(&deep).unwrap();
    fs::write(
        deep.join("level3.md"),
        "---\ntitle: level3\ntype: sr-test\nreview_type: recall\nsr:\n  next_review: 2026-01-01\n  interval: 1\n  ease: 2.5\n  reps: 0\n  last_reviewed: \"~\"\n---\n# Level 3\n",
    ).unwrap();

    // level 4: sub/deep/deeper/
    let deeper = deep.join("deeper");
    fs::create_dir_all(&deeper).unwrap();
    fs::write(
        deeper.join("level4.md"),
        "---\ntitle: level4\ntype: sr-test\nreview_type: recall\nsr:\n  next_review: 2026-01-01\n  interval: 1\n  ease: 2.5\n  reps: 0\n  last_reviewed: \"~\"\n---\n# Level 4\n",
    ).unwrap();

    dir
}

fn cleanup(dir: &std::path::Path) {
    let _ = fs::remove_dir_all(dir);
}

// ── query --depth ────────────────────────────────────────────────────────────

#[test]
fn query_depth1_finds_only_root() {
    let dir = make_nested_tree("q_d1");
    let out = fme()
        .args(["query", "title exists", "--folder", dir.to_str().unwrap(), "--depth", "1"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("root.md"), "depth=1 finds root: {stdout}");
    assert!(!stdout.contains("level2.md"), "depth=1 skips level2: {stdout}");
    assert!(!stdout.contains("level3.md"), "depth=1 skips level3: {stdout}");
    cleanup(&dir);
}

#[test]
fn query_depth2_finds_root_and_level2() {
    let dir = make_nested_tree("q_d2");
    let out = fme()
        .args(["query", "title exists", "--folder", dir.to_str().unwrap(), "--depth", "2"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("root.md"), "depth=2 finds root: {stdout}");
    assert!(stdout.contains("level2.md"), "depth=2 finds level2: {stdout}");
    assert!(!stdout.contains("level3.md"), "depth=2 skips level3: {stdout}");
    cleanup(&dir);
}

#[test]
fn query_depth3_finds_three_levels() {
    let dir = make_nested_tree("q_d3");
    let out = fme()
        .args(["query", "title exists", "--folder", dir.to_str().unwrap(), "--depth", "3"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("root.md"), "depth=3 finds root: {stdout}");
    assert!(stdout.contains("level2.md"), "depth=3 finds level2: {stdout}");
    assert!(stdout.contains("level3.md"), "depth=3 finds level3: {stdout}");
    assert!(!stdout.contains("level4.md"), "depth=3 skips level4: {stdout}");
    cleanup(&dir);
}

#[test]
fn query_depth0_unlimited_finds_all() {
    let dir = make_nested_tree("q_d0");
    let out = fme()
        .args(["query", "title exists", "--folder", dir.to_str().unwrap(), "--depth", "0"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("root.md"), "depth=0 finds root: {stdout}");
    assert!(stdout.contains("level2.md"), "depth=0 finds level2: {stdout}");
    assert!(stdout.contains("level3.md"), "depth=0 finds level3: {stdout}");
    assert!(stdout.contains("level4.md"), "depth=0 finds level4: {stdout}");
    cleanup(&dir);
}

#[test]
fn query_default_depth_is_3() {
    let dir = make_nested_tree("q_default");
    let out = fme()
        .args(["query", "title exists", "--folder", dir.to_str().unwrap()])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("root.md"), "default finds root: {stdout}");
    assert!(stdout.contains("level2.md"), "default finds level2: {stdout}");
    assert!(stdout.contains("level3.md"), "default finds level3: {stdout}");
    assert!(!stdout.contains("level4.md"), "default skips level4: {stdout}");
    cleanup(&dir);
}

// ── today --depth ────────────────────────────────────────────────────────────

#[test]
fn today_depth1_finds_only_root() {
    let dir = make_nested_tree("t_d1");
    let out = fme()
        .args(["today", "--folder", dir.to_str().unwrap(), "--depth", "1"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("root.md"), "today depth=1 finds root: {stdout}");
    assert!(!stdout.contains("level2.md"), "today depth=1 skips level2: {stdout}");
    assert!(stdout.contains("1 item(s) due"), "today depth=1 count: {stdout}");
    cleanup(&dir);
}

#[test]
fn today_depth0_finds_all() {
    let dir = make_nested_tree("t_d0");
    let out = fme()
        .args(["today", "--folder", dir.to_str().unwrap(), "--depth", "0"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("root.md"), "today depth=0 finds root: {stdout}");
    assert!(stdout.contains("level2.md"), "today depth=0 finds level2: {stdout}");
    assert!(stdout.contains("level3.md"), "today depth=0 finds level3: {stdout}");
    assert!(stdout.contains("level4.md"), "today depth=0 finds level4: {stdout}");
    assert!(stdout.contains("4 item(s) due"), "today depth=0 count: {stdout}");
    cleanup(&dir);
}

// ── stats --depth ────────────────────────────────────────────────────────────

#[test]
fn stats_depth1_counts_only_root() {
    let dir = make_nested_tree("s_d1");
    let out = fme()
        .args(["stats", "--folder", dir.to_str().unwrap(), "--depth", "1"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let total_line = stdout.lines().find(|l| l.contains("Total SR items:")).unwrap_or("");
    assert!(total_line.contains("1"), "stats depth=1 total=1: {stdout}");
    cleanup(&dir);
}

#[test]
fn stats_depth0_counts_all() {
    let dir = make_nested_tree("s_d0");
    let out = fme()
        .args(["stats", "--folder", dir.to_str().unwrap(), "--depth", "0"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let total_line = stdout.lines().find(|l| l.contains("Total SR items:")).unwrap_or("");
    assert!(total_line.contains("4"), "stats depth=0 total=4: {stdout}");
    cleanup(&dir);
}
