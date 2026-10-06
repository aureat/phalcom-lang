#!/usr/bin/env python3
"""Observed static verification for Phase 02 when no Phalcom executable exists."""

from __future__ import annotations

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]


def read(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


def fail(message: str) -> None:
    raise AssertionError(message)


def require(source: str, needle: str, where: str) -> None:
    if needle not in source:
        fail(f"{where}: missing {needle!r}")


def check_tests_first() -> None:
    expected = {
        "tests/core/settings.ph",
        "tests/core/status_match.ph",
        "tests/core/failure_origin.ph",
        "tests/core/context_cleanup.ph",
    }
    missing = sorted(path for path in expected if not (ROOT / path).is_file())
    if missing:
        fail("missing Phase 02 tests: " + ", ".join(missing))

    settings_test = read("tests/core/settings.ph")
    for value in ("100", "1000", "10000", "50", "20260723"):
        require(settings_test, value, "settings test")
    for update in (
        "maxExamples(500)", "maxDiscards(2000)", "maxShrinks(250)",
        "maxChoices(5000)", "seed(20260723)", "database(database)",
        "phases(const [Phase.Generate])", "statefulSteps(80)",
        "deadline(250)",
    ):
        require(settings_test, update, "settings immutable update test")
    require(settings_test, "standard.maxExamples(0)", "settings contracts")
    require(settings_test, "standard.phases(const [])", "settings contracts")

    require(read("tests/core/status_match.ph"), "item.match(", "status match test")
    require(read("tests/core/failure_origin.ph"), "first != second", "failure-origin test")
    require(read("tests/core/context_cleanup.ph"), "}.attempt()", "context cleanup test")


def check_core_model() -> None:
    manifest = read("phalcom.toml")
    version_match = re.search(r'version = "0\.1\.0-phase\.(\d+)"', manifest)
    if 'version = "0.1.0"' not in manifest and (not version_match or int(version_match.group(1)) < 2):
        fail("phalcom.toml: expected Phase 02 or later package version")

    phase = read("src/core/phase.ph")
    settings = read("src/core/settings.ph")
    status = read("src/core/status.ph")
    failure = read("src/core/failure.ph")
    statistics = read("src/core/statistics.ph")
    context = read("src/core/context.ph")
    errors = read("src/core/errors.ph")

    for path, source in {
        "phase.ph": phase,
        "settings.ph": settings,
        "status.ph": status,
        "failure.ph": failure,
        "statistics.ph": statistics,
    }.items():
        require(source, "@data", path)
        require(source, "@immutable", path)

    for path, source, variants in (
        ("phase.ph", phase, ("Explicit", "Reuse", "Generate", "Target", "Shrink", "Explain")),
        ("status.ph", status, ("Valid", "Invalid", "Overrun", "Interesting", "Passed", "Falsified", "Inconclusive", "Errored")),
    ):
        require(source, "@sealed", path)
        for variant in variants:
            require(source, f"@variant {variant}", path)

    for field in (
        "_maxExamples: Int", "_maxDiscards: Int", "_maxShrinks: Int",
        "_maxChoices: Int", "_seed: Option<Int>", "_database: Option<ExampleDatabase>",
        "_phases: List<Phase>", "_statefulSteps: Int", "_deadline: Option<Any>",
    ):
        require(settings, field, "settings.ph")

    require(settings, "Settings.new(", "settings standard")
    for literal in ("maxExamples: 100", "maxDiscards: 1000", "maxShrinks: 1000", "maxChoices: 10000", "statefulSteps: 50"):
        require(settings, literal, "settings standard")
    for updater in ("maxExamples(value: Int)", "maxDiscards(value: Int)", "maxShrinks(value: Int)", "maxChoices(value: Int)", "phases(value: List<Phase>)", "statefulSteps(value: Int)"):
        require(settings, updater, "settings updater")
    if settings.count("@requires") < 6:
        fail("settings.ph: expected contracts on all bounded updates")

    for field in ("_errorType: Class", "_module: Symbol", "_selector: Symbol", "_line: Int", "_column: Int", "_label: Option<Symbol>"):
        require(failure, field, "failure origin")
    require(failure, "sameSite(other: FailureOrigin)", "failure origin")

    for field in ("_validExamples: Int", "_discardedExamples: Int", "_successfulShrinks: Int", "_replayedExamples: Int", "_events: Map<Symbol, Int>"):
        require(statistics, field, "statistics")
    require(statistics, "class _StatisticsCollector", "statistics collector")
    require(statistics, "snapshot -> Statistics", "statistics collector")

    require(context, "class _PropertyContext", "context")
    require(context, "class _PropertyContextStack", "context")
    require(context, "}.ensure {", "context cleanup")
    require(errors, "class _HypothesisError", "errors")

    semantic_string = re.compile(r'\b(?:_kind|_status|kind|status)\s*=\s*"[^"]+"')
    public_core_classes = {
        "Settings", "Phase", "ExampleStatus", "PropertyResult",
        "FailureOrigin", "Failure", "Statistics",
        "StrategyResolutionError", "PropertyDiscoveryError", "ReporterFailure",
    }
    for path in sorted((ROOT / "src/core").glob("*.ph")):
        source = path.read_text(encoding="utf-8")
        if semantic_string.search(source):
            fail(f"free-form semantic string tag in {path.relative_to(ROOT)}")

        for class_name in re.findall(r"(?m)^class\s+([A-Za-z_][A-Za-z0-9_]*)", source):
            if class_name not in public_core_classes and not class_name.startswith("_"):
                fail(f"internal core class is not private-prefixed: {class_name}")
        for const_name in re.findall(r"(?m)^const\s+([A-Za-z_][A-Za-z0-9_]*)\s*=", source):
            if not const_name.startswith("_"):
                fail(f"internal core constant is not private-prefixed: {const_name}")


def check_facade_and_adapter() -> None:
    facade = read("src/hypothesis.ph")
    for import_line in (
        'import coreSettings from "core/settings"',
        'import corePhase from "core/phase"',
        'import coreStatus from "core/status"',
        'import coreFailure from "core/failure"',
        'import coreStatistics from "core/statistics"',
    ):
        require(facade, import_line, "root facade")
    for alias in (
        "const Settings = coreSettings.Settings",
        "const Phase = corePhase.Phase",
        "const PropertyResult = coreStatus.PropertyResult",
        "const Failure = coreFailure.Failure",
        "const Statistics = coreStatistics.Statistics",
        "const CheckConfig = coreSettings.Settings",
    ):
        require(facade, alias, "root facade")
    if (ROOT / "src/_internal/legacy_adapter.ph").exists() or (ROOT / "src/_internal/phase01_surface.ph").exists():
        fail("final release still contains compatibility implementation modules")
    engine = read("src/engine/engine.ph")
    require(engine, "status.overrun", "authoritative overrun classification")
    require(engine, "PropertyResult.errored", "authoritative overrun result")
    require(engine, "PropertyResult.inconclusive", "authoritative invalid-result classification")
    console = read("src/reporting/console.ph")
    require(console, "INCONCLUSIVE ", "authoritative reporter")
    require(console, "ERROR ", "authoritative reporter")
    require(console, "result.match(", "authoritative reporter")
    require(engine, "verifyFailure(", "authoritative final verification")
    require(engine, "shrinkFailure(", "authoritative structural shrinking")


def main() -> int:
    checks = [
        ("Phase 02 tests exist", check_tests_first),
        ("typed immutable core model", check_core_model),
        ("facade and final release integration", check_facade_and_adapter),
    ]
    failures: list[tuple[str, str]] = []
    for name, check in checks:
        try:
            check()
        except Exception as error:  # noqa: BLE001
            failures.append((name, str(error)))
            print(f"FAIL {name}")
            print(f"  {error}")
        else:
            print(f"PASS {name}")
    print()
    print(f"{len(checks) - len(failures)} passed, {len(failures)} failed")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
