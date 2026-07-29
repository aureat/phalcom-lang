# Phase 08 — Example Database Design

## Scope

Phase 08 replaces the temporary database bridge with authoritative typed memory and directory databases. It persists minimal failing semantic examples, reuses them before generation, and treats stale, corrupt, oversized, or invalid records as cache conditions rather than property failures.

## Considered storage layouts

### One file per example

This isolates corruption well, but creates unbounded directory fan-out, makes retention ordering expensive, and requires multi-file transactions when deduplicating or evicting entries.

### Append-only property log

This makes individual writes cheap, but requires a later compaction protocol, leaves torn tails and duplicate records, and complicates bounded file-size guarantees.

### One bounded file per database key

This stores an ordered set of records for one property in a single versioned payload. The complete payload is written to a sibling temporary file and atomically replaces the destination. Corruption is isolated to one property, retention is deterministic, and no compaction mechanism is required.

Phase 08 uses the third design.

## Database identity

`DatabaseKey` is immutable and contains:

- package name;
- module name;
- suite name;
- complete selector;
- ordered strategy fingerprint;
- engine format version.

The canonical key string is encoded inside every file. The filename uses a deterministic non-cryptographic digest of that canonical string. Reads verify the embedded canonical key, so digest collisions are cache misses rather than cross-property reuse.

Changing the property location, selector, strategy structure, or engine format version produces a different key and safely invalidates older records.

## Database protocol

`ExampleDatabase` exposes typed `fetch`, `save`, and `delete` operations over `DatabaseKey` and immutable `Example` values. `save` also accepts optional `FailureOrigin` metadata so persistent records preserve the failure site that produced them. Concrete databases provide a convenience two-argument `save` overload that records no origin.

Database operations are fail-soft. Missing entries, stale entries, malformed records, and recoverable I/O errors return an empty result or leave the cache unchanged. They never become property counterexamples.

## Record and codec model

The private immutable `_DatabaseRecord` contains:

- the semantic `Example`;
- optional `FailureOrigin`;
- a deterministic example signature.

`ExampleCodec` encodes a complete property file. The format has:

- a fixed magic string;
- a schema version;
- the canonical database key;
- record count;
- generation size;
- every primitive choice variant and its constraints;
- every semantic span and parent link;
- optional source-aware failure metadata;
- a payload checksum.

Strings, symbols, selectors, and byte sequences use explicit length-prefixed fields. Decoding validates all lengths, counts, variant tags, choice constraints, span ranges, parent references, key identity, and checksum before returning records. A partially valid file is rejected as a whole.

## Memory database

`MemoryDatabase` stores records by canonical key. It:

- copies returned lists;
- deduplicates by semantic example signature;
- places the newest save first;
- enforces the configured maximum entry count;
- deletes by example signature;
- never exposes mutable internal buckets.

## Directory database

`DirectoryDatabase` receives a root path and bounded retention settings. Each key maps to one file under a versioned subdirectory.

A save operation:

1. reads and decodes the current records;
2. removes a duplicate of the new example;
3. prepends the new record;
4. enforces maximum entry count and encoded byte size;
5. creates the parent directory;
6. writes the complete payload to a unique sibling temporary file;
7. flushes and closes the temporary file;
8. atomically replaces the destination;
9. removes any abandoned temporary file on failure.

A corrupt file is renamed to a `.corrupt` sibling when possible. If quarantine fails, the corrupt destination may be replaced by the next successful save. Reads still return an empty list.

The filesystem calls are isolated behind the private `_DatabaseFileSystem` protocol. The default adapter uses Phalcom's filesystem and path standard-library APIs; tests can use a scripted adapter to verify atomic ordering and recovery without process-global filesystem state.

## Runner integration

`PropertyDefinition.databaseKey` derives the key from reflected property identity and ordered strategy fingerprints. `PropertyRunner` asks the configured database for reusable examples before invoking the search engine. It does not eagerly delete valid fetched examples.

After execution:

- a falsified result saves the accepted minimal example and its `FailureOrigin`;
- a passed result deletes fetched stale candidates that did not reproduce;
- invalid or overrun reuse candidates are deleted after the search kernel rejects them;
- database failures remain cache failures and do not change `PropertyResult` classification.

The reuse phase therefore survives process boundaries while strategy or source changes invalidate entries safely.

## Limits and non-goals

Phase 08 did not add cross-process locking, distributed databases, remote storage, encryption, compression, or a public codec-extension protocol. Phase 11 adds adapter conformance, process-local path exclusion, and merge-on-write from the latest visible record set while preserving atomic replacement. Portable cross-process exclusion still requires a standardized runtime file-lock primitive; distributed storage, encryption, compression, and public codec extension remain outside this phase.

## Verification

The Phase 08 gate checks:

- complete typed key identity;
- memory save/fetch/delete and deduplication;
- codec coverage for choices, spans, generation size, and failure origin;
- checksum and structural validation;
- atomic temporary-write-and-replace ordering;
- corruption quarantine and fail-soft reads;
- bounded entries and byte size;
- runner key derivation, reuse-before-generation, stale deletion, and minimal-failure persistence;
- root façade migration and removal of duplicate database ownership from the legacy adapter;
- all Phase 01–07 gates and clean-archive checksums.
