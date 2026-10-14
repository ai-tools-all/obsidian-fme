use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

/// Builds the fixture tree once per test run in a private temp dir:
///   schema.toml
///   data/config.toml                  valid
///   experience/ai.toml                valid, with optional fields
///   experience/readme.md              valid markdown frontmatter
///   experience/invalid-missing.toml   missing `role` and `start_date`
///   experience/invalid-enum.toml      `role` not in allowed_values
///   experience/invalid-date.toml      `start_date` not a date
fn fixture_root() -> &'static PathBuf {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("fme-test-toml-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("data")).unwrap();
        fs::create_dir_all(dir.join("experience")).unwrap();
        let files = [
            (
                "schema.toml",
                "[fields.role]\nmandatory = true\nallowed_values = [\"engineer\", \"lead\", \"manager\"]\n\n\
                 [fields.start_date]\nmandatory = true\nformat = \"date\"\n\n\
                 [fields.company]\nmandatory = false\n",
            ),
            ("data/config.toml", "role = \"engineer\"\nstart_date = \"2024-01-15\"\n"),
            (
                "experience/ai.toml",
                "role = \"lead\"\nstart_date = \"2025-06-01\"\ncompany = \"Acme AI\"\n",
            ),
            (
                "experience/readme.md",
                "---\nrole: manager\nstart_date: 2023-03-01\n---\n# Readme\n",
            ),
            ("experience/invalid-missing.toml", "company = \"Nowhere\"\n"),
            (
                "experience/invalid-enum.toml",
                "role = \"invalid_role\"\nstart_date = \"2024-01-15\"\n",
            ),
            (
                "experience/invalid-date.toml",
                "role = \"engineer\"\nstart_date = \"not-a-date\"\n",
            ),
        ];
        for (rel, content) in files {
            fs::write(dir.join(rel), content).unwrap();
        }
        dir
    })
}

fn fixture(rel: &str) -> String {
    fixture_root().join(rel).to_string_lossy().to_string()
}

fn run_enforce(args: &[&str]) -> (bool, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_md-fme"))
        .args(args)
        .output()
        .expect("Failed to execute md-fme");

    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Combine both stdout and stderr for comprehensive output
    let output_text = format!("{}{}", stdout, stderr);

    (success, output_text)
}

#[test]
fn test_valid_toml_document() {
    let (success, output) = run_enforce(&[
        "enforce",
        "--file",
        &fixture("data/config.toml"),
        "--schema",
        &fixture("schema.toml"),
    ]);

    assert!(success, "Valid TOML should pass. Output: {}", output);
    assert!(
        output.contains("PASS config.toml"),
        "Should show PASS status"
    );
    assert!(output.contains("1 passed"), "Should report 1 passed");
}

#[test]
fn test_missing_mandatory_field() {
    let (success, output) = run_enforce(&[
        "enforce",
        "--file",
        &fixture("experience/invalid-missing.toml"),
        "--schema",
        &fixture("schema.toml"),
    ]);

    assert!(!success, "TOML with missing mandatory fields should fail");
    assert!(
        output.contains("missing `role`") && output.contains("missing `start_date`"),
        "Should report both missing fields. Output: {}",
        output
    );
}

#[test]
fn test_invalid_enum_value() {
    let (success, output) = run_enforce(&[
        "enforce",
        "--file",
        &fixture("experience/invalid-enum.toml"),
        "--schema",
        &fixture("schema.toml"),
    ]);

    assert!(!success, "TOML with invalid enum should fail");
    assert!(
        output.contains("invalid_role") && output.contains("not in"),
        "Should report invalid enum value. Output: {}",
        output
    );
}

#[test]
fn test_invalid_date_format() {
    let (success, output) = run_enforce(&[
        "enforce",
        "--file",
        &fixture("experience/invalid-date.toml"),
        "--schema",
        &fixture("schema.toml"),
    ]);

    assert!(!success, "TOML with invalid date should fail");
    assert!(
        output.contains("invalid date") && output.contains("not-a-date"),
        "Should report invalid date format. Output: {}",
        output
    );
}

#[test]
fn test_valid_toml_with_optional_fields() {
    let (success, output) = run_enforce(&[
        "enforce",
        "--file",
        &fixture("experience/ai.toml"),
        "--schema",
        &fixture("schema.toml"),
    ]);

    assert!(
        success,
        "Valid TOML with optional fields should pass. Output: {}",
        output
    );
    assert!(output.contains("PASS ai.toml"), "Should show PASS status");
}

#[test]
fn test_markdown_with_frontmatter() {
    let (success, output) = run_enforce(&[
        "enforce",
        "--file",
        &fixture("experience/readme.md"),
        "--schema",
        &fixture("schema.toml"),
    ]);

    assert!(
        success,
        "Markdown with valid frontmatter should pass. Output: {}",
        output
    );
    assert!(output.contains("PASS readme.md"), "Should show PASS status");
}

#[test]
fn test_mixed_folder_validation() {
    let (success, output) = run_enforce(&[
        "enforce",
        "--folder",
        &fixture("experience"),
        "--schema",
        &fixture("schema.toml"),
    ]);

    assert!(!success, "Folder with invalid files should fail overall");
    assert!(output.contains("PASS readme.md"), "Should pass readme.md");
    assert!(output.contains("PASS ai.toml"), "Should pass ai.toml");
    assert!(
        output.contains("FAIL invalid-date.toml"),
        "Should fail invalid-date.toml"
    );
    assert!(
        output.contains("FAIL invalid-enum.toml"),
        "Should fail invalid-enum.toml"
    );
    assert!(
        output.contains("FAIL invalid-missing.toml"),
        "Should fail invalid-missing.toml"
    );
    assert!(
        output.contains("2 passed, 3 failed"),
        "Should report 2 passed, 3 failed"
    );
}

#[test]
fn test_toml_folder_only() {
    let (success, output) = run_enforce(&[
        "enforce",
        "--folder",
        &fixture("data"),
        "--schema",
        &fixture("schema.toml"),
    ]);

    assert!(success, "Valid TOML folder should pass. Output: {}", output);
    assert!(
        output.contains("PASS config.toml"),
        "Should show PASS status"
    );
    assert!(output.contains("1 passed"), "Should report 1 passed");
}
