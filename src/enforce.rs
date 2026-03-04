use crate::{frontmatter, model::*, render};
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
                        errors.push(format!("`{field_name}` = \"{s}\" not in {:?}", allowed));
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
        filename.ends_with(&pattern[1..])
    } else if pattern.ends_with('*') {
        filename.starts_with(&pattern[..pattern.len() - 1])
    } else {
        filename == pattern
    }
}

fn is_excluded(filename: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|p| matches_pattern(filename, p))
}

fn hint_for_unfixable(fm: &toml::Value, schema: &Schema) -> Vec<String> {
    let mut hints = Vec::new();
    for (field_name, field_def) in &schema.fields {
        if let Some(allowed) = &field_def.allowed_values {
            if field_def.default.is_none() {
                if let Some(v) = frontmatter::get_nested(fm, field_name) {
                    let s = frontmatter::value_to_string(v);
                    if !allowed.contains(&s) {
                        hints.push(format!(
                            "HINT: `{field_name}` has allowed_values but no default — add `default = \"{}\"` to [fields.{field_name}] in schema.toml to enable --fix",
                            allowed[0]
                        ));
                    }
                }
            }
        }
    }
    hints
}

pub fn run_single_file(
    schema_path: &Path,
    file: &Path,
    fix: bool,
    json: bool,
) -> Result<(), String> {
    let schema_content = std::fs::read_to_string(schema_path)
        .map_err(|e| format!("Cannot read schema {}: {e}", schema_path.display()))?;
    let schema: Schema =
        toml::from_str(&schema_content).map_err(|e| format!("Invalid schema TOML: {e}"))?;

    let fname = file.file_name().unwrap_or_default().to_string_lossy().to_string();
    let mut results: Vec<EnforceFile> = Vec::new();
    let mut pass = 0u32;
    let mut fail = 0u32;

    let (doc, was_created, created_fields) = match frontmatter::read_file(file) {
        Ok(d) => (d, false, vec![]),
        Err(_) => {
            if !fix {
                fail += 1;
                results.push(EnforceFile {
                    file: fname,
                    status: FileStatus::Fail,
                    errors: vec!["no frontmatter".into()],
                    fixed: vec![],
                });
                let result = EnforceResult { passed: pass, failed: fail, results };
                render::get(json).enforce(&result);
                return Err("1 file(s) failed validation".to_string());
            }
            match create_frontmatter(file, &schema) {
                Ok(d) => {
                    let mut added: Vec<String> = schema
                        .fields
                        .keys()
                        .filter(|k| schema.fields[k.as_str()].mandatory)
                        .cloned()
                        .collect();
                    added.sort();
                    (d, true, added)
                }
                Err(e) => {
                    fail += 1;
                    results.push(EnforceFile {
                        file: fname,
                        status: FileStatus::Fail,
                        errors: vec![format!("fix error: {e}")],
                        fixed: vec![],
                    });
                    let result = EnforceResult { passed: pass, failed: fail, results };
                    render::get(json).enforce(&result);
                    return Err("1 file(s) failed validation".to_string());
                }
            }
        }
    };

    let errors = validate(&doc.frontmatter, &schema);
    if errors.is_empty() {
        pass += 1;
        if was_created {
            let mut fixed = vec!["created frontmatter".to_string()];
            fixed.extend(created_fields);
            results.push(EnforceFile { file: fname, status: FileStatus::Fixed, errors: vec![], fixed });
        } else {
            results.push(EnforceFile { file: fname, status: FileStatus::Pass, errors: vec![], fixed: vec![] });
        }
        let result = EnforceResult { passed: pass, failed: fail, results };
        render::get(json).enforce(&result);
        return Ok(());
    }

    if !fix {
        fail += 1;
        results.push(EnforceFile { file: fname, status: FileStatus::Fail, errors, fixed: vec![] });
        let result = EnforceResult { passed: pass, failed: fail, results };
        render::get(json).enforce(&result);
        return Err("1 file(s) failed validation".to_string());
    }

    let missing = find_fixable_fields(&doc.frontmatter, &schema);
    if missing.is_empty() {
        fail += 1;
        let mut all_errors = errors;
        all_errors.extend(hint_for_unfixable(&doc.frontmatter, &schema));
        results.push(EnforceFile { file: fname, status: FileStatus::Fail, errors: all_errors, fixed: vec![] });
        let result = EnforceResult { passed: pass, failed: fail, results };
        render::get(json).enforce(&result);
        return Err("1 file(s) failed validation".to_string());
    }

    if let Err(e) = apply_fix(file, &doc, &schema, &missing) {
        fail += 1;
        results.push(EnforceFile {
            file: fname,
            status: FileStatus::Fail,
            errors: vec![format!("fix error: {e}")],
            fixed: vec![],
        });
        let result = EnforceResult { passed: pass, failed: fail, results };
        render::get(json).enforce(&result);
        return Err("1 file(s) failed validation".to_string());
    }

    let mut added = missing.clone();
    added.sort();

    let re_doc = match frontmatter::read_file(file) {
        Ok(d) => d,
        Err(_) => {
            fail += 1;
            results.push(EnforceFile {
                file: fname,
                status: FileStatus::Fail,
                errors: vec!["re-read failed after fix".into()],
                fixed: added,
            });
            let result = EnforceResult { passed: pass, failed: fail, results };
            render::get(json).enforce(&result);
            return Err("1 file(s) failed validation".to_string());
        }
    };

    let re_errors = validate(&re_doc.frontmatter, &schema);
    if re_errors.is_empty() {
        pass += 1;
        results.push(EnforceFile { file: fname, status: FileStatus::Fixed, errors: vec![], fixed: added });
    } else {
        fail += 1;
        results.push(EnforceFile { file: fname, status: FileStatus::Fail, errors: re_errors, fixed: added });
    }

    let result = EnforceResult { passed: pass, failed: fail, results };
    render::get(json).enforce(&result);

    if fail > 0 {
        Err("1 file(s) failed validation".to_string())
    } else {
        Ok(())
    }
}

pub fn run(
    schema_path: &Path,
    folder: &Path,
    fix: bool,
    exclude: Option<&str>,
    depth: usize,
    json: bool,
) -> Result<(), String> {
    let schema_content = std::fs::read_to_string(schema_path)
        .map_err(|e| format!("Cannot read schema {}: {e}", schema_path.display()))?;
    let schema: Schema =
        toml::from_str(&schema_content).map_err(|e| format!("Invalid schema TOML: {e}"))?;

    let files = frontmatter::collect_md_files(folder, depth);
    if files.is_empty() {
        return Err("No .md files found".to_string());
    }

    let exclude_patterns = parse_exclude_patterns(exclude);
    let mut pass = 0u32;
    let mut fail = 0u32;
    let mut results: Vec<EnforceFile> = Vec::new();

    for file in &files {
        let fname = file.file_name().unwrap_or_default().to_string_lossy().to_string();

        if is_excluded(&fname, &exclude_patterns) {
            results.push(EnforceFile {
                file: fname,
                status: FileStatus::Skip,
                errors: vec![],
                fixed: vec![],
            });
            continue;
        }

        let (doc, was_created, created_fields) = match frontmatter::read_file(file) {
            Ok(d) => (d, false, vec![]),
            Err(_) => {
                if !fix {
                    fail += 1;
                    results.push(EnforceFile {
                        file: fname,
                        status: FileStatus::Fail,
                        errors: vec!["no frontmatter".into()],
                        fixed: vec![],
                    });
                    continue;
                }
                match create_frontmatter(file, &schema) {
                    Ok(d) => {
                        let mut added: Vec<String> = schema
                            .fields
                            .keys()
                            .filter(|k| schema.fields[k.as_str()].mandatory)
                            .cloned()
                            .collect();
                        added.sort();
                        (d, true, added)
                    }
                    Err(e) => {
                        fail += 1;
                        results.push(EnforceFile {
                            file: fname,
                            status: FileStatus::Fail,
                            errors: vec![format!("fix error: {e}")],
                            fixed: vec![],
                        });
                        continue;
                    }
                }
            }
        };

        let errors = validate(&doc.frontmatter, &schema);
        if errors.is_empty() {
            pass += 1;
            if was_created {
                let mut fixed = vec!["created frontmatter".to_string()];
                fixed.extend(created_fields);
                results.push(EnforceFile { file: fname, status: FileStatus::Fixed, errors: vec![], fixed });
            } else {
                results.push(EnforceFile { file: fname, status: FileStatus::Pass, errors: vec![], fixed: vec![] });
            }
            continue;
        }

        if !fix {
            fail += 1;
            results.push(EnforceFile {
                file: fname,
                status: FileStatus::Fail,
                errors,
                fixed: vec![],
            });
            continue;
        }

        let missing = find_fixable_fields(&doc.frontmatter, &schema);
        if missing.is_empty() {
            fail += 1;
            let mut all_errors = errors;
            all_errors.extend(hint_for_unfixable(&doc.frontmatter, &schema));
            results.push(EnforceFile {
                file: fname,
                status: FileStatus::Fail,
                errors: all_errors,
                fixed: vec![],
            });
            continue;
        }

        if let Err(e) = apply_fix(file, &doc, &schema, &missing) {
            fail += 1;
            results.push(EnforceFile {
                file: fname,
                status: FileStatus::Fail,
                errors: vec![format!("fix error: {e}")],
                fixed: vec![],
            });
            continue;
        }

        let mut added = missing.clone();
        added.sort();

        let re_doc = match frontmatter::read_file(file) {
            Ok(d) => d,
            Err(_) => {
                fail += 1;
                results.push(EnforceFile {
                    file: fname,
                    status: FileStatus::Fail,
                    errors: vec!["re-read failed after fix".into()],
                    fixed: added,
                });
                continue;
            }
        };

        let re_errors = validate(&re_doc.frontmatter, &schema);
        if re_errors.is_empty() {
            pass += 1;
            results.push(EnforceFile {
                file: fname,
                status: FileStatus::Fixed,
                errors: vec![],
                fixed: added,
            });
        } else {
            fail += 1;
            results.push(EnforceFile {
                file: fname,
                status: FileStatus::Fail,
                errors: re_errors,
                fixed: added,
            });
        }
    }

    let result = EnforceResult { passed: pass, failed: fail, results };
    render::get(json).enforce(&result);

    if fail > 0 {
        Err(format!("{fail} file(s) failed validation"))
    } else {
        Ok(())
    }
}
