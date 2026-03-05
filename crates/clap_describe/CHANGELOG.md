## [unreleased]

### 🚀 Features

- *(clap_describe)* Add CommandSchema data model with serde
- *(clap_describe)* Add extract_schema command introspection
- *(clap_describe)* Add to_markdown() renderer for CommandSchema
- *(clap_describe)* Add Describe Args struct with handle() helper
- *(clap_describe)* Add Deserialize to model structs for round-trip support

### 🚜 Refactor

- Convert to cargo workspace with crates/md-fme and crates/clap_describe

### 🧪 Testing

- *(clap_describe)* Add shared test fixtures for integration tests
- *(clap_describe)* Add JSON snapshot tests for all fixture commands
- *(clap_describe)* Add Markdown snapshot tests for all fixture commands
- *(clap_describe)* Add edge case tests for empty, hidden, nested commands
- *(clap_describe)* Add JSON round-trip tests verifying serialize/deserialize symmetry

### ⚙️ Miscellaneous Tasks

- *(clap_describe)* Bump version to 0.1.1 and add README
