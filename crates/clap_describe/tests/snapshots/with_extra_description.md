# `enforce`

Validate that markdown files in a folder have frontmatter matching the specified schema. Reports errors and optionally fixes them.

**Usage:** `enforce [OPTIONS]`

## Options

| Flag | Required | Default | Possible Values | Description |
|------|----------|---------|-----------------|-------------|
| `-s, --schema <PATH>` | no | - | - | Path to schema file |
| `-f, --folder <FOLDER>` | no | . | - | Folder to scan |
| `--fix` | no | - | - | Auto-fix issues where possible |
| `-e, --exclude <PATTERN>` | no | - | - | Glob pattern to exclude |
| `-d, --depth <N>` | no | 10 | - | Maximum directory depth |
| `--format <FORMAT>` | no | text | json, text, github | Output format |

## Notes

This command is used by AI agents to validate vault structure.
Run with --fix to auto-repair.
