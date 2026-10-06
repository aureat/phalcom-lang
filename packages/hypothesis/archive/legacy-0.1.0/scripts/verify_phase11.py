#!/usr/bin/env python3
"""Observed source/static verification for Phase 11 extension hardening."""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

DEFAULT_ROOT = Path(__file__).resolve().parents[1]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=DEFAULT_ROOT)
    return parser.parse_args()


def verifier(root: Path) -> int:
    def read(path: str) -> str:
        return (root / path).read_text(encoding="utf-8")

    def require(source: str, marker: str, context: str) -> None:
        if marker not in source:
            raise AssertionError(f"{context}: missing {marker!r}")

    def reject(source: str, marker: str, context: str) -> None:
        if marker in source:
            raise AssertionError(f"{context}: unexpected {marker!r}")

    def check_contract_files() -> None:
        required = {
            "tests/conformance/provider.ph",
            "tests/conformance/strategy.ph",
            "tests/conformance/shrink_pass.ph",
            "tests/conformance/database.ph",
            "tests/conformance/reporter.ph",
            "tests/integration/provider_equivalence.ph",
            "tests/regression/duplicate_shrink_candidates.ph",
            "tests/regression/span_stack_linear.ph",
            "tests/regression/database_merge_on_write.ph",
            "tests/regression/database_signature_roundtrip.ph",
            "tests/regression/reporter_failure_boundary.ph",
            "benchmarks/primitive_generation.ph",
            "benchmarks/nested_lists.ph",
            "benchmarks/replay.ph",
            "benchmarks/integer_shrinking.ph",
            "benchmarks/stateful_shrinking.ph",
            "benchmarks/README.md",
            "scripts/verify_phase11_mutations.py",
        }
        missing = sorted(path for path in required if not (root / path).is_file())
        if missing:
            raise AssertionError("missing Phase 11 contracts: " + ", ".join(missing))
        require(read("tests/integration/provider_equivalence.ph"), "systemData.example.choices", "provider equivalence")
        require(read("tests/regression/duplicate_shrink_candidates.ph"), "DuplicatePass", "shrink regression")
        require(read("tests/regression/database_signature_roundtrip.ph"), "record.signature", "database signature regression")
        require(read("benchmarks/README.md"), "No benchmark timing is simulated", "benchmark boundary")

    def check_providers() -> None:
        source = read("src/choices/provider.ph")
        for marker in (
            "protocol ChoiceProvider",
            "protocol ChoiceProviderFactory",
            "class SystemRandomChoiceProvider",
            "class ScriptedChoiceProvider",
            "class SystemRandomProviderFactory",
            "class ScriptedProviderFactory",
            "class _ChoiceNormalization",
            "_ChoiceNormalization.normalize",
            "_ScriptedProviderExhausted",
        ):
            require(source, marker, "choice providers")
        data = read("src/choices/data.ph")
        require(data, "generate(provider: ChoiceProvider", "DrawData provider overload")
        engine = read("src/engine/engine.ph")
        require(engine, "resolvedChoiceProviderFactory", "engine provider factory")
        require(engine, "factory.create", "fresh provider per example")
        settings = read("src/core/settings.ph")
        require(settings, "choiceProvider(factory: ChoiceProviderFactory)", "settings provider factory")

    def check_strategy_and_shrink_extensions() -> None:
        strategy = read("src/strategies/strategy.ph")
        combinators = read("src/strategies/combinators.ph")
        shrinker = read("src/engine/shrinker.ph")
        shrink_pass = read("src/engine/shrink_pass.ph")
        require(strategy, "protocol Strategy<out T>", "strategy protocol")
        require(combinators, "class StrategyBase<T>", "public strategy base")
        require(shrink_pass, "protocol ShrinkPass", "shrink-pass protocol")
        require(shrinker, "new(passes: List<ShrinkPass>)", "typed shrink pipeline")
        require(shrinker, "seenSignatures", "duplicate candidate suppression")
        require(shrinker, "proposal.signature", "candidate signature dedupe")
        reject(combinators, "class _StrategyBase<T>", "retired private strategy base")

    def check_database_and_reporter_hardening() -> None:
        directory = read("src/database/directory.ph")
        reporter = read("src/reporting/reporter.ph")
        engine = read("src/engine/engine.ph")
        errors = read("src/core/errors.ph")
        for marker in (
            "class _DirectoryLockTable",
            "withPathLock",
            "self._records(key)",
            "merge-on-write",
        ):
            require(directory, marker, "directory hardening")
        for marker in (
            "class _CheckedReporter",
            "ReporterFailure",
            "extension failure",
        ):
            require(reporter + "\n" + errors, marker, "reporter boundary")
        require(engine, "_CheckedReporter.new", "engine checked reporter")
        require(reporter, "for reporter in _reporters", "composite forwarding")

    def check_performance_fixes() -> None:
        buffer = read("src/choices/buffer.ph")
        data = read("src/choices/data.ph")
        example = read("src/choices/example.ph")
        database = read("src/database/database.ph")
        codec = read("src/database/codec.ph")
        require(buffer, "_openSpans.removeAt(_openSpans.size - 1)", "span stack pop")
        require(buffer, "_closedSpans.at(id, put: Some.new(closed))", "linear span ordering")
        require(data, "_labelStack.removeAt(_labelStack.size - 1)", "label stack pop")
        require(data, "_sizeStack.removeAt(_sizeStack.size - 1)", "size stack pop")
        require(example, "parts.join", "example signature builder")
        require(database, "parts.join", "database signature builder")
        require(codec, "databaseModel._DatabaseSignatures.example(example)", "database signature validation")
        reject(buffer, "while id < _nextSpanId {\n      for span in _spans", "quadratic span ordering")

    def check_public_api_docs_version() -> None:
        facade = read("src/hypothesis.ph")
        docs = read("docs/extension-api.md")
        design = read("docs/design/phase-11-hardening.md")
        manifest = read("phalcom.toml")
        for marker in (
            "const ChoiceProvider =",
            "const ChoiceProviderFactory =",
            "const SystemRandomChoiceProvider =",
            "const ScriptedChoiceProvider =",
            "const StrategyBase =",
            "const ShrinkPass =",
            "const Shrinker =",
            "const ReporterFailure =",
        ):
            require(facade, marker, "root facade")
        for marker in (
            "Provider contract",
            "Strategy contract",
            "Shrink-pass contract",
            "Database contract",
            "Reporter contract",
            "Conformance matrix",
            "Performance boundary",
        ):
            require(docs, marker, "extension docs")
        require(design, "Phase 11 Extension Hardening Design", "phase design")
        if 'version = "0.1.0"' not in manifest:
            require(manifest, 'version = "0.1.0-phase.11"', "manifest version")

    def check_imports_privacy_and_placeholders() -> None:
        missing: list[str] = []
        for path in sorted((root / "src").rglob("*.ph")):
            source = path.read_text(encoding="utf-8")
            for module in re.findall(r'(?m)^import\s+[^\n]+\s+from\s+"([^"]+)"', source):
                if not (root / "src" / f"{module}.ph").exists():
                    missing.append(f"{path.relative_to(root)} -> {module}")
        if missing:
            raise AssertionError("missing internal imports: " + ", ".join(missing))
        reject(read("docs/extension-api.md"), "remain Phase 11 work", "stale Phase 11 deferral")
        reject(read("README.md"), "remain Phase 11 work", "stale README deferral")
        reject(read("src/hypothesis.ph"), "Stable root façade for the Phase 10 package", "stale facade phase")

    checks = [
        ("Phase 11 contracts exist", check_contract_files),
        ("choice providers and engine injection", check_providers),
        ("strategy and shrink-pass extensions", check_strategy_and_shrink_extensions),
        ("database and reporter hardening", check_database_and_reporter_hardening),
        ("performance corrections", check_performance_fixes),
        ("public API docs and version", check_public_api_docs_version),
        ("imports privacy and placeholders", check_imports_privacy_and_placeholders),
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


def main() -> int:
    args = parse_args()
    return verifier(args.root.resolve())


if __name__ == "__main__":
    sys.exit(main())
