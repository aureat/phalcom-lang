#!/usr/bin/env python3
"""Observed source/static verification for Phase 09 stateful testing."""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(source: str, marker: str, context: str) -> None:
    if marker not in source:
        raise AssertionError(f"{context}: missing {marker!r}")


def reject(source: str, marker: str, context: str) -> None:
    if marker in source:
        raise AssertionError(f"{context}: unexpected {marker!r}")


def require_order(source: str, markers: tuple[str, ...], context: str) -> None:
    positions = [source.find(marker) for marker in markers]
    if any(position < 0 for position in positions):
        missing = [marker for marker, position in zip(markers, positions) if position < 0]
        raise AssertionError(f"{context}: missing ordered markers {missing!r}")
    if positions != sorted(positions):
        raise AssertionError(f"{context}: markers are out of order: {markers!r}")


def check_tests() -> None:
    names = {
        "broken_counter.ph", "when_applicability.ph", "initializer_order.ph",
        "bundle_publish.ph", "reference_resolution.ph", "consumed_bundle.ph",
        "invariant_policy.ph", "teardown_guarantees.ph",
        "middle_action_deletion.ph", "invalid_dependency_shrink.ph",
        "reproduction_names.ph", "persistence_replay.ph", "failure_origin.ph", "discovery_validation.ph",
        "compatibility_aliases.ph",
    }
    missing = sorted(name for name in names if not (ROOT / "tests/stateful" / name).is_file())
    if missing:
        raise AssertionError("missing Phase 09 fixtures: " + ", ".join(missing))
    require(read("tests/stateful/broken_counter.ph"), "normalActionCount", "counter fixture")
    require(read("tests/stateful/when_applicability.ph"), "discardedExamples", "applicability fixture")
    require(read("tests/stateful/middle_action_deletion.ph"), "#irrelevant", "middle deletion fixture")
    require(read("tests/stateful/reproduction_names.ph"), "key1 = state.createKey", "reproduction fixture")
    require(read("tests/stateful/persistence_replay.ph"), "replayedExamples", "persistence fixture")
    discovery = read("tests/stateful/discovery_validation.ph")
    for marker in ("MissingPredicateMachine", "PredicateArityMachine", "PredicateTypeMachine", "InvalidTargetMachine"):
        require(discovery, marker, "discovery validation fixture")
    teardown = read("tests/stateful/teardown_guarantees.ph")
    for marker in ("PassingTeardownMachine", "FailingTeardownMachine", "RejectingTeardownMachine", "ReplayInvalidationTeardownMachine", "InternalErrorTeardownMachine"):
        require(teardown, marker, "teardown fixture")


def check_attributes_and_bundles() -> None:
    attributes = read("src/stateful/attributes.ph")
    bundle = read("src/stateful/bundle.ph")
    for marker in (
        "class Rule extends Attribute", "class Initialize extends Attribute",
        "class StateInvariant extends Attribute", "class When extends Attribute",
        "class Teardown extends Attribute", "@On(Method)",
    ):
        require(attributes, marker, "stateful attributes")
    require(attributes, "passive reflected metadata", "attribute semantics")
    require(bundle, "class Bundle<T>", "bundle")
    for marker in ("select ->", "consume ->", "publish ->", "fingerprint -> String"):
        require(bundle, marker, "bundle")
    reject(bundle, "unavailable in Phase 01", "bundle")


def check_descriptive_model() -> None:
    argument = read("src/stateful/argument.ph")
    action = read("src/stateful/action.ph")
    rule = read("src/stateful/rule.ph")
    scenario = read("src/stateful/scenario.ph")
    for marker in ("@sealed", "class RuleArgument", "@variant Draw", "@variant Select", "@variant Consume"):
        require(argument, marker, "rule argument model")
    for marker in ("class ResultReference", "class LiteralArgument", "class ReferenceArgument"):
        require(argument, marker, "action argument model")
    for marker in ("class StateAction", "resultReference", "executableLine"):
        require(action, marker, "state action")
    for marker in ("class RuleDefinition", "class StateMachineMetadata", "fingerprint -> String"):
        require(rule, marker, "rule metadata")
    for marker in ("class StateScenario", "normalActionCount", "executable -> String", "selectors"):
        require(scenario, marker, "state scenario")
    for source, context in ((argument, "argument"), (action, "action"), (rule, "rule"), (scenario, "scenario")):
        reject(source, 'kind: String', context)
        reject(source, '== "rule"', context)


def check_discovery_and_execution() -> None:
    machine = read("src/stateful/machine.ph")
    runner = read("src/stateful/runner.ph")
    errors = read("src/core/errors.ph")
    for marker in (
        "class StateMachine", "class _StatefulDiscovery", "stableSelectors",
        "state machine has no @Rule methods", "duplicate or contradictory",
        "parameters.size", "validateWhenPredicate", "validateTargetType",
        "StateMachineMetadata.create",
    ):
        require(machine, marker, "stateful discovery")
    for marker in (
        "class _StatefulContext", "class _StatefulScenarioStrategy",
        "class _StatefulExecutor", "class _StatefulTarget", "class Stateful",
        "DrawData", "data.withSpan", "label: #stateAction", "discardable: true",
        "label: #stateSteps", "label: Some.new(#length)", "applicableRules", "if available.size == 0",
        "runInitializers", "runInvariants", "publish", "consumeReference",
        "executionResult", "teardownResult", "secondaryError",
        "SearchEngine.new().check", "DatabaseKey.create", "ExampleCodec.engineFormatVersion",
    ):
        require(runner, marker, "stateful execution")
    require_order(
        runner,
        ("self.runInitializers(data)", "self.runInvariants()", "label: #stateSteps"),
        "initializer/invariant/step order",
    )
    require_order(
        runner,
        ("_context.record(action)", "definition.method.invokeOn", "_context.publish"),
        "partial scenario and result publication order",
    )
    require_order(
        runner,
        ("const executionResult", "const teardownResult", "self.finish"),
        "structural teardown order",
    )
    for marker in ("class _InvalidStatefulReplay extends _EngineOverrun", "class _StatefulDiscoveryError"):
        require(errors, marker, "stateful errors")
    reject(runner, "prefix truncation", "stateful shrinking")
    reject(runner, "class _StatefulShrinker", "shared shrinker")


def check_failure_reporting_and_reproduction() -> None:
    runner = read("src/stateful/runner.ph")
    console = read("src/reporting/console.ph")
    json = read("src/reporting/json.ph")
    reproduction = read("src/reporting/reproduction.ph")
    for marker in ("class _StatefulFailure", "failureOrigin", "statefulScenario", "primaryError"):
        require(runner, marker, "stateful failure wrapper")
    for marker in ("Falsifying stateful scenario:", "statefulScenario.executable", "Secondary teardown failure"):
        require(console, marker, "console stateful rendering")
    require(json, '"statefulScenario"', "JSON stateful rendering")
    require(reproduction, "statefulExecutable", "reproduction integration")


def check_persistence_and_fingerprint() -> None:
    runner = read("src/stateful/runner.ph")
    rule = read("src/stateful/rule.ph")
    for marker in (
        "strategyFingerprint: metadata.fingerprint", "database.fetch", "database.save",
        "database.delete", "stale", "stateful-v1",
    ):
        require(runner + "\n" + rule, marker, "stateful persistence")
    require(rule, "argument.fingerprint", "stateful metadata fingerprint")
    require(rule, "target.fingerprint", "stateful metadata fingerprint")
    require(rule, "whenSelector", "stateful metadata fingerprint")


def check_facade_and_migration() -> None:
    facade = read("src/hypothesis.ph")
    manifest = read("phalcom.toml")
    for module in (
        'import statefulAttributes from "stateful/attributes"',
        'import statefulBundle from "stateful/bundle"',
        'import statefulMachine from "stateful/machine"',
        'import statefulRunner from "stateful/runner"',
    ):
        require(facade, module, "stateful façade")
    for alias in (
        "const StateMachine = statefulMachine.StateMachine",
        "const Stateful = statefulRunner.Stateful",
        "const Rule = statefulAttributes.Rule",
        "const Initialize = statefulAttributes.Initialize",
        "const StateInvariant = statefulAttributes.StateInvariant",
        "const When = statefulAttributes.When",
        "const Teardown = statefulAttributes.Teardown",
        "const Bundle = statefulBundle.Bundle",
        "const RuleBasedStateMachine = statefulMachine.StateMachine",
        "const Invariant = statefulAttributes.StateInvariant",
    ):
        require(facade, alias, "stateful façade")
    if (ROOT / "src/_internal/legacy_adapter.ph").exists() or (ROOT / "src/_internal/phase01_surface.ph").exists():
        raise AssertionError("final release still contains compatibility implementation modules")
    version = re.search(r'version = "0\.1\.0-phase\.(\d+)"', manifest)
    if 'version = "0.1.0"' not in manifest and (not version or int(version.group(1)) < 9):
        raise AssertionError("manifest version: expected Phase 09 or later")

def check_example_docs_privacy_imports() -> None:
    example = read("examples/stateful_database.ph")
    docs = read("docs/stateful.md")
    for marker in ("Bundle<Bytes>", "@Rule", "@When", "@StateInvariant", "@Teardown", "Stateful.check"):
        require(example, marker, "stateful database example")
    require(docs, "Rule-based stateful testing", "stateful docs")
    public = {
        "Rule", "Initialize", "StateInvariant", "When", "Teardown", "Bundle",
        "RuleArgument", "ResultReference", "LiteralArgument", "ReferenceArgument",
        "StateAction", "StateScenario", "RuleDefinition", "StateMachineMetadata",
        "StateMachine", "Stateful",
    }
    for path in sorted((ROOT / "src/stateful").glob("*.ph")):
        source = path.read_text(encoding="utf-8")
        reject(source, "Phase 01 module boundary", str(path.relative_to(ROOT)))
        for class_name in re.findall(r"(?m)^class\s+([A-Za-z_][A-Za-z0-9_]*)", source):
            if class_name not in public and not class_name.startswith("_"):
                raise AssertionError(f"internal stateful class is not private-prefixed: {class_name}")
    missing: list[str] = []
    for path in sorted((ROOT / "src").rglob("*.ph")):
        source = path.read_text(encoding="utf-8")
        for module in re.findall(r'(?m)^import\s+[^\n]+\s+from\s+"([^"]+)"', source):
            if not (ROOT / "src" / f"{module}.ph").exists():
                missing.append(f"{path.relative_to(ROOT)} -> {module}")
    if missing:
        raise AssertionError("missing internal imports: " + ", ".join(missing))


def main() -> int:
    checks = [
        ("Phase 09 tests exist", check_tests),
        ("passive attributes and typed bundles", check_attributes_and_bundles),
        ("immutable stateful descriptive model", check_descriptive_model),
        ("reflective discovery and incremental execution", check_discovery_and_execution),
        ("failure origin reporting and reproduction", check_failure_reporting_and_reproduction),
        ("stateful persistence and deterministic fingerprint", check_persistence_and_fingerprint),
        ("root facade and compatibility migration", check_facade_and_migration),
        ("example docs privacy imports and placeholders", check_example_docs_privacy_imports),
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
