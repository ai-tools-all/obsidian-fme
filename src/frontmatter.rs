use std::path::Path;
use toml::Value;

pub struct Document {
    pub frontmatter: Value,
    pub raw: String,
}

fn parse_toml_content(content: &str) -> Option<Document> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("+++") {
        return None;
    }
    let after_first = &trimmed[3..];
    let end = after_first.find("\n+++")?;
    let toml_str = &after_first[..end];
    let frontmatter: Value = toml::from_str(toml_str).ok()?;
    Some(Document {
        frontmatter,
        raw: content.to_string(),
    })
}

fn parse_yaml_content(content: &str) -> Option<Document> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }
    let after_first = &trimmed[3..];
    let end = after_first.find("\n---")?;
    let yaml_str = &after_first[..end];
    let yaml_val: serde_yaml::Value = serde_yaml::from_str(yaml_str).ok()?;
    let frontmatter = yaml_to_toml(&yaml_val);
    Some(Document {
        frontmatter,
        raw: content.to_string(),
    })
}

pub fn parse(content: &str) -> Option<Document> {
    parse_toml_content(content).or_else(|| parse_yaml_content(content))
}

pub fn read_file(path: &Path) -> Result<Document, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    parse(&content).ok_or_else(|| format!("No frontmatter in {}", path.display()))
}

fn yaml_to_toml(yaml: &serde_yaml::Value) -> Value {
    match yaml {
        serde_yaml::Value::Null => Value::String("~".to_string()),
        serde_yaml::Value::Bool(b) => Value::Boolean(*b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::String(n.to_string())
            }
        }
        serde_yaml::Value::String(s) => Value::String(s.clone()),
        serde_yaml::Value::Sequence(seq) => {
            let items: Vec<Value> = seq
                .iter()
                .filter(|v| !v.is_null())
                .map(yaml_to_toml)
                .collect();
            Value::Array(items)
        }
        serde_yaml::Value::Mapping(map) => {
            let mut table = toml::map::Map::new();
            for (k, v) in map {
                let key = match k {
                    serde_yaml::Value::String(s) => s.clone(),
                    other => format!("{other:?}"),
                };
                if !v.is_null() {
                    table.insert(key, yaml_to_toml(v));
                }
            }
            Value::Table(table)
        }
        _ => Value::String(format!("{yaml:?}")),
    }
}

pub fn serialize_toml(frontmatter: &Value) -> String {
    let body = match frontmatter {
        Value::Table(t) => {
            let mut out = String::new();
            let mut inline_keys = Vec::new();
            let mut table_keys = Vec::new();
            for (k, v) in t {
                if matches!(v, Value::Table(_)) {
                    table_keys.push(k);
                } else {
                    inline_keys.push(k);
                }
            }
            for k in &inline_keys {
                let v = &t[k.as_str()];
                out.push_str(&format_kv(k, v));
            }
            for k in &table_keys {
                let v = &t[k.as_str()];
                if let Value::Table(inner) = v {
                    out.push_str(&format!("\n[{k}]\n"));
                    for (ik, iv) in inner {
                        out.push_str(&format_kv(ik, iv));
                    }
                }
            }
            out
        }
        _ => toml::to_string(frontmatter).unwrap_or_default(),
    };
    format!("+++\n{body}+++")
}

fn format_kv(key: &str, value: &Value) -> String {
    match value {
        Value::String(s) => format!("{key} = \"{s}\"\n"),
        Value::Integer(i) => format!("{key} = {i}\n"),
        Value::Float(f) => {
            if f.fract() == 0.0 {
                format!("{key} = {f:.1}\n")
            } else {
                format!("{key} = {f}\n")
            }
        }
        Value::Boolean(b) => format!("{key} = {b}\n"),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(|v| format_inline_value(v)).collect();
            format!("{key} = [{}]\n", items.join(", "))
        }
        Value::Table(_) => {
            toml::to_string(&toml::map::Map::from_iter([(key.to_string(), value.clone())]))
                .unwrap_or_default()
        }
        _ => format!("{key} = {value}\n"),
    }
}

fn format_inline_value(v: &Value) -> String {
    match v {
        Value::String(s) => format!("\"{s}\""),
        Value::Integer(i) => format!("{i}"),
        Value::Float(f) => format!("{f}"),
        Value::Boolean(b) => format!("{b}"),
        _ => format!("{v}"),
    }
}

fn frontmatter_boundaries(raw: &str) -> Option<(usize, usize, bool)> {
    let trimmed = raw.trim_start();
    let skip = raw.len() - trimmed.len();
    if trimmed.starts_with("+++") {
        let after = &trimmed[3..];
        let end = after.find("\n+++")?;
        let start = skip;
        let close = skip + 3 + end + 1 + 3;
        Some((start, close, true))
    } else if trimmed.starts_with("---") {
        let after = &trimmed[3..];
        let end = after.find("\n---")?;
        let start = skip;
        let close = skip + 3 + end + 1 + 3;
        Some((start, close, false))
    } else {
        None
    }
}

pub fn replace_frontmatter(raw: &str, new_fm: &Value) -> String {
    let (start, close, _is_toml) = match frontmatter_boundaries(raw) {
        Some(b) => b,
        None => return raw.to_string(),
    };
    let toml_block = serialize_toml(new_fm);
    let mut result = String::with_capacity(raw.len());
    result.push_str(&raw[..start]);
    result.push_str(&toml_block);
    result.push_str(&raw[close..]);
    result
}

pub fn has_sr_block(fm: &Value) -> bool {
    fm.get("sr").is_some()
}

pub fn write_raw(path: &Path, content: &str) -> Result<(), String> {
    std::fs::write(path, content)
        .map_err(|e| format!("Cannot write {}: {e}", path.display()))
}

pub fn read_raw(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read {}: {e}", path.display()))
}

pub fn collect_md_files(folder: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(folder)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if p.is_file()
            && p.extension().is_some_and(|ext| ext == "md")
            && p.file_name().is_some_and(|n| n != "claude.md" && n != "schema.toml")
        {
            files.push(p.to_path_buf());
        }
    }
    files.sort();
    files
}

pub fn get_nested<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    let parts: Vec<&str> = key.split('.').collect();
    let mut current = value;
    for part in parts {
        current = current.get(part)?;
    }
    Some(current)
}

pub fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => {
            if f.fract() == 0.0 && !f.is_nan() && !f.is_infinite() {
                format!("{f:.0}")
            } else {
                format!("{f}")
            }
        }
        Value::Boolean(b) => b.to_string(),
        Value::Datetime(d) => d.to_string(),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(value_to_string).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Table(t) => {
            let items: Vec<String> = t
                .iter()
                .map(|(k, v)| format!("{k}: {}", value_to_string(v)))
                .collect();
            format!("{{{}}}", items.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_yaml_frontmatter() {
        let content = "---\ntitle: Test\ntags:\n  - rust\n  - cli\n---\n# Body\n";
        let doc = parse(content).unwrap();
        assert_eq!(
            doc.frontmatter.get("title").unwrap().as_str().unwrap(),
            "Test"
        );
        let tags = doc.frontmatter.get("tags").unwrap().as_array().unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].as_str().unwrap(), "rust");
    }

    #[test]
    fn parse_toml_frontmatter() {
        let content = "+++\ntitle = \"Test\"\ntags = [\"rust\", \"cli\"]\n+++\n# Body\n";
        let doc = parse(content).unwrap();
        assert_eq!(
            doc.frontmatter.get("title").unwrap().as_str().unwrap(),
            "Test"
        );
        let tags = doc.frontmatter.get("tags").unwrap().as_array().unwrap();
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn toml_preferred_over_yaml() {
        let content = "+++\ntitle = \"TOML\"\n+++\n# Body\n";
        let doc = parse(content).unwrap();
        assert_eq!(
            doc.frontmatter.get("title").unwrap().as_str().unwrap(),
            "TOML"
        );
    }

    #[test]
    fn yaml_to_toml_conversion() {
        let yaml: serde_yaml::Value = serde_yaml::from_str(
            "title: Hello\ncount: 42\nactive: true\ntags:\n  - a\n  - b\n",
        )
        .unwrap();
        let toml_val = yaml_to_toml(&yaml);
        assert_eq!(toml_val.get("title").unwrap().as_str().unwrap(), "Hello");
        assert_eq!(toml_val.get("count").unwrap().as_integer().unwrap(), 42);
        assert!(toml_val.get("active").unwrap().as_bool().unwrap());
        assert_eq!(toml_val.get("tags").unwrap().as_array().unwrap().len(), 2);
    }

    #[test]
    fn serialize_round_trip() {
        let content = "+++\ntitle = \"Test\"\nid = 42\ntags = [\"a\", \"b\"]\n+++\n# Body\n";
        let doc = parse(content).unwrap();
        let serialized = serialize_toml(&doc.frontmatter);
        assert!(serialized.starts_with("+++\n"));
        assert!(serialized.ends_with("+++"));
        assert!(serialized.contains("title = \"Test\""));
        assert!(serialized.contains("id = 42"));
    }

    #[test]
    fn replace_yaml_with_toml() {
        let raw = "---\ntitle: Old\ntags:\n  - a\n---\n# Body\n";
        let doc = parse(raw).unwrap();
        let result = replace_frontmatter(raw, &doc.frontmatter);
        assert!(result.starts_with("+++\n"));
        assert!(result.contains("+++\n# Body"));
        assert!(result.contains("title = \"Old\""));
    }

    #[test]
    fn get_nested_works() {
        let content = "+++\n[sr]\nnext_review = \"2026-01-01\"\ninterval = 5\n+++\n";
        let doc = parse(content).unwrap();
        let nr = get_nested(&doc.frontmatter, "sr.next_review").unwrap();
        assert_eq!(nr.as_str().unwrap(), "2026-01-01");
    }

    #[test]
    fn value_to_string_types() {
        assert_eq!(value_to_string(&Value::String("hi".into())), "hi");
        assert_eq!(value_to_string(&Value::Integer(42)), "42");
        assert_eq!(value_to_string(&Value::Float(2.5)), "2.5");
        assert_eq!(value_to_string(&Value::Boolean(true)), "true");
    }

    #[test]
    fn has_sr_block_check() {
        let with_sr = "+++\ntitle = \"X\"\n[sr]\nnext_review = \"2026-01-01\"\n+++\n";
        let without_sr = "+++\ntitle = \"X\"\n+++\n";
        let doc1 = parse(with_sr).unwrap();
        let doc2 = parse(without_sr).unwrap();
        assert!(has_sr_block(&doc1.frontmatter));
        assert!(!has_sr_block(&doc2.frontmatter));
    }

    #[test]
    fn yaml_null_filtered() {
        let yaml: serde_yaml::Value =
            serde_yaml::from_str("tags:\n  - a\n  -\n  - b\n").unwrap();
        let toml_val = yaml_to_toml(&yaml);
        let tags = toml_val.get("tags").unwrap().as_array().unwrap();
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn yaml_nested_table_conversion() {
        let yaml: serde_yaml::Value = serde_yaml::from_str(
            "sr:\n  next_review: 2026-01-01\n  interval: 1\n  ease: 2.5\n",
        )
        .unwrap();
        let toml_val = yaml_to_toml(&yaml);
        let sr = toml_val.get("sr").unwrap().as_table().unwrap();
        assert_eq!(sr.get("next_review").unwrap().as_str().unwrap(), "2026-01-01");
        assert_eq!(sr.get("interval").unwrap().as_integer().unwrap(), 1);
    }
}
