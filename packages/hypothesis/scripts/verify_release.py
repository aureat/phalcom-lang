#!/usr/bin/env python3
"""Run the complete source/static release gate, optionally twice."""
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def run(command: list[str]) -> None:
    print("$", " ".join(command))
    subprocess.run(command, cwd=ROOT, check=True)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--single-pass", action="store_true")
    args = parser.parse_args()
    passes = 1 if args.single_pass else 2
    for iteration in range(1, passes + 1):
        print(f"=== release verification pass {iteration}/{passes} ===")
        run([sys.executable, "scripts/verify_phase11_mutations.py"])
        for phase in range(1, 13):
            run([sys.executable, f"scripts/verify_phase{phase:02d}.py"])
    print("All source/static release gates passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
