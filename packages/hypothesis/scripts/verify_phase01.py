#!/usr/bin/env python3
"""Observed Phase 01 compatibility verification for the final package."""
from __future__ import annotations

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]

STABLE_NAMES = {
    "Given", "GivenArgs", "Case", "WithSettings", "Settings", "Phase", "Strategy", "Gen",
    "StrategyRegistry", "arbitrary", "strategy", "Property", "PropertyBuilder", "PropertySuite", "Assert",
    "PropertyResult", "PropertyId", "Failure", "Statistics",
    "PropertyAssertionError", "StrategyResolutionError", "PropertyDiscoveryError",
    "DatabaseKey", "ExampleDatabase", "MemoryDatabase", "DirectoryDatabase", "StateMachine", "Stateful", "Rule",
    "Initialize", "StateInvariant", "When", "Teardown", "Bundle", "Reporter", "ConsoleReporter", "JsonReporter",
    "Choice", "Example", "PropertyRun", "PropertySuiteResult", "ReportEvent", "NullReporter", "RecordingReporter",
    "CompositeReporter", "ReproductionToken", "Reproduction",
}
COMPATIBILITY_NAMES = {
    "Check", "CheckConfig", "PropertyRunner", "PropertyReporter", "RuleBasedStateMachine", "Invariant",
}
PLANNED_SOURCE_FILES = {
    "src/hypothesis.ph",
    "src/core/errors.ph", "src/core/settings.ph", "src/core/phase.ph", "src/core/status.ph",
    "src/core/failure.ph", "src/core/statistics.ph", "src/core/context.ph",
    "src/choices/choice.ph", "src/choices/request.ph", "src/choices/span.ph", "src/choices/example.ph",
    "src/choices/buffer.ph", "src/choices/provider.ph", "src/choices/data.ph",
    "src/strategies/strategy.ph", "src/strategies/combinators.ph", "src/strategies/primitives.ph",
    "src/strategies/collections.ph", "src/strategies/composite.ph", "src/strategies/attributes.ph",
    "src/strategies/derivation.ph", "src/strategies/registry.ph", "src/strategies/gen.ph",
    "src/engine/specification.ph", "src/engine/evaluator.ph", "src/engine/complexity.ph",
    "src/engine/shrink_pass.ph", "src/engine/shrinker.ph", "src/engine/search.ph", "src/engine/engine.ph",
    "src/property/attributes.ph", "src/property/assertion.ph", "src/property/target.ph",
    "src/property/inference.ph", "src/property/discovery.ph", "src/property/builder.ph", "src/property/runner.ph",
    "src/database/key.ph", "src/database/database.ph", "src/database/memory.ph", "src/database/codec.ph",
    "src/database/directory.ph", "src/stateful/attributes.ph", "src/stateful/machine.ph", "src/stateful/rule.ph",
    "src/stateful/bundle.ph", "src/stateful/argument.ph", "src/stateful/action.ph", "src/stateful/scenario.ph",
    "src/stateful/runner.ph", "src/reporting/event.ph", "src/reporting/reporter.ph", "src/reporting/console.ph",
    "src/reporting/json.ph", "src/reporting/reproduction.ph", "src/_internal/sequences.ph",
    "src/_internal/fingerprints.ph", "src/_internal/ordering.ph",
}


def fail(message: str) -> None:
    raise AssertionError(message)


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def facade_aliases(source: str) -> set[str]:
    return set(re.findall(r"(?m)^const\s+([A-Za-z_][A-Za-z0-9_]*)\s*=", source))


def imported_names(source: str) -> set[str]:
    return set(re.findall(r"(?m)^import\s+([A-Za-z_][A-Za-z0-9_]*)\s+from\s+hypothesis\s*$", source))


def strip_comments_and_strings(source: str) -> str:
    source = re.sub(r"//.*", "", source)
    return re.sub(r'"(?:\\.|[^"\\])*"', '""', source)


def assert_balanced(path: Path) -> None:
    source = strip_comments_and_strings(path.read_text(encoding="utf-8"))
    pairs = {"(": ")", "[": "]", "{": "}"}
    stack: list[str] = []
    for char in source:
        if char in pairs:
            stack.append(char)
        elif char in pairs.values():
            if not stack or pairs[stack.pop()] != char:
                fail(f"unbalanced delimiter in {path.relative_to(ROOT)}")
    if stack:
        fail(f"unclosed delimiter in {path.relative_to(ROOT)}")


def active_files() -> list[Path]:
    files: list[Path] = []
    for base in (ROOT / "src", ROOT / "tests/integration", ROOT / "examples"):
        files.extend(sorted(base.rglob("*.ph")))
    return files


def check_package_imports() -> None:
    missing = sorted(path for path in PLANNED_SOURCE_FILES if not (ROOT / path).is_file())
    if missing:
        fail("missing planned source files: " + ", ".join(missing))
    manifest = read("phalcom.toml")
    if 'name = "hypothesis"' not in manifest or 'root-module = "hypothesis"' not in manifest:
        fail("manifest does not expose hypothesis")
    facade = read("src/hypothesis.ph")
    expected = STABLE_NAMES | COMPATIBILITY_NAMES
    aliases = facade_aliases(facade)
    if expected - aliases:
        fail("missing façade aliases: " + ", ".join(sorted(expected - aliases)))
    imported = imported_names(read("tests/integration/package_loads.ph"))
    if expected - imported:
        fail("package import fixture lost Phase 01 names: " + ", ".join(sorted(expected - imported)))
    if "legacy_adapter" in facade or "phase01_surface" in facade:
        fail("final façade still depends on a compatibility implementation module")
    for path in active_files():
        assert_balanced(path)


def check_compatibility_acceptance() -> None:
    facade = read("src/hypothesis.ph")
    acceptance = read("tests/integration/current_acceptance.ph")
    migration = read("tests/integration/broad_v1_migration.ph")
    direct_aliases = (
        "const Check = propertyAttributes.WithSettings",
        "const CheckConfig = coreSettings.Settings",
        "const RuleBasedStateMachine = statefulMachine.StateMachine",
        "const Invariant = statefulAttributes.StateInvariant",
    )
    for marker in direct_aliases:
        if marker not in facade:
            fail(f"compatibility alias does not delegate directly: {marker}")
    if not (ROOT / "src/reporting/console.ph").read_text(encoding="utf-8").count("class PropertyReporter"):
        fail("compatibility reporter factory is missing")
    if "seed(20260723)" not in acceptance or "self.assertTrue(n < 10)" not in acceptance:
        fail("deterministic broad-v1 acceptance behavior was lost")
    if "PropertyReporter.console.report(run)" not in acceptance:
        fail("compatibility reporter is not exercised")
    if 'from "hypothesis"' not in migration or "if (" in migration:
        fail("broad-v1 migration fixture is not an executable current-syntax fixture")
    forbidden = {
        "retired constructor syntax": re.compile(r"\bconstruct\s+"),
        "parenthesized if": re.compile(r"\bif\s+\("),
        "parenthesized while": re.compile(r"\bwhile\s+\("),
        "parenthesized for": re.compile(r"\bfor\s+\("),
    }
    for path in active_files():
        source = path.read_text(encoding="utf-8")
        for label, pattern in forbidden.items():
            if pattern.search(source):
                fail(f"{label} found in {path.relative_to(ROOT)}")
    builder = read("src/property/builder.ph")
    context = read("src/core/context.ph")
    if "}.ensure {" not in builder + "\n" + context:
        fail("property-context cleanup is not structurally guaranteed")
    for marker in ("Property.current.note(value)", "Property.current.event(label)", "Property.event(as)"):
        if marker not in builder:
            fail(f"public observation is not connected: {marker}")
    if "target(score: Number" in builder:
        fail("public targeting remains exposed without an optimizer")


def main() -> int:
    checks = [
        ("package imports", check_package_imports),
        ("broad-v1 compatibility through authoritative aliases", check_compatibility_acceptance),
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
    print(f"{len(checks)-len(failures)} passed, {len(failures)} failed")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
