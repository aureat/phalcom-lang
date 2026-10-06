#!/usr/bin/env python3
"""Observed source/static verification for Phase 07 reporting."""
from __future__ import annotations
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(source: str, needle: str, where: str) -> None:
    if needle not in source:
        raise AssertionError(f"{where}: missing {needle!r}")


def reject(source: str, needle: str, where: str) -> None:
    if needle in source:
        raise AssertionError(f"{where}: forbidden {needle!r}")


def check_tests_and_goldens() -> None:
    required = {
        "tests/reporting/event_ordering.ph",
        "tests/reporting/failure_notes.ph",
        "tests/reporting/event_statistics.ph",
        "tests/reporting/health_and_flaky.ph",
        "tests/reporting/json_schema.ph",
        "tests/reporting/reproduction_token.ph",
        "tests/reporting/targeting_removed.ph",
        "tests/golden/reporting/pass.txt",
        "tests/golden/reporting/failure.txt",
        "tests/golden/reporting/health.txt",
        "tests/golden/reporting/flaky.txt",
        "tests/golden/reporting/property.jsonl",
    }
    missing = sorted(path for path in required if not (ROOT / path).is_file())
    if missing:
        raise AssertionError("missing Phase 07 tests/goldens: " + ", ".join(missing))
    require(read("tests/reporting/event_ordering.ph"), "RecordingReporter", "event ordering test")
    require(read("tests/reporting/failure_notes.ph"), "Property.note", "failure notes test")
    require(read("tests/reporting/event_statistics.ph"), "Property.classify", "event statistics test")
    require(read("tests/reporting/reproduction_token.ph"), "Reproduction.replay", "reproduction test")
    require(read("tests/reporting/targeting_removed.ph"), "respondsTo(#target)", "targeting decision test")


def check_event_model() -> None:
    event = read("src/reporting/event.ph")
    reporter = read("src/reporting/reporter.ph")
    for marker in ("@data", "@immutable", "@sealed", "class ReportEvent"):
        require(event, marker, "report event")
    for variant in (
        "SuiteStarted", "PropertyStarted", "PhaseStarted",
        "ExampleAccepted", "ExampleRejected", "FailureFound",
        "ShrinkAccepted", "HealthCheckFailed", "PropertyFinished", "SuiteFinished",
    ):
        require(event, f"@variant {variant}", "report event")
    require(reporter, "protocol Reporter", "reporter protocol")
    require(reporter, "handle(event: ReportEvent) -> None", "reporter protocol")
    for name in ("NullReporter", "RecordingReporter", "CompositeReporter"):
        require(reporter, f"class {name}", "reporter implementations")
    reject(event, "Phase 01 module boundary", "report event")
    reject(reporter, "Phase 01 module boundary", "reporter")


def check_observations() -> None:
    builder = read("src/property/builder.ph")
    context = read("src/core/context.ph")
    failure = read("src/core/failure.ph")
    statistics = read("src/core/statistics.ph")
    evaluator = read("src/engine/evaluator.ph")
    require(builder, "Property.current.note(value)", "Property.note")
    require(builder, "Property.current.event(label)", "Property.event")
    require(builder, "if condition {", "Property.classify")
    require(builder, "Property.event(as)", "Property.classify")
    reject(builder, "target(score: Number", "public targeting")
    reject(context, "_targets", "property context targeting")
    require(failure, "notes: List<Any>", "failure note capture")
    require(evaluator, "notes: context.notes", "evaluator note capture")
    require(statistics, "recordFailure(context", "failure observation statistics")
    require(statistics, "eventCounts", "statistics event accessor")


def check_engine_delivery() -> None:
    engine = read("src/engine/engine.ph")
    shrinker = read("src/engine/shrinker.ph")
    runner = read("src/property/runner.ph")
    require(engine, 'import reportingEvent from "reporting/event"', "engine event import")
    require(engine, 'import reportingReporter from "reporting/reporter"', "engine reporter import")
    require(engine, "check(spec: Any, reporter: Any)", "reporter-aware engine")
    require(engine, "ReportEvent.phaseStarted", "phase events")
    require(engine, "ReportEvent.exampleAccepted", "accepted example events")
    require(engine, "ReportEvent.exampleRejected", "rejected example events")
    require(engine, "ReportEvent.failureFound", "failure events")
    require(engine, "ReportEvent.healthCheckFailed", "health events")
    require(shrinker, "reporter: Any", "reporter-aware shrinker")
    require(shrinker, "ReportEvent.shrinkAccepted", "shrink events")
    require(runner, "reporter: Reporter", "reporter-aware runner")
    require(runner, "ReportEvent.suiteStarted", "suite lifecycle")
    require(runner, "ReportEvent.propertyStarted", "property lifecycle")
    require(runner, "ReportEvent.propertyFinished", "property lifecycle")
    require(runner, "ReportEvent.suiteFinished", "suite lifecycle")
    require(runner, "_settings: Settings", "PropertyRun settings")


def check_reproduction() -> None:
    source = read("src/reporting/reproduction.ph")
    for marker in ("@data", "@immutable", "class ReproductionToken"):
        require(source, marker, "reproduction token")
    for field in ("_propertyId", "_example: Example", "_settings: Settings", "_text: String"):
        require(source, field, "reproduction token")
    require(source, '"phalcom-hypothesis:v1:"', "token version")
    require(source, "class Reproduction", "reproduction service")
    require(source, "Phase.Reuse", "reuse-only replay")
    require(source, "SearchEngine.new().check", "authoritative replay")
    require(source, "run.explicitFailure", "explicit-case exclusion")
    reject(source, "Phase 01 module boundary", "reproduction")


def check_renderers() -> None:
    console = read("src/reporting/console.ph")
    json = read("src/reporting/json.ph")
    require(console, "class ConsoleReporter", "console reporter")
    for text in ("Falsifying example:", "Notes:", "Observations:", "HEALTH CHECK", "FLAKY", "Reproduce:"):
        require(console, text, "console reporter")
    require(console, "class PropertyReporter", "compatibility reporter factory")
    require(json, "class JsonReporter", "JSON reporter")
    require(json, '"schemaVersion"', "JSON schema")
    require(json, '"type"', "JSON schema")
    require(json, "jsonLines", "JSON lines")
    require(json, "escape(value: String)", "JSON escaping")
    reject(console, "Phase 01 module boundary", "console reporter")
    reject(json, "Phase 01 module boundary", "JSON reporter")


def check_facade_and_migration() -> None:
    facade = read("src/hypothesis.ph")
    manifest = read("phalcom.toml")
    for module in (
        'import reportingEvent from "reporting/event"',
        'import reportingReporter from "reporting/reporter"',
        'import reportingConsole from "reporting/console"',
        'import reportingJson from "reporting/json"',
        'import reportingReproduction from "reporting/reproduction"',
    ):
        require(facade, module, "reporting façade")
    for alias in (
        "const ReportEvent = reportingEvent.ReportEvent",
        "const Reporter = reportingReporter.Reporter",
        "const ConsoleReporter = reportingConsole.ConsoleReporter",
        "const PropertyReporter = reportingConsole.PropertyReporter",
        "const JsonReporter = reportingJson.JsonReporter",
        "const ReproductionToken = reportingReproduction.ReproductionToken",
    ):
        require(facade, alias, "reporting façade")
    if (ROOT / "src/_internal/legacy_adapter.ph").exists() or (ROOT / "src/_internal/phase01_surface.ph").exists():
        raise AssertionError("final release still contains compatibility implementation modules")
    version = re.search(r'version = "0\.1\.0-phase\.(\d+)"', manifest)
    if 'version = "0.1.0"' not in manifest and (not version or int(version.group(1)) < 7):
        raise AssertionError("manifest version: expected Phase 07 or later")

def check_imports_privacy_and_placeholders() -> None:
    public = {
        "ReportEvent", "Reporter", "NullReporter", "RecordingReporter",
        "CompositeReporter", "ConsoleReporter", "PropertyReporter",
        "JsonReporter", "ReproductionToken", "Reproduction",
    }
    for path in sorted((ROOT / "src/reporting").glob("*.ph")):
        source = path.read_text(encoding="utf-8")
        reject(source, "Phase 01 module boundary", str(path.relative_to(ROOT)))
        for class_name in re.findall(r"(?m)^class\s+([A-Za-z_][A-Za-z0-9_]*)", source):
            if class_name not in public and not class_name.startswith("_"):
                raise AssertionError(f"internal reporting class is not private-prefixed: {class_name}")
    missing: list[str] = []
    for path in sorted((ROOT / "src").rglob("*.ph")):
        source = path.read_text(encoding="utf-8")
        for module in re.findall(r'(?m)^import\s+[^\n]+\s+from\s+"([^"]+)"', source):
            target = ROOT / "src" / f"{module}.ph"
            if not target.exists():
                missing.append(f"{path.relative_to(ROOT)} -> {module}")
    if missing:
        raise AssertionError("missing internal imports: " + ", ".join(missing))


def main() -> int:
    checks = [
        ("Phase 07 tests and goldens", check_tests_and_goldens),
        ("typed report event and reporter model", check_event_model),
        ("notes, events, classifications, and targeting decision", check_observations),
        ("engine and runner event delivery", check_engine_delivery),
        ("reproduction tokens", check_reproduction),
        ("console and JSON renderers", check_renderers),
        ("root façade and reporting ownership migration", check_facade_and_migration),
        ("reporting privacy, imports, and placeholder removal", check_imports_privacy_and_placeholders),
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
