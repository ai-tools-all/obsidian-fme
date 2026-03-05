#!/usr/bin/env bash
# Common helpers — source this from other scripts: source "$(dirname "$0")/helper.sh"

die() { echo "❌ $*" >&2; exit 1; }
info() { echo "→ $*"; }

# Read from crate Cargo.toml (workspace layout: crates/md-fme/)
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CRATE_TOML="$REPO_ROOT/crates/md-fme/Cargo.toml"
BINARY=$(grep '^name' "$CRATE_TOML" | head -1 | cut -d'"' -f2)
VERSION=$(grep '^version' "$CRATE_TOML" | head -1 | cut -d'"' -f2)

require_clean_tree() {
    [[ -z "$(git -C "$REPO_ROOT" status --porcelain)" ]] || die "Working tree is dirty — commit or stash changes first"
}

require_tag() {
    local tag="v$VERSION"
    git -C "$REPO_ROOT" rev-parse "$tag" &>/dev/null || die "Tag $tag not found — run ./scripts/release.sh first"
}

require_tracked_and_clean() {
    local f
    for f in "$@"; do
        git -C "$REPO_ROOT" ls-files --error-unmatch "$f" &>/dev/null || die "$f is not tracked by git"
        [[ -z "$(git -C "$REPO_ROOT" status --porcelain "$f")" ]] || die "$f has uncommitted changes"
    done
}
