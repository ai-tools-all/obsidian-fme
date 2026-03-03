#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/helper.sh"

[[ $# -eq 1 ]] || { echo "Current: $BINARY v$VERSION"; echo "Usage: ./scripts/release.sh <new-version>  (e.g. ./scripts/release.sh 0.9.2)"; exit 1; }

NEW=$1
[[ "$NEW" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "Version must be semver X.Y.Z (got: $NEW)"
[[ -f "$REPO_ROOT/cliff.toml" ]] || die "cliff.toml not found"

info "Checking working tree..."
require_clean_tree
require_tracked_and_clean Cargo.toml Cargo.lock CHANGELOG.md cliff.toml README.md

info "$BINARY: $VERSION → $NEW"

sed -i "0,/version = \"$VERSION\"/s//version = \"$NEW\"/" "$REPO_ROOT/Cargo.toml"

info "Generating changelog..."
git -C "$REPO_ROOT" cliff --tag "v$NEW" -o "$REPO_ROOT/CHANGELOG.md"

info "Committing..."
git -C "$REPO_ROOT" add Cargo.toml Cargo.lock CHANGELOG.md
git -C "$REPO_ROOT" commit -m "chore(release): bump version to $NEW and generate changelog"

info "Tagging v$NEW..."
git -C "$REPO_ROOT" tag "v$NEW"

info "Done. Run ./scripts/local_ship.sh to install, or ./scripts/publish.sh to publish to crates.io"
