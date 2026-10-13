#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 <workload> [samples-per-level]" >&2
  exit 2
fi

workload="$1"
samples="${2:-5}"
script_dir="$(cd "$(dirname "$0")" && pwd)"
for level in off summary detailed trace; do
  "$script_dir/run-profile.sh" "$workload" "$level" "$samples"
done
