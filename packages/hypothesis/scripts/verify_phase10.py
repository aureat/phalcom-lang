#!/usr/bin/env python3
"""Observed source/static verification for Phase 10 derived strategies."""
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


def check_tests() -> None:
    required = {
        "tests/strategies/derived_data.ph",
        "tests/strategies/sealed_variants.ph",
        "tests/strategies/recursive_variants.ph",
        "tests/strategies/derived_generic_fields.ph",
        "tests/strategies/resolution_path.ph",
        "tests/strategies/constrained_constructor.ph",
        "tests/strategies/annotated_strategy.ph",
        "tests/strategies/custom_registration_precedence.ph",
        "tests/property/inferred_domain_models.ph",
    }
    missing = sorted(path for path in required if not (ROOT / path).is_file())
    if missing:
        raise AssertionError("missing Phase 10 fixtures: " + ", ".join(missing))
    require(read("tests/strategies/derived_data.ph"), "@arbitrary", "data fixture")
    require(read("tests/strategies/sealed_variants.ph"), "sealed(Token", "sealed fixture")
    require(read("tests/strategies/recursive_variants.ph"), "generationSize: 0", "recursive fixture")
    require(read("tests/strategies/resolution_path.ph"), "resolution path", "path fixture")
    require(read("tests/strategies/constrained_constructor.ph"), "custom strategy", "contract fixture")
    require(read("tests/strategies/annotated_strategy.ph"), "@strategy(UserId)", "provider fixture")
    require(read("tests/property/inferred_domain_models.ph"), "@Given", "inference fixture")


def check_attributes() -> None:
    source = read("src/strategies/attributes.ph")
    for marker in (
        "class arbitrary extends Attribute",
        "class strategy extends Attribute",
        "@On(Class)",
        "@On(Method)",
        "targetType -> Any",
        "passive reflected metadata",
    ):
        require(source, marker, "derivation attributes")


def check_constructor_derivation() -> None:
    source = read("src/strategies/derivation.ph")
    for marker in (
        "class _ConstructorStrategy",
        "class _DerivedStrategy",
        "constructor.parameters",
        "parameter.type",
        "parameter.label",
        "constructor.invokeOn",
        "data.withSpan",
        "label: #derivedConstructor",
        "unsafe automatic derivation",
        "constrained constructor",
        "custom strategy",
    ):
        require(source, marker, "constructor derivation")
    reject(source, ".filter {", "contract derivation")


def check_registry_precedence_and_paths() -> None:
    source = read("src/strategies/registry.ph")
    for marker in (
        "register(type: Any, strategy: Strategy<Any>)",
        "register(type: Any, use: Strategy<Any>)",
        "register(provider: Class)",
        "attributesOfType(attributes.strategy)",
        "_entries.at(type)",
        "_derived.at(type)",
        "derivation._Derivation.derive",
        "resolution path: ",
        "path.add",
        "_resolving",
    ):
        require(source, marker, "strategy registry")
    exact = source.find("const exact = _entries.at(type)")
    derived = source.find("const cached = _derived.at(type)")
    automatic = source.find("derivation._Derivation.derive")
    if min(exact, derived, automatic) < 0 or not (exact < derived < automatic):
        raise AssertionError("strategy registry: exact/custom/cache/derivation precedence is not explicit")


def check_sealed_recursion() -> None:
    source = read("src/strategies/derivation.ph")
    for marker in (
        "stableVariants",
        "terminalVariants",
        "recursiveVariants",
        "Gen.oneOf",
        "Gen.recursive",
        "recursive-sealed",
        "containsType",
        "recursiveReplacement",
        "no terminal variant",
    ):
        require(source, marker, "sealed derivation")


def check_facade_docs_examples() -> None:
    facade = read("src/hypothesis.ph")
    docs = read("docs/inference.md")
    design = read("docs/design/phase-10-derivation.md")
    derived = read("examples/derived_data.ph")
    recursive = read("examples/recursive_expression.ph")
    manifest = read("phalcom.toml")
    for marker in (
        'import strategyAttributes from "strategies/attributes"',
        "const arbitrary = strategyAttributes.arbitrary",
        "const strategy = strategyAttributes.strategy",
    ):
        require(facade, marker, "root facade")
    for marker in ("Automatic derivation", "Resolution precedence", "constrained constructors", "resolution path"):
        require(docs, marker, "inference docs")
    require(design, "Phase 10 Derivation Design", "design document")
    for marker in ("@arbitrary", "@Given", "Point"):
        require(derived, marker, "derived-data example")
    for marker in ("@sealed", "@variant", "Expression"):
        require(recursive, marker, "recursive example")
    match = re.search(r'version = "0\.1\.0-phase\.(\d+)"', manifest)
    if 'version = "0.1.0"' not in manifest and (match is None or int(match.group(1)) < 10):
        raise AssertionError("manifest version: expected Phase 10 or later")


def check_privacy_imports_placeholders() -> None:
    public = {"arbitrary", "strategy", "StrategyRegistry", "StrategyBase", "Draw"}
    for path in sorted((ROOT / "src/strategies").glob("*.ph")):
        source = path.read_text(encoding="utf-8")
        for class_name in re.findall(r"(?m)^class\s+([A-Za-z_][A-Za-z0-9_]*)", source):
            if class_name not in public and not class_name.startswith("_") and class_name not in {"Gen"}:
                raise AssertionError(f"internal strategy class is not private-prefixed: {class_name}")
    missing: list[str] = []
    for path in sorted((ROOT / "src").rglob("*.ph")):
        source = path.read_text(encoding="utf-8")
        for module in re.findall(r'(?m)^import\s+[^\n]+\s+from\s+"([^"]+)"', source):
            if not (ROOT / "src" / f"{module}.ph").exists():
                missing.append(f"{path.relative_to(ROOT)} -> {module}")
    if missing:
        raise AssertionError("missing internal imports: " + ", ".join(missing))
    reject(read("docs/inference.md"), "Automatic data-class and sealed-variant derivation remains deferred", "deferred derivation text")
    reject(read("examples/derived_data.ph"), "Reserved for the Phase 10", "derived example placeholder")
    reject(read("examples/recursive_expression.ph"), "Reserved for the Phase 10", "recursive example placeholder")


def main() -> int:
    checks = [
        ("Phase 10 tests exist", check_tests),
        ("passive derivation attributes", check_attributes),
        ("constructor and data-class derivation", check_constructor_derivation),
        ("registry precedence and resolution paths", check_registry_precedence_and_paths),
        ("sealed and recursive derivation", check_sealed_recursion),
        ("facade docs examples and version", check_facade_docs_examples),
        ("privacy imports and placeholder removal", check_privacy_imports_placeholders),
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
