#!/usr/bin/env python3
"""Replay retained observations; never overwrite evidence or rebuild the runtime."""
import argparse
from pathlib import Path
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--check', action='store_true', help='compare stdout and exit status with retained observations')
parser.add_argument('--only', nargs='+', help='probe stems, such as 03-async 09-guard-only')
parser.add_argument('--state', action='store_true', help='compile/run only the Rust heap-state probe using the pinned compiler')
args = parser.parse_args()
evidence = Path(__file__).resolve().parent
root = evidence.parents[3]
probes = evidence / 'probes'
failed = False

def record(stem, result):
    global failed
    print(f'{stem}: exit {result.returncode}\n{result.stdout}', flush=True)
    if result.stderr and result.returncode:
        print(result.stderr, flush=True)
    if args.check:
        expected = (probes / f'{stem}.stdout').read_text()
        expected_code = int((probes / f'{stem}.exit').read_text())
        matches = result.stdout == expected and result.returncode == expected_code
        print('matches retained observation' if matches else 'DIFFERS from retained observation', flush=True)
        failed |= not matches

if args.state:
    libraries = list((root / 'target/debug/deps').glob('libphalcom_core-*.rlib'))
    if not libraries:
        raise SystemExit('Build phalcom-core with the pinned toolchain first.')
    library = max(libraries, key=lambda p: p.stat().st_mtime)
    with tempfile.TemporaryDirectory(prefix='phalcom-concurrency-audit-') as temporary:
        executable = Path(temporary) / 'state-probe'
        subprocess.run(['rustup', 'run', 'nightly-2026-07-10', 'rustc', '--edition=2024',
                        str(probes / '10-state-inspection.rs'), '-L', f'dependency={root / "target/debug/deps"}',
                        '--extern', f'phalcom_core={library}', '-o', str(executable)], cwd=root, check=True, timeout=60)
        record('10-state-inspection', subprocess.run([str(executable)], cwd=root, capture_output=True, text=True, timeout=60))
else:
    files = sorted(probes.glob('*.ph'))
    if args.only:
        known = {p.stem for p in files}
        missing = set(args.only) - known
        if missing:
            raise SystemExit(f'Unknown probes: {sorted(missing)}')
        files = [p for p in files if p.stem in args.only]
    for source in files:
        record(source.stem, subprocess.run([str(root / 'target/debug/phalcom'), '--plain', str(source)],
                                          cwd=root, capture_output=True, text=True, timeout=60))
raise SystemExit(1 if failed else 0)
