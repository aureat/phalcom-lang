#!/usr/bin/env python3
"""Observed static acceptance gate for Phase 05 engine ownership."""

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
        "tests/engine/integer_minimum.ph",
        "tests/engine/middle_span_deletion.ph",
        "tests/engine/recursive_subtree_deletion.ph",
        "tests/engine/complexity_ordering.ph",
        "tests/engine/failure_origin_preservation.ph",
        "tests/engine/flaky_verification.ph",
        "tests/engine/phase_ordering.ph",
        "tests/engine/invalid_overrun_candidates.ph",
        "tests/engine/find_minimal.ph",
        "tests/engine/deterministic_minimal_replay.ph",
    ]
    for path in required:
        source = read(path)
        require(source, "Phase 05", path)
        require(source, "System.print(\"PASS", path)

    require(read(required[0]), "Assert.equal(10", "integer boundary fixture")
    require(read(required[1]), "const [1, 2]", "middle deletion fixture")
    require(read(required[2]), "Tuple.fromList(const [#leaf, #leaf])", "recursive fixture")
    require(read(required[3]), ".lessThan(", "complexity fixture")
    require(read(required[4]), "failure.origin.sameSite", "origin fixture")
    require(read(required[5]), "errors._FlakyFailure", "flaky fixture")
    require(read(required[6]), "const [1]", "phase order fixture")
    require(read(required[7]), "ExampleStatus.invalid", "invalid candidate fixture")
    require(read(required[7]), "ExampleStatus.overrun", "overrun candidate fixture")
    require(read(required[8]), "Some.new(10)", "find fixture")
    require(read(required[9]), "sameOrigin", "deterministic replay fixture")


def check_engine_model() -> None:
    specification = read("src/engine/specification.ph")
    evaluator = read("src/engine/evaluator.ph")
    search = read("src/engine/search.ph")
    errors = read("src/core/errors.ph")
    failure = read("src/core/failure.ph")

    require(specification, "class PropertySpec<T...>", "property specification")
    require(specification, "@data", "property specification")
    require(specification, "@immutable", "property specification")
    for field in ("_target", "_strategies", "_explicitExamples", "_reuseExamples", "_settings"):
        require(specification, field, "property specification")

    require(search, "class _SearchResult", "find search result")
    require(search, "@variant Found", "find result variant")
    require(search, "@variant Evaluated", "ordinary result variant")
    reject(search, "extends Error", "find search result")

    require(evaluator, "class _Evaluator", "engine evaluator")
    require(evaluator, "ExampleStatus.valid", "valid classification")
    require(evaluator, "ExampleStatus.invalid", "invalid classification")
    require(evaluator, "ExampleStatus.overrun", "overrun classification")
    require(evaluator, "ExampleStatus.interesting", "interesting classification")
    require(evaluator, "coreContext._propertyContexts.with", "context cleanup")
    require(failure, "failureOrigin", "source-aware failure capture")
    require(errors, "class _FlakyFailure", "flaky error taxonomy")
    require(errors, "class _UnsatisfiedAssumptions", "discard exhaustion taxonomy")
    require(errors, "class _NoSuchExample", "find exhaustion taxonomy")


def check_structural_shrinker() -> None:
    complexity = read("src/engine/complexity.ph")
    shrink_pass = read("src/engine/shrink_pass.ph")
    shrinker = read("src/engine/shrinker.ph")
    example = read("src/choices/example.ph")
    composite = read("src/strategies/composite.ph")
    ordering = read("src/_internal/ordering.ph")
    fingerprints = read("src/_internal/fingerprints.ph")

    require(complexity, "class ExampleComplexity", "complexity model")
    require(complexity, "lessThan(other: ExampleComplexity) -> Bool", "strict complexity order")
    for component in ("choiceCount", "structuralWeight", "choiceWeight", "signature"):
        require(complexity, component, "complexity tuple")

    require(shrink_pass, "protocol ShrinkPass", "shrink pass protocol")
    for class_name in (
        "_DeleteDiscardableSpans",
        "_ShortenTrailingChoices",
        "_MinimizeBranchIndices",
        "_MinimizeIntegerChoices",
        "_MinimizeIntegerBlocks",
        "_SimplifyBytesAndText",
        "_MinimizeRecursiveStructures",
    ):
        require(shrink_pass, f"class {class_name}", "ordered shrink passes")
    require(shrink_pass, "span.discardable", "discardable span deletion")
    require(shrink_pass, "#length", "collection length adjustment")
    require(shrink_pass, "#recursiveBranch", "recursive subtree deletion")
    require(shrink_pass, "#branch", "branch minimization")

    require(shrinker, "class Shrinker", "shrinker")
    require(shrinker, "candidateComplexity.lessThan(currentComplexity)", "strict accepted shrink")
    require(shrinker, "candidate.failed", "stable failure acceptance")
    require(shrinker, "sameOrigin", "failure-origin preservation")
    require(shrinker, "candidate.invalid", "invalid candidate rejection")
    require(shrinker, "candidate.overrun", "overrun candidate rejection")
    require(shrinker, "acceptedComplexities", "complexity trace")

    require(example, "deleteRange(start: Int, end: Int) -> Example", "immutable range deletion")
    require(example, "adjustedAfterDeletion", "span range adjustment")
    require(composite, "label: #recursiveBranch", "recursive semantic spans")
    require(composite, "discardable: true", "discardable recursive spans")
    require(ordering, "class _Ordering", "private complexity ordering")
    require(fingerprints, "class _Fingerprints", "private example fingerprints")
    reject(ordering, "Phase 01 module boundary", "private complexity ordering")
    reject(fingerprints, "Phase 01 module boundary", "private example fingerprints")


def check_engine_phases_and_find() -> None:
    engine = read("src/engine/engine.ph")
    facade = read("src/hypothesis.ph")
    manifest = read("phalcom.toml")

    require(engine, "class SearchEngine", "search engine")
    for phase in (
        "Phase.Explicit", "Phase.Reuse", "Phase.Generate", "Phase.Shrink"
    ):
        require(engine, phase, "ordered engine phases")
    explicit_pos = engine.index("Phase.Explicit")
    reuse_pos = engine.index("Phase.Reuse")
    generate_pos = engine.index("Phase.Generate")
    shrink_pos = engine.index("Phase.Shrink")
    if not explicit_pos < reuse_pos < generate_pos < shrink_pos:
        raise AssertionError("engine phase handling is not in approved order")

    require(engine, "verifyFailure", "final failure replay")
    require(engine, "_FlakyFailure", "flaky classification")
    require(engine, "find<T>(", "engine find API")
    require(engine, "shrinkFound", "find structural shrink")
    require(engine, "return Some.new", "find value return")
    reject(engine, "FoundExample", "engine find implementation")
    reject(engine, "throw _Found", "engine find implementation")

    if "0.1.0" not in facade:
        raise AssertionError("root release marker is missing")
    version = re.search(r'version = "0\.1\.0-phase\.(\d+)"', manifest)
    if 'version = "0.1.0"' not in manifest and (not version or int(version.group(1)) < 5):
        raise AssertionError("manifest version: expected Phase 05 or later")


def check_adapter_delegation() -> None:
    stateful = read("src/stateful/runner.ph")
    builder = read("src/property/builder.ph")
    runner = read("src/property/runner.ph")
    facade = read("src/hypothesis.ph")
    require(stateful, "engineSpec.PropertySpec.check", "authoritative stateful property delegation")
    require(stateful, "engineSearch.SearchEngine.new().check", "authoritative stateful check delegation")
    require(builder, "engineSearch.SearchEngine.new().find", "property find delegation")
    require(runner, "engineSearch.SearchEngine.new().check", "reflective runner check delegation")
    require(facade, "const Stateful = statefulRunner.Stateful", "root stateful delegation")
    if (ROOT / "src/_internal/legacy_adapter.ph").exists():
        raise AssertionError("final release still contains the legacy adapter")


def check_privacy_imports_and_placeholders() -> None:
    public = {"PropertySpec", "SearchEngine", "ShrinkPass", "Shrinker", "ExampleComplexity"}
    forbidden_models = {"Settings", "Phase", "PropertyResult", "Failure", "Choice", "Example", "DrawData", "Strategy", "Gen"}
    for path in sorted((ROOT / "src/engine").glob("*.ph")):
        source = path.read_text(encoding="utf-8")
        reject(source, "Phase 01 module boundary", str(path.relative_to(ROOT)))
        for class_name in re.findall(r"(?m)^class\s+([A-Za-z_][A-Za-z0-9_]*)", source):
            if class_name in forbidden_models:
                raise AssertionError(f"engine redefines authoritative model: {class_name}")
            if class_name not in public and not class_name.startswith("_"):
                raise AssertionError(
                    f"internal engine class is not private-prefixed: {class_name}"
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
    for path in sorted(list((ROOT / "src/engine").glob("*.ph")) + list((ROOT / "tests/engine").glob("*.ph"))):
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
        ("Phase 05 tests exist", check_tests_first),
        ("engine specification and evaluation model", check_engine_model),
        ("structural shrink passes and complexity", check_structural_shrinker),
        ("ordered engine phases and find", check_engine_phases_and_find),
        ("final façade delegates to engine", check_adapter_delegation),
        ("engine privacy, imports, and placeholder removal", check_privacy_imports_and_placeholders),
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
