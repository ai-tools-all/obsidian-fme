# fme — Frontmatter Engine

CLI tool for managing Obsidian vault frontmatter. Validates schemas, queries fields, and runs spaced repetition (SM-2) workflows.

## Commands

| Command | What it does |
|---------|-------------|
| `enforce` | Validate frontmatter against a TOML schema |
| `query` | Search files using a DSL (`"difficulty = hard AND status = completed"`) |
| `today` | Show items due for spaced repetition review |
| `review` | Record an SM-2 review rating (0-5) for a file |
| `init-sr` | Add SR metadata block to a file |
| `stats` | Display SR analytics (streaks, forecasts, weak items) |

## Frontmatter Format

**TOML** (`+++` delimiters) is the canonical format. YAML (`---`) is supported for reading only — any write operation converts to TOML.

```
+++
type = "leetcode"
title = "Clone Graph"
status = "completed"
topics = ["graph", "dfs"]
difficulty = "medium"
date = "2026-02-22"
+++
```

## Install

```sh
cargo build --release
cp target/release/fme ~/bin/
```

## Usage

```sh
fme enforce ./vault --schema schema.toml
fme query ./vault "status = completed AND difficulty = hard"
fme today ./vault
fme review ./vault/file.md 4
fme stats ./vault
```
