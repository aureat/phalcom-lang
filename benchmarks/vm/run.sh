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
#   4. `cargo bench -p phalcom-core --features benchmarks --bench vm_bench`,
#      the Criterion micro-benches (send / arith / fiber), for regression
#      tripwires.
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
echo "==> bootstrap tripwire (whole-process ceiling ${BOOTSTRAP_CEILING_MS:-20} ms)"
# H7 / F13. Bootstrap regressed 5ms -> 180ms (35x) and passed every gate this
# harness had: the loop above only asks "did it run", the criterion benches
# amortize bootstrap inside a ~0.9s program, and the wren-suite table is
# single-run. A ceiling on `VM::new` is the one gate that would have caught it.
# Best-of-3: the check must fail on a 35x regression, not on a scheduling blip.
if bootstrap_ms="$(BIN="$bin" PH="$here/bootstrap.ph" python3 -c '
import os, subprocess, time, sys
best = min(
    (lambda t0: (subprocess.run([os.environ["BIN"], os.environ["PH"]],
                                capture_output=True), time.perf_counter() - t0)[1])(time.perf_counter())
    for _ in range(3)
)
print(f"{best * 1000:.1f}")
' 2>/dev/null)"; then
  ceiling="${BOOTSTRAP_CEILING_MS:-20}"
  if awk "BEGIN{exit !($bootstrap_ms > $ceiling)}"; then
    echo "FAIL     bootstrap.ph  ${bootstrap_ms} ms  (ceiling ${ceiling} ms)"
    echo "           | Bootstrap is VM::new re-compiling core.ph. A blowup here is a"
    echo "           | COMPILER regression, not a VM one — profile the compiler first."
    echo "           | perf-log findings F13; raise with BOOTSTRAP_CEILING_MS= if core.ph grew."
    rc=1
  else
    echo "PASS     bootstrap.ph  ${bootstrap_ms} ms  (ceiling ${ceiling} ms)"
  fi
else
  echo "SKIP     bootstrap.ph — python3 unavailable for timing"
fi

echo
echo "==> Skynet (Phalcom, whole-process, wall-clock + peak RSS)"
if /usr/bin/time -l "$bin" "$repo/benchmarks/concurrency/skynet.ph"; then
  echo "PASS     skynet.ph"
else
  echo "FAIL     skynet.ph" >&2
  rc=1
fi

wren_bin="${WREN_TEST:-$HOME/dev/repos/wren/bin/wren_test}"
echo
if [ -x "$wren_bin" ]; then
  echo "==> Skynet (Wren reference, whole-process, wall-clock + peak RSS)"
  if /usr/bin/time -l "$wren_bin" "$repo/benchmarks/concurrency/skynet.wren"; then
    echo "PASS     skynet.wren"
  else
    echo "FAIL     skynet.wren" >&2
    rc=1
  fi
else
  echo "==> Skynet (Wren reference): SKIPPED — no wren_test binary at"
  echo "    \"$wren_bin\". Set WREN_TEST=/path/to/wren_test to compare."
  echo "    (DEC-BENCH-B: never invent a number — see BASELINE.md provenance note.)"
fi

if [ "$skip_bench" -eq 0 ]; then
  echo
  echo "==> criterion micro-benches (send / arith / fiber)"
  if cargo bench -p phalcom-core --features benchmarks --bench vm_bench; then
    echo "PASS     Criterion micro-benches"
  else
    echo "FAIL     Criterion micro-benches" >&2
    rc=1
  fi
fi

exit "$rc"
