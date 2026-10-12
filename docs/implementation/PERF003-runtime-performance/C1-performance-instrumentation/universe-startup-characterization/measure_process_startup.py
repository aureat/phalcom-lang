#!/usr/bin/env python3
"""Capture repeated Python startup controls and one Telos CLI blocker probe."""

from __future__ import annotations

import argparse
import shlex
import subprocess
import time
from pathlib import Path

BASELINE_HEAD = "b00d75b3ac9f50d37cf9abfddfde00917e099031"


def run(label: str, command: list[str], cwd: Path, repetitions: int, stream) -> None:
    for sample in range(1, repetitions + 1):
        wrapped = ["/usr/bin/time", "-l", *command]
        started = time.perf_counter_ns()
        result = subprocess.run(wrapped, cwd=cwd, text=True, capture_output=True, check=False)
        wall_ns = time.perf_counter_ns() - started
        stream.write(f"CASE,{label},{sample},{result.returncode},{wall_ns}\n")
        stream.write(f"COMMAND,{shlex.join(wrapped)}\n")
        stream.write(f"STDOUT_BEGIN\n{result.stdout}STDOUT_END\n")
        stream.write(f"STDERR_BEGIN\n{result.stderr}STDERR_END\n")
        stream.flush()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--cwd", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--repetitions", type=int, default=5)
    parser.add_argument("--baseline-head", default=BASELINE_HEAD)
    args = parser.parse_args()
    actual_head = subprocess.run(
        ["git", "-C", str(args.cwd), "rev-parse", "HEAD"],
        text=True,
        capture_output=True,
        check=True,
    ).stdout.strip()
    if actual_head != args.baseline_head:
        raise SystemExit(f"baseline mismatch: expected {args.baseline_head}, found {actual_head}")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    fixture = "docs/implementation/PERF003-runtime-performance/C1-performance-instrumentation/universe-startup-characterization/fixtures/tiny.py"
    telos_fixture = "docs/implementation/PERF003-runtime-performance/C1-performance-instrumentation/universe-startup-characterization/fixtures/tiny.ph"
    binary = "target/debug/phalcom"
    with args.output.open("w") as stream:
        stream.write(f"# BASELINE_HEAD={args.baseline_head}\n")
        stream.write("# BRANCH=perf/universe-characterization\n")
        run("python", ["python3", fixture], args.cwd, args.repetitions, stream)
        run("python_no_site", ["python3", "-S", fixture], args.cwd, args.repetitions, stream)
        # The current baseline cannot complete this normal language path. One
        # failure probe is retained instead of reporting it as a timing sample.
        run("telos_blocker_probe", [binary, telos_fixture], args.cwd, 1, stream)


if __name__ == "__main__":
    main()
