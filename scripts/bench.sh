#!/usr/bin/env bash
# Benchmark entry point for Phalcom's whole-process and micro-benchmark lanes.
#
# Usage:
#   scripts/bench.sh vm [--skip-bench]
#   scripts/bench.sh criterion [criterion filter]
#   scripts/bench.sh perf [phalcom-perf args...]
#   scripts/bench.sh wren [benchmark names...]
#   scripts/bench.sh math [--strict] [files...]
#   scripts/bench.sh one <path-to-ph-file>

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

usage() {
  cat <<'USAGE'
Usage: scripts/bench.sh <lane> [args...]

  vm         release VM baseline: micro-programs, bootstrap, Skynet, Criterion
  criterion  Criterion VM micro-benches only
  perf       combined corpus + benchmarks timing report (subcommands: run, ab, compare, show, list, layout, baseline)
  wren       output-verified Phalcom-vs-Wren timings
  math       math benchmark self-checks
  one PATH   run one .ph benchmark with the release CLI

Examples:
  scripts/bench.sh vm --skip-bench
  scripts/bench.sh criterion bare_send
  scripts/bench.sh perf run --suite representation
  scripts/bench.sh perf layout
  scripts/bench.sh perf list
  scripts/bench.sh wren fib map_string
  scripts/bench.sh one benchmarks/wren-suite/fib.ph
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
  vm)
    ./benchmarks/vm/run.sh "$@"
    ;;
  criterion)
    cargo bench -p phalcom-core --features benchmarks --bench vm_bench -- "$@"
    ;;
  perf)
    target_dir="${CARGO_TARGET_DIR:-$(cargo metadata --format-version 1 --no-deps 2>/dev/null | grep -o '"target_directory":"[^"]*"' | cut -d'"' -f4 || echo target)}"
    cargo build --release -q -p phalcom-core --bin phalcom --bin phalcom-perf
    if [[ "$#" -eq 0 || "${1:-}" == -* ]]; then
      exec "$target_dir/release/phalcom-perf" run "$@"
    else
      exec "$target_dir/release/phalcom-perf" "$@"
    fi
    ;;
  wren)
    cargo build --release -q -p phalcom-core --bin phalcom
    python3 benchmarks/vm/compare-wren.py "$@"
    ;;
  math)
    ./benchmarks/math/run.sh "$@"
    ;;
  one)
    path="${1:-}"
    if [[ -z "$path" || "$#" -ne 1 ]]; then
      printf 'one requires exactly one .ph path\n' >&2
      usage >&2
      exit 2
    fi
    target_dir="${CARGO_TARGET_DIR:-target}"
    cargo build --release -q -p phalcom-core --bin phalcom
    exec "$target_dir/release/phalcom" "$path"
    ;;
  *)
    printf 'unknown benchmark lane: %s\n\n' "$lane" >&2
    usage >&2
    exit 2
    ;;
esac
