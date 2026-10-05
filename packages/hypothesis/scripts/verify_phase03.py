#!/usr/bin/env python3
"""Observed static verification for Phase 03 when no Phalcom executable exists."""

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
        "tests/choices/choice_variants.ph",
        "tests/choices/buffer_freeze.ph",
        "tests/choices/span_tree.ph",
        "tests/choices/generate_replay.ph",
        "tests/choices/overrun.ph",
    }
    missing = sorted(path for path in expected if not (ROOT / path).is_file())
    if missing:
        fail("missing Phase 03 tests: " + ", ".join(missing))

    variants = read("tests/choices/choice_variants.ph")
    for token in ("Choice.integer", "Choice.boolean", "Choice.index", "Choice.bytes", "ChoiceRequest.integer", "ChoiceRequest.bytes"):
        require(variants, token, "choice variants test")
    require(variants, "shrinkTarget", "choice request test")
    require(variants, "attempt().isErr", "choice contract test")

    freeze = read("tests/choices/buffer_freeze.ph")
    require(freeze, "const frozen = choices.freeze", "buffer freeze test")
    require(freeze, "exposed.add(", "deep freeze test")

    spans = read("tests/choices/span_tree.ph")
    require(spans, "withSpan(label: #composite", "span tree test")
    require(spans, "Some.new(0)", "span parent test")

    replay = read("tests/choices/generate_replay.ph")
    require(replay, "DrawData.generate", "generate replay test")
    require(replay, "DrawData.replay", "generate replay test")
    require(replay, "Assert.equal(generatedExample, replayedExample)", "normalized replay test")

    overrun = read("tests/choices/overrun.ph")
    require(overrun, "status.overrun", "overrun test")
    require(overrun, "Assert.isFalse(status.failed)", "overrun classification test")


def check_choice_model() -> None:
    manifest = read("phalcom.toml")
    match = re.search(r'version = "0\.1\.0-phase\.(\d+)"', manifest)
    if 'version = "0.1.0"' not in manifest and (not match or int(match.group(1)) < 3):
        fail("phalcom.toml: expected Phase 03 or later package version")

    choice = read("src/choices/choice.ph")
    request = read("src/choices/request.ph")
    span = read("src/choices/span.ph")
    example = read("src/choices/example.ph")
    buffer = read("src/choices/buffer.ph")

    for path, source in {
        "choice.ph": choice,
        "request.ph": request,
        "span.ph": span,
        "example.ph": example,
    }.items():
        require(source, "@data", path)
        require(source, "@immutable", path)

    for path, source in (("choice.ph", choice), ("request.ph", request)):
        require(source, "@sealed", path)
        for variant in ("Integer", "Boolean", "Index", "Bytes"):
            require(source, f"@variant {variant}", path)

    for field in ("value:", "min:", "max:", "shrinkTowards:", "label:"):
        require(choice, field, "choice variants")
    require(choice, "simplifications", "choice simplification compatibility")
    require(choice, "withValue", "choice replacement compatibility")
    require(choice, "class _ChoiceBytes", "byte choice copies")
    if choice.count("_ChoiceBytes.copy") < 4:
        fail("choice.ph: byte values and targets are not copied at API boundaries")

    if request.count("@requires") < 8:
        fail("request.ph: expected contracts for bounds, sizes, and shrink targets")
    for factory in ("integer(", "boolean(", "index(", "bytes("):
        require(request, factory, "choice request factories")
    require(request, "shrinkTarget", "choice request API")

    for field in ("_id: Int", "_label: Symbol", "_start: Int", "_end: Int", "_parent: Option<Int>", "_discardable: Bool"):
        require(span, field, "span.ph")
    require(span, "@requires(start <= end)", "span bounds")

    for field in ("_choiceValues: List<Choice>", "_spanValues: List<Span>", "_generationSize: Int"):
        require(example, field, "example.ph")
    for method in ("empty -> Example", "choices -> List<Choice>", "spans -> List<Span>", "signature", "replace(", "prefix("):
        require(example, method, "example compatibility")

    require(buffer, "class ChoiceBuffer", "buffer.ph")
    require(buffer, "class _OpenSpan", "buffer.ph")
    require(buffer, "withSpan(", "buffer span API")
    require(buffer, "}.ensure {", "buffer span cleanup")
    require(buffer, "freeze -> Example", "buffer freeze")
    require(buffer, "Example.from(", "buffer freeze copy")
    require(buffer, "_closedSpans.at(id, put: Some.new(closed))", "stable linear span ordering")


def check_providers_and_replay() -> None:
    provider = read("src/choices/provider.ph")
    data = read("src/choices/data.ph")
    errors = read("src/core/errors.ph")

    require(provider, "protocol ChoiceProvider", "provider protocol")
    require(provider, "choose(request: ChoiceRequest) -> Choice", "provider protocol")
    require(provider, "class SystemRandomChoiceProvider", "random provider")
    require(provider, "class _ReplayChoiceProvider", "replay provider")
    require(provider, "request.match(", "typed provider dispatch")
    require(provider, "_cursor++", "replay cursor")
    require(provider, "_ChoiceNormalization.normalize", "provider normalization")
    require(provider, "Random", "random provider")

    require(data, "class DrawData", "DrawData")
    for factory in ("generate(", "replay("):
        require(data, factory, "DrawData factories")
    for draw in ("drawInt(", "drawBool(", "drawIndex(", "drawBytes("):
        require(data, draw, "typed draw API")
    require(data, "if _buffer.size >= _maxChoices", "choice budget")
    require(data, "ExampleStatus.overrun", "overrun classification")
    require(data, "ExampleStatus.interesting", "interesting classification")
    require(data, "withSpan(", "DrawData span API")
    require(data, "consumedChoices", "replay consumption")

    for error_name in ("_ChoiceOverrun", "_ReplayExhausted", "_InvalidReplayChoice", "_ChoiceBudgetExceeded", "_UnclosedSpan"):
        require(errors, f"class {error_name}", "choice error taxonomy")
    require(errors, "extends _EngineOverrun", "overrun taxonomy")

    random_class = re.search(r"class SystemRandomChoiceProvider.*?(?=\nclass |\Z)", provider, re.S)
    replay_class = re.search(r"class _ReplayChoiceProvider.*?(?=\nclass |\Z)", provider, re.S)
    if not random_class or not replay_class:
        fail("provider classes could not be isolated")
    if "Random" in replay_class.group(0):
        fail("replay provider must not consult randomness")


def check_adapter_integration() -> None:
    facade = read("src/hypothesis.ph")
    for module_import in (
        'import choiceModel from "choices/choice"',
        'import exampleModel from "choices/example"',
        'import choiceData from "choices/data"',
    ):
        require(facade, module_import, "root facade")
    for alias in (
        "const Choice = choiceModel.Choice",
        "const Example = exampleModel.Example",
        "const DrawData = choiceData.DrawData",
    ):
        require(facade, alias, "root facade")
    if (ROOT / "src/_internal/legacy_adapter.ph").exists():
        fail("final release still contains the legacy adapter")
    evaluator = read("src/engine/evaluator.ph")
    require(evaluator, "Example.empty", "authoritative explicit example")
    require(evaluator, "DrawData.generate", "authoritative generation")
    require(evaluator, "DrawData.replay", "authoritative replay")
    require(evaluator, "errors._EngineOverrun", "authoritative overrun classification")
    if "0.1.0" not in facade:
        fail("root facade does not identify the final release")


def main() -> int:
    checks = [
        ("Phase 03 tests exist", check_tests_first),
        ("typed choices, spans, and immutable examples", check_choice_model),
        ("choice providers and deterministic replay", check_providers_and_replay),
        ("facade and final release integration", check_adapter_integration),
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
