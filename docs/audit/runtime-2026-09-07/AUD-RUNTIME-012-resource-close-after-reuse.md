# AUD-RUNTIME-012 - Resource close loses idempotency after slot reuse

## Classification

- Severity: High
- Category: Resource-v2 lifecycle / stale-handle semantics
- Confidence: Confirmed table-level sequence and public execution failure.

## Observed

`ResourceTable::close` marks a row closed and returns its index to the free
list. Reopening the index increments its generation. Closing the old handle
after reuse therefore returns `StaleHandle` instead of the stream protocol's
idempotent success for an already-closed resource. The stale handle must never
close the new occupant.

## Executed evidence

The public sequence `A.close -> open B -> A.close` raised
`UseAfterCloseError` while `B.isClosed == false`. This confirms the old
handle cannot be made idempotent across generation reuse without risking the
new occupant. Resource-v2 stale-handle isolation is working; the remaining
contract decision is to distinguish same-generation idempotence from stale
handle rejection.

## Reproducer and repair direction

Execute open A, close A, open B, close A, and verify B remains open. The
canonical repair must preserve exact generational identity while retaining
wrapper-local closed identity or an equivalent tombstone decision; do not make
stale handles close the reused row.
