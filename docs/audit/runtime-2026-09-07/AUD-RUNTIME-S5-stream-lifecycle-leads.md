# AUD-RUNTIME-S5 — Stream lifecycle continuation

Read-only checkpoint at HEAD `1e22b57ff8cd0c61b12aeca4128671c9fabdef6e`. New source-established problems are [007](AUD-RUNTIME-007-buffered-writer-flush-contract.md) and [008](AUD-RUNTIME-008-resource-handle-lossy-transport.md). No executable reproduction or Cargo validation was performed. Relevant source files had no working-tree diff.

## L25 — BufferedReader does not close its inner reader

**Confirmed source contract gap; not executed.** Stream protocol §5.5 requires BufferedReader.close to close the inner reader. `collections/bytes.ph::BufferedReader` defines a constructor and read only; it inherits Resource.close, which closes its own `_handle` and returns Ok. That base implementation does not know `_inner`. No close override appears in this class. Next: verify current loading and close the wrapper around a BytesReader, inspecting both isClosed values. Keep wrapper registration distinct from the inner resource's registration.

## L26 — Resource close loses idempotency after slot reuse

**Confirmed source sequence; public execution unresolved.** `ResourceTable::close` marks a row closed and immediately pushes its index to the free list. `open` reuses that index and increments the generation. A subsequent close of the old handle returns StaleHandle because generation comparison precedes the closed check; `resource_raw_close` turns that into a raised use-after-close error. Thus table-level close is idempotent before reuse but not after reuse, although stream protocol §3.1 requires closing an already-closed resource to succeed. Never close the new occupant to satisfy this law. Next: establish how wrapper-local closed identity can be retained separately, and verify open A → close A → open B → close A at the public API.

## L27 — Zero-capacity reads blur EOF and no-op

**Unresolved contract detail.** BytesReader.read returns zero for an empty destination even when data remains. BufferedReader.read with no cached data calls the inner reader before accounting for a zero-size destination, then may return zero while retaining fetched bytes. The stream law says zero means EOF but does not visibly specify the zero-capacity exception in the examined section. Next: resolve the allowed zero-size input contract and whether an empty read may advance/block the inner source. Do not label an EOF bug without resolving this precondition.

## L28 — Outstanding buffered operations share mutable state

**Unresolved scheduling/ownership contract.** BufferedWriter.flush captures a chunk, then clears shared `_len` in a future callback. A later small write can append to the shared buffer before the first flush settles unless another layer serializes access. BufferedReader likewise reuses `_buf` for outstanding reads. Future.then has both immediate and deferred callback branches. Next: establish whether callers must serialize operations; if overlap is allowed, trace two pending operations and check ownership of each buffer range. No concurrency failure was executed.

## Positive findings

BytesReader snapshots its source, and BytesWriter copies accepted chunks, preventing later caller mutation from changing retained data on these immediate paths. Dirty BufferedWriter.close tests pending length before closing either layer. Keep these properties while investigating partial transfer and asynchronous ownership.

## Continuation limits

The public loader and full stream conformance harness were not verified. Resource allocation-site diagnostics and pending-byte leak reporting are additional visible questions but were not investigated in this slice; they are not new confirmed findings. Preserve audit-only scope and do not turn these source traces into claims of executed failures.
