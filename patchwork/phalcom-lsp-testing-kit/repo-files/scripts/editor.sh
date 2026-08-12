#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

usage() {
  cat <<'USAGE'
Usage: scripts/editor.sh <lane>

Lanes:
  lsp         Run all phalcom-lsp unit + integration tests
  vsphalcom   Build phalcom-lsp, then run VS Code extension-host tests
  all         Run lsp then vsphalcom
  vsix        Build/package the VS Code extension as an installable VSIX
USAGE
}

lane="${1:-}"
case "$lane" in
  lsp)
    cargo test -p phalcom-lsp
    ;;
  vsphalcom)
    cargo build -p phalcom-lsp
    npm --prefix tools/vsphalcom test
    ;;
  all)
    cargo test -p phalcom-lsp
    cargo build -p phalcom-lsp
    npm --prefix tools/vsphalcom test
    ;;
  vsix)
    npm --prefix tools/vsphalcom run vsix
    ;;
  -h|--help|"")
    usage
    ;;
  *)
    echo "unknown editor test lane: $lane" >&2
    usage >&2
    exit 2
    ;;
esac
