#!/usr/bin/env python3
"""Observed Phase 06 static verification for environments without Phalcom."""

from __future__ import annotations

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(source: str, needle: str, context: str) -> None:
    if needle not in source:
        raise AssertionError(f"missing {needle!r} in {context}")


def reject(source: str, needle: str, context: str) -> None:
    if needle in source:
        raise AssertionError(f"unexpected {needle!r} in {context}")


def check_tests_first() -> None:
    required = [
        "tests/property/inferred_given.ph",
        "tests/property/inference_error.ph",
        "tests/property/explicit_arity.ph",
        "tests/property/named_overrides.ph",
        "tests/property/explicit_case_names.ph",
        "tests/property/builder_api.ph",
        "tests/property/assertion_origins.ph",
        "tests/property/runner_output.ph",
        "tests/property/settings_metadata.ph",
    ]
    for path in required:
        if not (ROOT / path).is_file():
            raise AssertionError(f"missing Phase 06 fixture: {path}")

    inferred = read(required[0])
    for annotation in ("Int", "Bool", "String", "Bytes", "Option<Int>", "List<Bool>", "Tuple<Int, String>"):
        require(inferred, annotation, "inference fixture")
    require(read(required[1]), "parameter 'value'", "missing annotation fixture")
    require(read(required[2]), "expected 2 strategies, received 1", "arity fixture")
    require(read(required[3]), ".for(#count, use:", "named override fixture")
    require(read(required[4]), "property.namedArguments", "explicit case fixture")
    require(read(required[4]), "property.explicitFailure", "explicit case fixture")
    require(read(required[5]), ".given(Gen.int", "builder fixture")
    require(read(required[5]), ".using(Settings.standard", "builder fixture")
    require(read(required[6]), "failureOrigin.sameSite", "assertion origin fixture")
    require(read(required[7]), '"2 passed, 0 failed"', "runner output fixture")
    require(read(required[8]), "@WithSettings(", "settings metadata fixture")
    require(read(required[8]), "definition.settings.maxExamples", "settings metadata fixture")


def check_attributes_and_inference() -> None:
    attributes = read("src/property/attributes.ph")
    inference = read("src/property/inference.ph")
    registry = read("src/strategies/registry.ph")
    errors = read("src/core/errors.ph")

    for name in ("class Given", "class GivenArgs", "class GivenMode", "class Case", "class WithSettings"):
        require(attributes, name, "property attributes")
    for variant in ("@variant Inferred", "@variant Explicit", "@variant Overrides"):
        require(attributes, variant, "Given modes")
    require(attributes, "duplicate override", "GivenArgs duplicate validation")
    require(attributes, "for(name: Symbol, use: Strategy<Any>)", "GivenArgs API")

    require(inference, "class _ReflectedParameter", "reflected parameter model")
    require(inference, "class _StrategyInference", "strategy inference")
    require(inference, "method.parameters", "parameter reflection")
    require(inference, "parameter.type", "type reflection")
    require(inference, "given.mode.match", "Given mode resolution")
    require(inference, "expected " + "", "arity diagnostics")
    require(inference, "unknown @Given override", "override diagnostics")

    require(registry, "type.origin", "applied type decomposition")
    require(registry, "type.arguments", "applied type arguments")
    for origin in ("Option", "List", "Tuple", "Set", "Map", "Result"):
        require(registry, f"origin == {origin}", "recursive registry")
    for constructor in ("Gen.option", "Gen.list", "Gen.tuple", "Gen.set", "Gen.map", "Gen.result"):
        require(registry, constructor, "recursive registry")

    require(errors, "class StrategyResolutionError", "public inference diagnostic")
    require(errors, "class PropertyDiscoveryError", "public discovery diagnostic")


def check_targets_assertions_and_builder() -> None:
    target = read("src/property/target.ph")
    assertion = read("src/property/assertion.ph")
    builder = read("src/property/builder.ph")

    require(target, "class _MethodTarget", "method invocation target")
    require(target, ".invokeOn(_receiver, arguments)", "method invocation target")
    require(target, "class _BlockTarget", "block invocation target")
    require(target, ".callWith(arguments)", "block invocation target")

    require(assertion, "class PropertyAssertionError", "public assertion error")
    require(assertion, "failureOrigin -> FailureOrigin", "assertion origin")
    require(assertion, "SourceLocation.caller", "caller source capture")
    require(assertion, "class Assert", "assertion API")
    for selector in ("equal(", "true(", "false(", "fail("):
        require(assertion, selector, "assertion API")

    require(builder, "class PropertyBuilder<T...>", "property builder")
    require(builder, "given<T...>(*strategies", "Property.given")
    require(builder, "using(settings: Settings)", "PropertyBuilder.using")
    require(builder, "check(body: Block)", "PropertyBuilder.check")
    require(builder, "engineSearch.SearchEngine.new().check", "builder engine delegation")
    require(builder, "class PropertySuite", "property suite")
    reject(builder, "Phase 01 module boundary", "property builder")


def check_discovery_and_runner() -> None:
    discovery = read("src/property/discovery.ph")
    runner = read("src/property/runner.ph")
    specification = read("src/engine/specification.ph")

    require(discovery, "class PropertyId", "property identity")
    require(discovery, "class PropertyDefinition", "discovered property")
    require(discovery, "class PropertyDiscovery", "property discovery")
    require(discovery, "attributesOfType(attributes.Given)", "Given discovery")
    require(discovery, "attributesOfType(attributes.Case)", "Case discovery")
    require(discovery, "attributesOfType(attributes.WithSettings)", "settings discovery")
    require(discovery, "inference._StrategyInference.resolve", "strategy inference integration")
    require(discovery, "parameterNames", "reflected parameter names")

    require(runner, "class PropertyRun", "named property result")
    require(runner, "namedArguments -> Map<Symbol, Any>", "named arguments")
    require(runner, "explicitFailure -> Bool", "explicit case identity")
    require(runner, "class PropertySuiteResult", "suite result")
    require(runner, "summaryLines -> List<String>", "acceptance summary")
    require(runner, "class PropertyRunner", "reflective runner")
    require(runner, "discovery.PropertyDiscovery.discover", "runner discovery")
    require(runner, "engineSearch.SearchEngine.new().check", "runner engine delegation")
    require(runner, "definition.explicitExamples", "explicit case delegation")

    require(specification, "_parameterNames", "engine spec parameter names")
    require(specification, "parameterNames: List<Symbol>", "engine spec parameter names")


def check_facade_and_adapter_migration() -> None:
    facade = read("src/hypothesis.ph")
    manifest = read("phalcom.toml")
    for module in (
        'import propertyAttributes from "property/attributes"',
        'import propertyAssertion from "property/assertion"',
        'import propertyBuilder from "property/builder"',
        'import propertyDiscovery from "property/discovery"',
        'import propertyRunner from "property/runner"',
    ):
        require(facade, module, "property façade")
    for alias in (
        "const Given = propertyAttributes.Given",
        "const Case = propertyAttributes.Case",
        "const WithSettings = propertyAttributes.WithSettings",
        "const Property = propertyBuilder.Property",
        "const PropertySuite = propertyBuilder.PropertySuite",
        "const PropertyRunner = propertyRunner.PropertyRunner",
        "const Check = propertyAttributes.WithSettings",
    ):
        require(facade, alias, "property façade")
    if (ROOT / "src/_internal/legacy_adapter.ph").exists() or (ROOT / "src/_internal/phase01_surface.ph").exists():
        raise AssertionError("final release still contains compatibility implementation modules")
    require(read("src/property/runner.ph"), "class PropertySuiteResult", "authoritative runner result")
    require(read("src/reporting/console.ph"), "renderSuiteSummary", "authoritative reporting bridge")
    version_match = re.search(r'version = "0\.1\.0-phase\.(\d+)"', manifest)
    if 'version = "0.1.0"' not in manifest and (not version_match or int(version_match.group(1)) < 6):
        raise AssertionError("manifest version predates Phase 06")


def check_privacy_imports_and_placeholders() -> None:
    public = {
        "Given", "GivenArgs", "GivenMode", "Case", "WithSettings",
        "PropertyAssertionError", "Assert", "Property", "PropertyBuilder",
        "PropertySuite", "PropertyId", "PropertyDefinition",
        "PropertyDiscovery", "PropertyRun", "PropertySuiteResult", "PropertyRunner",
    }
    for path in sorted((ROOT / "src/property").glob("*.ph")):
        source = path.read_text(encoding="utf-8")
        reject(source, "Phase 01 module boundary", str(path.relative_to(ROOT)))
        for class_name in re.findall(r"(?m)^class\s+([A-Za-z_][A-Za-z0-9_]*)", source):
            if class_name not in public and not class_name.startswith("_"):
                raise AssertionError(
                    f"internal property class is not private-prefixed: {class_name}"
                )

    missing: list[str] = []
    for path in sorted((ROOT / "src").rglob("*.ph")):
        source = path.read_text(encoding="utf-8")
        for module in re.findall(r'(?m)^import\s+[^\n]+\s+from\s+"([^"]+)"', source):
            target = ROOT / "src" / f"{module}.ph"
            if not target.exists():
                missing.append(f"{path.relative_to(ROOT)} -> {module}")
    if missing:
        raise AssertionError("missing internal imports: " + ", ".join(missing))

    pairs = {"(": ")", "[": "]", "{": "}"}
    paths = list((ROOT / "src/property").glob("*.ph")) + list((ROOT / "tests/property").glob("*.ph"))
    for path in sorted(paths):
        source = re.sub(r"//.*", "", path.read_text(encoding="utf-8"))
        source = re.sub(r'"(?:\\.|[^"\\])*"', '""', source)
        stack: list[str] = []
        for char in source:
            if char in pairs:
                stack.append(char)
            elif char in pairs.values():
                if not stack or pairs[stack.pop()] != char:
                    raise AssertionError(f"unbalanced delimiter in {path.relative_to(ROOT)}")
        if stack:
            raise AssertionError(f"unclosed delimiter in {path.relative_to(ROOT)}")


def main() -> int:
    checks = [
        ("Phase 06 tests exist", check_tests_first),
        ("property attributes and reflective inference", check_attributes_and_inference),
        ("targets, assertions, and builder", check_targets_assertions_and_builder),
        ("property discovery and runner", check_discovery_and_runner),
        ("root façade and final ownership", check_facade_and_adapter_migration),
        ("property privacy, imports, and placeholder removal", check_privacy_imports_and_placeholders),
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
