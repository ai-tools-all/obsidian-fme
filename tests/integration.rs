use std::fs;
use std::path::Path;
use std::process::Command;

fn fme_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_md-fme"))
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

fn with_fix_temp_dir(test_name: &str, md_content: &str, f: impl FnOnce(&Path, &Path)) {
    let dir = std::env::temp_dir().join(format!("fme_fix_{test_name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let schema_src = Path::new("tests/fixtures/fix_schema.toml");
    let schema_dst = dir.join("schema.toml");
    fs::copy(schema_src, &schema_dst).unwrap();
    let md_path = dir.join("test_file.md");
    fs::write(&md_path, md_content).unwrap();
    f(&dir, &md_path);
    let _ = fs::remove_dir_all(&dir);
}

fn with_exclude_temp_dir(
    test_name: &str,
    files: &[(&str, &str)],
    f: impl FnOnce(&Path),
) {
    let dir = std::env::temp_dir().join(format!("fme_excl_{test_name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let schema_src = Path::new("tests/fixtures/fix_schema.toml");
    let schema_dst = dir.join("schema.toml");
    fs::copy(schema_src, &schema_dst).unwrap();
    for (name, content) in files {
        fs::write(dir.join(name), content).unwrap();
    }
    f(&dir);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn fix_adds_missing_fields_and_converts_yaml_to_toml() {
    let yaml_content = "---\ntags:\n  - cybersecurity\n  - blogs\n---\n\n## Body content\n";
    with_fix_temp_dir("converts", yaml_content, |dir, md_path| {
        let output = fme_bin()
            .args([
                "enforce",
                "--folder", dir.to_str().unwrap(),
                "--fix",
            ])
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("FIXED"), "Should print FIXED: {stdout}");
        assert!(stdout.contains("added:"), "Should list added fields: {stdout}");

        let after = fs::read_to_string(md_path).unwrap();
        assert!(after.starts_with("+++"), "Should convert to TOML: {after}");
        assert!(after.contains("status = \"attempted\""), "Should have status with default: {after}");
        assert!(after.contains("date = \"2026-01-01\""), "Should have date with default: {after}");
        assert!(after.contains("type = \"\""), "Should have type with empty default: {after}");
        assert!(after.contains("Body content"), "Body preserved: {after}");
    });
}

#[test]
fn fix_uses_schema_defaults() {
    let yaml_content = "---\ntags:\n  - test\n---\n\n## Body\n";
    with_fix_temp_dir("defaults", yaml_content, |dir, md_path| {
        fme_bin()
            .args([
                "enforce",
                "--folder", dir.to_str().unwrap(),
                "--fix",
            ])
            .output()
            .unwrap();

        let after = fs::read_to_string(md_path).unwrap();
        assert!(after.contains("status = \"attempted\""), "status should use schema default 'attempted': {after}");
        assert!(after.contains("date = \"2026-01-01\""), "date should use schema default '2026-01-01': {after}");
    });
}

#[test]
fn fix_creates_frontmatter_when_missing() {
    let no_fm = "# Just a heading\n\nSome body text.\n";
    with_fix_temp_dir("nofm", no_fm, |dir, md_path| {
        let output = fme_bin()
            .args([
                "enforce",
                "--folder", dir.to_str().unwrap(),
                "--fix",
            ])
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("FIXED"), "Should fix no-frontmatter files: {stdout}");
        assert!(stdout.contains("created frontmatter"), "Should mention created: {stdout}");

        let after = fs::read_to_string(md_path).unwrap();
        assert!(after.starts_with("+++"), "Should have TOML frontmatter: {after}");
        assert!(after.contains("# Just a heading"), "Body preserved: {after}");
        assert!(after.contains("Some body text"), "Body preserved: {after}");
    });
}

#[test]
fn no_fix_fails_on_missing_frontmatter() {
    let no_fm = "# Just a heading\n\nSome body text.\n";
    with_fix_temp_dir("nofm_noflag", no_fm, |dir, md_path| {
        let output = fme_bin()
            .args([
                "enforce",
                "--folder", dir.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("FAIL"), "Should FAIL without --fix: {stdout}");
        assert!(stdout.contains("no frontmatter"), "Should mention no frontmatter: {stdout}");

        let after = fs::read_to_string(md_path).unwrap();
        assert_eq!(after, no_fm, "File unchanged without --fix");
    });
}

#[test]
fn fix_inserts_empty_string_when_no_default() {
    let yaml_content = "---\nstatus: attempted\ndate: 2026-01-01\n---\n\n## Body\n";
    with_fix_temp_dir("nodefault", yaml_content, |dir, md_path| {
        let output = fme_bin()
            .args([
                "enforce",
                "--folder", dir.to_str().unwrap(),
                "--fix",
            ])
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("FIXED"), "Should fix the file: {stdout}");
        assert!(stdout.contains("type"), "Should list 'type' as added: {stdout}");

        let after = fs::read_to_string(md_path).unwrap();
        assert!(after.contains("type = \"\""), "type field should have empty string default: {after}");
    });
}

#[test]
fn fix_replaces_invalid_allowed_value_with_default() {
    let yaml_content = "---\nstatus: bogus\ndate: 2026-01-01\ntype: leetcode\n---\n\n## Body\n";
    with_fix_temp_dir("invalid_val", yaml_content, |dir, md_path| {
        let output = fme_bin()
            .args([
                "enforce",
                "--folder", dir.to_str().unwrap(),
                "--fix",
            ])
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("FIXED"), "Should fix invalid value: {stdout}");
        assert!(stdout.contains("status"), "Should list 'status' as fixed: {stdout}");

        let after = fs::read_to_string(md_path).unwrap();
        assert!(after.contains("status = \"attempted\""), "status should be replaced with default: {after}");
        assert!(after.contains("type = \"leetcode\""), "Existing valid fields preserved: {after}");
    });
}

#[test]
fn exclude_skips_matching_files() {
    let good = "---\nstatus: attempted\ndate: 2026-01-01\ntype: leetcode\n---\n\n## Good\n";
    let bad = "---\ntags:\n  - test\n---\n\n## Bad\n";
    with_exclude_temp_dir("skip", &[("good.md", good), ("bad.md", bad)], |dir| {
        let output = fme_bin()
            .args([
                "enforce",
                "--folder", dir.to_str().unwrap(),
                "--exclude", "bad.md",
            ])
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("SKIP"), "Should print SKIP for bad.md: {stdout}");
        assert!(stdout.contains("bad.md"), "Should mention bad.md: {stdout}");
        assert!(stdout.contains("excluded"), "Should say excluded: {stdout}");
        assert!(stdout.contains("PASS"), "good.md should pass: {stdout}");
        assert!(!stdout.contains("FAIL"), "No files should fail: {stdout}");
        assert!(output.status.success(), "Command should succeed: {}", String::from_utf8_lossy(&output.stderr));
    });
}

#[test]
fn exclude_with_wildcard() {
    let valid = "---\nstatus: attempted\ndate: 2026-01-01\ntype: leetcode\n---\n\n## Valid\n";
    let test1 = "---\ntags:\n  - a\n---\n\n## Test1\n";
    let test2 = "---\ntags:\n  - b\n---\n\n## Test2\n";
    with_exclude_temp_dir(
        "wildcard",
        &[("valid.md", valid), ("test_one.md", test1), ("test_two.md", test2)],
        |dir| {
            let output = fme_bin()
                .args([
                    "enforce",
                    "--folder", dir.to_str().unwrap(),
                    "--exclude", "test*",
                ])
                .output()
                .unwrap();
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("SKIP"), "Should skip test files: {stdout}");
            assert!(stdout.contains("test_one.md") || stdout.contains("test_two.md"),
                "Should mention skipped test files: {stdout}");
            assert!(stdout.contains("PASS"), "valid.md should pass: {stdout}");
            assert!(!stdout.contains("FAIL"), "No files should fail: {stdout}");
            assert!(output.status.success(), "Command should succeed");
        },
    );
}

#[test]
fn exclude_multiple_patterns() {
    let valid = "---\nstatus: attempted\ndate: 2026-01-01\ntype: leetcode\n---\n\n## Valid\n";
    let readme = "# README\nNo frontmatter here.\n";
    let index = "---\ntags:\n  - nav\n---\n\n## Index\n";
    with_exclude_temp_dir(
        "multi",
        &[("valid.md", valid), ("README.md", readme), ("index.md", index)],
        |dir| {
            let output = fme_bin()
                .args([
                    "enforce",
                    "--folder", dir.to_str().unwrap(),
                    "--exclude", "README.md,index.md",
                ])
                .output()
                .unwrap();
            let stdout = String::from_utf8_lossy(&output.stdout);
            let skip_count = stdout.matches("SKIP").count();
            assert_eq!(skip_count, 2, "Should skip exactly 2 files: {stdout}");
            assert!(stdout.contains("PASS"), "valid.md should pass: {stdout}");
            assert!(!stdout.contains("FAIL"), "No files should fail: {stdout}");
            assert!(output.status.success(), "Command should succeed");
        },
    );
}
