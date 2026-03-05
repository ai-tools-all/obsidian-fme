mod common;

use clap_describe::{extract_schema, extract_schema_with_extra};

#[test]
fn empty_command_has_name_and_usage_only() {
    let cmd = common::empty_command();
    let schema = extract_schema(&cmd);
    assert_eq!(schema.name, "empty");
    assert!(schema.about.is_none());
    assert!(schema.long_about.is_none());
    assert!(schema.positional_args.is_empty());
    assert!(schema.options.is_empty());
    assert!(schema.subcommands.is_empty());
    assert!(schema.extra_description.is_none());
    assert!(!schema.usage.is_empty());
}

#[test]
fn positional_only_has_no_options() {
    let cmd = common::positional_only_command();
    let schema = extract_schema(&cmd);
    assert_eq!(schema.positional_args.len(), 2);
    assert!(schema.options.is_empty());
    assert!(schema.subcommands.is_empty());
}

#[test]
fn hidden_subcommands_excluded() {
    let cmd = common::command_with_subcommands();
    let schema = extract_schema(&cmd);
    let names: Vec<&str> = schema.subcommands.iter().map(|s| s.name.as_str()).collect();
    assert!(!names.contains(&"hidden-cmd"));
    assert_eq!(schema.subcommands.len(), 3); // init, build, deploy
}

#[test]
fn hidden_args_excluded() {
    let cmd = common::realistic_command();
    let schema = extract_schema(&cmd);
    let names: Vec<&str> = schema.options.iter().map(|a| a.name.as_str()).collect();
    assert!(!names.contains(&"hidden-internal"));
}

#[test]
fn nested_subcommands_only_shows_direct_children() {
    let cmd = common::nested_subcommands();
    let schema = extract_schema(&cmd);
    assert_eq!(schema.subcommands.len(), 1);
    assert_eq!(schema.subcommands[0].name, "level1");
}

#[test]
fn possible_values_preserved_in_order() {
    let cmd = common::realistic_command();
    let schema = extract_schema(&cmd);
    let format_arg = schema.options.iter().find(|a| a.name == "format").unwrap();
    assert_eq!(format_arg.possible_values, vec!["json", "text", "github"]);
}

#[test]
fn default_values_captured() {
    let cmd = common::realistic_command();
    let schema = extract_schema(&cmd);
    let folder = schema.options.iter().find(|a| a.name == "folder").unwrap();
    assert_eq!(folder.default_value.as_deref(), Some("."));
    let depth = schema.options.iter().find(|a| a.name == "depth").unwrap();
    assert_eq!(depth.default_value.as_deref(), Some("10"));
}

#[test]
fn bool_flags_dont_take_value() {
    let cmd = common::realistic_command();
    let schema = extract_schema(&cmd);
    let fix = schema.options.iter().find(|a| a.name == "fix").unwrap();
    assert!(!fix.takes_value);
    assert!(!fix.required);
}

#[test]
fn extra_description_none_by_default() {
    let cmd = common::realistic_command();
    let schema = extract_schema(&cmd);
    assert!(schema.extra_description.is_none());
}

#[test]
fn extra_description_with_extra() {
    let cmd = common::realistic_command();
    let schema = extract_schema_with_extra(&cmd, Some("Agent notes".into()));
    assert_eq!(schema.extra_description.as_deref(), Some("Agent notes"));
}

#[test]
fn json_omits_empty_optional_fields() {
    let cmd = common::empty_command();
    let schema = extract_schema(&cmd);
    let json = serde_json::to_string_pretty(&schema).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(val.get("about").is_none());
    assert!(val.get("long_about").is_none());
    assert!(val.get("positional_args").is_none());
    assert!(val.get("options").is_none());
    assert!(val.get("subcommands").is_none());
    assert!(val.get("extra_description").is_none());
    assert!(val.get("name").is_some());
    assert!(val.get("usage").is_some());
}

#[test]
fn markdown_empty_command_has_no_tables() {
    let cmd = common::empty_command();
    let md = extract_schema(&cmd).to_markdown();
    assert!(!md.contains("## Arguments"));
    assert!(!md.contains("## Options"));
    assert!(!md.contains("## Subcommands"));
    assert!(!md.contains("## Notes"));
}
