#!/usr/bin/env bash
set -euo pipefail

BINARY=$(grep '^name' Cargo.toml | head -1 | cut -d'"' -f2)
CURRENT=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)

if [ -z "${1:-}" ]; then
    echo "Current: $BINARY v$CURRENT"
    echo "Usage: ./ship.sh <new-version>"
    echo "Example: ./ship.sh 0.3.0"
    exit 1
fi

NEW=$1
echo "==> $BINARY: $CURRENT → $NEW"

sed -i "0,/version = \"$CURRENT\"/s//version = \"$NEW\"/" Cargo.toml

echo "==> Building release..."
cargo build --release

echo "==> Installing to ~/bin..."
rm -f ~/bin/"$BINARY" && cp target/release/"$BINARY" ~/bin/"$BINARY"

INSTALLED=$("$BINARY" --version 2>&1 || true)
echo "==> Installed: $INSTALLED"

if echo "$INSTALLED" | grep -q "$NEW"; then
    echo "==> Done."
else
    echo "==> WARNING: version mismatch!"
    exit 1
fi
