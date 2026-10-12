#!/usr/bin/env python3
"""Parse PHALCOM_TEST_TIMINGS lines without silently dropping malformed rows."""

from __future__ import annotations

import argparse
import csv
import math
import re
import statistics
from pathlib import Path


PHASES = [
    ("compile_io_ms", "io"),
    ("compile_parse_ms", "parse"),
    ("compile_link_ms", "discovery+link"),
    ("semantic_session_init_ms", "semantic_session_init"),
    ("semantic_update_ms", "semantic_update"),
    ("compile_projection_ms", "user_projection"),
    ("canonical_universe_product_ms", "canonical_universe_product"),
    ("vm_kernel_native_ms", "kernel+native"),
    ("vm_universe_compile_ms", "universe_vm_compile"),
    ("vm_universe_execute_ms", "universe_initializers"),
    ("user_execution_ms", "user_execution"),
]

TIMING_RE = re.compile(
    r"^\[TIMING\] (?P<fixture>.+?) - "
    r"io: (?P<io>[^,]+), "
    r"parse: (?P<parse>[^,]+), "
    r"discovery\+link: (?P<discovery_link>[^,]+), "
    r"semantic_session_init: (?P<semantic_session_init>[^,]+), "
    r"semantic_update: (?P<semantic_update>[^,]+), "
    r"user_projection: (?P<user_projection>[^,]+), "
    r"canonical_universe_product: (?P<canonical_universe_product>[^,]+), "
    r"kernel\+native: (?P<kernel_native>[^,]+), "
    r"universe_vm_compile: (?P<universe_vm_compile>[^,]+), "
    r"universe_initializers: (?P<universe_initializers>[^,]+), "
    r"user_execution: (?P<user_execution>[^,]+)$"
)
DURATION_RE = re.compile(r"^(?P<number>\d+(?:\.\d+)?)\s*(?P<unit>ns|µs|us|ms|s)$")


def duration_ms(raw: str) -> float:
    match = DURATION_RE.fullmatch(raw.strip())
    if not match:
        raise ValueError(f"unrecognized duration {raw!r}")
    number = float(match.group("number"))
    unit = match.group("unit")
    factor = {"ns": 1e-6, "µs": 1e-3, "us": 1e-3, "ms": 1.0, "s": 1e3}[unit]
    return number * factor


def parse_log(path: Path) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    malformed: list[str] = []
    raw = path.read_text(errors="replace")
    baseline = re.search(r"(?m)^# BASELINE_HEAD=(?P<sha>[0-9a-f]{40})$", raw)
    if not baseline:
        raise ValueError(f"{path} has no full BASELINE_HEAD metadata")
    for line_number, line in enumerate(raw.splitlines(), start=1):
        if "[TIMING]" not in line:
            continue
        normalized = line[line.index("[TIMING]") :].strip()
        match = TIMING_RE.fullmatch(normalized)
        if not match:
            malformed.append(f"line {line_number}: {line}")
            continue
        ordinal = len(rows) + 1
        row: dict[str, object] = {
            "baseline_head": baseline.group("sha"),
            "fixture": match.group("fixture"),
            "ordinal": ordinal,
            "cold_or_warm": "cold" if ordinal == 1 else "warm",
        }
        failure = re.search(
            rf"(?m)^{re.escape(match.group('fixture'))} failed: (?P<reason>.+)$", raw
        )
        row["outcome"] = "failed" if failure else "no_matching_failure_line"
        row["failure_reason"] = failure.group("reason") if failure else ""
        groups = {
            "compile_io_ms": "io",
            "compile_parse_ms": "parse",
            "compile_link_ms": "discovery_link",
            "semantic_session_init_ms": "semantic_session_init",
            "semantic_update_ms": "semantic_update",
            "compile_projection_ms": "user_projection",
            "canonical_universe_product_ms": "canonical_universe_product",
            "vm_kernel_native_ms": "kernel_native",
            "vm_universe_compile_ms": "universe_vm_compile",
            "vm_universe_execute_ms": "universe_initializers",
            "user_execution_ms": "user_execution",
        }
        for column, group in groups.items():
            row[column] = duration_ms(match.group(group))
        if failure and "program compile failure" in failure.group("reason"):
            # The support harness emits zero-initialized VM timing fields when
            # compilation aborts before VM construction. Keep these unobserved
            # fields blank instead of treating them as measured zero-cost work.
            for column in (
                "canonical_universe_product_ms",
                "vm_kernel_native_ms",
                "vm_universe_compile_ms",
                "vm_universe_execute_ms",
                "user_execution_ms",
            ):
                row[column] = None
        row["measured_phase_total_ms"] = sum(
            float(row[column]) for column, _ in PHASES if row[column] is not None
        )
        rows.append(row)
    if malformed:
        details = "\n".join(malformed)
        raise ValueError(f"malformed timing records in {path}:\n{details}")
    if not rows:
        raise ValueError(f"no [TIMING] records found in {path}")
    return rows


def percentile(values: list[float], probability: float) -> float:
    ordered = sorted(values)
    index = max(0, math.ceil(probability * len(ordered)) - 1)
    return ordered[index]


def make_summary(rows: list[dict[str, object]], raw_paths: list[Path]) -> str:
    columns = [name for name, _ in PHASES]
    all_total = sum(float(row["measured_phase_total_ms"]) for row in rows)
    failures = [row for row in rows if row["outcome"] == "failed"]
    lines = [
        "Baseline SHAs: " + ", ".join(f"`{sha}`" for sha in sorted({str(row['baseline_head']) for row in rows})),
        "Branch: `perf/universe-characterization`",
        "Raw timing logs: " + ", ".join(f"`{path}`" for path in raw_paths),
        f"Parsed fixture records: {len(rows)}",
        f"Matching fixture failures: {len(failures)}",
        "Failed samples are partial. Blank phase fields mean the phase was not observed; they are excluded from totals.",
        "",
        "| Rank | Phase | Count | Sum (ms) | Share of observed phase sum | Mean (ms) | Median (ms) | P90 (ms) | P95 (ms) | Min (ms) | Max (ms) |",
        "|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|",
    ]
    totals: list[tuple[str, float, list[float]]] = []
    for column in columns:
        values = [float(row[column]) for row in rows if row[column] is not None]
        label = column.removesuffix("_ms")
        totals.append((label, sum(values), values))
    totals.sort(key=lambda item: item[1], reverse=True)
    for rank, (label, phase_sum, values) in enumerate(totals, start=1):
        share = (phase_sum / all_total * 100.0) if all_total else 0.0
        if not values:
            lines.append(f"| {rank} | {label} | 0 | N/A | N/A | N/A | N/A | N/A | N/A | N/A | N/A |")
        else:
            p95 = f"{percentile(values, 0.95):.3f}" if len(values) >= 20 else "N/A (<20 samples)"
            lines.append(
                f"| {rank} | {label} | {len(values)} | {phase_sum:.3f} | {share:.2f}% | {statistics.mean(values):.3f} | "
                f"{statistics.median(values):.3f} | {percentile(values, 0.90):.3f} | {p95} | {min(values):.3f} | {max(values):.3f} |"
            )
    total_values = [float(row["measured_phase_total_ms"]) for row in rows]
    p95_total = f"{percentile(total_values, 0.95):.3f}" if len(total_values) >= 20 else "N/A (<20 samples)"
    lines.append(
        f"| — | measured_phase_total_ms | {len(total_values)} | {sum(total_values):.3f} | 100.00% | "
        f"{statistics.mean(total_values):.3f} | {statistics.median(total_values):.3f} | "
        f"{percentile(total_values, 0.90):.3f} | {p95_total} | "
        f"{min(total_values):.3f} | {max(total_values):.3f} |"
    )
    lines.extend(
        [
            "",
            "## First fixture versus warm fixtures by process",
            "",
            "| Process | Phase | First (ms) | Warm median (ms) | First / warm median |",
            "|---|---|---:|---:|---:|",
        ]
    )
    process_ids = sorted({str(row["process_id"]) for row in rows})
    for process_id in process_ids:
        process_rows = [row for row in rows if row["process_id"] == process_id]
        first_row = min(process_rows, key=lambda row: int(row["ordinal"]))
        warm_rows = [row for row in process_rows if int(row["ordinal"]) > int(first_row["ordinal"])]
        for column, _ in PHASES:
            first_value = first_row[column]
            warm_values = [float(row[column]) for row in warm_rows if row[column] is not None]
            if first_value is None:
                lines.append(f"| {process_id} | {column.removesuffix('_ms')} | N/A | N/A | N/A |")
            elif not warm_values:
                lines.append(f"| {process_id} | {column.removesuffix('_ms')} | {float(first_value):.3f} | N/A (no warm sample) | N/A |")
            else:
                warm_median = statistics.median(warm_values)
                ratio = float(first_value) / warm_median if warm_median else float("inf")
                lines.append(f"| {process_id} | {column.removesuffix('_ms')} | {float(first_value):.3f} | {warm_median:.3f} | {ratio:.3f}x |")
    return "\n".join(lines) + "\n"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, required=True, action="append", help="raw corpus timing log; repeat for multiple processes")
    parser.add_argument("--csv", type=Path, required=True, help="parsed measurement CSV")
    parser.add_argument("--summary", type=Path, required=True, help="summary Markdown output")
    args = parser.parse_args()

    rows: list[dict[str, object]] = []
    for input_path in args.input:
        for row in parse_log(input_path):
            row["process_id"] = input_path.stem
            rows.append(row)
    args.csv.parent.mkdir(parents=True, exist_ok=True)
    fieldnames = ["baseline_head", "process_id", "fixture", "ordinal", "cold_or_warm", "outcome", "failure_reason", *[name for name, _ in PHASES], "measured_phase_total_ms"]
    with args.csv.open("w", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=fieldnames)
        writer.writeheader()
        for row in rows:
            writer.writerow(
                {
                    **row,
                    **{name: (f"{float(row[name]):.9f}" if row[name] is not None else "") for name, _ in PHASES},
                    "measured_phase_total_ms": f"{float(row['measured_phase_total_ms']):.9f}",
                }
            )
    args.summary.parent.mkdir(parents=True, exist_ok=True)
    args.summary.write_text(make_summary(rows, args.input))
    print(f"parsed {len(rows)} timing rows; malformed timing lines: 0")
    print(f"csv: {args.csv}")
    print(f"summary: {args.summary}")


if __name__ == "__main__":
    main()
