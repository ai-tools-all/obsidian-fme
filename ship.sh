#!/usr/bin/env bash
set -euo pipefail

die() { echo "❌ $*" >&2; exit 1; }
info() { echo "→ $*"; }

BINARY=$(grep '^name' Cargo.toml | head -1 | cut -d'"' -f2)
CURRENT=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)

[[ $# -eq 1 ]] || { echo "Current: $BINARY v$CURRENT"; echo "Usage: ./ship.sh <new-version>  (e.g. ./ship.sh 0.9.2)"; exit 1; }

NEW=$1
[[ "$NEW" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "Version must be semver X.Y.Z (got: $NEW)"

info "Checking working tree..."
[[ -z "$(git status --porcelain)" ]] || die "Commit or stash changes before releasing"

info "$BINARY: $CURRENT → $NEW"

sed -i "0,/version = \"$CURRENT\"/s//version = \"$NEW\"/" Cargo.toml

info "Building release..."
cargo build --release

info "Generating changelog..."
git cliff --tag "v$NEW" -o CHANGELOG.md

info "Committing..."
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore(release): bump version to $NEW and generate changelog"

info "Tagging v$NEW..."
git tag "v$NEW"

info "Installing to ~/bin..."
rm -f ~/bin/"$BINARY" && cp target/release/"$BINARY" ~/bin/"$BINARY"

INSTALLED=$("$BINARY" --version 2>&1 || true)
info "Installed: $INSTALLED"

echo "$INSTALLED" | grep -q "$NEW" && info "Done." || die "Version mismatch after install!"
