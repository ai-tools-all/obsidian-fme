# Plan: YAML → TOML Frontmatter Migration

## Goal
Read both formats, always write TOML. Any update to a YAML file converts it to TOML (`+++`).

## Steps

### 1. Dual-parse in `frontmatter.rs`
- Add `parse_toml()` — extract content between `+++` delimiters, parse as `toml::Value`
- Modify `parse()` → try TOML first (`+++`), fall back to YAML (`---`)
- Unify into common `toml::Value` internally (YAML values get converted on parse)
- `Document.frontmatter` type changes from `serde_yaml::Value` → `toml::Value`

### 2. TOML writer
- `fn serialize_toml(frontmatter: &toml::Value) -> String` — renders `+++\n...\n+++`
- `fn replace_frontmatter(raw: &str, new_fm: &toml::Value) -> String` — swaps old block (either `---` or `+++`) with TOML output, preserves body
- Update `insert_before_closing()` and `update_sr_field()` to work with TOML format

### 3. Adapt consumers
- `enforce.rs` — already reads TOML schemas; update field access from `serde_yaml::Value` → `toml::Value`
- `query.rs` — update comparisons/accessors for `toml::Value` (types differ slightly)
- `sr.rs` — update `init-sr` and `review` to write TOML; SR block becomes TOML table `[sr]`
- `display.rs` — update `value_to_string` for `toml::Value`

### 4. Tests
- `tests/fixtures/` has 7 real files (6 YAML, 1 TOML)
- Unit tests: parse YAML → get `toml::Value`, parse TOML → get `toml::Value`, round-trip
- Integration tests: read YAML fixture → update field → verify output is `+++` TOML
- Edge cases: empty tags, datetime values, nested SR block, inline vs multiline arrays

### 5. Cleanup (later)
- Once vault is fully migrated, remove YAML fallback path
- Remove `serde_yaml` dependency

## File Impact
| File | Changes |
|------|---------|
| `Cargo.toml` | May need `toml` features for datetime |
| `frontmatter.rs` | Major — dual parse, TOML writer, type change |
| `enforce.rs` | Medium — `serde_yaml::Value` → `toml::Value` |
| `query.rs` | Medium — value access/comparison changes |
| `sr.rs` | Medium — write path produces TOML |
| `display.rs` | Small — `value_to_string` for `toml::Value` |
| `main.rs` | Small — no structural changes |
