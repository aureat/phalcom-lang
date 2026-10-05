#!/usr/bin/env python3
"""Mutation checks proving the Phase 11 source gate detects contract loss."""
from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VERIFY = ROOT / "scripts" / "verify_phase11.py"

MUTATIONS = {
    "provider-normalization": ("src/choices/provider.ph", "_ChoiceNormalization.normalize", "_ChoiceNormalization.missing"),
    "candidate-deduplication": ("src/engine/shrinker.ph", "seenSignatures", "lostSignatures"),
    "directory-lock": ("src/database/directory.ph", "withPathLock", "withoutPathLock"),
    "reporter-boundary": ("src/reporting/reporter.ph", "_CheckedReporter", "_UncheckedReporter"),
    "linear-span-order": ("src/choices/buffer.ph", "_closedSpans.at(id, put: Some.new(closed))", "_closedSpans.add(closed)"),
}


def main() -> int:
    failures: list[str] = []
    for name, (relative, before, after) in MUTATIONS.items():
        with tempfile.TemporaryDirectory(prefix="phalcom-hypothesis-phase11-mutation-") as temp:
            target = Path(temp) / "project"
            shutil.copytree(ROOT, target)
            path = target / relative
            source = path.read_text(encoding="utf-8")
            if before not in source:
                failures.append(f"{name}: source marker missing before mutation")
                continue
            path.write_text(source.replace(before, after), encoding="utf-8")
            completed = subprocess.run(
                [sys.executable, str(VERIFY), "--root", str(target)],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.STDOUT,
                check=False,
            )
            if completed.returncode == 0:
                failures.append(f"{name}: verifier accepted mutated source")
            else:
                print(f"PASS mutation {name}")
    if failures:
        for failure in failures:
            print(f"FAIL {failure}")
        return 1
    print(f"\n{len(MUTATIONS)} passed, 0 failed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
