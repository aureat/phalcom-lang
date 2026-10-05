# Phase 08 Example Database Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed, versioned, bounded memory and directory databases that persist minimal counterexamples and safely reuse them across processes.

**Architecture:** `DatabaseKey` derives stable identity from reflected property and strategy metadata. `ExampleCodec` serializes complete semantic examples and optional failure origins into a validated versioned payload. `MemoryDatabase` and `DirectoryDatabase` implement one typed protocol; the directory implementation stores one bounded atomically replaced file per key and treats corruption as a recoverable cache miss.

**Tech Stack:** Phalcom source, Python source/static verifiers, ZIP/SHA-256 checkpoint tooling.

## Global Constraints

- Preserve every Phase 01–07 public contract and verifier guarantee.
- Write tests and the Phase 08 verifier before production source.
- Type annotations remain reflective metadata, not runtime dispatch or enforcement.
- Database failures, stale examples, corruption, invalid replay, and overrun remain cache/search conditions, never property counterexamples.
- Persist only semantic examples and source-aware metadata; never serialize `toString` output as the authoritative record.
- Use current Phalcom syntax and `_`-prefix implementation-only top-level names.
- Run real Phalcom tests only when a `phalcom` executable is available; otherwise report source/static verification separately.
- The final artifact is the complete project archive `phalcom-hypothesis-phase-08-database.zip`.

---

### Task 1: Phase 08 Contract Gate and Failing Fixtures

**Files:**
- Create: `scripts/verify_phase08.py`
- Create: `tests/database/key_stability.ph`
- Create: `tests/database/memory_database.ph`
- Create: `tests/database/codec_roundtrip.ph`
- Create: `tests/database/corruption_recovery.ph`
- Create: `tests/database/atomic_replace.ph`
- Create: `tests/database/retention_limits.ph`
- Create: `tests/database/process_reuse.ph`
- Create: `tests/database/stale_replay.ph`

**Interfaces:**
- Consumes: Phase 07 project tree and checkpoint protocol.
- Produces: an eight-check source verifier and executable Phalcom acceptance fixtures covering every Phase 08 requirement.

- [ ] **Step 1: Write fixtures that name the intended API**

```phalcom
const key = DatabaseKey.create(
  package: #hypothesisTests,
  module: #databaseProperties,
  suite: #CodecProperties,
  selector: #roundTrips(value:),
  strategyFingerprint: "list(int,-10,10)",
  engineFormatVersion: 1
)

const database = MemoryDatabase.new(maxEntries: 4)
database.save(key, example, failureOrigin: Some.new(origin))
Assert.equal(example, database.fetch(key).at(0))
```

Directory fixtures must use a scripted filesystem and assert the operation order `writeTemporary`, `flush`, `replaceAtomic` and corruption quarantine.

- [ ] **Step 2: Write `verify_phase08.py`**

The verifier must check:

```python
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
```

- [ ] **Step 3: Run the verifier and record the red state**

Run:

```sh
python3 -m py_compile scripts/verify_phase08.py
python3 scripts/verify_phase08.py
```

Expected: one fixture-existence check may pass; all implementation checks fail because `src/database/*.ph` are placeholders.

---

### Task 2: Typed Key, Protocol, and Memory Database

**Files:**
- Replace: `src/database/key.ph`
- Replace: `src/database/database.ph`
- Replace: `src/database/memory.ph`
- Modify: `src/_internal/fingerprints.ph`
- Test: `tests/database/key_stability.ph`
- Test: `tests/database/memory_database.ph`

**Interfaces:**
- Consumes: `PropertyId`, strategy `fingerprint`, `Example`, `FailureOrigin`.
- Produces: `DatabaseKey`, `ExampleDatabase`, `MemoryDatabase`, `_DatabaseRecord`, deterministic key and example fingerprints.

- [ ] **Step 1: Implement immutable key identity**

```phalcom
@data
@immutable
class DatabaseKey {
  const _package: Symbol
  const _module: Symbol
  const _suite: Symbol
  const _selector: Symbol
  const _strategyFingerprint: String
  const _engineFormatVersion: Int

  canonical -> String { ... }
  fileStem -> String { ... }
}
```

`canonical` must length-prefix fields or otherwise escape delimiters unambiguously. `fileStem` uses deterministic FNV-1a hexadecimal encoding and is not treated as collision-proof.

- [ ] **Step 2: Implement the protocol and record model**

```phalcom
protocol ExampleDatabase {
  fetch(key: DatabaseKey) -> List<Example>
  save(
    key: DatabaseKey,
    example: Example,
    failureOrigin: Option<FailureOrigin>
  ) -> ExampleDatabase
  delete(key: DatabaseKey, example: Example) -> ExampleDatabase
}

@data
@immutable
class _DatabaseRecord {
  const _example: Example
  const _failureOrigin: Option<FailureOrigin>
  const _signature: String
}
```

- [ ] **Step 3: Implement bounded memory storage**

`MemoryDatabase.new(maxEntries:)` validates a positive limit, deduplicates by example signature, prepends the newest record, copies fetched lists, and truncates to the limit.

- [ ] **Step 4: Run the Phase 08 verifier**

Run:

```sh
python3 scripts/verify_phase08.py
```

Expected: key/protocol and memory checks pass; codec, directory, runner, and façade checks remain red.

---

### Task 3: Versioned Semantic Example Codec

**Files:**
- Replace: `src/database/codec.ph`
- Modify: `src/database/database.ph`
- Test: `tests/database/codec_roundtrip.ph`

**Interfaces:**
- Consumes: `DatabaseKey`, `_DatabaseRecord`, all `Choice` variants, `Span`, `Example`, `FailureOrigin`.
- Produces: `ExampleCodec.encode(key:, records:) -> Bytes` and `ExampleCodec.decode(payload:, expectedKey:) -> Result<List<_DatabaseRecord>, _DatabaseDecodeError>`.

- [ ] **Step 1: Define the format constants and cursor helpers**

```phalcom
class ExampleCodec {
  @class
  magic -> String => "PHALCOM-HYPOTHESIS-DB"

  @class
  schemaVersion -> Int => 1
}
```

Private writer/reader helpers must encode and validate explicit integer, boolean, index, and bytes choice tags; span fields; generation size; and optional failure-origin fields.

- [ ] **Step 2: Add payload integrity**

The final field is a deterministic 32-bit checksum over all prior bytes. Decode rejects wrong magic, version, key, checksum, unknown tags, impossible counts, invalid choice constraints, invalid spans, trailing bytes, and oversized declared fields.

- [ ] **Step 3: Verify codec coverage**

Run:

```sh
python3 scripts/verify_phase08.py
```

Expected: codec check passes; directory, runner, and façade checks remain red.

---

### Task 4: Atomic Directory Database

**Files:**
- Replace: `src/database/directory.ph`
- Test: `tests/database/corruption_recovery.ph`
- Test: `tests/database/atomic_replace.ph`
- Test: `tests/database/retention_limits.ph`

**Interfaces:**
- Consumes: `ExampleCodec`, `_DatabaseRecord`, `DatabaseKey`.
- Produces: `DirectoryDatabase`, private `_DatabaseFileSystem` protocol, default filesystem adapter, bounded atomic persistence.

- [ ] **Step 1: Implement filesystem isolation**

```phalcom
protocol _DatabaseFileSystem {
  read(path: Any) -> Result<Bytes, Error>
  createDirectories(path: Any) -> Result<None, Error>
  writeTemporary(path: Any, payload: Bytes) -> Result<Any, Error>
  flush(file: Any) -> Result<None, Error>
  close(file: Any) -> Result<None, Error>
  replaceAtomic(source: Any, destination: Any) -> Result<None, Error>
  quarantine(path: Any) -> Result<None, Error>
  remove(path: Any) -> Result<None, Error>
}
```

- [ ] **Step 2: Implement fail-soft reads**

Missing files return an empty list. Decode failures trigger quarantine and return an empty list. Other recoverable read errors return an empty list without changing property results.

- [ ] **Step 3: Implement bounded atomic saves**

`DirectoryDatabase.new(root:, maxEntries:, maxFileBytes:)` reads current records, deduplicates, prepends, trims by count, repeatedly drops oldest records until the encoded payload is within the byte limit, then writes/flushed/closes/replaces atomically. An abandoned temporary file is removed on failure.

- [ ] **Step 4: Implement delete**

Deleting the final record removes the property file. Otherwise it atomically rewrites the bounded payload.

- [ ] **Step 5: Run the Phase 08 verifier**

Expected: directory check passes; runner and façade checks remain red.

---

### Task 5: Runner Integration and Ownership Migration

**Files:**
- Modify: `src/property/discovery.ph`
- Modify: `src/property/runner.ph`
- Modify: `src/core/settings.ph`
- Modify: `src/hypothesis.ph`
- Modify: `src/_internal/legacy_adapter.ph`
- Modify: `src/_internal/phase01_surface.ph`
- Modify: `tests/integration/package_loads.ph`
- Test: `tests/database/process_reuse.ph`
- Test: `tests/database/stale_replay.ph`

**Interfaces:**
- Consumes: `DatabaseKey`, `ExampleDatabase`, `MemoryDatabase`, `DirectoryDatabase`.
- Produces: typed settings, `PropertyDefinition.databaseKey`, persistent reuse before generation, minimal-failure save, stale-entry deletion, canonical root exports.

- [ ] **Step 1: Type settings**

Change `_database: Option<Any>` and related selectors to `Option<ExampleDatabase>` while retaining compatibility selector names.

- [ ] **Step 2: Derive the key**

`PropertyDefinition.databaseKey` combines package/module/suite/selector, ordered strategy fingerprints, and `ExampleCodec.engineFormatVersion`.

- [ ] **Step 3: Integrate reuse**

`_PropertyReuse.fetch(definition:)` calls `database.fetch(definition.databaseKey)` before search. It does not delete fetched entries eagerly.

After search, save the minimal falsifying example with its failure origin. Delete only fetched examples that do not match the accepted failure or whose replay is invalid/stale.

- [ ] **Step 4: Migrate root ownership**

Replace legacy/surface aliases with authoritative database module imports and aliases. Remove `ExampleDatabase`, `MemoryExampleDatabase`, `_LegacySearchBridge`, and the Phase 01 `DirectoryDatabase` placeholder.

- [ ] **Step 5: Run all phase verifiers**

```sh
python3 scripts/verify_phase08.py
python3 scripts/verify_phase07.py
python3 scripts/verify_phase06.py
python3 scripts/verify_phase05.py
python3 scripts/verify_phase04.py
python3 scripts/verify_phase03.py
python3 scripts/verify_phase02.py
python3 scripts/verify_phase01.py
```

Expected: all green. If an inherited verifier hard-codes obsolete database ownership, update only that assertion while preserving the original semantic guarantee.

---

### Task 6: Documentation, Checkpoint, and Clean Archive

**Files:**
- Update: `README.md`
- Update: `CHANGELOG.md`
- Update: `CHECKPOINT.md`
- Update: `TEST-RESULTS.md`
- Replace: `docs/database.md`
- Update: `docs/extension-api.md`
- Update: `phalcom.toml`
- Update: `SHA256SUMS`
- Create: `phalcom-hypothesis-phase-08-database.zip`

**Interfaces:**
- Consumes: completed Phase 08 tree and all verifier output.
- Produces: complete Phase 08 checkpoint with exact observed/static limitations and Phase 09 next-step declaration.

- [ ] **Step 1: Document behavior and limitations**

Document key invalidation, atomic replacement, corruption quarantine, retention limits, last-writer-wins concurrency, and cache-failure semantics. State explicitly whether a `phalcom` executable was available.

- [ ] **Step 2: Set checkpoint metadata**

Set the manifest version to `0.1.0-phase.08`. Name Phase 09 as `Stateful bundles, rules, applicability, and structural programs`.

- [ ] **Step 3: Rebuild checksums**

Generate SHA-256 entries for every project file except `SHA256SUMS` itself.

- [ ] **Step 4: Verify the working tree**

Run Python compilation, Phase 01–08 verifiers, import checks, placeholder/privacy scans, baseline preservation, and `sha256sum -c SHA256SUMS`.

- [ ] **Step 5: Create and verify a clean archive**

Create the full ZIP, extract it into a separate directory, run ZIP integrity, checksums, and every verifier against the extracted tree.

Expected final archive: `phalcom-hypothesis-phase-08-database.zip`.
