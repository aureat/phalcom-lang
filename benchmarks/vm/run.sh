#!/usr/bin/env bash
#
# run.sh — one-command reproduction of the U-BENCH Tier 0 baseline
# (docs/spec/v0.2/performance.md §4 Tier 0, docs/forge/units/U-BENCH/plan.md).
#
# Runs, in order:
#   1. The three isolating micro-programs (benchmarks/vm/*.ph) as whole
#      processes, confirming each still executes cleanly (the gate: "a
#      benchmark that errors is not a baseline").
#   2. Skynet (benchmarks/concurrency/skynet.ph) under `/usr/bin/time -l`,
#      whole-process wall-clock + peak RSS — the number BASELINE.md
#      records and every later tier re-measures against (law P1).
#   3. The same for Skynet's Wren reference (benchmarks/concurrency/skynet.wren),
#      if a `wren_test` binary is available (DEC-BENCH-B: never invent a
#      number — skip with a note if not).
#   4. `cargo bench -p phalcom-core --bench vm_bench`, the criterion
#      micro-benches (send / arith / fiber), for regression tripwires.
#
# This does NOT regenerate BASELINE.md's attribution profile (that used
# macOS `sample`/`dtrace`, a one-off interactive capture, not scripted here
# since its output path and PID timing are inherently non-reproducible
# byte-for-byte) — it reproduces every *number*, which is what P1 requires.
#
# Usage:
#   benchmarks/vm/run.sh              # everything
#   benchmarks/vm/run.sh --skip-bench # skip the criterion pass (faster)
#   WREN_TEST=/path/to/wren_test benchmarks/vm/run.sh   # override Wren binary
#
set -u

here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
cd "$repo" || exit 2

skip_bench=0
for arg in "$@"; do
  case "$arg" in
    --skip-bench) skip_bench=1 ;;
  esac
done

echo "==> building phalcom CLI (release)"
if ! cargo build -rq -p phalcom-core --bin phalcom; then
  echo "BUILD FAILED — cannot run benchmarks." >&2
  exit 2
fi
bin="$repo/target/release/phalcom"

rc=0

echo
echo "==> micro-programs (execution gate)"
for f in "$here"/*.ph; do
  name="$(basename "$f")"
  if out="$("$bin" "$f" 2>&1)"; then
    echo "PASS     $name  ($out)"
  else
    echo "FAIL     $name"
    printf '%s\n' "$out" | sed 's/^/           | /'
    rc=1
  fi
done

echo
echo "==> Skynet (Phalcom, whole-process, wall-clock + peak RSS)"
/usr/bin/time -l "$bin" "$repo/benchmarks/concurrency/skynet.ph"

wren_bin="${WREN_TEST:-$HOME/dev/repos/wren/bin/wren_test}"
echo
if [ -x "$wren_bin" ]; then
  echo "==> Skynet (Wren reference, whole-process, wall-clock + peak RSS)"
  /usr/bin/time -l "$wren_bin" "$repo/benchmarks/concurrency/skynet.wren"
else
  echo "==> Skynet (Wren reference): SKIPPED — no wren_test binary at"
  echo "    \"$wren_bin\". Set WREN_TEST=/path/to/wren_test to compare."
  echo "    (DEC-BENCH-B: never invent a number — see BASELINE.md provenance note.)"
fi

if [ "$skip_bench" -eq 0 ]; then
  echo
  echo "==> criterion micro-benches (send / arith / fiber)"
  cargo bench -p phalcom-core --bench vm_bench
fi

exit "$rc"
