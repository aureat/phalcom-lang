#!/usr/bin/env python3
"""Parse measure_process_startup.py output and reject malformed samples."""

from __future__ import annotations

import argparse
import csv
import re
from pathlib import Path


CASE_RE = re.compile(
    r"^CASE,(?P<case>[^,]+),(?P<sample>\d+),(?P<exit_code>-?\d+),(?P<wall_ns>\d+)\n"
    r"(?P<body>.*?)(?=^CASE,|\Z)",
    re.MULTILINE | re.DOTALL,
)
COMMAND_RE = re.compile(r"^COMMAND,(?P<command>.+)$", re.MULTILINE)
RSS_RE = re.compile(r"^\s*(?P<bytes>\d+)\s+maximum resident set size$", re.MULTILINE)


def parse(path: Path) -> list[dict[str, object]]:
    full_raw = path.read_text(errors="replace")
    baseline = re.search(r"(?m)^# BASELINE_HEAD=(?P<sha>[0-9a-f]{40})$", full_raw)
    if not baseline:
        raise ValueError(f"{path} has no full BASELINE_HEAD metadata")
    raw = "".join(
        line for line in full_raw.splitlines(keepends=True)
        if not line.startswith("# BASELINE_HEAD=") and not line.startswith("# BRANCH=")
    )
    rows: list[dict[str, object]] = []
    cursor = 0
    for match in CASE_RE.finditer(raw):
        if raw[cursor : match.start()].strip():
            raise ValueError(f"unparsed output before sample at byte {cursor}")
        body = match.group("body")
        command = COMMAND_RE.search(body)
        rss = RSS_RE.search(body)
        if not command:
            raise ValueError(f"sample {match.group('case')}#{match.group('sample')} has no COMMAND")
        if not rss:
            raise ValueError(f"sample {match.group('case')}#{match.group('sample')} has no RSS measurement")
        rows.append(
            {
                "baseline_head": baseline.group("sha"),
                "case": match.group("case"),
                "sample": int(match.group("sample")),
                "exit_code": int(match.group("exit_code")),
                "outcome": "success" if int(match.group("exit_code")) == 0 else "failed_or_blocked",
                "wall_ms": f"{int(match.group('wall_ns')) / 1_000_000:.6f}",
                "max_rss_bytes": int(rss.group("bytes")),
                "command": command.group("command"),
            }
        )
        cursor = match.end()
    if raw[cursor:].strip():
        raise ValueError(f"unparsed trailing output at byte {cursor}")
    if not rows:
        raise ValueError("no CASE samples found")
    return rows


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--csv", type=Path, required=True)
    args = parser.parse_args()
    rows = parse(args.input)
    args.csv.parent.mkdir(parents=True, exist_ok=True)
    with args.csv.open("w", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=list(rows[0]))
        writer.writeheader()
        writer.writerows(rows)
    print(f"parsed {len(rows)} samples; malformed/unparsed output: 0")
    print(f"csv: {args.csv}")


if __name__ == "__main__":
    main()
