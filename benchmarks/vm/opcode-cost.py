#!/usr/bin/env python3
"""opcode-cost.py — what does one Phalcom instruction cost?

Answers perf-log hole H3. Nothing else in the harness can: the dispatch loop is a
single `match` in a single function, so `sample` attributes every opcode arm to
`run_until_inner` (27-35% of ticks, per perf-log §4) and prices none of them.

## The protocol, and why it is two builds

Counting instructions costs an increment per instruction — the same class of
per-opcode work that `vm-trace`'s span cost a measured 18.2% of arith wall-clock
(perf-log 003). So a timing read from a counting build is wrong.

**Counts are deterministic**: the same program retires the same instruction mix in
both builds. So this script runs each benchmark twice —

  1. counts   <- `--features opcode-histogram` build (stderr histogram)
  2. wall     <- default build (best-of-N)

- and divides. The counter never touches the number it produces. Do not "simplify"
this into one run.

## What the number means

`ns/instr = wall / total` is a true mean over the *executed mix*, not a price per
opcode: a `Loop` and an `Invoke` land in the same average. The mix column is what
makes it readable — comparing the mean across benchmarks with different mixes is
the signal. Pricing one opcode needs a differential (two programs differing by a
known count of a single opcode); this script makes that constructible by reporting
exact per-opcode counts, but does not itself perform it.

Wall-clock includes process start and bootstrap (~5 ms), which the instruction
count also covers (bootstrap compiles and runs `core.ph`), so the two are measured
over the same interval.

Usage:
    benchmarks/vm/opcode-cost.py                    # default benchmark set
    benchmarks/vm/opcode-cost.py bare_send for      # named programs
    REPS=5 benchmarks/vm/opcode-cost.py             # more repetitions
    PH_BIN=... PH_HIST_BIN=... benchmarks/vm/opcode-cost.py

Build the two binaries first:
    cargo build -r -p phalcom-core --bin phalcom
    cp target/release/phalcom /tmp/ph-default
    cargo build -r -p phalcom-core --bin phalcom --features opcode-histogram
    cp target/release/phalcom /tmp/ph-hist
"""

import os
import re
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PH_BIN = os.environ.get("PH_BIN", str(ROOT / "target/release/phalcom"))
PH_HIST_BIN = os.environ.get("PH_HIST_BIN", "/tmp/ph-hist")
REPS = int(os.environ.get("REPS", "3"))

# (label, path) — whole programs, so each carries bootstrap exactly once.
DEFAULT_SET = [
    ("bare_send", "benchmarks/vm/bare_send.ph"),
    ("arith_send", "benchmarks/vm/arith_send.ph"),
    ("rest_fallback_send", "benchmarks/vm/rest_fallback_send.ph"),
    ("fiber_churn", "benchmarks/vm/fiber_churn.ph"),
    ("bootstrap", "benchmarks/vm/bootstrap.ph"),
    ("for", "benchmarks/wren-suite/for.ph"),
    ("fib", "benchmarks/wren-suite/fib.ph"),
    ("method_call", "benchmarks/wren-suite/method_call.ph"),
    ("string_equals", "benchmarks/wren-suite/string_equals.ph"),
    ("binary_trees", "benchmarks/wren-suite/binary_trees.ph"),
    ("map_numeric", "benchmarks/wren-suite/map_numeric.ph"),
    ("fibers", "benchmarks/wren-suite/fibers.ph"),
]

HIST_LINE = re.compile(r"^\s*(\w+)\s+(\d+)\s+([\d.]+)%$")
TOTAL_LINE = re.compile(r"^opcode histogram: (\d+) instructions retired")


def counts_for(path):
    """Returns (total, {opcode: count}) from a histogram build. Timing here is
    meaningless by construction — only the counts are read."""
    proc = subprocess.run([PH_HIST_BIN, path], capture_output=True, text=True, cwd=ROOT)
    total, per = None, {}
    for line in proc.stderr.splitlines():
        m = TOTAL_LINE.match(line)
        if m:
            total = int(m.group(1))
            continue
        m = HIST_LINE.match(line)
        if m and m.group(1) != "TOTAL":
            per[m.group(1)] = int(m.group(2))
    return total, per


def wall_for(path):
    """Best-of-REPS wall-clock from the default build."""
    best = None
    for _ in range(REPS):
        start = time.perf_counter()
        proc = subprocess.run([PH_BIN, path], capture_output=True, text=True, cwd=ROOT)
        elapsed = time.perf_counter() - start
        if proc.returncode != 0:
            return None
        best = elapsed if best is None else min(best, elapsed)
    return best


def main():
    for binary, label in ((PH_BIN, "PH_BIN"), (PH_HIST_BIN, "PH_HIST_BIN")):
        if not Path(binary).exists():
            sys.exit(f"{label} not found: {binary}\nSee this script's docstring for the two builds it needs.")

    wanted = sys.argv[1:]
    bench = [(n, p) for n, p in DEFAULT_SET if not wanted or n in wanted]
    if not bench:
        sys.exit(f"no benchmark matched {wanted}; known: {', '.join(n for n, _ in DEFAULT_SET)}")

    print(f"{'benchmark':<16}{'wall':>9}{'instructions':>15}{'ns/instr':>11}{'Minstr/s':>11}  top opcodes")
    print("-" * 96)

    rows = []
    for name, path in bench:
        total, per = counts_for(path)
        wall = wall_for(path)
        if total is None or wall is None:
            print(f"{name:<16}  FAILED (non-zero exit, or no histogram on stderr)")
            continue
        ns = (wall * 1e9) / total
        mips = total / wall / 1e6
        top = ", ".join(f"{k} {v * 100 // total}%" for k, v in list(per.items())[:3])
        # A program that retires almost nothing spends its wall-clock somewhere
        # other than the dispatch loop (bootstrap.ph: ~660 instructions, ~5ms of
        # it compiling core.ph). Its ns/instr prices the *compiler*, not an
        # instruction, so it is reported and then excluded from the spread rather
        # than silently averaged in.
        compile_bound = total < 1_000_000
        print(f"{name:<16}{wall:>8.3f}s{total:>15,}{ns:>11.1f}{mips:>11.1f}  {top}"
              + ("   <- compile-bound, not an instruction price" if compile_bound else ""))
        if not compile_bound:
            rows.append((name, ns))

    if len(rows) > 1:
        lo = min(rows, key=lambda r: r[1])
        hi = max(rows, key=lambda r: r[1])
        print("-" * 96)
        print(f"spread (execution-bound rows only): {lo[0]} {lo[1]:.1f} ns/instr .. "
              f"{hi[0]} {hi[1]:.1f} ns/instr  ({hi[1] / lo[1]:.1f}x)")
        print("Means over each program's own mix — not a per-opcode price. See the docstring.")
        print("A high ns/instr means each instruction did more work (allocation, GC, hashing),")
        print("not that more instructions ran — that is what the instruction count is for.")


if __name__ == "__main__":
    main()
