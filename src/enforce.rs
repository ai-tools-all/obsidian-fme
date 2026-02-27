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
    #[serde(default)]
    default: Option<String>,
}

fn validate(fm: &toml::Value, schema: &Schema) -> Vec<String> {
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
    errors
}

fn find_fixable_fields(fm: &toml::Value, schema: &Schema) -> Vec<String> {
    let mut fixable = Vec::new();
    for (name, def) in &schema.fields {
        if !def.mandatory {
            continue;
        }
        match frontmatter::get_nested(fm, name) {
            None => fixable.push(name.clone()),
            Some(v) => {
                if let Some(allowed) = &def.allowed_values {
                    let s = frontmatter::value_to_string(v);
                    if !allowed.contains(&s) && def.default.is_some() {
                        fixable.push(name.clone());
                    }
                }
            }
        }
    }
    fixable
}

fn apply_fix(
    file: &Path,
    doc: &frontmatter::Document,
    schema: &Schema,
    fixable: &[String],
) -> Result<(), String> {
    let mut fm = doc.frontmatter.clone();
    let table = fm
        .as_table_mut()
        .ok_or_else(|| "Frontmatter is not a table".to_string())?;
    for field_name in fixable {
        let default_val = schema.fields.get(field_name).and_then(|d| d.default.clone());
        let val = default_val.unwrap_or_default();
        table.insert(field_name.clone(), toml::Value::String(val));
    }
    let new_raw = frontmatter::replace_frontmatter(&doc.raw, &fm);
    frontmatter::write_raw(file, &new_raw)
}

fn create_frontmatter(file: &Path, schema: &Schema) -> Result<frontmatter::Document, String> {
    let raw = frontmatter::read_raw(file)?;
    let mut table = toml::map::Map::new();
    for (name, def) in &schema.fields {
        if def.mandatory {
            let val = def.default.clone().unwrap_or_default();
            table.insert(name.clone(), toml::Value::String(val));
        }
    }
    let fm = toml::Value::Table(table);
    let toml_block = frontmatter::serialize_toml(&fm);
    let new_raw = format!("{toml_block}\n{raw}");
    frontmatter::write_raw(file, &new_raw)?;
    frontmatter::read_file(file)
}

const DEFAULT_EXCLUDES: &[&str] = &["README.md", "AGENTS.md", "index.md"];

fn parse_exclude_patterns(exclude: Option<&str>) -> Vec<String> {
    let mut patterns: Vec<String> = DEFAULT_EXCLUDES.iter().map(|s| s.to_string()).collect();
    if let Some(s) = exclude {
        for p in s.split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()) {
            if !patterns.contains(&p) {
                patterns.push(p);
            }
        }
    }
    patterns
}

fn matches_pattern(filename: &str, pattern: &str) -> bool {
    if pattern.starts_with('*') {
        let suffix = &pattern[1..];
        filename.ends_with(suffix)
    } else if pattern.ends_with('*') {
        let prefix = &pattern[..pattern.len() - 1];
        filename.starts_with(prefix)
    } else {
        filename == pattern
    }
}

fn is_excluded(filename: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|p| matches_pattern(filename, p))
}

pub fn run(schema_path: &Path, folder: &Path, fix: bool, exclude: Option<&str>) -> Result<(), String> {
    let schema_content = std::fs::read_to_string(schema_path)
        .map_err(|e| format!("Cannot read schema {}: {e}", schema_path.display()))?;
    let schema: Schema = toml::from_str(&schema_content)
        .map_err(|e| format!("Invalid schema TOML: {e}"))?;

    let files = frontmatter::collect_md_files(folder);
    if files.is_empty() {
        return Err("No .md files found".to_string());
    }

    let exclude_patterns = parse_exclude_patterns(exclude);
    let mut pass = 0u32;
    let mut fail = 0u32;

    for file in &files {
        let fname = file.file_name().unwrap_or_default().to_string_lossy();

        if is_excluded(&fname, &exclude_patterns) {
            println!("{} {} — excluded", "SKIP".yellow().bold(), fname);
            continue;
        }
        let doc = match frontmatter::read_file(file) {
            Ok(d) => d,
            Err(_) => {
                if !fix {
                    fail += 1;
                    println!("{} {} — no frontmatter", "FAIL".red().bold(), fname);
                    continue;
                }
                match create_frontmatter(file, &schema) {
                    Ok(d) => {
                        let all_fields: Vec<String> = schema.fields.keys().filter(|k| schema.fields[k.as_str()].mandatory).cloned().collect();
                        let mut added = all_fields;
                        added.sort();
                        println!(
                            "{} {} — created frontmatter, added: {}",
                            "FIXED".yellow().bold(),
                            fname,
                            added.join(", ")
                        );
                        d
                    }
                    Err(e) => {
                        fail += 1;
                        println!("{} {} — fix error: {e}", "FAIL".red().bold(), fname);
                        continue;
                    }
                }
            }
        };

        let errors = validate(&doc.frontmatter, &schema);

        if errors.is_empty() {
            pass += 1;
            println!("{} {}", "PASS".green().bold(), fname);
            continue;
        }

        if !fix {
            fail += 1;
            println!("{} {} — {}", "FAIL".red().bold(), fname, errors.join("; "));
            continue;
        }

        let missing = find_fixable_fields(&doc.frontmatter, &schema);
        if missing.is_empty() {
            fail += 1;
            println!("{} {} — {}", "FAIL".red().bold(), fname, errors.join("; "));
            continue;
        }

        if let Err(e) = apply_fix(file, &doc, &schema, &missing) {
            fail += 1;
            println!("{} {} — fix error: {e}", "FAIL".red().bold(), fname);
            continue;
        }

        let mut added: Vec<String> = missing.clone();
        added.sort();
        println!(
            "{} {} — added: {}",
            "FIXED".yellow().bold(),
            fname,
            added.join(", ")
        );

        let re_doc = match frontmatter::read_file(file) {
            Ok(d) => d,
            Err(_) => {
                fail += 1;
                println!("{} {} — re-read failed after fix", "FAIL".red().bold(), fname);
                continue;
            }
        };
        let re_errors = validate(&re_doc.frontmatter, &schema);
        if re_errors.is_empty() {
            pass += 1;
            println!("{} {}", "PASS".green().bold(), fname);
        } else {
            fail += 1;
            println!(
                "{} {} — {}",
                "FAIL".red().bold(),
                fname,
                re_errors.join("; ")
            );
        }
    }

    let total = pass + fail;
    println!(
        "\n{} passed, {} failed out of {} files",
        pass.to_string().green(),
        if fail > 0 {
            fail.to_string().red().to_string()
        } else {
            "0".to_string()
        },
        total
    );

    if fail > 0 {
        Err(format!("{fail} file(s) failed validation"))
    } else {
        Ok(())
    }
}
