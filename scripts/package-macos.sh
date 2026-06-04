#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

export PATH="$HOME/.cargo/bin:$PATH"

npm run tauri -- build

APP_DIR="src-tauri/target/release/bundle/macos"
DMG_DIR="src-tauri/target/release/bundle/dmg"
APP_PATH="$(find "$APP_DIR" -maxdepth 1 -name "*.app" -print -quit)"
DMG_PATH="$(find "$DMG_DIR" -maxdepth 1 -name "*.dmg" -print -quit)"

if [[ -z "$APP_PATH" ]]; then
  echo "APP_NOT_FOUND: $APP_DIR" >&2
  exit 1
fi

if [[ -z "$DMG_PATH" ]]; then
  echo "DMG_NOT_FOUND: $DMG_DIR" >&2
  exit 1
fi

codesign --force --deep --sign - --entitlements src-tauri/entitlements.plist "$APP_PATH"
codesign --verify --deep --strict --verbose=2 "$APP_PATH"
hdiutil verify "$DMG_PATH"

INSTALL_PATH="/Applications/BatchRename Pro.app"
rm -rf "$INSTALL_PATH"
ditto "$APP_PATH" "$INSTALL_PATH"

SHA256="$(shasum -a 256 "$DMG_PATH" | awk '{print $1}')"

echo "APP_PATH=$APP_PATH"
echo "INSTALLED_APP_PATH=$INSTALL_PATH"
echo "DMG_PATH=$DMG_PATH"
echo "SHA256=$SHA256"
