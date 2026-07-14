#!/usr/bin/env python3
"""compare-wren.py — output-verified Phalcom-vs-Wren timing of the ported suite.

Runs each `benchmarks/wren-suite/<name>.{ph,wren}` pair as whole processes,
best-of-N wall-clock, and **compares stdout byte-for-byte** (after dropping
Wren's `elapsed:` timing line, which the Phalcom ports do not print — no
`System.clock` exists). A row where the two outputs disagree, or where either
side exits non-zero, is reported as a FAIL: a benchmark that computes the
wrong answer is not a measurement (law P1, `docs/spec/v0.2/performance.md`).
The suite README promises "correctness-verified, not just 'it ran'" — this
script is what mechanizes that promise, instead of re-checking by eye.

Siblings:
  - `run.sh` — Tier 0 baseline reproduction (Skynet + criterion).
  - `../../phalcom-core/benches/vm_bench.rs` — criterion micro-benches.

Usage:
    benchmarks/vm/compare-wren.py                 # whole suite
    benchmarks/vm/compare-wren.py for map_string  # named rows only
    REPS=5 benchmarks/vm/compare-wren.py          # more repetitions
    PH_BIN=... WREN_TEST=... benchmarks/vm/compare-wren.py

Exits non-zero if any row fails, so it can gate a perf commit.
"""

import os
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
SUITE = os.path.join(REPO, "benchmarks", "wren-suite")
PH = os.environ.get("PH_BIN", os.path.join(REPO, "target", "release", "phalcom"))
WREN = os.environ.get("WREN_TEST", os.path.expanduser("~/dev/repos/wren/bin/wren_test"))
REPS = int(os.environ.get("REPS", "3"))


def normalize(text):
    """Comparable stdout: trimmed lines, minus Wren's `elapsed:` timing line."""
    lines = [line.rstrip() for line in text.strip().splitlines()]
    return [line for line in lines if not line.startswith("elapsed:")]


def best_of(cmd, reps=REPS):
    """Runs `cmd` `reps` times, returning (fastest wall-clock, stdout, exit code)."""
    best, out, rc = None, "", 0
    for _ in range(reps):
        start = time.perf_counter()
        proc = subprocess.run(cmd, capture_output=True, text=True)
        elapsed = time.perf_counter() - start
        if best is None or elapsed < best:
            best, out, rc = elapsed, proc.stdout, proc.returncode
    return best, out, rc


def main():
    if not os.path.exists(PH):
        sys.exit(f"no phalcom binary at {PH} — run `cargo build -r -p phalcom-core --bin phalcom`")
    have_wren = os.path.exists(WREN)
    if not have_wren:
        print(f"note: no wren_test at {WREN} — timing Phalcom only, no comparison.\n")

    names = sorted(f[:-3] for f in os.listdir(SUITE) if f.endswith(".ph"))
    if len(sys.argv) > 1:
        names = [n for n in names if n in sys.argv[1:]]

    failures = 0
    for name in names:
        ph_src = os.path.join(SUITE, f"{name}.ph")
        wren_src = os.path.join(SUITE, f"{name}.wren")
        ph_time, ph_out, ph_rc = best_of([PH, ph_src])

        wren_time, wren_out, wren_rc = None, None, 0
        if have_wren and os.path.exists(wren_src):
            wren_time, wren_out, wren_rc = best_of([WREN, wren_src])

        matches = wren_out is not None and normalize(ph_out) == normalize(wren_out)
        ok = ph_rc == 0 and wren_rc == 0 and (matches or wren_out is None)
        ratio = f"{ph_time / wren_time:.1f}x" if wren_time else "n/a"
        wren_cell = f"{wren_time:.3f}s" if wren_time else "n/a"
        print(f"{name:16} phalcom={ph_time:7.3f}s  wren={wren_cell:>8}  {ratio:>6}  "
              f"{'ok' if ok else 'FAIL'}")

        if not ok:
            failures += 1
            print(f"    phalcom (rc={ph_rc}): {normalize(ph_out)[:4]}")
            print(f"    wren    (rc={wren_rc}): {normalize(wren_out)[:4] if wren_out else None}")
        sys.stdout.flush()

    if failures:
        print(f"\n{failures} row(s) FAILED — output mismatch or non-zero exit. "
              "The timings above are not a baseline.")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
