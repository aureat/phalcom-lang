#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]

REQUIRED_FILES = {
    "src/typing.ph": ["const Type =", "const AppliedType =", "const TypeParameter ="],
    "src/typing/type.ph": ["@protocol", "class Type", "currentApplication"],
    "src/typing/type_descriptor.ph": ["@abstract", "class TypeDescriptor"],
    "src/typing/variance.ph": ["class Variance", "@variant Invariant", "@variant Covariant", "@variant Contravariant"],
    "src/typing/type_parameter.ph": ["class TypeParameter", "_owner", "_index", "substitute(using:"],
    "src/typing/generic_signature.ph": ["class GenericSignature", "validate(arguments:"],
    "src/typing/type_environment.ph": ["class TypeEnvironment", "bind(parameter:", "substitute(type:"],
    "src/typing/applied_type.ph": ["class AppliedType", "_origin", "_arguments", "forward(selector:"],
    "src/typing/applied_member.ph": ["class AppliedMethod", "_environment", "executable"],
    "src/typing/type_constructor.ph": ["class TypeConstructor", "<...>"],
    "src/typing/type_runtime.ph": ["class TypeRuntime", "@native", "<...>", "currentApplication", "forwardClassSide"],
    "src/typing/errors.ph": ["class TypeError", "class TypeArgumentCountError", "class TypeBoundError"],
    "tests/acceptance/typing_core_acceptance.ph": ["Box<Int>", "Type.currentApplication", "TypeParameter"],
    "tests/acceptance/type_parameter_identity.ph": ["Left<T>", "Right<T>", "owner ==="],
    "tests/acceptance/type_application_validation.ph": ["T: Vehicle", "T in (Int, String)", "TypeBoundError"],
    "tests/acceptance/applied_forwarding.ph": ["Factory<Int>.make", "currentApplication", "value.class === Product"],
    "tests/acceptance/member_substitution.ph": ["Pair<String, Int>", "returnType == String", "executable ==="],
    "README.md": ["Phase 1", "reified type expressions only"],
    "STATUS.md": ["Implemented in Phase 1", "Deferred to Phase 2"],
}


def fail(message: str) -> None:
    print(f"FAIL: {message}")
    raise SystemExit(1)


def main() -> None:
    for relative, needles in REQUIRED_FILES.items():
        path = ROOT / relative
        if not path.exists():
            fail(f"missing {relative}")
        text = path.read_text(encoding="utf-8")
        for needle in needles:
            if needle not in text:
                fail(f"{relative} lacks required surface: {needle}")

    sources = "\n".join(
        path.read_text(encoding="utf-8")
        for path in sorted((ROOT / "src").rglob("*.ph"))
    )

    forbidden = {
        r"\bconstruct\b": "deprecated construct keyword",
        r"\bTODO\b": "TODO placeholder",
        r"\bTBD\b": "TBD placeholder",
        r"NotImplemented": "NotImplemented placeholder",
    }
    for pattern, description in forbidden.items():
        if re.search(pattern, sources):
            fail(f"source contains {description}")

    runtime = (ROOT / "src/typing/type_runtime.ph").read_text(encoding="utf-8")
    if runtime.count("@native") < 4:
        fail("native boundary must expose at least four source anchors")

    parameter = (ROOT / "src/typing/type_parameter.ph").read_text(encoding="utf-8")
    if "_name == other.name" in parameter and "_owner === other.owner" not in parameter:
        fail("TypeParameter equality must include owner identity")

    applied = (ROOT / "src/typing/applied_type.ph").read_text(encoding="utf-8")
    if "TypeRuntime.forwardClassSide" not in applied:
        fail("AppliedType must forward all non-reserved class-side messages")

    print(f"PASS: {len(REQUIRED_FILES)} required files and typing-core invariants verified")


if __name__ == "__main__":
    main()
