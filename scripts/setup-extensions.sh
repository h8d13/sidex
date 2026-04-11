#!/usr/bin/env bash
# Populate extensions/ from upstream VSCode. Run once after cloning.

set -euo pipefail

VSCODE_VERSION="1.110.0"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXTENSIONS_DIR="$REPO_ROOT/extensions"

if [[ -d "$EXTENSIONS_DIR" && "$(ls -A "$EXTENSIONS_DIR" 2>/dev/null | wc -l)" -gt 10 ]]; then
  echo "extensions/ already populated — skipping."
  exit 0
fi

mkdir -p "$EXTENSIONS_DIR"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

echo "Downloading VSCode ${VSCODE_VERSION}..."
curl -L --progress-bar \
  "https://github.com/microsoft/vscode/archive/refs/tags/${VSCODE_VERSION}.tar.gz" \
  -o "$TMP_DIR/vscode.tar.gz"

tar -xzf "$TMP_DIR/vscode.tar.gz" -C "$TMP_DIR" --wildcards "vscode-${VSCODE_VERSION}/extensions/*"
cp -r "$TMP_DIR/vscode-${VSCODE_VERSION}/extensions/." "$EXTENSIONS_DIR/"
echo "Done — $(ls "$EXTENSIONS_DIR" | wc -l | tr -d ' ') extensions installed."
