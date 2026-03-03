#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/helper.sh"

info "Sanity checks for $BINARY v$VERSION..."

require_clean_tree
require_tracked_and_clean Cargo.toml Cargo.lock CHANGELOG.md
require_tag

info "All checks passed. Publishing $BINARY v$VERSION to crates.io..."
cargo publish --manifest-path "$REPO_ROOT/Cargo.toml"

info "Done. $BINARY v$VERSION published."
