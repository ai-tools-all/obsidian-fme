#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/helper.sh"

info "Building release binary for $BINARY v$VERSION..."
cargo build --release --manifest-path "$REPO_ROOT/Cargo.toml"

info "Installing to ~/bin..."
rm -f ~/bin/"$BINARY"
cp "$REPO_ROOT/target/release/$BINARY" ~/bin/"$BINARY"

INSTALLED=$("$BINARY" --version 2>&1 || true)
info "Installed: $INSTALLED"

echo "$INSTALLED" | grep -q "$VERSION" && info "Done." || die "Version mismatch — expected $VERSION, got: $INSTALLED"
