use std::process::Command;

fn run_enforce(args: &[&str]) -> (bool, String) {
    let cargo_manifest = env!("CARGO_MANIFEST_DIR");
    let binary_path = format!("{}/../../target/debug/md-fme", cargo_manifest);
    
    let output = Command::new(&binary_path)
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
        "/tmp/fme-test-toml/data/config.toml",
        "--schema",
        "/tmp/fme-test-toml/schema.toml",
    ]);

    assert!(success, "Valid TOML should pass. Output: {}", output);
    assert!(output.contains("PASS config.toml"), "Should show PASS status");
    assert!(output.contains("1 passed"), "Should report 1 passed");
}

#[test]
fn test_missing_mandatory_field() {
    let (success, output) = run_enforce(&[
        "enforce",
        "--file",
        "/tmp/fme-test-toml/experience/invalid-missing.toml",
        "--schema",
        "/tmp/fme-test-toml/schema.toml",
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
        "/tmp/fme-test-toml/experience/invalid-enum.toml",
        "--schema",
        "/tmp/fme-test-toml/schema.toml",
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
        "/tmp/fme-test-toml/experience/invalid-date.toml",
        "--schema",
        "/tmp/fme-test-toml/schema.toml",
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
        "/tmp/fme-test-toml/experience/ai.toml",
        "--schema",
        "/tmp/fme-test-toml/schema.toml",
    ]);

    assert!(success, "Valid TOML with optional fields should pass. Output: {}", output);
    assert!(output.contains("PASS ai.toml"), "Should show PASS status");
}

#[test]
fn test_markdown_with_frontmatter() {
    let (success, output) = run_enforce(&[
        "enforce",
        "--file",
        "/tmp/fme-test-toml/experience/readme.md",
        "--schema",
        "/tmp/fme-test-toml/schema.toml",
    ]);

    assert!(success, "Markdown with valid frontmatter should pass. Output: {}", output);
    assert!(output.contains("PASS readme.md"), "Should show PASS status");
}

#[test]
fn test_mixed_folder_validation() {
    let (success, output) = run_enforce(&[
        "enforce",
        "--folder",
        "/tmp/fme-test-toml/experience",
        "--schema",
        "/tmp/fme-test-toml/schema.toml",
    ]);

    assert!(!success, "Folder with invalid files should fail overall");
    assert!(output.contains("PASS readme.md"), "Should pass readme.md");
    assert!(output.contains("PASS ai.toml"), "Should pass ai.toml");
    assert!(output.contains("FAIL invalid-date.toml"), "Should fail invalid-date.toml");
    assert!(
        output.contains("FAIL invalid-enum.toml"),
        "Should fail invalid-enum.toml"
    );
    assert!(
        output.contains("FAIL invalid-missing.toml"),
        "Should fail invalid-missing.toml"
    );
    assert!(output.contains("2 passed, 3 failed"), "Should report 2 passed, 3 failed");
}

#[test]
fn test_toml_folder_only() {
    let (success, output) = run_enforce(&[
        "enforce",
        "--folder",
        "/tmp/fme-test-toml/data",
        "--schema",
        "/tmp/fme-test-toml/schema.toml",
    ]);

    assert!(success, "Valid TOML folder should pass. Output: {}", output);
    assert!(output.contains("PASS config.toml"), "Should show PASS status");
    assert!(output.contains("1 passed"), "Should report 1 passed");
}
