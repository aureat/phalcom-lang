#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 2 || $# -gt 3 ]]; then
  echo "usage: $0 <workload> <off|summary|detailed|trace> [samples]" >&2
  exit 2
fi

workload="$1"
level="$2"
samples="${3:-1}"
if [[ ! "$samples" =~ ^[1-9][0-9]*$ ]]; then
  echo "samples must be a positive integer" >&2
  exit 2
fi
if [[ ! "$level" =~ ^(off|summary|detailed|trace)$ ]]; then
  echo "unknown observability level: $level" >&2
  exit 2
fi
if [[ -z "${PHALCOM_PERF_BASELINE_HEAD:-}" ]]; then
  echo "set PHALCOM_PERF_BASELINE_HEAD to the exact committed baseline SHA" >&2
  exit 2
fi

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$repo_root"
execution_tree_head="$(git rev-parse HEAD)"
branch="$(git branch --show-current)"
if [[ -n "$(git status --porcelain)" ]]; then
  dirty=true
else
  dirty=false
fi
toolchain="$(rustup show active-toolchain | awk '{print $1}')"
export RUSTUP_TOOLCHAIN="$toolchain"

cargo build --release -p phalcom-core --example perf_observability_profile --features perf-observability
target_root="${CARGO_TARGET_DIR:-$repo_root/target}"
profile_bin="$target_root/release/examples/perf_observability_profile"
if [[ ! -x "$profile_bin" ]]; then
  echo "profile executable not found at $profile_bin" >&2
  exit 2
fi

run_stamp="$(date -u +%Y%m%dT%H%M%SZ)-$$"
output_dir="$target_root/perf/profiles/${workload}-${level}-${run_stamp}"
mkdir -p "$output_dir"

for ((index = 1; index <= samples; index += 1)); do
  report="$output_dir/sample-${index}.json"
  if PHALCOM_PERF_BASELINE_HEAD="$PHALCOM_PERF_BASELINE_HEAD" \
    PHALCOM_PERF_EXECUTION_TREE_HEAD="$execution_tree_head" \
    PHALCOM_PERF_SOURCE_REVISION="$execution_tree_head" \
    PHALCOM_PERF_BRANCH="$branch" \
    PHALCOM_PERF_DIRTY="$dirty" \
    "$profile_bin" "$workload" "$level" >"$report" 2>/dev/null; then
    exit_code=0
  else
    exit_code=$?
  fi
  printf '%s\n' "$exit_code" >"$output_dir/sample-${index}.exit-code"
done

printf '%s\n' "$output_dir"
