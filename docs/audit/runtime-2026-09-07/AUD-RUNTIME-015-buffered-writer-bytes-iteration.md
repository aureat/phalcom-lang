# AUD-RUNTIME-015 - BufferedWriter fixture depends on missing Bytes each protocol

## Classification

- Severity: Medium
- Category: Streams / collection protocol integration
- Confidence: Confirmed executed failure.

## Observed

The `BufferedWriter` stream fixture fails after reaching flush because the
runtime reports `Bytes(11) does not understand 'each(_)'`. The current `Bytes`
surface intentionally omits a local `each` override and exposes the canonical
iterator instead, while the active buffered stream implementation still sends
`each(_)` to transfer buffered chunks.

## Evidence

`cargo test -p phalcom-core --test language-corpus corpus::streams` executes
`streams_negative` successfully but `streams/buffered_stream.ph` produces only
the first three pending-byte lines and then the scheduled failure above. This
is distinct from `AUD-RUNTIME-011`: BufferedReader close ownership is repaired,
but the stream corpus remains blocked by this Bytes protocol mismatch.

## Repair direction

Make the buffered writer use the canonical Bytes iterator protocol or restore a
spec-authorized `each(_)` forwarding method. Preserve the no-user-code/native
bulk-operation boundary and add a focused buffered-stream regression before
changing the implementation.
