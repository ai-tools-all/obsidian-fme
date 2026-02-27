# fme — Frontmatter Engine

CLI for Obsidian vault frontmatter validation, querying, and SM-2 spaced repetition workflows.

## Install

```sh
cargo build --release
cp target/release/fme ~/bin/
```

## Quick Start

```sh
# Validate against schema & auto-fix missing mandatory fields
fme enforce --folder ./vault --fix

# Skip files matching a pattern
fme enforce --folder ./vault --exclude "template.md,README.md"

# Query with DSL
fme query "difficulty = hard AND status = completed" --folder .

# Show values for matching files
fme query "status = completed" --folder . --verbose

# Items due for review today
fme today --folder .

# Record review with quality rating (0-5)
fme review --file vault/problem.md --quality 4

# Initialize SR fields on a file/folder
fme init-sr --file vault/problem.md
fme init-sr --folder ./vault --review-type solve

# SR analytics (streaks, overdue items, forecast)
fme stats --folder .
```

## Frontmatter Format

Use **TOML** with `+++` delimiters (YAML `---` supported for reading, auto-converts to TOML on write):

```toml
+++
type = "leetcode"
title = "Clone Graph"
status = "completed"
topics = ["graph", "dfs"]
difficulty = "medium"
date = "2026-02-22"
sr = { next_review = "2026-03-01", interval = 5, ease = 2.6, reps = 3 }
+++
```

## Commands

| Command | Flags | Purpose |
|---------|-------|---------|
| `enforce` | `--schema`, `--folder`, `--fix`, `--exclude` | Validate against TOML schema, auto-fix with `--fix`, skip patterns with `--exclude` |
| `query` | `expression`, `--folder`, `--verbose` | DSL search: `=`, `!=`, `contains`, `<`, `<=`, `>`, `>=`, `exists`, `missing`; combine with `AND`, `OR` |
| `today` | `--folder` | Show SR items due today |
| `review` | `--file`, `--quality` | Record SM-2 review (0=fail, 3=ok, 5=perfect) |
| `init-sr` | `--file`, `--folder`, `--review-type` | Add SR metadata block |
| `stats` | `--folder` | Show SR metrics: streaks, overdue, load forecast |

## Schema Format

```toml
[fields.status]
mandatory = true
allowed_values = ["attempted", "completed", "revisited"]

[fields.difficulty]
mandatory = false
allowed_values = ["easy", "medium", "hard"]

[fields.date]
mandatory = true
format = "date"
```
