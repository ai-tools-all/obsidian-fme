use crate::frontmatter;
use colored::Colorize;
use std::collections::HashMap;
use std::path::Path;

#[derive(serde::Deserialize)]
struct Schema {
    fields: HashMap<String, FieldDef>,
}

#[derive(serde::Deserialize)]
struct FieldDef {
    mandatory: bool,
    #[serde(default)]
    allowed_values: Option<Vec<String>>,
    #[serde(default)]
    format: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    description: Option<String>,
}

pub fn run(schema_path: &Path, folder: &Path) -> Result<(), String> {
    let schema_content = std::fs::read_to_string(schema_path)
        .map_err(|e| format!("Cannot read schema {}: {e}", schema_path.display()))?;
    let schema: Schema = toml::from_str(&schema_content)
        .map_err(|e| format!("Invalid schema TOML: {e}"))?;

    let files = frontmatter::collect_md_files(folder);
    if files.is_empty() {
        return Err("No .md files found".to_string());
    }

    let mut pass = 0u32;
    let mut fail = 0u32;

    for file in &files {
        let fname = file.file_name().unwrap_or_default().to_string_lossy();
        let doc = match frontmatter::read_file(file) {
            Ok(d) => d,
            Err(_) => {
                fail += 1;
                println!("{} {} — no frontmatter", "FAIL".red().bold(), fname);
                continue;
            }
        };

        let fm = &doc.frontmatter;
        let mut errors: Vec<String> = Vec::new();

        for (field_name, field_def) in &schema.fields {
            let value = frontmatter::get_nested(fm, field_name);
            match value {
                None => {
                    if field_def.mandatory {
                        errors.push(format!("missing `{field_name}`"));
                    }
                }
                Some(v) => {
                    if let Some(allowed) = &field_def.allowed_values {
                        let s = frontmatter::value_to_string(v);
                        if !allowed.contains(&s) {
                            errors.push(format!(
                                "`{field_name}` = \"{s}\" not in {:?}",
                                allowed
                            ));
                        }
                    }
                    if let Some(fmt) = &field_def.format {
                        if fmt == "date" {
                            let s = frontmatter::value_to_string(v);
                            if chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").is_err()
                                && chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
                                    .is_err()
                            {
                                errors.push(format!("`{field_name}` invalid date: \"{s}\""));
                            }
                        }
                    }
                }
            }
        }

        if errors.is_empty() {
            pass += 1;
            println!("{} {}", "PASS".green().bold(), fname);
        } else {
            fail += 1;
            println!("{} {} — {}", "FAIL".red().bold(), fname, errors.join("; "));
        }
    }

    println!(
        "\n{} passed, {} failed out of {} files",
        pass.to_string().green(),
        if fail > 0 {
            fail.to_string().red().to_string()
        } else {
            "0".to_string()
        },
        files.len()
    );

    if fail > 0 {
        Err(format!("{fail} file(s) failed validation"))
    } else {
        Ok(())
    }
}
