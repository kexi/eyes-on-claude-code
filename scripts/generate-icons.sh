#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ICONS_DIR="$PROJECT_ROOT/src-tauri/icons"
SOURCE_ICON="$ICONS_DIR/icon.png"

if [ ! -f "$SOURCE_ICON" ]; then
    echo "Error: Source icon not found at $SOURCE_ICON"
    exit 1
fi

echo "Generating icons from $SOURCE_ICON..."

cd "$PROJECT_ROOT"

# Use Tauri CLI to generate all icon formats
pnpm tauri icon "$SOURCE_ICON"

echo "Icons generated successfully in $ICONS_DIR"
ls -la "$ICONS_DIR"
