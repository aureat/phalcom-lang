# Resources and Byte Streams

## Ownership

This specification owns the public resource and byte-stream types exported by
`universe.collections.bytes`. Resource lifecycle is explicit: collection does
not close a resource, and callers must use `close` or a documented finishing
operation. Accepted lifecycle and reactor decisions remain recorded by
[TDR-0057](../decisions/accepted/0057-resources-are-disposable-handles-not-finalized.md)
and [TDR-0056](../decisions/accepted/0056-io-is-future-shaped-reactor-owned.md).
Text encoding is owned by [`text.md`](text.md); byte storage itself is owned by
[`collections/bytes.md`](collections/bytes.md).

## Resource lifecycle

`Resource` is the closeable root class. `close` is synchronous and returns a
`Result`; closing an already-closed resource is successful and idempotent.
`isClosed` reports lifecycle state. Operations on a closed resource raise a
use-after-close error. No finalizer closes a resource during garbage
collection.

Reader and Writer participation is structural: consumers send `read` or
`write` and do not require a nominal Reader/Writer type. Asynchronous operations
return `Future`; close remains synchronous. Implementations must preserve
resource ownership and must not silently swallow failed writes or flushes.

## Paths and open modes

`OpenMode` provides the values `read`, `write`, `append`, and `readWrite`.
`Path.of(String)` constructs a path from UTF-8 text; `Path.ofBytes(Bytes)` makes
an independent byte copy. `bytes` returns a copy. Equality compares path bytes,
and `hash` is consistent with equality. `isAbsolute`, `join`, `parent`,
`fileName`, `extension`, and `components` provide lexical path operations.
`toString` is a lossy text rendering and is not a round-trip encoding for
arbitrary path bytes.

These operations are lexical. They do not promise filesystem existence,
canonicalization, symlink resolution, or platform-independent normalization.
An absolute right-hand path passed to `join` replaces the left side.

## In-memory streams

`BytesReader.new(source)` snapshots the source bytes. `read(destination)` copies
up to the destination capacity, advances the reader position by the count, and
returns that count in a Future. A zero count denotes end of stream. The reader
does not mutate the source buffer.

`BytesWriter.new()` accumulates copied chunks. `write(source)` snapshots the
supplied bytes and returns its byte count in a Future. `flush` is a no-op that
completes successfully. `toBytes` returns a new concatenated snapshot; later
writes do not mutate an already returned value.

## Buffered streams

`BufferedReader` and `BufferedWriter` wrap objects that implement the relevant
read/write protocol. Their buffering is explicit and bounded; callers choose
the wrapper rather than receiving hidden buffering from every writer.

`BufferedWriter.write` copies the submitted input. If it fits in the buffer, it
returns the full byte count after buffering it. If the input is at least the
buffer capacity, it drains pending bytes and forwards the copied input directly
to the wrapped writer, returning that writer's count; the wrapped writer may
accept fewer bytes than submitted. `flush` drains pending bytes and then
flushes the wrapped writer. While an asynchronous operation is active,
overlapping operations fail with `ConcurrentOperationError`.

`BufferedWriter.close` does not flush. If bytes remain pending it raises
`UnflushedError` and leaves the resource open so the caller can retry. `finish`
is the explicit drain, flush, then close operation. A wrapped writer that
returns an invalid byte count raises `StreamProtocolError`; a drain that reports
zero progress while bytes remain raises `WriteZeroError`. Errors during a
partial drain preserve the unwritten suffix for a later retry.

`BufferedReader.read` consumes buffered bytes before asking the wrapped reader
for more. It returns a Future of the number of bytes copied into the destination.
Closing the buffered reader closes its wrapped resource.

## Error identity

`UnflushedError`, `ConcurrentOperationError`, `StreamProtocolError`, and
`WriteZeroError` are distinct resource/stream errors. Invalid argument types or
counts use `ArgumentError`. Closed-resource access uses the platform's
use-after-close error. A host I/O failure remains an error from the underlying
operation; buffering does not convert it to success.
