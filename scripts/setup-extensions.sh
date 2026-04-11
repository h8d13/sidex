#!/usr/bin/env bash
# setup-extensions.sh — Populate the extensions/ directory with VSCode built-in extensions.
#
# Extensions are gitignored (large, sourced from upstream VSCode).
# Run this once after cloning before `npm run dev` or `npm run build`.
#
# Priority:
#   1. Copy from a local VSCode installation (fast, offline)
#   2. Download from VSCode GitHub release (fallback)

set -euo pipefail

VSCODE_VERSION="1.110.0"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXTENSIONS_DIR="$REPO_ROOT/extensions"

# Already populated — nothing to do.
if [[ -d "$EXTENSIONS_DIR" && "$(ls -A "$EXTENSIONS_DIR" 2>/dev/null | wc -l)" -gt 10 ]]; then
  echo "extensions/ already populated ($(ls "$EXTENSIONS_DIR" | wc -l | tr -d ' ') entries) — skipping."
  exit 0
fi

mkdir -p "$EXTENSIONS_DIR"

# ── Strategy 1: copy from local VSCode install ───────────────────────────────

VSCODE_CANDIDATES=(
  "/usr/share/code/resources/app/extensions"
  "/usr/lib/code/extensions"
  "/opt/visual-studio-code/resources/app/extensions"
  "$HOME/.vscode/extensions"
  "/Applications/Visual Studio Code.app/Contents/Resources/app/extensions"
  "/Applications/Cursor.app/Contents/Resources/app/extensions"
)

for candidate in "${VSCODE_CANDIDATES[@]}"; do
  if [[ -d "$candidate" && "$(ls -A "$candidate" 2>/dev/null | wc -l)" -gt 10 ]]; then
    echo "Found VSCode extensions at: $candidate"
    echo "Copying built-in extensions..."
    # Only copy language/grammar/theme extensions (skip user-installed ones)
    cp -r "$candidate"/. "$EXTENSIONS_DIR/"
    echo "Copied $(ls "$EXTENSIONS_DIR" | wc -l | tr -d ' ') extensions."
    exit 0
  fi
done

# ── Strategy 2: download from GitHub release ─────────────────────────────────

echo "No local VSCode installation found. Downloading from GitHub release..."

TARBALL_URL="https://github.com/microsoft/vscode/archive/refs/tags/${VSCODE_VERSION}.tar.gz"
TMP_DIR="$(mktemp -d)"
TMP_TAR="$TMP_DIR/vscode.tar.gz"

cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

echo "Downloading VSCode ${VSCODE_VERSION} source (this may take a while)..."
curl -L --progress-bar "$TARBALL_URL" -o "$TMP_TAR"

echo "Extracting extensions..."
# The tar contains vscode-1.110.0/extensions/
tar -xzf "$TMP_TAR" -C "$TMP_DIR" --wildcards "vscode-${VSCODE_VERSION}/extensions/*"

SRC="$TMP_DIR/vscode-${VSCODE_VERSION}/extensions"
if [[ ! -d "$SRC" ]]; then
  echo "ERROR: extensions not found in archive at $SRC"
  exit 1
fi

cp -r "$SRC"/. "$EXTENSIONS_DIR/"
echo "Done. $(ls "$EXTENSIONS_DIR" | wc -l | tr -d ' ') extensions installed."
echo ""
echo "Now run: node scripts/generate-extension-meta.js && npm run build"
