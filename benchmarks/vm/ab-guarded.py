#!/usr/bin/env python3
"""ab-guarded.py — alternating same-session A/B that REFUSES to run on a busy machine.

Why this exists (F22, 2026-07-15)
---------------------------------
An `INLINE_ARGS` probe was timed on this box at **load average 7.1-10.4 on 8 cores**
while a concurrent session ran `rustc` and edited `dispatch.rs` mid-run. The harness
reported four decimal places throughout. Every number was void:

  * the *baseline* binary drifted ~4% across passes -- the unchanged arm moving as
    much as any claimed effect;
  * 7-rep signals evaporated at 15 reps (fib +2.91% -> -0.08%);
  * `min` improved while `median` did not -- the fingerprint of rare uncontended
    runs, not of an effect.

This is H15's failure class, not a new one: `compare-wren.py` found no `wren_test`,
printed a note, marked every row `ok` and **exited 0**, freezing SCOREBOARD §1 for
three cuts. A harness that DEGRADES instead of FAILING is worse than no harness,
because it emits numbers people believe. So this one exits 3 and prints nothing.

The repo's measurement law (ADR-0051) says: under ~10%, read the **sign across
pairs**. Contention is precisely what destroys the sign, which is why the guard is
not optional at the effect sizes this file is used for.

Guard, in order of responsiveness
---------------------------------
1. Preflight: 1-min load under LOAD_MAX, and no competing rustc/cargo/cc/ld process.
2. Per-rep rescan. `getloadavg()` lags ~1 min, so a short rustc burst is INVISIBLE
   to it -- the process scan is what actually catches contention arriving mid-run.
3. Abort, never pause: a mid-run pause perturbs cache state. A contaminated run is
   discarded whole and retried from a quiet machine.

What it does NOT fix
--------------------
Alternation defends against slow monotone drift. It does NOT make a contended run
valid -- a burst overlapping one arm's runs skews that arm, and no rep count repairs
it. That is why the guard aborts rather than compensating.

Usage
-----
    benchmarks/vm/ab-guarded.py <reps> [quick|full] --bin base=/path --bin probe=/path

Binaries must be BUILT AND COPIED OUT BEFORE ANY TIMING (all arms, then time) -- see
the two-build protocol in instruments.md; `cargo build --features opcode-histogram`
overwrites `target/release/phalcom` with a counting build, which must never be timed.

Exit codes: 0 = numbers valid. 2 = usage/run error. 3 = machine not quiet (no output).
"""

import argparse
import os
import statistics
import subprocess
import sys
import time

NCPU = os.cpu_count() or 8
# Our own timed child contributes ~1.0; meaningfully above that is someone else's work.
SANE_LOAD_MAX = 4.0  # above this the guard is effectively off -- output is smoke, not data
LOAD_MAX = float(os.environ.get("LOAD_MAX", "1.5"))
REPO = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

ROWS_FULL = [
    ("arith_send", f"{REPO}/benchmarks/vm/arith_send.ph"),
    ("bare_send", f"{REPO}/benchmarks/vm/bare_send.ph"),
    ("rest_fallback_send", f"{REPO}/benchmarks/vm/rest_fallback_send.ph"),
    ("for", f"{REPO}/benchmarks/wren-suite/for.ph"),
    ("fib", f"{REPO}/benchmarks/wren-suite/fib.ph"),
    ("string_equals", f"{REPO}/benchmarks/wren-suite/string_equals.ph"),
    ("method_call", f"{REPO}/benchmarks/wren-suite/method_call.ph"),
    ("map_numeric", f"{REPO}/benchmarks/wren-suite/map_numeric.ph"),
]
QUICK = {"arith_send", "rest_fallback_send", "for", "map_numeric"}

OWN_PIDS = {os.getpid(), os.getppid()}
BUILD_PROCS = ("rustc", "cargo", "cc", "clang", "ld", "lld", "swift-frontend")


def competitors():
    """Build processes stealing our cores. Cheaper and far more responsive than load."""
    out = subprocess.run(["ps", "-Ao", "pid=,pcpu=,comm="], capture_output=True, text=True).stdout
    found = []
    for line in out.splitlines():
        parts = line.split(None, 2)
        if len(parts) < 3:
            continue
        try:
            pid, cpu = int(parts[0]), float(parts[1])
        except ValueError:
            continue
        if pid in OWN_PIDS or cpu < 20.0:
            continue
        name = os.path.basename(parts[2])
        if name in BUILD_PROCS:
            found.append((pid, cpu, name))
    return found


def quiet_check(where):
    load1 = os.getloadavg()[0]
    comp = competitors()
    if load1 > LOAD_MAX or comp:
        print(f"\nABORT ({where}): machine is not quiet -- no numbers produced.", file=sys.stderr)
        print(f"  load1={load1:.2f} (max {LOAD_MAX}, ncpu={NCPU})", file=sys.stderr)
        for pid, cpu, name in comp:
            print(f"  competitor: pid={pid} {name} {cpu:.0f}%", file=sys.stderr)
        print("  A contended A/B cannot be repaired by more reps (F22). Wait, then re-run.", file=sys.stderr)
        print("  Override only to smoke-test the harness: LOAD_MAX=99 (never for a number).", file=sys.stderr)
        sys.exit(3)


def run(binp, ph):
    t0 = time.perf_counter()
    p = subprocess.run([binp, ph], capture_output=True)
    return time.perf_counter() - t0, p.stdout, p.returncode


def main():
    ap = argparse.ArgumentParser(description="Load-guarded alternating A/B (F22).")
    ap.add_argument("--check-only", action="store_true", help="Only run quiet check and exit 0 if quiet, 3 if busy")
    ap.add_argument("--where", default="check", help="Label for where the quiet check is performed")
    ap.add_argument("reps", type=int, nargs="?", default=15)
    ap.add_argument("rowset", nargs="?", default="full", choices=["quick", "full"])
    ap.add_argument("--bin", action="append", required=False, metavar="LABEL=PATH",
                    help="repeat; first is the baseline. e.g. --bin base=/tmp/base --bin p4=/tmp/p4")
    a = ap.parse_args()

    if a.check_only:
        quiet_check(a.where)
        print(f"quiet check ({a.where}) OK: load1={os.getloadavg()[0]:.2f} ncpu={NCPU}")
        sys.exit(0)

    if not a.bin:
        sys.exit("need --bin flags unless --check-only is specified")

    arms = []
    for spec in a.bin:
        if "=" not in spec:
            sys.exit(f"--bin needs LABEL=PATH, got {spec!r}")
        label, path = spec.split("=", 1)
        if not os.path.exists(path):
            sys.exit(f"no such binary: {path}")
        arms.append((label, path))
    if len(arms) < 2:
        sys.exit("need >= 2 --bin arms")
    base = arms[0][0]
    rows = [r for r in ROWS_FULL if a.rowset == "full" or r[0] in QUICK]

    quiet_check("preflight")
    print(f"preflight OK: load1={os.getloadavg()[0]:.2f} ncpu={NCPU} "
          f"arms={[x[0] for x in arms]} base={base} reps={a.reps}")
    hdr = f"{'row':<15} " + " ".join(f"{l+'_min':>11}" for l, _ in arms)
    hdr += "   " + " ".join(f"{'d%_'+l:>9}" for l, _ in arms[1:]) + "   sign_vs_" + base
    print(hdr)
    print("-" * len(hdr))

    for name, ph in rows:
        t = {l: [] for l, _ in arms}
        outs = {}
        for r in range(a.reps):
            quiet_check(f"row={name} rep={r}")
            order = arms if r % 2 == 0 else list(reversed(arms))
            for label, binp in order:
                w, out, rc = run(binp, ph)
                if rc != 0:
                    sys.exit(f"{name}: {label} exited rc={rc}")
                if label in outs and outs[label] != out:
                    sys.exit(f"{name}: {label} nondeterministic stdout across reps")
                outs[label] = out
                t[label].append(w)
        if len({bytes(v) for v in outs.values()}) != 1:
            sys.exit(f"{name}: STDOUT DIFFERS ACROSS ARMS -- probe is not behavior-invariant")
        b = min(t[base])
        line = f"{name:<15} " + " ".join(f"{min(t[l]):11.4f}" for l, _ in arms)
        line += "   " + " ".join(f"{(min(t[l])-b)/b*100:+9.2f}" for l, _ in arms[1:])
        line += "   " + " ".join(
            f"{l}:{sum(1 for i in range(a.reps) if t[l][i] < t[base][i])}/{a.reps}"
            for l, _ in arms[1:])
        print(line)

    quiet_check("post-run")
    if LOAD_MAX > SANE_LOAD_MAX:
        print(f"\n*** SMOKE TEST ONLY -- LOAD_MAX={LOAD_MAX} disabled the guard. ***")
        print("*** The numbers above are NOT valid and must not be quoted (F22). ***")
    else:
        print("\npost-run quiet check OK -- numbers above are valid.")


if __name__ == "__main__":
    main()
