#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/helper.sh"

info "Sanity checks for $BINARY v$VERSION..."

require_clean_tree
require_tracked_and_clean Cargo.toml crates/md-fme/Cargo.toml Cargo.lock CHANGELOG.md cliff.toml README.md
require_tag

info "All checks passed. Publishing $BINARY v$VERSION to crates.io..."
cargo publish --manifest-path "$CRATE_TOML"

info "Done. $BINARY v$VERSION published."
