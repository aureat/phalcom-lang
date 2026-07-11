#!/usr/bin/env bash
# Single entry point for the /forge verification gate.
#
# Runs the lanes every later forge phase gates on: build, full test suite
# (unit + phalcom-ast lexer/parser insta snapshots + phalcom-core golden .ph
# corpus + object-model invariants), and clippy across the workspace.
#
# Usage:
#   scripts/verify.sh            # default gate (build + test + clippy)
#   scripts/verify.sh --fuzz     # also run a short fuzz smoke pass (needs nightly + cargo-fuzz)
#   scripts/verify.sh --miri     # also run miri on phalcom-ast (needs nightly + miri component)
#
# Exit code is non-zero if any lane fails. Fuzz/miri are opt-in and do not
# run by default: they need a nightly toolchain with extra components that
# may not be installed, and are smoke checks, not part of the merge gate.

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

run_fuzz=0
run_miri=0
for arg in "$@"; do
  case "$arg" in
    --fuzz) run_fuzz=1 ;;
    --miri) run_miri=1 ;;
    *) echo "unknown flag: $arg" >&2; exit 2 ;;
  esac
done

step() { printf '\n==> %s\n' "$*"; }

step "cargo build --workspace"
cargo build --workspace

step "cargo test --workspace"
# Includes: phalcom-ast lexer/parser insta snapshot tests, phalcom-core
# golden .ph corpus (tests/golden.rs), phalcom-core object-model invariants
# (tests/invariants.rs), and every other unit/doc test in the workspace.
cargo test --workspace

step "cargo clippy --workspace --all-targets"
cargo clippy --workspace --all-targets

if [ "$run_fuzz" -eq 1 ]; then
  step "cargo +nightly fuzz run parser (60s smoke)"
  cargo +nightly fuzz run parser --fuzz-dir fuzz -- -dict=fuzz/phalcom.dict -max_total_time=60
  step "cargo +nightly fuzz run lexer (60s smoke)"
  cargo +nightly fuzz run lexer --fuzz-dir fuzz -- -dict=fuzz/phalcom.dict -max_total_time=60
fi

if [ "$run_miri" -eq 1 ]; then
  step "cargo +nightly miri test -p phalcom-ast"
  cargo +nightly miri test -p phalcom-ast
fi

step "verify: all lanes green"
