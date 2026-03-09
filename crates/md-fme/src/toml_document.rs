use std::path::Path;
use toml::Value;

pub struct TomlDocument {
    pub content: Value,
    #[allow(dead_code)]
    pub raw: String,
}

pub fn parse(content: &str) -> Option<TomlDocument> {
    let content_val: Value = toml::from_str(content).ok()?;
    Some(TomlDocument {
        content: content_val,
        raw: content.to_string(),
    })
}

pub fn read_file(path: &Path) -> Result<TomlDocument, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    parse(&content).ok_or_else(|| format!("Invalid TOML in {}", path.display()))
}

#[allow(dead_code)]
pub fn write_raw(path: &Path, content: &str) -> Result<(), String> {
    std::fs::write(path, content)
        .map_err(|e| format!("Cannot write {}: {e}", path.display()))
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
    fn parse_valid_toml() {
        let content = "name = \"test\"\nrole = \"engineer\"\n";
        let doc = parse(content).unwrap();
        assert_eq!(doc.content.get("name").unwrap().as_str().unwrap(), "test");
        assert_eq!(doc.content.get("role").unwrap().as_str().unwrap(), "engineer");
    }

    #[test]
    fn parse_invalid_toml_fails() {
        let content = "name = \"test\"\nrole = unclosed";
        let result = parse(content);
        assert!(result.is_none());
    }

    #[test]
    fn get_nested_works() {
        let content = "name = \"test\"\n[sr]\nnext_review = \"2026-01-01\"\n";
        let doc = parse(content).unwrap();
        let nr = get_nested(&doc.content, "sr.next_review").unwrap();
        assert_eq!(nr.as_str().unwrap(), "2026-01-01");
    }

    #[test]
    fn value_to_string_types() {
        assert_eq!(value_to_string(&Value::String("hi".into())), "hi");
        assert_eq!(value_to_string(&Value::Integer(42)), "42");
        assert_eq!(value_to_string(&Value::Float(2.5)), "2.5");
        assert_eq!(value_to_string(&Value::Boolean(true)), "true");
    }
}
