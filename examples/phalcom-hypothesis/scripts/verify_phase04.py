#!/usr/bin/env python3
"""Observed static acceptance gate for Phase 04 strategy ownership."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(relative: str) -> str:
    path = ROOT / relative
    if not path.exists():
        raise AssertionError(f"missing {relative}")
    return path.read_text(encoding="utf-8")


def require(source: str, needle: str, where: str) -> None:
    if needle not in source:
        raise AssertionError(f"missing {needle!r} in {where}")


def reject(source: str, needle: str, where: str) -> None:
    if needle in source:
        raise AssertionError(f"unexpected {needle!r} in {where}")


def check_tests_first() -> None:
    required = [
        "tests/strategies/primitives.ph",
        "tests/strategies/combinators.ph",
        "tests/strategies/collections.ph",
        "tests/strategies/composite_recursive.ph",
        "tests/strategies/registry.ph",
        "tests/strategies/invalid_construction.ph",
        "tests/strategies/deterministic_replay.ph",
    ]
    for path in required:
        source = read(path)
        require(source, "Phase 04", path)
        require(source, "System.print(\"PASS", path)

    replay = read("tests/strategies/deterministic_replay.ph")
    for api in (
        "Gen.int", "Gen.bool", "Gen.float", "Gen.bytes", "Gen.text",
        "Gen.just", "Gen.sampledFrom", "Gen.oneOf", "Gen.option",
        "Gen.result", "Gen.list", "Gen.set", "Gen.map", "Gen.tuple",
        "Gen.build", "Gen.deferred", "Gen.recursive",
    ):
        require(replay, api, "standard-strategy replay fixture")
    require(replay, "DrawData.generate", "standard-strategy replay fixture")
    require(replay, "DrawData.replay", "standard-strategy replay fixture")
    require(replay, "Assert.equal(generatedExample, replayed.example)", "normalized strategy replay")


def check_strategy_protocol_and_combinators() -> None:
    strategy = read("src/strategies/strategy.ph")
    combinators = read("src/strategies/combinators.ph")
    errors = read("src/core/errors.ph")
    data = read("src/choices/data.ph")

    require(strategy, "protocol Strategy<out T>", "strategy protocol")
    for signature in (
        "draw(data: DrawData) -> T",
        "map<U>(transform: [T] -> U) -> Strategy<U>",
        "filter(predicate: [T] -> Bool) -> Strategy<T>",
        "flatMap<U>(transform: [T] -> Strategy<U>) -> Strategy<U>",
        "named(label: Symbol) -> Strategy<T>",
        "fingerprint -> String",
    ):
        require(strategy, signature, "strategy protocol")

    for class_name in (
        "StrategyBase",
        "_JustStrategy",
        "_MappedStrategy",
        "_FilteredStrategy",
        "_FlatMappedStrategy",
        "_NamedStrategy",
        "_OneOfStrategy",
    ):
        require(combinators, f"class {class_name}", "combinators")
    require(combinators, "data.recordRejection", "filter rejection accounting")
    require(combinators, "errors._InvalidStrategy", "flatMap validation")
    require(combinators, "data.withSpan", "named strategy span")
    require(errors, "class _InvalidStrategy", "strategy error taxonomy")
    require(errors, "class _RejectedExample", "filter rejection taxonomy")
    require(data, "rejectionCount -> Int", "DrawData rejection count")
    require(data, "recordRejection", "DrawData rejection accounting")
    require(data, "ExampleStatus.invalid", "DrawData rejection classification")


def check_standard_strategies() -> None:
    primitives = read("src/strategies/primitives.ph")
    collections = read("src/strategies/collections.ph")
    composite = read("src/strategies/composite.ph")
    gen = read("src/strategies/gen.ph")

    for class_name in (
        "_IntStrategy",
        "_BoolStrategy",
        "_FloatStrategy",
        "_BytesStrategy",
        "_TextStrategy",
        "_SampledFromStrategy",
    ):
        require(primitives, f"class {class_name}", "primitive strategies")
    for draw in ("drawInt(", "drawBool(", "drawBytes(", "drawIndex("):
        require(primitives, draw, "primitive typed draws")
    require(primitives, "scale -> Int => 1000000", "deterministic float encoding")

    for class_name in (
        "_ListStrategy",
        "_SetStrategy",
        "_MapStrategy",
        "_TupleStrategy",
        "_OptionStrategy",
        "_ResultStrategy",
    ):
        require(collections, f"class {class_name}", "collection strategies")
    require(collections, "label: #element", "list element spans")
    require(collections, "discardable: true", "discardable list elements")
    require(collections, "data.withSpan(label: #list", "list root span")

    for class_name in (
        "Draw",
        "_BuildStrategy",
        "_DeferredStrategy",
        "_RecursiveStrategy",
        "_SizedStrategy",
    ):
        require(composite, f"class {class_name}", "composite strategies")
    require(composite, "if data.size == 0", "recursive size-zero base")
    require(composite, "withGenerationSize", "recursive size reduction")

    require(gen, "class Gen", "Gen façade")
    for api in (
        "int -> Strategy<Int>", "bool -> Strategy<Bool>",
        "float -> Strategy<Float>", "bytes -> Strategy<Bytes>",
        "text -> Strategy<String>", "just<T>", "sampledFrom<T>",
        "oneOf<T>", "option<T>", "result<T, E>", "list<T>",
        "set<T>", "map<K, V>", "tuple(", "build<T>",
        "deferred<T>", "recursive<T>",
    ):
        require(gen, api, "Gen API")


def check_registry_and_facade() -> None:
    registry = read("src/strategies/registry.ph")
    facade = read("src/hypothesis.ph")
    manifest = read("phalcom.toml")

    require(registry, "class StrategyRegistry", "registry")
    require(registry, "standard -> StrategyRegistry", "registry defaults")
    for built_in in ("Int", "Bool", "Float", "Bytes", "String"):
        require(registry, built_in, "registry built-ins")
    for alias in (
        "const Strategy = strategyModel.Strategy",
        "const StrategyBase = strategyCombinators.StrategyBase",
        "const Gen = strategyGen.Gen",
        "const StrategyRegistry = strategyRegistry.StrategyRegistry",
    ):
        require(facade, alias, "strategy facade")
    if "0.1.0" not in facade:
        raise AssertionError("root facade does not identify the final release")
    phase_version = re.search(r'version = "0\.1\.0-phase\.(\d+)"', manifest)
    if 'version = "0.1.0"' not in manifest and (not phase_version or int(phase_version.group(1)) < 4):
        raise AssertionError("manifest version: expected Phase 04 or later")
    if (ROOT / "src/_internal/legacy_adapter.ph").exists():
        raise AssertionError("final release still contains the legacy adapter")
    stateful = read("src/stateful/runner.ph")
    require(stateful, "class _StatefulScenarioStrategy extends strategyCombinators.StrategyBase", "authoritative stateful strategy")


def check_privacy_and_no_placeholders() -> None:
    public = {"Strategy", "StrategyBase", "Gen", "StrategyRegistry", "Draw", "arbitrary", "strategy"}
    for path in sorted((ROOT / "src/strategies").glob("*.ph")):
        source = path.read_text(encoding="utf-8")
        reject(source, "Phase 01 scaffold placeholder", str(path.relative_to(ROOT)))
        for class_name in re.findall(r"(?m)^class\s+([A-Za-z_][A-Za-z0-9_]*)", source):
            if class_name not in public and not class_name.startswith("_"):
                raise AssertionError(
                    f"internal strategy class is not private-prefixed: {class_name}"
                )


def main() -> int:
    checks = [
        ("Phase 04 tests exist", check_tests_first),
        ("typed strategy protocol and combinators", check_strategy_protocol_and_combinators),
        ("standard, composite, and recursive strategies", check_standard_strategies),
        ("registry and final façade ownership", check_registry_and_facade),
        ("strategy privacy and placeholder removal", check_privacy_and_no_placeholders),
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
