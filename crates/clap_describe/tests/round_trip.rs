mod common;

use clap_describe::{extract_schema, extract_schema_with_extra, CommandSchema};

#[test]
fn round_trip_empty_command() {
    let cmd = common::empty_command();
    let schema = extract_schema(&cmd);
    let json = serde_json::to_string_pretty(&schema).unwrap();
    let deserialized: CommandSchema = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, schema.name);
    assert_eq!(deserialized.usage, schema.usage);
    assert_eq!(deserialized.about, schema.about);
}

#[test]
fn round_trip_realistic_command() {
    let cmd = common::realistic_command();
    let schema = extract_schema(&cmd);
    let json = serde_json::to_string_pretty(&schema).unwrap();
    let deserialized: CommandSchema = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name, schema.name);
    assert_eq!(deserialized.about, schema.about);
    assert_eq!(deserialized.long_about, schema.long_about);
    assert_eq!(deserialized.usage, schema.usage);
    assert_eq!(
        deserialized.positional_args.len(),
        schema.positional_args.len()
    );
    assert_eq!(deserialized.options.len(), schema.options.len());
    assert_eq!(deserialized.subcommands.len(), schema.subcommands.len());

    for (orig, deser) in schema.options.iter().zip(deserialized.options.iter()) {
        assert_eq!(orig.name, deser.name);
        assert_eq!(orig.short, deser.short);
        assert_eq!(orig.long, deser.long);
        assert_eq!(orig.required, deser.required);
        assert_eq!(orig.default_value, deser.default_value);
        assert_eq!(orig.possible_values, deser.possible_values);
        assert_eq!(orig.help, deser.help);
        assert_eq!(orig.value_name, deser.value_name);
        assert_eq!(orig.takes_value, deser.takes_value);
    }
}

#[test]
fn round_trip_with_subcommands() {
    let cmd = common::command_with_subcommands();
    let schema = extract_schema(&cmd);
    let json = serde_json::to_string_pretty(&schema).unwrap();
    let deserialized: CommandSchema = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.subcommands.len(), schema.subcommands.len());
    for (orig, deser) in schema
        .subcommands
        .iter()
        .zip(deserialized.subcommands.iter())
    {
        assert_eq!(orig.name, deser.name);
        assert_eq!(orig.about, deser.about);
    }
}

#[test]
fn round_trip_with_extra_description() {
    let cmd = common::realistic_command();
    let schema = extract_schema_with_extra(&cmd, Some("Agent-specific notes".into()));
    let json = serde_json::to_string_pretty(&schema).unwrap();
    let deserialized: CommandSchema = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.extra_description, schema.extra_description);
}

#[test]
fn round_trip_json_is_identical() {
    let cmd = common::realistic_command();
    let schema = extract_schema(&cmd);
    let json1 = serde_json::to_string_pretty(&schema).unwrap();
    let deserialized: CommandSchema = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string_pretty(&deserialized).unwrap();
    assert_eq!(
        json1, json2,
        "Re-serialized JSON should be identical to original"
    );
}
