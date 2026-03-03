## [0.9.3] - 2026-03-03

### 🐛 Bug Fixes

- *(sr)* Show relative path instead of filename in stats and today

### ⚙️ Miscellaneous Tasks

- Update Cargo.lock
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
