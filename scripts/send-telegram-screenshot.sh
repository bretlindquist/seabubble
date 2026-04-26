#!/usr/bin/env bash
set -euo pipefail

TARGET="${1:-8519796167}"
CAPTION="${2:-Fresh screenshot}"
BASE_DIR="/Users/bretlindquist/.openclaw/workspace/tmp"
PNG_PATH="$BASE_DIR/latest-screenshot.png"
JPG_PATH="$BASE_DIR/latest-screenshot.jpg"

mkdir -p "$BASE_DIR"
/usr/sbin/screencapture -x "$PNG_PATH"
/opt/homebrew/bin/ffmpeg -y -i "$PNG_PATH" -q:v 3 "$JPG_PATH" >/dev/null 2>&1
openclaw message send \
  --channel telegram \
  --account default \
  --target "$TARGET" \
  --message "$CAPTION" \
  --media "$JPG_PATH" \
  --json
