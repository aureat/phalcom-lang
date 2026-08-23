#!/usr/bin/env bash
# Single entry point for the /forge verification gate.
#
# The default gate favors fast local feedback: it runs the full workspace test
# suite and Clippy across every target. Use --full when the ordinary non-test
# workspace build must also be verified, such as before a merge or release.
#
# Usage:
#   scripts/verify.sh            # fast gate: test + clippy
#   scripts/verify.sh --full     # full gate: build + test + clippy
#   scripts/verify.sh --fuzz     # also run short parser/lexer fuzz smoke passes
#   scripts/verify.sh --miri     # also run Miri on phalcom-ast
#
# Flags may be combined. Fuzz and Miri require a nightly toolchain and their
# respective components/tools. Any failed lane makes the script exit non-zero.

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

run_build=0
run_fuzz=0
run_miri=0

for arg in "$@"; do
  case "$arg" in
    --full) run_build=1 ;;
    --fuzz) run_fuzz=1 ;;
    --miri) run_miri=1 ;;
    -h|--help)
      cat <<'USAGE'
Usage: scripts/verify.sh [--full] [--fuzz] [--miri]

  --full  Also verify the ordinary non-test workspace build.
  --fuzz  Run 60-second parser and lexer fuzz smoke passes.
  --miri  Run Miri tests for phalcom-ast.
USAGE
      exit 0
      ;;
    *)
      printf 'unknown flag: %s\n' "$arg" >&2
      exit 2
      ;;
  esac
done

step() { printf '\n==> %s\n' "$*"; }

step "cargo run -p phalcom-native-surface-gen -- --root . --check"
cargo run -p phalcom-native-surface-gen -- --root . --check

step "canonical native sentinel drift check"
if rg -n "MemberAstRef::INVALID|native.*usize::MAX|usize::MAX.*native" phalcom-lsp/src phalcom-native-surface/src; then
  printf 'canonical native sentinel remains in source\n' >&2
  exit 1
fi

if (( run_build )); then
  step "cargo build --workspace"
  cargo build --workspace
fi

step "cargo nextest run --workspace --no-fail-fast --test-threads=2"
# LSP integration cases each start a semantic worker that bootstraps the full
# core surface. Unbounded binary concurrency starves those workers and turns
# their 30-second readiness assertions into host-load flakes. Two test
# processes retain parallel coverage without oversubscribing the gate.
# Doctests are not supported by nextest, so retain Cargo's dedicated doc lane.
cargo nextest run --workspace --no-fail-fast --test-threads=2

step "cargo test --workspace --doc"
cargo test --workspace --doc

step "cargo clippy --workspace --all-targets"
cargo clippy --workspace --all-targets

if (( run_fuzz )); then
  step "cargo +nightly fuzz run parser (60s smoke)"
  cargo +nightly fuzz run parser \
    --fuzz-dir fuzz \
    -- \
    -dict=fuzz/phalcom.dict \
    -max_total_time=60

  step "cargo +nightly fuzz run lexer (60s smoke)"
  cargo +nightly fuzz run lexer \
    --fuzz-dir fuzz \
    -- \
    -dict=fuzz/phalcom.dict \
    -max_total_time=60
fi

if (( run_miri )); then
  step "cargo +nightly miri test -p phalcom-ast"
  cargo +nightly miri test -p phalcom-ast
fi

step "verify: all requested lanes green"
