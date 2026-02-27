# md-fme — Frontmatter Engine

CLI for Obsidian vault frontmatter validation, querying, and SM-2 spaced repetition workflows.

## Install

```sh
cargo build --release
cp target/release/md-fme ~/bin/
```

## Quick Start

```sh
# Validate against schema.toml auto-discovered in the folder
md-fme enforce --folder ./vault

# Validate with an explicit schema file
md-fme enforce --schema ./schemas/my-schema.toml --folder ./vault

# Auto-fix missing mandatory fields
md-fme enforce --folder ./vault --fix

# Skip files matching a pattern
md-fme enforce --folder ./vault --exclude "template.md,README.md"

# Query with DSL
md-fme query "difficulty = hard AND status = completed" --folder .

# Show values for matching files
md-fme query "status = completed" --folder . --verbose

# Items due for review today
md-fme today --folder .

# Record review with quality rating (0-5)
md-fme review --file vault/problem.md --quality 4

# Initialize SR fields on a file/folder
md-fme init-sr --file vault/problem.md
md-fme init-sr --folder ./vault --review-type solve

# SR analytics (streaks, overdue items, forecast)
md-fme stats --folder .
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

Place a `schema.toml` in your vault folder — `enforce` picks it up automatically. Pass `--schema <path>` to use a file elsewhere.

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

`--fix` adds missing mandatory fields using the first `allowed_values` entry as the default.

## Complete schema.toml Reference

Every field you want to enforce gets its own `[fields.<name>]` block.
The four keys per block are:

| Key | Type | When to use |
|-----|------|-------------|
| `mandatory` | bool | `true` → file fails if the field is absent |
| `allowed_values` | string array | restrict the field to a fixed set of values |
| `format` | `"date"` | validate the value is a date (`YYYY-MM-DD`) |
| `default` | string | value written by `--fix` when the field is missing or invalid |

```toml
# schema.toml — drop this in your vault root (or pass via --schema)
#
# fme enforce auto-discovers schema.toml inside --folder.
# Use --fix to have fme write defaults into files that fail.

# ── mandatory + constrained ──────────────────────────────────────────
# Use when the field must always be present AND must be one of a fixed set.
# The first allowed_values entry becomes the --fix default.
[fields.status]
mandatory = true
allowed_values = ["attempted", "completed", "revisited", "skipped"]
default = "attempted"          # written by --fix when missing or invalid

[fields.difficulty]
mandatory = true
allowed_values = ["easy", "medium", "hard"]
default = "medium"

# ── mandatory + free-form ────────────────────────────────────────────
# Use when the field must exist but any string value is acceptable.
# Set a sensible default so --fix can populate it.
[fields.title]
mandatory = true
default = ""                   # --fix inserts an empty string; fill it in manually

[fields.type]
mandatory = true
allowed_values = ["leetcode", "system-design", "concept", "project"]
default = "concept"

# ── mandatory + date format ──────────────────────────────────────────
# Use when the field must exist AND must be a valid YYYY-MM-DD date.
# format = "date" rejects strings like "today" or "02/22/2026".
# Combine with default only if you want --fix to stamp a placeholder date.
[fields.date]
mandatory = true
format = "date"
# no default here — let --fix skip it so you don't end up with a wrong date

# ── optional + constrained ───────────────────────────────────────────
# Use when the field is optional but, if present, must be one of the list.
# mandatory = false means missing files still PASS; wrong values still FAIL.
[fields.review_type]
mandatory = false
allowed_values = ["solve", "read", "implement"]

# ── optional + date format ───────────────────────────────────────────
# Use for fields like last_reviewed that should be a real date when set.
[fields.last_reviewed]
mandatory = false
format = "date"

# ── optional + free-form ─────────────────────────────────────────────
# Use for informational fields you want to allow but not enforce.
# Omitting allowed_values means any string (or array) value is accepted.
[fields.source]
mandatory = false

# ── nested fields ────────────────────────────────────────────────────
# fme supports dot-notation for nested TOML tables (e.g. sr.interval).
# Use when you want to validate a field inside a sub-table.
[fields."sr.ease"]
mandatory = false              # present only on SR-tracked files

[fields."sr.next_review"]
mandatory = false
format = "date"
```
