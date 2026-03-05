## [0.9.6] - 2026-03-05

### 🚀 Features

- *(clap_describe)* Add CommandSchema data model with serde
- *(clap_describe)* Add extract_schema command introspection
- *(clap_describe)* Add to_markdown() renderer for CommandSchema
- *(clap_describe)* Add Describe Args struct with handle() helper
- Integrate clap_describe --describe into all subcommands

### 🐛 Bug Fixes

- *(scripts)* Update Cargo.lock before committing in release.sh; guard dirty tree in local_ship.sh

### 🚜 Refactor

- Convert to cargo workspace with crates/md-fme and crates/clap_describe

### 🧪 Testing

- Add integration tests for --describe on all subcommands

### ⚙️ Miscellaneous Tasks

- *(beads)* Add clap_describe epic issues and research docs
## [0.9.5] - 2026-03-04

### 🚀 Features

- *(enforce)* Add --file flag to validate a single markdown file

### 🐛 Bug Fixes

- *(scripts)* Require cliff.toml and README.md tracked and clean before release

### ⚙️ Miscellaneous Tasks

- Update Cargo.lock
- *(release)* Bump version to 0.9.5 and generate changelog
## [0.9.4] - 2026-03-03

### 🚀 Features

- Add render layer with tree view and --json global flag

### ⚙️ Miscellaneous Tasks

- *(release)* Bump version to 0.9.4 and generate changelog
## [0.9.3] - 2026-03-03

### 🐛 Bug Fixes

- *(sr)* Show relative path instead of filename in stats and today

### ⚙️ Miscellaneous Tasks

- Update Cargo.lock
- *(release)* Bump version to 0.9.3 and generate changelog
## [0.9.2] - 2026-03-03

### 🚀 Features

- Frontmatter engine with TOML-first read/write
- Add --fix and --exclude flags to enforce command
- Hint on --fix when allowed_values has no default; add query DSL tests
- Add --depth parameter for recursive directory scanning
- Integrate git-cliff for automated changelog generation

### 🐛 Bug Fixes

- Warn on stderr when skipping unreadable files
- Remove duplicate subcommands list from long_about help text
- Unify -h and help output by dropping long_about
- Use rm -f before cp in ship.sh to avoid stale binary
- *(query)* Suppress warnings by default, show only with --verbose
- Commit Cargo.lock and changelog in ship.sh before release
- *(sr)* Suppress verbose 'No frontmatter' warnings in stats and today

### 📚 Documentation

- Add README, test fixtures, and ship script
- Expand enforce docs and add complete schema.toml reference

### ⚡ Performance

- *(release)* Disable overflow checks to reduce binary size

### 🧪 Testing

- *(sr)* Add SR integration tests for today, review, stats, and query

### ⚙️ Miscellaneous Tasks

- Update README, bump to 0.6.0, init beads tracker
- Add MIT license
- Ignore /docs/ directory
- Bump version to 0.8.0
- Remove plan
- Rename crate to md-fme for crates.io publish
- Update all fme references to md-fme in src, tests, and README
- Bump version to 0.8.1
- Bump version to 0.8.2
- Bump version to 0.9.0
- Bump version to 0.9.1
- Updated cargo.lock
- Restructure release scripts into scripts/ with shared helper
- *(release)* Bump version to 0.9.2 and generate changelog
