#!/usr/bin/env python3
"""Observed source/static verification for the Phase 12 final release."""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

PUBLIC_EXPORTS = {
    "Given", "GivenArgs", "Case", "WithSettings", "Settings", "Phase",
    "Choice", "ChoiceRequest", "Example", "DrawData", "ChoiceProvider",
    "ChoiceProviderFactory", "SystemRandomChoiceProvider", "ScriptedChoiceProvider",
    "SystemRandomProviderFactory", "ScriptedProviderFactory", "Strategy",
    "StrategyBase", "Gen", "StrategyRegistry", "arbitrary", "strategy",
    "Property", "PropertyBuilder", "PropertySuite", "Assert",
    "PropertyAssertionError", "PropertyRunner", "PropertyRun",
    "PropertySuiteResult", "PropertyResult", "PropertyId", "Failure",
    "Statistics", "StrategyResolutionError", "ReporterFailure",
    "PropertyDiscoveryError", "HealthCheckFailure", "FlakyFailure",
    "ShrinkPass", "Shrinker", "ReportEvent", "Reporter", "NullReporter",
    "RecordingReporter", "CompositeReporter", "ConsoleReporter",
    "PropertyReporter", "JsonReporter", "ReproductionToken", "Reproduction",
    "DatabaseKey", "ExampleDatabase", "MemoryDatabase", "DirectoryDatabase",
    "StateMachine", "Stateful", "Rule", "Initialize", "StateInvariant",
    "When", "Teardown", "Bundle", "Check", "CheckConfig",
    "RuleBasedStateMachine", "Invariant",
}


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def aliases(source: str) -> set[str]:
    return set(re.findall(r"(?m)^const\s+([A-Za-z_][A-Za-z0-9_]*)\s*=", source))


def imported_names(source: str) -> set[str]:
    return set(re.findall(r"(?m)^import\s+([A-Za-z_][A-Za-z0-9_]*)\s+from\s+hypothesis\s*$", source))


def check_release_metadata() -> None:
    manifest = read("phalcom.toml")
    require('version = "0.1.0"' in manifest, "manifest is not the final 0.1.0 release")
    require("phase." not in manifest, "prerelease phase marker remains in manifest")
    for path in (
        "docs/public-api.md",
        "docs/migration-from-monolith.md",
        "docs/design/phase-12-release.md",
        "tests/integration/release_facade.ph",
        "tests/integration/release_compatibility.ph",
        "tests/integration/release_examples.ph",
        "tests/integration/release_persistence.ph",
        "scripts/verify_release.py",
    ):
        require((ROOT / path).is_file(), f"missing release artifact: {path}")


def check_legacy_removed() -> None:
    forbidden = (
        "legacy",
        "src/_internal/legacy_adapter.ph",
        "src/_internal/phase01_surface.ph",
        "scripts/migrate_phase01.py",
        "tests/legacy",
    )
    present = [path for path in forbidden if (ROOT / path).exists()]
    require(not present, "historical implementation artifacts remain: " + ", ".join(present))
    facade = read("src/hypothesis.ph")
    require("legacy_adapter" not in facade, "root façade still imports the legacy adapter")
    require("phase01_surface" not in facade, "root façade still imports the Phase 01 surface")
    for path in (ROOT / "src").rglob("*.ph"):
        source = path.read_text(encoding="utf-8")
        require("_Legacy" not in source, f"legacy implementation name remains in {path.relative_to(ROOT)}")


def check_facade_inventory() -> None:
    facade = read("src/hypothesis.ph")
    actual = aliases(facade)
    require(actual == PUBLIC_EXPORTS, f"façade mismatch; missing={sorted(PUBLIC_EXPORTS-actual)}, extra={sorted(actual-PUBLIC_EXPORTS)}")
    imported = imported_names(read("tests/integration/package_loads.ph"))
    require(imported == PUBLIC_EXPORTS, f"package import mismatch; missing={sorted(PUBLIC_EXPORTS-imported)}, extra={sorted(imported-PUBLIC_EXPORTS)}")
    docs = read("docs/public-api.md")
    documented = set(re.findall(r"(?m)^- `([A-Za-z_][A-Za-z0-9_]*)`(?:\s|$)", docs))
    require(documented == PUBLIC_EXPORTS, f"public API docs mismatch; missing={sorted(PUBLIC_EXPORTS-documented)}, extra={sorted(documented-PUBLIC_EXPORTS)}")
    for marker in (
        "const Check = propertyAttributes.WithSettings",
        "const CheckConfig = coreSettings.Settings",
        "const RuleBasedStateMachine = statefulMachine.StateMachine",
        "const Invariant = statefulAttributes.StateInvariant",
    ):
        require(marker in facade, f"compatibility alias is not authoritative: {marker}")


def check_release_fixtures() -> None:
    facade = read("tests/integration/release_facade.ph")
    compatibility = read("tests/integration/release_compatibility.ph")
    examples = read("tests/integration/release_examples.ph")
    persistence = read("tests/integration/release_persistence.ph")
    require("PASS release facade" in facade, "release façade fixture lacks success sentinel")
    require("CheckConfig.standard" in compatibility and "RuleBasedStateMachine" in compatibility, "compatibility fixture is incomplete")
    for name in ("arithmetic", "codec", "collections", "derived_data", "parser_roundtrip", "recursive_expression", "stateful_database"):
        require(name in examples, f"example inventory omits {name}")
        require((ROOT / "examples" / f"{name}.ph").is_file(), f"missing executable example: {name}.ph")
    require("DirectoryDatabase" in persistence and "20260723" in persistence, "persistence fixture lacks deterministic cross-process contract")
    require((ROOT / "tests/integration/broad_v1_migration.ph").is_file(), "broad-v1 migration fixture was not preserved")


def check_source_hygiene() -> None:
    cache_artifacts = [path for path in ROOT.rglob("*") if "__pycache__" in path.parts or path.suffix == ".pyc"]
    require(not cache_artifacts, "Python cache artifacts remain in release tree")
    forbidden = {
        "retired constructor syntax": re.compile(r"\bconstruct\s+"),
        "parenthesized if": re.compile(r"\bif\s+\("),
        "parenthesized while": re.compile(r"\bwhile\s+\("),
        "parenthesized for": re.compile(r"\bfor\s+\("),
        "placeholder marker": re.compile(r"\b(?:TODO|TBD|placeholder implementation)\b", re.IGNORECASE),
    }
    active_roots = (ROOT / "src", ROOT / "examples", ROOT / "tests")
    failures: list[str] = []
    for base in active_roots:
        for path in sorted(base.rglob("*.ph")):
            source = path.read_text(encoding="utf-8")
            for label, pattern in forbidden.items():
                if pattern.search(source):
                    failures.append(f"{label}: {path.relative_to(ROOT)}")
    require(not failures, "source hygiene failures: " + "; ".join(failures))


def check_test_matrix() -> None:
    required_dirs = (
        "tests/core", "tests/choices", "tests/strategies", "tests/engine",
        "tests/property", "tests/reporting", "tests/database", "tests/stateful",
        "tests/conformance", "tests/regression", "tests/integration", "tests/golden",
    )
    for path in required_dirs:
        require((ROOT / path).is_dir(), f"missing release test category: {path}")
    release_script = read("scripts/verify_release.py")
    require("passes = 1 if args.single_pass else 2" in release_script, "release gate is not two-pass by default")
    require("verify_phase11_mutations.py" in release_script, "release gate omits mutation verification")
    require("range(1, 13)" in release_script, "release gate omits a phase verifier")


def check_docs_and_final_summary() -> None:
    readme = read("README.md")
    changelog = read("CHANGELOG.md")
    migration = read("docs/migration-from-monolith.md")
    release = read("docs/design/phase-12-release.md")
    for marker in ("Hypothesis for Phalcom", "phalcom test --all", "0.1.0"):
        require(marker in readme, f"README missing {marker!r}")
    require("## 0.1.0" in changelog, "CHANGELOG lacks final release section")
    for marker in ("CheckConfig", "WithSettings", "RuleBasedStateMachine", "StateMachine", "PropertyReporter", "ConsoleReporter"):
        require(marker in migration, f"migration notes omit {marker}")
    for marker in ("All tests passed.", "No legacy syntax found.", "No placeholder implementations found.", "All public façade imports resolved."):
        require(marker in release, f"release design omits required final summary line: {marker}")


def check_imports() -> None:
    missing: list[str] = []
    for path in sorted((ROOT / "src").rglob("*.ph")):
        source = path.read_text(encoding="utf-8")
        for module in re.findall(r'(?m)^import\s+[^\n]+\s+from\s+"([^"]+)"', source):
            if not (ROOT / "src" / f"{module}.ph").is_file():
                missing.append(f"{path.relative_to(ROOT)} -> {module}")
    require(not missing, "missing internal imports: " + ", ".join(missing))


def main() -> int:
    checks = [
        ("release metadata and artifacts", check_release_metadata),
        ("historical implementation removed", check_legacy_removed),
        ("public façade inventory", check_facade_inventory),
        ("release integration fixtures", check_release_fixtures),
        ("source hygiene", check_source_hygiene),
        ("complete test matrix", check_test_matrix),
        ("release documentation", check_docs_and_final_summary),
        ("internal imports", check_imports),
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
