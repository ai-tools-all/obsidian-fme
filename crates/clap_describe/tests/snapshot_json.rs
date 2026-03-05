mod common;

use clap_describe::extract_schema;

fn json_output(cmd: &clap::Command) -> String {
    let schema = extract_schema(cmd);
    serde_json::to_string_pretty(&schema).unwrap()
}

#[test]
fn snapshot_empty_command_json() {
    let cmd = common::empty_command();
    let actual = json_output(&cmd);
    let expected = include_str!("snapshots/empty_command.json");
    assert_eq!(
        actual.trim(),
        expected.trim(),
        "Snapshot mismatch for empty_command.json. If intentional, update the snapshot file."
    );
}

#[test]
fn snapshot_positional_only_json() {
    let cmd = common::positional_only_command();
    let actual = json_output(&cmd);
    let expected = include_str!("snapshots/positional_only.json");
    assert_eq!(
        actual.trim(),
        expected.trim(),
        "Snapshot mismatch for positional_only.json. If intentional, update the snapshot file."
    );
}

#[test]
fn snapshot_realistic_command_json() {
    let cmd = common::realistic_command();
    let actual = json_output(&cmd);
    let expected = include_str!("snapshots/realistic_command.json");
    assert_eq!(
        actual.trim(),
        expected.trim(),
        "Snapshot mismatch for realistic_command.json. If intentional, update the snapshot file."
    );
}

#[test]
fn snapshot_command_with_subcommands_json() {
    let cmd = common::command_with_subcommands();
    let actual = json_output(&cmd);
    let expected = include_str!("snapshots/command_with_subcommands.json");
    assert_eq!(
        actual.trim(),
        expected.trim(),
        "Snapshot mismatch for command_with_subcommands.json. If intentional, update the snapshot file."
    );
}
