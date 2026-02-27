use std::fs;
use std::path::Path;
use std::process::Command;

fn fme_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_fme"))
}

fn fixture(name: &str) -> String {
    let path = Path::new("tests/fixtures").join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Cannot read {}: {e}", path.display()))
}

fn with_temp_fixture(name: &str, f: impl FnOnce(&Path)) {
    let src = Path::new("tests/fixtures").join(name);
    let tmp = std::env::temp_dir().join(format!("fme_test_{name}"));
    fs::copy(&src, &tmp).unwrap();
    f(&tmp);
    let _ = fs::remove_file(&tmp);
}

#[test]
fn parse_yaml_fixture_leetcode_rich() {
    let content = fixture("leetcode_rich.md");
    assert!(content.starts_with("---"));
    assert!(content.contains("type: leetcode"));

    let output = fme_bin()
        .args(["query", "difficulty = medium AND status = completed", "--folder", "tests/fixtures"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("leetcode_rich.md"));
}

#[test]
fn parse_toml_fixture() {
    let output = fme_bin()
        .args(["query", "difficulty = hard", "--folder", "tests/fixtures"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("toml_frontmatter.md"));
}

#[test]
fn parse_yaml_multiline_tags() {
    let output = fme_bin()
        .args(["query", "tags contains cybersecurity", "--folder", "tests/fixtures"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("simple_tags.md"));
}

#[test]
fn parse_yaml_inline_array() {
    let output = fme_bin()
        .args(["query", "tags contains workflow", "--folder", "tests/fixtures"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("workflow_inline_tags.md"));
}

#[test]
fn parse_yaml_mixed_types() {
    let output = fme_bin()
        .args(["query", "type = daily-log", "--folder", "tests/fixtures"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("daily_log.md"));
}

#[test]
fn parse_yaml_boolean_field() {
    let output = fme_bin()
        .args(["query", "study_block = true", "--folder", "tests/fixtures"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("daily_log.md"));
}

#[test]
fn parse_yaml_datetime_field() {
    let output = fme_bin()
        .args(["query", "topics contains bfs", "--folder", "tests/fixtures"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("leetcode_datetime.md"));
}

#[test]
fn init_sr_converts_yaml_to_toml() {
    with_temp_fixture("simple_tags.md", |tmp| {
        let before = fs::read_to_string(tmp).unwrap();
        assert!(before.starts_with("---"));

        let output = fme_bin()
            .args(["init-sr", "--file", tmp.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(output.status.success(), "init-sr failed: {}", String::from_utf8_lossy(&output.stderr));

        let after = fs::read_to_string(tmp).unwrap();
        assert!(after.starts_with("+++"), "Should be converted to TOML: {after}");
        assert!(after.contains("[sr]"), "Should have [sr] table");
        assert!(after.contains("next_review"), "Should have next_review");
        assert!(after.contains("review_type"), "Should have review_type");
        assert!(after.contains("Pegasus"), "Body content preserved");
    });
}

#[test]
fn init_sr_on_toml_stays_toml() {
    with_temp_fixture("toml_frontmatter.md", |tmp| {
        let output = fme_bin()
            .args(["init-sr", "--file", tmp.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(output.status.success());

        let after = fs::read_to_string(tmp).unwrap();
        assert!(after.starts_with("+++"));
        assert!(after.contains("[sr]"));
        assert!(after.contains("Test TOML Frontmatter"), "Body preserved");
    });
}

#[test]
fn verbose_query_shows_toml_fields() {
    let output = fme_bin()
        .args(["query", "type = leetcode", "--folder", "tests/fixtures", "--verbose"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("toml_frontmatter.md"));
    assert!(stdout.contains("difficulty: hard"));
}
