---
title: "Add recursive depth parameter to commands (--depth)"
status: open
priority: medium
---

## Problem

`md-fme today`, `md-fme query`, and `md-fme stats` only scan a single directory level (non-recursive). This is limiting for nested project structures where SR items are scattered across subdirectories.

Example limitation:
```bash
# Works (finds items in this folder only)
md-fme query "sr.next_review <= today" --folder ./projects/search-relevance/learning-sessions

# Doesn't work (ignores ./projects/*/learning-sessions/ and nested subfolders)
md-fme query "sr.next_review <= today" --folder ./projects
```

## Proposed Solution

Add optional `--depth` parameter to control recursion depth:

```bash
# Default: depth = 3 (search up to 3 levels deep)
md-fme query "sr.next_review <= today" --folder ./projects

# Explicit depth
md-fme query "sr.next_review <= today" --folder ./projects --depth 5

# No recursion (current behavior)
md-fme query "sr.next_review <= today" --folder . --depth 1

# Unlimited recursion
md-fme query "sr.next_review <= today" --folder . --depth 0
```

Apply to all commands:
- `md-fme query <expr> --folder <path> [--depth <n>]`
- `md-fme today --folder <path> [--depth <n>]`
- `md-fme stats --folder <path> [--depth <n>]`

## Behavior

- `--depth 0` = unlimited (walk entire tree)
- `--depth 1` = current behavior (non-recursive, top-level only)
- `--depth 3` = default (most nested projects fit in 3 levels: `projects/project-name/subfolder/`)
- `--depth N` where N > 0 = recurse N levels deep

## Impact

- Enables natural SR organization in nested project folders
- Backwards compatible (default 3 levels covers most use cases)
- Simplifies CLI usage: no need to manually specify leaf folders
