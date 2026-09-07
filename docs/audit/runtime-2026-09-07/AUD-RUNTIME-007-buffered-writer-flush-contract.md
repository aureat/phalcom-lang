# AUD-RUNTIME-007 — Buffered flush forgets unaccepted bytes

## Classification

Severity: High for data correctness. **Confirmed by source control flow; not executed.** Examined HEAD `1e22b57ff8cd0c61b12aeca4128671c9fabdef6e`. No source changes or tests in this slice.

## Contract and implementation

The [stream protocol](../../spec/current/stdlib/stream-protocol.md), §§2, 5.3 and 6, permits short writes, defines pending bytes as those not yet handed to the inner writer, and requires flush to make accepted bytes durable at the next layer.

`core/universe/src/collections/bytes.ph::BufferedWriter.flush` copies the pending buffer into a chunk, calls `_inner.write(chunk)`, then accepts a `bytesWritten` callback parameter but never reads it. The callback sets `_len = 0` unconditionally and returns a fulfilled Future. `concurrency/fiber.ph::Future.then` passes the fulfilled value into the callback and flattens its result; it does not retry short writes.

## Concrete consequence derived from source

With three pending bytes and an inner writer that accepts one byte and fulfills with count 1, the callback sets pending to zero. The other two bytes are no longer represented as pending output. `finish` then calls `close`, whose dirty-buffer guard now sees zero. This is a valid short-write scenario under the protocol, not malformed input. It is an analytical trace, not a retained execution log.

A second flush-contract gap is visible: neither the empty-buffer nor nonempty-buffer branch calls `_inner.flush`. For a nested buffered writer, completing the outer write need not make bytes durable at the next layer. The reference BytesWriter accepts entire writes immediately, so it masks both distinctions.

## Root cause and direction

The implementation treats successful Future completion as complete transfer, discarding the quantitative write result and the separate flush obligation. Preserve the suffix until it is accepted; define progress handling for zero-count writes; and propagate the inner flush completion. Do not repair this by changing the protocol to forbid short writes.

Preserve the useful dirty-close design: current close checks `_len` before closing either layer and keeps pending bytes intact when it raises. Close must remain synchronous; residual work belongs in flush/finish.

## Evidence and limitations

Primary anchors: `bytes.ph` BufferedWriter at line 489, flush at line 528, close/finish immediately after; `fiber.ph::Future.then` at line 287. Existing `streams/buffered_stream.ph` and `buffered_writer_boundary.ph` wrap BytesWriter and inspect small successful writes. They were read, not run. No claim of complete test inventory, loader validation or source-level reproduction is made.

Future verification: a deterministic short-writing in-memory sink, zero-progress sink, rejected write, nested buffered writers and clean flush. Check exact output bytes, retained suffix, pending count and inner flush calls. Resolve overlapping outstanding writes separately rather than assuming serialization.
