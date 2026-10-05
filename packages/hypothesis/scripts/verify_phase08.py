#!/usr/bin/env python3
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
        "tests/database/key_stability.ph",
        "tests/database/memory_database.ph",
        "tests/database/codec_roundtrip.ph",
        "tests/database/corruption_recovery.ph",
        "tests/database/atomic_replace.ph",
        "tests/database/retention_limits.ph",
        "tests/database/process_reuse.ph",
        "tests/database/stale_replay.ph",
    }
    missing = sorted(path for path in required if not (ROOT / path).is_file())
    if missing:
        raise AssertionError("missing Phase 08 tests: " + ", ".join(missing))
    require(read("tests/database/key_stability.ph"), "strategyFingerprint", "key fixture")
    require(read("tests/database/codec_roundtrip.ph"), "FailureOrigin", "codec fixture")
    require(read("tests/database/atomic_replace.ph"), "replaceAtomic", "atomic fixture")
    require(read("tests/database/process_reuse.ph"), "nextProcess", "process fixture")


def check_key_and_protocol() -> None:
    key = read("src/database/key.ph")
    protocol = read("src/database/database.ph")
    for marker in ("@data", "@immutable", "class DatabaseKey"):
        require(key, marker, "database key")
    for field in (
        "_package: Symbol", "_module: Symbol", "_suite: Symbol",
        "_selector: Symbol", "_strategyFingerprint: String",
        "_engineFormatVersion: Int",
    ):
        require(key, field, "database key")
    for marker in ("canonical -> String", "fileStem -> String", "fnv1a"):
        require(key, marker, "database key")
    require(protocol, "protocol ExampleDatabase", "database protocol")
    require(protocol, "fetch(key: DatabaseKey) -> List<Example>", "database protocol")
    require(protocol, "failureOrigin: Option<FailureOrigin>", "database protocol")
    require(protocol, "class _DatabaseRecord", "database record")


def check_memory_database() -> None:
    source = read("src/database/memory.ph")
    require(source, "class MemoryDatabase", "memory database")
    require(source, "maxEntries: Int", "memory database")
    require(source, "existing.signature != record.signature", "memory deduplication")
    require(source, "while kept.size > _maxEntries", "memory retention")
    require(source, "entryCount", "memory inspection")
    reject(source, "Phase 01 module boundary", "memory database")


def check_codec() -> None:
    source = read("src/database/codec.ph")
    require(source, "class ExampleCodec", "codec")
    require(source, '"PHALCOM-HYPOTHESIS-DB"', "codec magic")
    require(source, "engineFormatVersion", "codec version")
    for marker in (
        "Choice.Integer", "Choice.Boolean", "Choice.Index", "Choice.Bytes",
        "generationSize", "Span.create", "FailureOrigin.new",
        "checksum", "expectedKey", "trailing bytes",
    ):
        require(source, marker, "codec")
    require(source, "Result<List<databaseModel._DatabaseRecord>", "codec result")
    reject(source, "toString output", "codec authority")


def check_directory_database() -> None:
    source = read("src/database/directory.ph")
    require(source, "protocol _DatabaseFileSystem", "directory filesystem")
    for marker in (
        "class DirectoryDatabase", "maxEntries: Int", "maxFileBytes: Int",
        "writeTemporary", "flush", "close", "replaceAtomic", "quarantine",
        "while payload.size > _maxFileBytes", "files.remove(temporary)",
    ):
        require(source, marker, "directory database")
    require(source, "ExampleCodec.decode", "directory decode")
    require(source, "ExampleCodec.encode", "directory encode")
    reject(source, "Phase 01 module boundary", "directory database")


def check_runner_integration() -> None:
    discovery = read("src/property/discovery.ph")
    runner = read("src/property/runner.ph")
    settings = read("src/core/settings.ph")
    require(discovery, "databaseKey -> DatabaseKey", "property database key")
    require(discovery, "strategy.fingerprint", "strategy key fingerprint")
    require(discovery, "ExampleCodec.engineFormatVersion", "engine format key")
    require(runner, "definition.databaseKey", "runner typed key")
    require(runner, "failureOrigin: Some.new(value.failure.origin)", "failure metadata save")
    require(runner, "stale", "stale reuse cleanup")
    require(settings, "Option<ExampleDatabase>", "typed settings database")
    for marker in ("database.fetch(id.toString)", "database.save(id.toString", "database.delete(id.toString"):
        reject(runner, marker, "string database key")


def check_facade_and_migration() -> None:
    facade = read("src/hypothesis.ph")
    manifest = read("phalcom.toml")
    for marker in (
        'import databaseModel from "database/database"',
        'import databaseKey from "database/key"',
        'import databaseMemory from "database/memory"',
        'import databaseDirectory from "database/directory"',
        "const DatabaseKey = databaseKey.DatabaseKey",
        "const ExampleDatabase = databaseModel.ExampleDatabase",
        "const MemoryDatabase = databaseMemory.MemoryDatabase",
        "const DirectoryDatabase = databaseDirectory.DirectoryDatabase",
    ):
        require(facade, marker, "database façade")
    if (ROOT / "src/_internal/legacy_adapter.ph").exists() or (ROOT / "src/_internal/phase01_surface.ph").exists():
        raise AssertionError("final release still contains compatibility implementation modules")
    version = re.search(r'version = "0\.1\.0-phase\.(\d+)"', manifest)
    if 'version = "0.1.0"' not in manifest and (not version or int(version.group(1)) < 8):
        raise AssertionError("manifest version: expected Phase 08 or later")

def check_privacy_imports_placeholders() -> None:
    public = {"DatabaseKey", "ExampleDatabase", "MemoryDatabase", "DirectoryDatabase", "ExampleCodec"}
    for path in sorted((ROOT / "src/database").glob("*.ph")):
        source = path.read_text(encoding="utf-8")
        reject(source, "Phase 01 module boundary", str(path.relative_to(ROOT)))
        for class_name in re.findall(r"(?m)^class\s+([A-Za-z_][A-Za-z0-9_]*)", source):
            if class_name not in public and not class_name.startswith("_"):
                raise AssertionError(f"internal database class is not private-prefixed: {class_name}")
    missing: list[str] = []
    for path in sorted((ROOT / "src").rglob("*.ph")):
        source = path.read_text(encoding="utf-8")
        for module in re.findall(r'(?m)^import\s+[^\n]+\s+from\s+"([^"]+)"', source):
            target = ROOT / "src" / f"{module}.ph"
            if not target.exists():
                missing.append(f"{path.relative_to(ROOT)} -> {module}")
    if missing:
        raise AssertionError("missing internal imports: " + ", ".join(missing))


def main() -> int:
    checks = [
        ("Phase 08 tests exist", check_tests),
        ("typed database key and protocol", check_key_and_protocol),
        ("memory database behavior", check_memory_database),
        ("versioned semantic example codec", check_codec),
        ("atomic directory persistence and recovery", check_directory_database),
        ("runner reuse and persistence integration", check_runner_integration),
        ("root facade and database ownership migration", check_facade_and_migration),
        ("database privacy imports and placeholder removal", check_privacy_imports_placeholders),
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
