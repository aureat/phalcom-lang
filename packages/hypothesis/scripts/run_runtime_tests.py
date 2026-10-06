#!/usr/bin/env python3
"""Run Hypothesis .ph runtime tests in isolated Phalcom processes."""
from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path


PACKAGE_ROOT = Path(__file__).resolve().parents[1]
RUNTIME_ROOT = PACKAGE_ROOT / "tests" / "runtime"
SOURCE_ROOT = RUNTIME_ROOT / "src"
DEFAULT_TIMEOUT_SECONDS = 60


def resolve_executable(value: str) -> Path | None:
    """Resolve an explicit path or executable name from the caller or repo."""
    expanded = Path(value).expanduser()
    candidates: list[Path] = []

    if expanded.is_absolute():
        candidates.append(expanded)
    else:
        candidates.append(Path.cwd() / expanded)
        candidates.append(PACKAGE_ROOT.parents[1] / expanded)

    found_on_path = shutil.which(value)
    if found_on_path:
        candidates.append(Path(found_on_path))

    for candidate in candidates:
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return candidate.resolve()
    return None


def find_executable(requested: str | None) -> Path:
    if requested:
        resolved = resolve_executable(requested)
        if resolved is None:
            raise ValueError(f"Phalcom executable is not runnable: {requested}")
        return resolved

    configured = os.environ.get("PHALCOM_BIN")
    if configured:
        resolved = resolve_executable(configured)
        if resolved is None:
            raise ValueError(f"PHALCOM_BIN is not runnable: {configured}")
        return resolved

    repository_binary = PACKAGE_ROOT.parents[1] / "target" / "debug" / "phalcom"
    if repository_binary.is_file() and os.access(repository_binary, os.X_OK):
        return repository_binary.resolve()

    on_path = shutil.which("phalcom")
    if on_path:
        return Path(on_path).resolve()

    raise ValueError(
        "Phalcom was not found; pass --phalcom, set PHALCOM_BIN, build "
        "target/debug/phalcom, or put phalcom on PATH"
    )


def test_name(path: Path) -> str:
    relative = path.relative_to(SOURCE_ROOT).with_suffix("")
    parts = list(relative.parts)
    if parts[0] == "blocked":
        parts = parts[1:]
    filename = parts[-1]
    if not filename.endswith("-test"):
        raise ValueError(f"runtime test filename must end in -test.ph: {path}")
    parts[-1] = filename.removesuffix("-test")
    parts = [part.replace("-", "_") for part in parts]
    return ".".join(parts)


def blocked_reason(path: Path) -> str | None:
    for line in path.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if stripped.startswith("// BLOCKED:"):
            return stripped.removeprefix("// BLOCKED:").strip()
    return None


def collect_tests(inputs: list[Path]) -> list[Path]:
    if not inputs:
        return sorted(SOURCE_ROOT.rglob("*-test.ph"))

    collected: set[Path] = set()
    for item in inputs:
        candidates = [item] if item.is_absolute() else [Path.cwd() / item, PACKAGE_ROOT.parents[1] / item]
        path = next((candidate.resolve() for candidate in candidates if candidate.exists()), None)
        if path is None:
            raise ValueError(f"test path does not exist: {item}")
        if not path.is_relative_to(SOURCE_ROOT.resolve()):
            raise ValueError(f"test path must be inside {SOURCE_ROOT}: {item}")
        if path.is_dir():
            collected.update(path.rglob("*-test.ph"))
        elif path.is_file() and path.name.endswith("-test.ph"):
            collected.add(path)
        else:
            raise ValueError(f"expected a *-test.ph file or directory: {item}")

    return sorted(collected)


def captured_text(value: str | bytes | None) -> str:
    if value is None:
        return ""
    if isinstance(value, bytes):
        return value.decode("utf-8", errors="replace")
    return value


def report_output(label: str, output: str) -> None:
    if output:
        print(f"  {label}:", flush=True)
        for line in output.rstrip().splitlines():
            print(f"    {line}", flush=True)


def run_one(executable: Path, fixture: Path, timeout: int) -> bool:
    name = test_name(fixture)
    marker = f"PASS {name}"
    try:
        completed = subprocess.run(
            [str(executable), str(fixture)],
            cwd=RUNTIME_ROOT,
            check=False,
            capture_output=True,
            text=True,
            timeout=timeout,
        )
    except subprocess.TimeoutExpired as error:
        print(f"FAIL {name}: timed out after {timeout}s", flush=True)
        report_output("stdout", captured_text(error.stdout))
        report_output("stderr", captured_text(error.stderr))
        return False
    except OSError as error:
        print(f"FAIL {name}: could not start Phalcom: {error}", flush=True)
        return False

    if completed.returncode == 0 and marker in completed.stdout.splitlines():
        print(f"PASS {name}", flush=True)
        return True

    reason = f"exit code {completed.returncode}"
    if marker not in completed.stdout.splitlines():
        reason += f", missing marker {marker!r}"
    print(f"FAIL {name}: {reason}", flush=True)
    report_output("stdout", completed.stdout)
    report_output("stderr", completed.stderr)
    return False


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--phalcom",
        metavar="PATH",
        help="Phalcom executable (then PHALCOM_BIN, target/debug/phalcom, PATH)",
    )
    parser.add_argument(
        "--timeout",
        type=int,
        default=DEFAULT_TIMEOUT_SECONDS,
        metavar="SECONDS",
        help=f"per-test process timeout (default: {DEFAULT_TIMEOUT_SECONDS})",
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="return failure if any known-blocked runtime test is present",
    )
    parser.add_argument(
        "tests",
        nargs="*",
        type=Path,
        help="optional runtime test files or directories; defaults to the full runtime corpus",
    )
    args = parser.parse_args()

    if args.timeout <= 0:
        parser.error("--timeout must be positive")

    try:
        fixtures = collect_tests(args.tests)
    except ValueError as error:
        parser.error(str(error))

    if not fixtures:
        print(f"No runtime tests found under {SOURCE_ROOT}", file=sys.stderr)
        return 2

    needs_executable = any(
        fixture.relative_to(SOURCE_ROOT).parts[0] != "blocked"
        for fixture in fixtures
    )
    try:
        executable = find_executable(args.phalcom) if needs_executable else None
    except ValueError as error:
        parser.error(str(error))

    passed = 0
    failed = 0
    blocked = 0
    for fixture in fixtures:
        if fixture.relative_to(SOURCE_ROOT).parts[0] == "blocked":
            reason = blocked_reason(fixture)
            if reason:
                print(f"BLOCKED {test_name(fixture)}: {reason}", flush=True)
                blocked += 1
            else:
                print(
                    f"FAIL {test_name(fixture)}: blocked fixture is missing its // BLOCKED: reason",
                    flush=True,
                )
                failed += 1
        else:
            assert executable is not None
            if run_one(executable, fixture, args.timeout):
                passed += 1
            else:
                failed += 1

    print(f"\n{passed} passed, {failed} failed, {blocked} blocked", flush=True)
    return 1 if failed or (args.strict and blocked) else 0


if __name__ == "__main__":
    raise SystemExit(main())
