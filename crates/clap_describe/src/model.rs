use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSchema {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_about: Option<String>,
    pub usage: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub positional_args: Vec<ArgSchema>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<ArgSchema>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub subcommands: Vec<SubcommandSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgSchema {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short: Option<char>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long: Option<String>,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub possible_values: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_name: Option<String>,
    pub takes_value: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubcommandSummary {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_schema_serializes_to_json() {
        let schema = CommandSchema {
            name: "mycli".to_string(),
            about: Some("A test CLI".to_string()),
            long_about: None,
            usage: "mycli [OPTIONS]".to_string(),
            positional_args: vec![],
            options: vec![ArgSchema {
                name: "verbose".to_string(),
                short: Some('v'),
                long: Some("verbose".to_string()),
                required: false,
                default_value: None,
                possible_values: vec![],
                help: Some("Enable verbose output".to_string()),
                value_name: None,
                takes_value: false,
            }],
            subcommands: vec![],
            extra_description: None,
        };

        let json = serde_json::to_string_pretty(&schema).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(val["name"], "mycli");
        assert_eq!(val["about"], "A test CLI");
        assert_eq!(val["usage"], "mycli [OPTIONS]");
        assert_eq!(val["options"][0]["name"], "verbose");
        assert_eq!(val["options"][0]["short"], "v");

        assert!(val.get("long_about").is_none());
        assert!(val.get("positional_args").is_none());
        assert!(val.get("subcommands").is_none());
        assert!(val.get("extra_description").is_none());
    }

    #[test]
    fn arg_schema_omits_empty_fields() {
        let arg = ArgSchema {
            name: "input".to_string(),
            short: None,
            long: None,
            required: true,
            default_value: None,
            possible_values: vec![],
            help: Some("Input file".to_string()),
            value_name: None,
            takes_value: true,
        };

        let json = serde_json::to_string(&arg).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(val.get("short").is_none());
        assert!(val.get("long").is_none());
        assert!(val.get("default_value").is_none());
        assert!(val.get("possible_values").is_none());
        assert!(val.get("value_name").is_none());

        assert_eq!(val["name"], "input");
        assert_eq!(val["required"], true);
        assert_eq!(val["takes_value"], true);
        assert_eq!(val["help"], "Input file");
    }
}
