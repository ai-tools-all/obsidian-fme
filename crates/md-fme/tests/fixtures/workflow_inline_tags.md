---
title: Reading + Blog Workflow
tags: [workflow, reading, writing, blog]
created: 2026-02-14
---

# Reading + Blog Workflow

Workflow for consuming content and producing blog posts.

**Primitive system:** See `primitive-system.md`

---

## Base Directory Context for writing blogposts

**Blog posts are created in:**
```
/home/abhishek/Downloads/experiments/rusty_learns/bytevault/blog/
```

All blogpost files go to this location by default. File naming: `{YYYY}-{title-slug}.md`

---

## Type: reading

### Schema

```yaml
---
type: reading
title: string
url: string                    # source URL or "local" for files
source: string                 # where discovered (twitter, newsletter, etc.)
topics: [string]               # thematic tags
status: queued | in-progress | read
priority: low | medium | high
added: YYYY-MM-DD
read_date: YYYY-MM-DD | null
goals: [string]                # which goals this contributes to
blog_post: string | null       # filename if turned into post
related: [string]              # other files in vault
---

# Notes

Key points from the reading.

# Insights

- What I learned
- What surprised me
- What I want to explore further

# Quotes (optional)

Notable excerpts worth revisiting.
```

### File Naming

`reading/{YYYY}-{title-slug}.md`

Examples:
- `reading/2026-table-formats-lance.md`
- `reading/2026-octostore-distributed-locking.md`

---

## Type: blogpost

### Schema

```yaml
---
type: blogpost
title: string
topics: [string]
status: draft (default) | published
draft: true (default)
created: YYYY-MM-DD
published: YYYY-MM-DD | null
platform: linkedin | blog | dev.to | x
url: string | null             # published URL
derived_from: [string]         # readings that inspired this
---

# Content

The actual post content.
```

### File Naming & Location

**Path:** `/home/abhishek/Downloads/experiments/rusty_learns/bytevault/blog/{YYYY}-{title-slug}.md`

**Examples:**
- `bytevault/blog/2026-greptime-high-cardinality.md`
- `bytevault/blog/2026-lance-vector-storage.md`

---

## Workflow Steps

### Step 1: Discover

User finds a link/article/codebase.

**Agent creates file:**
```yaml
---
type: reading
title: "Octostore - Distributed Locking"
url: https://github.com/...
source: twitter
topics: [distributed-systems, locking]
status: queued
priority: medium
added: 2026-02-14
read_date: null
goals: [G3-openTurboPuffer]
blog_post: null
related: []
---

# Notes

(Empty - to fill when reading)

# Insights

(Empty - to fill when reading)
```

---

### Step 2: Prioritize

User asks "what should I read?"

**Agent queries:**
1. `status: queued` + `priority: high`
2. Or matches current focus topics (from goals)
3. Or matches `goals: [active-goal]`

**Returns:** List of 3-5 suggested readings with reasons.

---

### Step 3: Read

User reads and shares notes.

**Agent updates:**
- `status: read`
- `read_date: YYYY-MM-DD`
- Fills `# Notes` and `# Insights` sections

---

### Step 4: Find Related

User asks "what else should I read on this topic?"

**Agent queries:**
1. `topics: [overlap]` in reading files
2. `research-links/*.toml` for passive links not yet in reading/
3. Suggests 2-3 related items

---

### Step 5: Blog

User asks "what can I write about?"

**Agent queries:**
1. `type: reading` + `status: read` + `blog_post: null`
2. Has content in `# Insights`
3. Returns candidates ranked by insight depth

**User writes post → Agent updates:**
- `blog_post: 2026-topic-name.md`
- Creates blogpost file in `bytevault/blog/` with:
  - `status: draft`
  - `draft: true`
  - `derived_from: [reading-file]`

---

## Query Reference

| Query | Command |
|-------|---------|
| What's queued? | `grep "status: queued" reading/*.md` |
| What did I read this week? | `grep "read_date: 2026-02-Wxx" reading/*.md` |
| Readings on vector-dbs? | `grep "topics:.*vector-db" reading/*.md` |
| Blog post candidates? | `grep -L "blog_post: " reading/*.md` + status: read |
| Everything for G3? | `grep "goals:.*G3" reading/*.md` |
| What to read next? | Query queued + priority high + match focus topics |

---

## Integration with research-links/

`research-links/*.toml` = passive links (discovered but not committed to reading)

**Flow:**
1. User discovers link → add to `research-links/Wxx.toml`
2. User decides to read → promote to `reading/*.md` with full schema
3. Agent can suggest from research-links when prioritizing

---

## Example Session

```
User: "I found this article on LanceDB: https://..."

Agent: Creates `reading/2026-lancedb-multi-base.md`
       status: queued, priority: medium, topics: [vector-dbs]

User: "What should I read next?"

Agent: Queries queued items
       Suggests: "LanceDB multi-base (matches G3-openTurboPuffer)"

User: "Read it, here are my notes: [notes]"

Agent: Updates file: status: read, read_date, fills notes/insights

User: "What can I write about?"

Agent: Finds read items with insights but no blog_post
       Suggests: "LanceDB multi-base - you have 3 strong insights"

User: "Wrote 2026-lancedb-multi-base.md about it"

Agent: Creates `bytevault/blog/2026-lancedb-multi-base.md`
       Sets: status: draft, draft: true, derived_from: [reading-file]
       Updates reading file: blog_post field
```

---

*Reading + Blog Workflow | 2026-02-14*
