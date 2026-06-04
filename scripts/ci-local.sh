#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

export PATH="$HOME/.cargo/bin:$PATH"

verify() {
  npm run typecheck
  npm run build
  (cd src-tauri && cargo test)
}

case "${1:-verify}" in
  verify)
    verify
    ;;
  package)
    verify
    scripts/package-macos.sh
    ;;
  *)
    echo "Usage: scripts/ci-local.sh [verify|package]" >&2
    exit 2
    ;;
esac
