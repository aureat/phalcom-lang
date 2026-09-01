#!/usr/bin/env bash
# Compare rustc internal thread-pool sizes using cold target directories.
#
# Cargo jobs stay fixed at one so this isolates -Zthreads. The workspace is on
# nightly because -Zthreads is unstable. sccache is bypassed deliberately.
#
# Usage:
#   scripts/bench-rustc-threads.sh             # test 1, 2, 4, 6 threads
#   scripts/bench-rustc-threads.sh 2 4 6      # test selected values
#
# Each run keeps its Cargo timing HTML and terminal log under target/.

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

if (($# > 0)); then
  thread_counts=("$@")
else
  thread_counts=(1 2 4 6)
fi

cargo_jobs="${PHALCOM_CARGO_JOBS:-1}"
bench_root="${PHALCOM_BENCH_ROOT:-$PWD/target/bench-rustc-threads-$(date +%Y%m%d-%H%M%S)}"
mkdir -p "$bench_root"

printf 'Cold build benchmark: rustc threads=%s, Cargo jobs=%s\n' \
  "${thread_counts[*]}" "$cargo_jobs"
printf 'Results root: %s\n' "$bench_root"
printf 'sccache: bypassed\n\n'

for threads in "${thread_counts[@]}"; do
  run_dir="$(mktemp -d "$bench_root/threads-${threads}.XXXXXX")"
  printf '=== rustc threads=%s, Cargo jobs=%s ===\n' "$threads" "$cargo_jobs"
  printf 'Target and report directory: %s\n' "$run_dir"

  {
    /usr/bin/time -p env \
      RUSTC_WRAPPER= \
      CARGO_BUILD_RUSTC_WRAPPER= \
      CARGO_TARGET_DIR="$run_dir" \
      RUSTFLAGS="-Zunstable-options -Zthreads=${threads} -Ctarget-cpu=native" \
      cargo test --workspace --no-run --locked --timings -j "$cargo_jobs"
  } 2>&1 | tee "$run_dir/timing.log"

  printf 'Cargo timing report: %s/cargo-timings/cargo-timing.html\n\n' "$run_dir"
done
