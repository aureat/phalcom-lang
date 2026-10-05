#!/usr/bin/env bash
# Focused test entry point for the consolidated Cargo targets.
#
# Usage:
#   scripts/test.sh ast [cargo test args...]
#   scripts/test.sh core [cargo test args...]
#   scripts/test.sh core-integration [cargo test args...]
#   scripts/test.sh lang [label] [-- cargo test args...]
#   scripts/test.sh invariants [cargo test args...]
#   scripts/test.sh lsp [cargo test args...]
#   scripts/test.sh repl [cargo test args...]
#   scripts/test.sh workspace
#   scripts/test.sh full

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

usage() {
  cat <<'USAGE'
Usage: scripts/test.sh <lane> [args...]

Focused lanes:
  ast               AST lexer/parser integration target
  core              all phalcom-core tests
  core-integration  phalcom-core integration target (--test core)
  lang [label]      language acceptance corpus (--test language-corpus), optionally one label
  invariants        object-model invariant target (--test core invariants)
  lsp               all LSP integration stages
  repl              REPL integration target

Gates:
  workspace         ./scripts/verify.sh
  full              ./scripts/verify.sh --full

Arguments after the lane are passed to Cargo. For test-harness arguments,
use `--`, for example: scripts/test.sh lang concurrency -- --nocapture
USAGE
}

lane="${1:-}"
if [[ -z "$lane" || "$lane" == "-h" || "$lane" == "--help" ]]; then
  usage
  [[ -n "$lane" ]] && exit 0
  exit 2
fi
shift

case "$lane" in
  ast)
    cargo test -p phalcom-ast --test integration "$@"
    ;;
  core)
    cargo test -p phalcom-core "$@"
    ;;
  core-integration)
    cargo test -p phalcom-core --test core "$@"
    ;;
  lang)
    label="${1:-}"; [[ -n "$label" ]] && shift
    if [[ -n "$label" ]]; then
      cargo test -p phalcom-core --test language-corpus "corpus::$label" "$@"
    else
      cargo test -p phalcom-core --test language-corpus "$@"
    fi
    ;;
  invariants)
    cargo test -p phalcom-core --test core invariants "$@"
    ;;
  lsp)
    cargo test -p phalcom-lsp "$@"
    ;;
  repl)
    cargo test -p phalcom-repl --test repl_phase_b "$@"
    ;;
  workspace)
    ./scripts/verify.sh
    ;;
  full)
    ./scripts/verify.sh --full
    ;;
  *)
    printf 'unknown test lane: %s\n\n' "$lane" >&2
    usage >&2
    exit 2
    ;;
esac
