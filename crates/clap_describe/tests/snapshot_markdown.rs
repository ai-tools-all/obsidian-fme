mod common;

use clap_describe::{extract_schema, extract_schema_with_extra};

fn markdown_output(cmd: &clap::Command) -> String {
    extract_schema(cmd).to_markdown()
}

#[test]
fn snapshot_empty_command_markdown() {
    let cmd = common::empty_command();
    let actual = markdown_output(&cmd);
    let expected = include_str!("snapshots/empty_command.md");
    assert_eq!(
        actual.trim(),
        expected.trim(),
        "Snapshot mismatch for empty_command.md. If intentional, update the snapshot file."
    );
}

#[test]
fn snapshot_positional_only_markdown() {
    let cmd = common::positional_only_command();
    let actual = markdown_output(&cmd);
    let expected = include_str!("snapshots/positional_only.md");
    assert_eq!(
        actual.trim(),
        expected.trim(),
        "Snapshot mismatch for positional_only.md. If intentional, update the snapshot file."
    );
}

#[test]
fn snapshot_realistic_command_markdown() {
    let cmd = common::realistic_command();
    let actual = markdown_output(&cmd);
    let expected = include_str!("snapshots/realistic_command.md");
    assert_eq!(
        actual.trim(),
        expected.trim(),
        "Snapshot mismatch for realistic_command.md. If intentional, update the snapshot file."
    );
}

#[test]
fn snapshot_command_with_subcommands_markdown() {
    let cmd = common::command_with_subcommands();
    let actual = markdown_output(&cmd);
    let expected = include_str!("snapshots/command_with_subcommands.md");
    assert_eq!(
        actual.trim(),
        expected.trim(),
        "Snapshot mismatch for command_with_subcommands.md. If intentional, update the snapshot file."
    );
}

#[test]
fn snapshot_with_extra_description_markdown() {
    let cmd = common::realistic_command();
    let schema = extract_schema_with_extra(
        &cmd,
        Some(
            "This command is used by AI agents to validate vault structure.\n\
             Run with --fix to auto-repair."
                .to_string(),
        ),
    );
    let actual = schema.to_markdown();
    let expected = include_str!("snapshots/with_extra_description.md");
    assert_eq!(
        actual.trim(),
        expected.trim(),
        "Snapshot mismatch for with_extra_description.md. If intentional, update the snapshot file."
    );
}
