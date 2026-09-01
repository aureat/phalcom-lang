#!/usr/bin/env bash
# Compare Cargo build-job counts using cold target directories.
#
# sccache is bypassed deliberately. Otherwise the first run populates the
# compiler cache and later runs measure cache hits instead of compilation.
# Rustc flags come from the workspace .cargo/config.toml; only Cargo jobs vary.
#
# Usage:
#   scripts/bench-build-jobs.sh             # test 1, 2, 4, 6, 8 jobs
#   scripts/bench-build-jobs.sh 2 4 8      # test selected job counts
#
# Each run keeps its Cargo timing HTML and terminal log under target/.

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

if (($# > 0)); then
  job_counts=("$@")
else
  job_counts=(1 2 4 6 8)
fi

bench_root="${PHALCOM_BENCH_ROOT:-$PWD/target/bench-build-jobs-$(date +%Y%m%d-%H%M%S)}"
mkdir -p "$bench_root"

printf 'Cold build benchmark: Cargo jobs=%s\n' "${job_counts[*]}"
printf 'Results root: %s\n' "$bench_root"
printf 'sccache: bypassed\n\n'

for jobs in "${job_counts[@]}"; do
  run_dir="$(mktemp -d "$bench_root/jobs-${jobs}.XXXXXX")"
  printf '=== jobs=%s ===\n' "$jobs"
  printf 'Target and report directory: %s\n' "$run_dir"

  {
    /usr/bin/time -p env \
      RUSTC_WRAPPER= \
      CARGO_BUILD_RUSTC_WRAPPER= \
      CARGO_TARGET_DIR="$run_dir" \
      cargo test --workspace --no-run --locked --timings -j "$jobs"
  } 2>&1 | tee "$run_dir/timing.log"

  printf 'Cargo timing report: %s/cargo-timings/cargo-timing.html\n\n' "$run_dir"
done
