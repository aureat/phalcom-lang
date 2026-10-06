# Example Databases

Phase 08 provides typed in-memory and process-persistent databases for minimal failing semantic examples.

## Identity

Every persisted property uses an immutable `DatabaseKey` containing:

```phalcom
DatabaseKey.create(
  package: packageName,
  module: moduleName,
  suite: suiteName,
  selector: selector,
  strategyFingerprint: fingerprint,
  engineFormatVersion: ExampleCodec.engineFormatVersion
)
```

All six fields participate in equality and the canonical key. A property move, selector change, strategy-shape change, or engine-format change therefore creates a new cache identity.

The directory filename uses a deterministic FNV-1a digest of the canonical key. The full canonical key is also encoded inside the file and verified on read, so a filename-digest collision is handled as a cache miss rather than cross-property reuse.

## Protocol

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
```

`MemoryDatabase` and `DirectoryDatabase` also provide `save(key, example)` as a convenience overload.

Database adapters are fail-soft. Missing, stale, corrupt, oversized, or otherwise unreadable entries do not fail the property and are never classified as counterexamples.

## Memory database

```phalcom
const database = MemoryDatabase.new(maxEntries: 16)
const settings = Settings.standard.database(database)
```

The memory database:

- stores newest entries first;
- deduplicates using the complete semantic example structure;
- copies fetched lists;
- enforces a per-key entry limit;
- deletes by semantic signature.

## Directory database

```phalcom
const database = DirectoryDatabase.new(
  root: ".phalcom-hypothesis/examples",
  maxEntries: 16,
  maxFileBytes: 1048576
)
```

Each property key owns one file below a schema-versioned directory. A save reads the current records, deduplicates and bounds them, encodes the complete replacement payload, writes a sibling temporary file, flushes and closes it, then atomically replaces the destination.

A malformed or oversized file is quarantined to a `.corrupt-*` sibling when possible and treated as empty. If quarantine itself fails, the next successful save may replace the corrupt destination.

Phase 11 serializes overlapping writes to the same path within the process and rereads the latest visible bucket before replacement. Atomic replacement remains the portable cross-process integrity boundary; true cross-process exclusion requires a standardized runtime file-lock primitive.

## Codec

`ExampleCodec` uses a versioned structured byte format containing:

- a fixed magic value and schema version;
- the canonical database key;
- every record's complete semantic signature;
- generation size;
- integer, Boolean, index, and bytes choices with all constraints and shrink targets;
- semantic spans, parent links, and discardability;
- optional `FailureOrigin` metadata;
- a checksum over the complete body.

Decode validates the key, checksum, lengths, counts, tags, choice contracts, span ranges, parent containment, parent cycles, and trailing bytes. A file is accepted only when the entire payload is valid.

The codec is an implementation detail rather than the database extension boundary. Custom database adapters exchange `Example` values through `ExampleDatabase`; Phase 11 conformance fixtures cover copy isolation, deduplication, retention, deletion, and recoverable-failure behavior.

## Runner behavior

`PropertyRunner` fetches reusable examples before generation. It does not eagerly remove them.

After the search:

- the accepted minimal failure is saved with its source-aware origin;
- older fetched examples with different semantic signatures are deleted;
- a passing or inconclusive property deletes fetched examples that no longer reproduce;
- explicit-case failures are never persisted;
- cache failures do not change `PropertyResult` classification.

This preserves reuse across processes while making source and strategy drift self-healing.
