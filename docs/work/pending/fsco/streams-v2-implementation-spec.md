# Streams v2 Implementation Specification

> **Status:** proposed replacement implementation specification for the shipped
> U-STREAMS unit.
>
> This specification preserves the existing public Reader / Writer / Seekable
> protocol surface and the four reference stream classes:
>
> - `BytesReader < Resource`
> - `BytesWriter < Resource`
> - `BufferedReader < Resource`
> - `BufferedWriter < Resource`
>
> It supersedes the shipped `streams-implementation-spec.md` where that record
> conflicts with Resource v2, short-write correctness, wrapper ownership,
> overlapping-operation semantics, or layered flush behavior.
>
> **Dependencies:** shipped U-BYTES, Resource v2, and the shipped `Future`
> implementation. U-FS remains downstream.
>
> **Floor delta: zero.** This unit adds no native primitive. New error classes
> are ordinary `.ph` classes.
>
> The central purpose of this revision is to make the in-memory reference streams
> and buffering layer correct for a genuinely asynchronous, short-writing `File`
> implementation rather than only for `BytesWriter`, whose writes always accept
> the complete input.

## 1. Why a v2 revision is required

The shipped stream architecture is fundamentally sound:

```text
Reader / Writer / Seekable
        informal protocols
              |
              v
     Resource for lifecycle
              |
              v
Future for can-block operations
```

However, several implementation assumptions are only valid while every inner writer
is an immediately-settled `BytesWriter`.

The shipped `BufferedWriter.flush` performs one inner `write` and then clears the entire
buffer. The Writer protocol explicitly permits short writes. Once `File` or another
real writer accepts only a prefix, that implementation silently discards the suffix.

The shipped buffering layer also has no rule for overlapping unresolved operations.
That becomes observable as soon as an inner operation settles asynchronously.

The Resource implementation has independently moved to Resource v2:

```text
register_    -> attach_
isClosed guard -> ensureOpen_
```

and the stream constructors/guards must migrate with it.

Finally, layered `flush` must propagate to the wrapped writer. Draining this wrapper's
buffer but failing to call `inner.flush` makes nested buffering semantically incorrect.

This revision fixes those issues before U-FS is allowed to depend on the stream layer.

## 2. Public surface retained

No existing stream selector is renamed or removed.

### Reader

```text
read(Bytes) -> Future
```

### Writer

```text
write(Bytes) -> Future
flush        -> Future
```

### Resource

```text
close    -> Result       // synchronous
isClosed -> Bool
```

### BufferedWriter additions already in the protocol

```text
pending -> Int
finish  -> Future
```

### Reference-type-specific surface

```text
BytesWriter#toBytes -> Bytes
```

No `writeAll` public selector is added by this unit.

No capacity argument is added to either buffering constructor.

## 3. Settlement and error-channel discipline

The shipped `Future` is a one-channel fulfilled/rejected state machine. Reactor-backed
IO settles a Future to the semantic equivalent of:

```text
Ok(value)  -> Future fulfillment
Err(error) -> Future rejection
```

There MUST NOT be a nested:

```text
Future<Result<T, E>>
```

shape.

For stream operations:

- successful `read` fulfills with an integer byte count;
- successful `write` fulfills with an integer byte count;
- successful `flush` fulfills with `None`;
- successful `finish` fulfills with the synchronous `close` result;
- asynchronous IO failure rejects the Future;
- a failure detected only after an inner Future settles also rejects the returned Future;
- synchronous contract/precondition violations raise before a Future is returned where
  they can be detected at call time.

Examples of synchronous raises:

```text
wrong argument type
use after close
overlapping operation
dirty close
```

Examples of Future rejection:

```text
inner writer IO failure
inner reader IO failure
invalid asynchronous count returned by an inner protocol implementation
zero-progress failure while the buffering layer is required to write all pending bytes
```

`await` naturally re-raises a Future rejection through the existing Future contract.

## 4. Protocol clarifications introduced by v2

### 4.1 Zero-length reads

The old shorthand:

```text
0 means EOF
```

is only unambiguous for a non-empty destination.

The clarified law is:

> For `dst.size > 0`, a fulfilled count of `0` means EOF. For `dst.size == 0`,
> `read(dst)` fulfills immediately with `0`, consumes nothing, and does not probe the
> underlying stream; that result is not evidence of EOF.

Every Reader implementation in this unit follows that rule.

### 4.2 Zero-length writes

For `src.size == 0`:

```text
write(src) -> already-fulfilled Future(0)
```

and the inner writer is not invoked.

### 4.3 Write source ownership

Once a conforming `write(src)` call returns, the writer MUST NOT subsequently depend on
the caller-owned mutable contents of `src`.

The caller may legally do:

```phalcom
const f = writer.write(bytes)
bytes.fill(0)
f.await
```

without changing which bytes the original write represents.

Therefore an asynchronously executing concrete writer must snapshot or take native
ownership of the source bytes before returning.

`BytesWriter` already satisfies this by copying.

U-FS must satisfy it by copying into owned worker-job data.

`BufferedWriter` must satisfy it in every branch where use of `src` is delayed until
after another asynchronous operation.

### 4.4 Read destination ownership

`read(dst)` is intentionally different. The reader fills caller-owned `dst` and may do
so until the returned Future settles.

The caller MUST NOT mutate or repurpose `dst` while the read Future is unresolved.

This is the ordinary output-buffer contract and does not require a destination copy.

## 5. Resource v2 migration

All four reference stream classes remain `Resource` subclasses.

### 5.1 Constructors

Remove every constructor assignment of the form:

```phalcom
_handle = Resource.register_("BytesReader")
```

and equivalent spellings.

Use the Resource v2 instance operation:

```phalcom
self.attach_("BytesReader")
```

`attach_` MUST be the final potentially failing constructor action.

Constructor ordering is therefore:

```text
1. validate arguments;
2. allocate/copy ordinary managed state;
3. initialize all stream fields;
4. attach the Resource row;
5. return.
```

A failure in steps 1-3 creates no resource-table entry.

There is no `_handle` declaration/assignment in these `.ph` constructors. Resource v2
owns the inherited handle slot.

### 5.2 Operational guards

Remove hand-written:

```phalcom
if self.isClosed {
  throw UseAfterCloseError.new(...)
}
```

guards.

Every operation that requires a live stream begins with:

```phalcom
self.ensureOpen_
```

so Resource v2 owns:

- closed detection;
- stale detection;
- malformed-handle detection;
- resource kind;
- open/close/attempt diagnostic attribution.

Stream classes do not reconstruct lifecycle diagnostics.

### 5.3 Selectors that require an open resource

The following selectors MUST guard with `ensureOpen_`:

```text
BytesReader#read

BytesWriter#write
BytesWriter#flush
BytesWriter#toBytes

BufferedReader#read

BufferedWriter#write
BufferedWriter#flush
BufferedWriter#finish
BufferedWriter#pending
```

`close` and `isClosed` are the Resource lifecycle exceptions.

A dirty `BufferedWriter#close` performs its own state check described in §11 rather
than calling `ensureOpen_` in a way that would break idempotent double close.

## 6. Wrapper ownership

### 6.1 Buffered wrappers own the inner resource

`BufferedReader` and `BufferedWriter` are owning wrappers.

A successful clean close consumes:

```text
outer wrapper
+
inner Resource
```

This aligns the accepted stream close law with actual lifetime ownership.

### 6.2 Constructor precondition

Because ownership implies closeability, a buffered wrapper requires its inner object to
be a `Resource`.

This is the one axis Phalcom deliberately reifies as a class.

The data protocol remains structural:

```text
BufferedReader:
    inner is Resource
    inner conforms to Reader by responding correctly to read(_)

BufferedWriter:
    inner is Resource
    inner conforms to Writer by responding correctly to write(_) and flush
```

Do not introduce `Reader` or `Writer` marker classes.

Constructor validation may check:

```phalcom
inner.is(Resource)
```

but does not type-test for a Reader/Writer class that does not exist.

An object that is a Resource but violates the informal data protocol fails when the
required selector is sent or when its settlement violates §8.

### 6.3 No optional ownership mode

There is no:

```text
ownsInner:
closeInner:
leaveOpen:
```

parameter in v2.

A non-owning buffering abstraction, if ever needed, is a separate API decision rather
than an options flag on these reference classes.

## 7. Overlapping-operation law

### 7.1 Rule

A stateful stream wrapper permits at most one unresolved state-mutating operation.

For `BufferedReader`:

```text
read
```

is state-mutating.

For `BufferedWriter`:

```text
write          when it requires asynchronous drain/delegation
flush
finish
```

may be unresolved and therefore own the operation gate.

While such an operation is unresolved, another state-mutating operation raises
synchronously:

```text
ConcurrentOperationError
```

`close` also raises `ConcurrentOperationError` while an operation is unresolved because
the Resource close contract is synchronous and may not wait for that operation.

### 7.2 Why reject rather than secretly serialize

V2 deliberately rejects overlap rather than installing an implicit per-stream Future
queue.

An invisible queue would introduce:

- hidden memory growth;
- cancellation ordering;
- close-behind-queued-work semantics;
- backpressure policy;
- error fan-out policy.

Those are larger abstractions and do not belong in the first filesystem-capable stream
substrate.

The rejection model is also the dynamic equivalent of the exclusive mutable access that
prevents overlapping stream mutation in ownership-oriented systems.

### 7.3 `_busy`

`BufferedReader` and `BufferedWriter` each carry:

```phalcom
_busy = false
```

A helper or equivalent implementation discipline MUST ensure:

```text
operation starts       -> _busy = true
Future fulfills        -> _busy = false
Future rejects         -> _busy = false
synchronous start raise-> _busy restored to false
```

No failure path may permanently wedge a stream busy.

The implementation MUST handle synchronous exceptions thrown while invoking the inner
stream before an unresolved Future has been safely registered.

Using the existing Phalcom exception machinery plus `Future.then` / `Future.catch` is
acceptable. The exact helper spelling is implementation-local.

### 7.4 Immediately-settled operations

A purely synchronous state mutation that returns an already-settled Future need not leave
`_busy` true after return.

Examples:

```text
small BufferedWriter write that fits entirely in its local buffer
BytesReader read
BytesWriter write
```

It must still reject if `_busy` was already true because another operation owns the
wrapper state.

### 7.5 Observers

`isClosed` is always available.

`BufferedWriter#pending` is an observer, not a state mutation. It may be read while an
operation is busy and returns the wrapper's current committed local pending count.

During an in-progress drain, implementations MAY continue to report the pre-drain `_len`
until the drain either succeeds or fails; `pending` is a snapshot, not a progress meter.

## 8. Inner protocol result validation

Because buffering wraps structural protocols, it must defend its own invariants from
malformed inner implementations.

### 8.1 Writer count

For a call:

```text
inner.write(bytes_of_length_n)
```

a fulfilled count MUST be an integer satisfying:

```text
0 <= count <= n
```

A negative, fractional, non-numeric, or greater-than-requested count is a protocol
violation.

When detected after settlement, reject the outer Future with:

```text
StreamProtocolError
```

### 8.2 Reader count

For:

```text
inner.read(buffer_of_capacity_n)
```

a fulfilled count MUST be an integer satisfying:

```text
0 <= count <= n
```

The same invalid-count cases reject with `StreamProtocolError`.

### 8.3 Zero-progress during required draining

The public Writer protocol allows a write to report a short count. A zero count on a
non-empty input cannot be retried indefinitely by a write-all loop.

When `BufferedWriter` is draining already-accepted buffered bytes and an inner writer
fulfills:

```text
0
```

for a non-empty remainder, the drain rejects with:

```text
WriteZeroError
```

This is not represented as EOF and does not discard pending data.

A conforming asynchronous/nonblocking writer should keep its Future pending until it can
make progress rather than settling `0` as a would-block signal.

## 9. `BytesReader`

### 9.1 State

```text
_data : Bytes    immutable snapshot owned by reader
_pos  : Int
Resource handle : inherited, attached by Resource v2
```

### 9.2 Construction

```phalcom
@constructor
new(source) {
  // validate source is Bytes
  // _data = source.slice(0, source.size)
  // _pos = 0
  // self.attach_("BytesReader")  // final fallible action
}
```

The snapshot is load-bearing: mutation of the original `source` after construction cannot
change future reads.

### 9.3 `read(dst)`

Order:

```text
1. ensureOpen_
2. validate dst is Bytes
3. if dst.size == 0, return Future.value(0)
4. remaining = _data.size - _pos
5. n = min(dst.size, remaining)
6. if n > 0, bulk-copy that prefix to dst and advance _pos
7. return Future.value(n)
```

At exhaustion and non-empty `dst`:

```text
n == 0
```

is stable EOF. Repeated reads continue to fulfill `0`.

No per-octet `.ph` loop is introduced.

## 10. `BytesWriter`

### 10.1 State

```text
_chunks : List<Bytes>
Resource handle : inherited
```

### 10.2 Construction

Initialize `_chunks`, then:

```phalcom
self.attach_("BytesWriter")
```

as the final fallible constructor action.

### 10.3 `write(src)`

Order:

```text
1. ensureOpen_
2. validate src is Bytes
3. zero length -> Future.value(0)
4. append src.slice(0, src.size)
5. return Future.value(src.size)
```

The copy occurs before return, satisfying §4.3.

### 10.4 `flush`

`BytesWriter` has no lower buffering layer:

```text
ensureOpen_
return Future.value(None)
```

### 10.5 `toBytes`

`toBytes` requires an open writer.

It computes aggregate size with checked integer/allocation semantics from Bytes v2,
allocates one result `Bytes`, and bulk-copies chunks in order.

No unchecked aggregate-size wrap is allowed.

### 10.6 `close`

Inherited Resource v2 close.

The stored chunks are managed data, not an external resource.

## 11. `BufferedWriter`

### 11.1 State

```text
_inner : Resource conforming to Writer
_buf   : Bytes(8192)
_len   : Int            // committed bytes pending in this wrapper
_busy  : Bool
Resource handle : inherited
```

Buffer capacity remains exactly:

```text
8192
```

in one documented location.

No capacity constructor parameter is added.

### 11.2 Construction

Conceptual order:

```text
validate inner is Resource
_inner = inner
_buf   = Bytes.new(8192)
_len   = 0
_busy  = false
self.attach_("BufferedWriter")
```

`attach_` is last.

### 11.3 Core invariant

When not busy:

```text
0 <= _len <= _buf.size
pending bytes are exactly _buf[0 .. _len]
```

During an asynchronous drain, the implementation may retain a snapshot and local accepted
offset. On settlement it atomically commits either:

```text
full success:
    _len = 0

failure after accepting prefix k:
    _buf[0 .. old_len-k] = original[k .. old_len]
    _len = old_len-k
```

A failure may never cause accepted bytes to be re-sent accidentally or unaccepted bytes
to be discarded.

## 12. BufferedWriter internal drain

### 12.1 `_drainBuffer_` concept

V2 needs an internal operation with semantics:

```text
write every byte currently accepted into this wrapper
to _inner, tolerating legal short writes,
without calling _inner.flush.
```

It is internal composition, not a new public selector.

If `_len == 0`, it returns an already-fulfilled Future.

Otherwise:

1. snapshot `_buf[0 .. _len]`;
2. retain a local accepted offset initially `0`;
3. call `_inner.write(remaining)`;
4. validate the fulfilled count;
5. if count > 0, advance the accepted offset;
6. if bytes remain, repeat through Future continuation composition;
7. if count == 0 while bytes remain, reject `WriteZeroError`;
8. on full completion set `_len = 0`;
9. on rejection preserve exactly the unaccepted suffix in `_buf` and update `_len`.

The loop MUST NOT assume one inner `write` accepts the whole chunk.

### 12.2 No `await` inside implementation machinery

`_drainBuffer_` composes Futures with the existing continuation machinery rather than
blocking the caller with an internal `await`.

The public operation returns the composed Future immediately.

### 12.3 Snapshot lifetime

The drain snapshot is wrapper-owned managed data captured by the continuation chain.
It may remain live until the operation settles.

No worker thread ever receives a Phalcom `Bytes`; U-FS remains responsible for converting
File writes to owned plain worker data.

## 13. BufferedWriter `write(src)`

### 13.1 Entry

Order:

```text
1. ensureOpen_
2. validate src is Bytes
3. reject if _busy
4. src.size == 0 -> Future.value(0)
```

### 13.2 Small write that fits

If:

```text
src.size < _buf.size
and
_len + src.size <= _buf.size
```

copy `src` synchronously into `_buf` at `_len`, increase `_len`, and fulfill with the
complete `src.size`.

This path is immediately settled and does not remain busy.

### 13.3 Small write that requires a drain

If:

```text
src.size < _buf.size
and
_len + src.size > _buf.size
```

the caller's source must be copied **before return** because its eventual insertion is
delayed until the current buffer drains:

```text
owned = src.slice(0, src.size)
```

Then one gated asynchronous operation:

```text
drain current buffer completely
then copy owned into now-empty _buf
set _len = owned.size
fulfill with owned.size
```

If draining fails, the new source is not accepted into the wrapper and the returned Future
rejects.

The old buffer retains exactly its unwritten suffix.

### 13.4 Large write bypass

If:

```text
src.size >= _buf.size
```

large writes bypass local buffering.

If `_len == 0`:

```text
delegate immediately to _inner.write(src)
```

and return the inner accepted count after validation.

The inner writer is responsible for §4.3 source ownership once its call returns.

If `_len > 0`, ordering requires the old buffered bytes to be drained first. Since use of
the new source is delayed, snapshot it synchronously:

```text
owned = src.slice(0, src.size)
```

Then:

```text
drain old buffer completely
then _inner.write(owned)
fulfill with the validated accepted count
```

The direct large write is still allowed to short-write. `BufferedWriter#write` does not
turn every public write into write-all.

Only bytes that had already been accepted into the wrapper's internal buffer are subject
to the mandatory write-all drain.

### 13.5 Ordering law

If a `BufferedWriter.write` accepts bytes A into its local buffer before a later write B,
no byte from B may be handed to the inner writer before all accepted bytes of A have been
handed off.

## 14. BufferedWriter `flush`

### 14.1 Layered meaning

Public `flush` means:

```text
1. hand every byte accepted by this wrapper to the inner writer;
2. then call inner.flush;
3. fulfill None only after both succeed.
```

Therefore `flush` MUST propagate even when this wrapper's own `_len == 0`.

The following implementation is wrong:

```text
if _len == 0:
    Future.value(None)
```

because an inner buffered layer may still contain pending data.

### 14.2 Internal composition

Conceptually:

```text
_drainBuffer_()
    then _inner.flush
    then fulfill None
```

Do not conflate `_drainBuffer_` with `flush`.

Internal buffer drains used only to make room for later writes MUST NOT impose an
unnecessary durability/flush barrier on the inner writer.

### 14.3 Failure states

If `_drainBuffer_` rejects:

- `inner.flush` is not called;
- this wrapper retains the unwritten suffix;
- the public Future rejects.

If all local bytes are accepted but `inner.flush` rejects:

- this wrapper's `_len` is `0`;
- bytes are already owned by the inner layer and are not reconstructed locally;
- the Future rejects;
- a later retry of `flush` calls `inner.flush` again even with `_len == 0`.

## 15. BufferedWriter `pending`

```text
ensureOpen_
return _len
```

`pending` means bytes still retained in **this wrapper's local buffer**.

It does not recursively include pending bytes in nested inner buffering.

It is an `Int`, not a generic fractional Number.

## 16. BufferedWriter `close`

`close` remains synchronous, never flushes, and never waits.

### 16.1 Decision order

```text
1. if already closed:
       return inherited idempotent Ok
2. if _busy:
       raise ConcurrentOperationError
3. if _len > 0:
       raise UnflushedError
4. attempt to close the owned inner Resource
5. once inner.close returns a Result, close this wrapper's Resource row
6. return the inner close Result (wrapper managed close is infallible in this unit)
```

### 16.2 Dirty close

If `_len > 0`:

```text
raise UnflushedError
```

The diagnostic MUST contain the exact `_len`.

The operation closes neither:

```text
wrapper
inner
```

and does not mutate `_buf` or `_len`.

A subsequent:

```phalcom
writer.finish.await
```

may recover normally.

### 16.3 Why inner closes before wrapper state

The inner close may itself synchronously raise a precondition error.

The canonical example is nested buffering:

```text
outer BufferedWriter: local pending == 0
inner BufferedWriter: pending > 0
```

Calling `outer.close` causes `inner.close` to raise `UnflushedError`.

In that case the outer wrapper MUST remain open so the caller can still recover through:

```phalcom
outer.finish.await
```

Therefore the outer Resource row is not closed before the inner close call has returned
its `Result`.

### 16.4 Inner close returning `Err`

Resource v2 consumes a resource once a native close attempt occurs even when the close
returns an error.

Accordingly, if:

```text
inner.close -> Err(e)
```

the outer wrapper is then marked closed and returns that same `Err(e)`.

A second outer `close` returns idempotent `Ok` and does not retry the inner close.

### 16.5 Double close

If the wrapper is already closed:

```text
close -> Ok
```

without sending another `close` to the inner resource.

Idempotence does not mean repeating side effects.

## 17. BufferedWriter `finish`

`finish` is one atomic lifecycle operation from the wrapper's overlap perspective.

It returns a Future and performs:

```text
flush fully through the inner layer
then close the ownership chain
```

### 17.1 Entry

```text
ensureOpen_
reject synchronously if _busy
acquire the operation gate
```

### 17.2 Success

On successful flush:

```text
perform the clean close sequence from §16 internally
fulfill with its Result
release _busy
```

No unrelated write/flush/close may interleave between the successful flush and close.

### 17.3 Flush failure

If local drain or inner flush rejects:

```text
do not close wrapper
do not close inner
release _busy
reject with the original failure
```

The caller may inspect state and retry.

### 17.4 Close failure

If flush succeeds and inner close returns `Err(e)`:

```text
wrapper becomes closed
Future fulfills with Err(e)
```

because `close` itself is a synchronous `Result`-returning operation.

A synchronous precondition raise from the close sequence becomes a Future rejection when it
occurs inside the `finish` continuation; the operation gate MUST still be released.

### 17.5 Clean `finish`

Even with `_len == 0`, `finish` still calls `inner.flush` before close.

This is required for nested buffering.

## 18. `BufferedReader`

### 18.1 State

```text
_inner : Resource conforming to Reader
_buf   : Bytes(8192)
_pos   : Int
_len   : Int
_busy  : Bool
Resource handle : inherited
```

Invariant when not busy:

```text
0 <= _pos <= _len <= _buf.size
unconsumed local bytes are _buf[_pos .. _len]
```

### 18.2 Construction

Order:

```text
validate inner is Resource
_inner = inner
_buf   = Bytes.new(8192)
_pos   = 0
_len   = 0
_busy  = false
self.attach_("BufferedReader")
```

`attach_` is final.

### 18.3 `read(dst)` entry

```text
1. ensureOpen_
2. validate dst is Bytes
3. reject if _busy
4. if dst.size == 0, Future.value(0)
```

### 18.4 Serve local read-ahead first

If:

```text
_pos < _len
```

copy:

```text
min(dst.size, _len - _pos)
```

bytes to `dst`, advance `_pos`, and return an already-fulfilled Future.

The wrapper does not unnecessarily refill merely because the destination could hold more.

Reader `read` is allowed to return a short positive count.

### 18.5 Refill

If local read-ahead is exhausted:

```text
_pos == _len
```

perform one gated:

```text
_inner.read(_buf)
```

operation.

When it fulfills:

- validate `0 <= count <= _buf.size`;
- if `count == 0`, set `_pos = 0`, `_len = 0`, fulfill `0`;
- otherwise set `_pos = 0`, `_len = count`;
- copy at most `dst.size` bytes from the new buffer;
- advance `_pos`;
- fulfill the copied count.

If the inner Future rejects:

- local state remains a valid exhausted-buffer state;
- release `_busy`;
- reject with the same error.

If the inner count is malformed, reject `StreamProtocolError`.

### 18.6 EOF

Once an inner non-empty refill reports `0`, repeated reads may call the inner reader again;
the protocol does not require `BufferedReader` to permanently cache EOF.

A stable source such as `BytesReader` or `File` will continue to report `0`.

Implementations MAY cache EOF as an optimization if doing so cannot hide source semantics, but
v2 does not require an `_eof` field.

## 19. BufferedReader `close`

`BufferedReader` owns its inner Resource.

Decision order:

```text
1. if already closed:
       return inherited idempotent Ok
2. if _busy:
       raise ConcurrentOperationError
3. call inner.close
4. once it returns a Result, close this wrapper
5. return the inner Result
```

Unread read-ahead bytes are discarded; they are not durable output and require no flush.

As with BufferedWriter, if the inner close synchronously raises a precondition error, the outer
wrapper remains open.

If inner close returns `Err`, the wrapper is still consumed/closed and returns that `Err`.

Double close does not resend inner close.

## 20. New `.ph` error classes

The v2 stream implementation uses:

```phalcom
class UnflushedError is Error {}
class ConcurrentOperationError is Error {}
class StreamProtocolError is Error {}
class WriteZeroError is Error {}
```

`UnflushedError` already exists in the shipped implementation.

The other three are pure `.ph` classes and add zero primitives.

### `ConcurrentOperationError`

Synchronous programmer/precondition error.

Examples:

```text
read while a BufferedReader refill is unresolved
flush while a BufferedWriter drain is unresolved
close while finish is unresolved
```

### `StreamProtocolError`

Represents a malformed result from a structural inner Reader/Writer.

Examples:

```text
writer reports -1
writer reports requested + 1
reader reports buffer size + 1
reader reports a fractional count
```

When discovered asynchronously, it rejects the outer Future.

### `WriteZeroError`

Represents inability of the buffering layer to make progress while it is obligated to deliver
already-accepted non-empty pending bytes.

It rejects the operation Future and preserves the still-unwritten data.

## 21. Shutdown and leak reporting

### 21.1 Resource v2 remains authoritative

The VM resource table reports every still-open stream Resource at orderly shutdown.

`BufferedReader` and `BufferedWriter` therefore remain visible through the ordinary Resource
leak mechanism.

### 21.2 Remove the unimplementable exact dirty-count shutdown requirement

The old stream implementation contract required the Resource-table exit reporter to distinguish a
dirty `BufferedWriter` and report its exact pending byte count.

Resource v2 deliberately stores no `Value`/`ObjRef` and cannot inspect a `.ph` instance's `_len`.

V2 therefore changes the shutdown requirement:

```text
open BufferedWriter at shutdown
    -> report as leaked BufferedWriter resource
```

The reporter MAY include a static note:

```text
buffered data may remain unflushed
```

but MUST NOT claim the writer is definitely dirty unless it has actual state evidence.

No native "update leak metadata on every buffer mutation" seam is added.

### 21.3 Exact count remains at the actionable failure point

The exact pending count is mandatory on synchronous dirty `close`, where `_len` is directly
available.

That is the point where the programmer can still recover.

### 21.4 Busy resources at shutdown

Streams do not invent shutdown cancellation.

U-REACTOR is responsible for stopping producers/completions before Resource v2 performs its final
leak snapshot and drain.

A stream implementation must not mutate its wrapper state from a worker thread.

## 22. File-by-file implementation plan

### 22.1 `phalcom-core/core/core.ph`

Replace the shipped stream blocks in place.

#### Resource migration

For all four classes:

```text
remove `_handle = Resource.register_(...)`
add final `self.attach_(...)`
replace manual isClosed guards with `ensureOpen_`
```

#### Error classes

Add:

```phalcom
ConcurrentOperationError
StreamProtocolError
WriteZeroError
```

beside `UnflushedError`.

#### `BytesReader`

Patch constructor and `read` per §9.

#### `BytesWriter`

Patch constructor and guard `write`, `flush`, `toBytes` per §10.

#### `BufferedWriter`

Rewrite rather than incrementally patch the existing one-write `flush`.

Add `_busy`.

Implement private/internal helpers sufficient to express:

```text
validated inner count
write-all local drain
busy acquisition/release
clean ownership close
flush propagation
```

Exact underscore selector names are implementation-local and do not join the public stream
protocol.

#### `BufferedReader`

Add `_busy`, Resource v2 migration, count validation, overlap rejection, and owning close.

### 22.2 No Rust primitive changes

This unit should not need a new native primitive.

If implementation work appears to require one, stop and justify why the behavior cannot be
expressed through the existing:

```text
Bytes bulk primitives
Future continuation machinery
Resource v2 primitives
```

before changing the floor.

### 22.3 Census/invariants

Primitive census delta remains:

```text
0
```

No bootstrapped class is added by Streams v2.

Resource's existing bootstrapped class/invariants remain covered by Resource v2.

## 23. Required synthetic harness streams

Testing only against `BytesReader` / `BytesWriter` is insufficient because they settle
immediately and never short-write.

Add test-only `.ph` Resource subclasses.

### 23.1 `ShortWriter`

Behavior:

```text
accept at most N bytes per write
record accepted bytes
flush succeeds
close is observable/countable
```

Used to prove repeated short-write drain.

### 23.2 `ZeroWriter`

For any non-empty write:

```text
Future.value(0)
```

Used to prove `WriteZeroError` and no infinite continuation loop.

### 23.3 `PrefixThenErrorWriter`

Behavior:

```text
first write accepts a prefix
next write rejects with test error
```

Used to prove the exact unwritten suffix remains pending.

### 23.4 `RejectingFlushWriter`

Accepts writes but rejects `flush`.

Used to prove:

```text
local buffer can be empty
outer flush still rejects
finish does not close after flush failure
```

### 23.5 `ControlledWriter`

Returns unresolved Futures that the fixture settles manually.

Used to prove overlap rejection, close-while-busy behavior, and source snapshotting.

### 23.6 `ControlledReader`

Returns an unresolved read Future.

Used to prove overlap rejection, close-while-busy, and destination/refill sequencing.

### 23.7 Malformed-count streams

Writer/reader variants that report:

```text
negative
fractional
non-number
greater than request/capacity
```

Used to prove `StreamProtocolError`.

## 24. Test matrix — Resource v2 migration

Required tests for each reference stream class:

```text
construction attaches exactly one Resource row
constructor failure before attach creates no row
isClosed false after successful construction
close succeeds
double close succeeds
operation after close raises Resource v2 UseAfterCloseError
diagnostic points to user open/close/attempt locations per Resource v2
```

No stream test should depend on the old `register_` selector.

## 25. Test matrix — BytesReader / BytesWriter

### BytesReader

```text
snapshot source
partial reads
read exactly to end
repeated EOF with non-empty dst
zero-length dst returns 0 without advancing
use after close
wrong dst type
```

### BytesWriter

```text
snapshot every source
chunk order
zero-length write
flush on open writer
toBytes aggregate
toBytes after close raises use-after-close
flush after close raises use-after-close
checked aggregate allocation behavior from Bytes v2
```

## 26. Test matrix — BufferedWriter short-write correctness

### 26.1 Full drain through repeated short writes

Fill wrapper with known bytes.

Inner `ShortWriter` accepts, for example:

```text
3 bytes per write
```

Call `flush`.

Assert:

```text
all bytes arrive, in order
inner.write called repeatedly
pending == 0
inner.flush called exactly once after drain
```

### 26.2 One-byte writer

Use an inner writer accepting one byte per call across a modest fixture.

Proves the continuation loop does not assume large progress.

### 26.3 Zero progress

Dirty wrapper over `ZeroWriter`.

`flush` rejects `WriteZeroError`.

Assert:

```text
pending unchanged
buffered data preserved
inner.flush not called
wrapper remains open
```

### 26.4 Prefix then failure

Inner accepts prefix K, then rejects.

Assert:

```text
returned Future rejects original error
pending == original_len - K
retained bytes equal exact unwritten suffix
retry with healthy inner behavior does not resend prefix
```

If the fixture cannot swap inner behavior, make the test writer recover on a later call.

### 26.5 Malformed count

Inner reports more bytes than requested.

Assert:

```text
StreamProtocolError
pending data preserved according to zero accepted valid progress
no out-of-bounds Bytes operation occurs
```

## 27. Test matrix — flush propagation

### Empty outer / dirty inner

Create:

```text
BufferedWriter(
    BufferedWriter(
        BytesWriter
    )
)
```

Arrange:

```text
outer.pending == 0
inner.pending > 0
```

Call outer `flush`.

Assert:

```text
inner is drained
base BytesWriter receives bytes
both wrapper pending counts become 0
```

This specifically catches the shipped:

```text
if _len == 0 -> Future.value(None)
```

bug.

### Inner flush rejection

Own buffer drains successfully, then inner `flush` rejects.

Assert:

```text
outer.pending == 0
outer remains open
flush Future rejects
retry calls inner.flush again
```

## 28. Test matrix — source mutation after call

### Delayed small write

Arrange wrapper nearly full so a new small write must wait for drain.

```phalcom
const src = Bytes.fromList(...)
const future = writer.write(src)
src.fill(0)
```

Settle the controlled drain.

Assert the newly buffered/written bytes are the pre-mutation snapshot.

### Delayed large bypass

Arrange non-empty old buffer and call a large bypass write.

Mutate original `src` immediately after `write` returns.

After old-buffer drain completes, assert inner receives the original large-write bytes.

These are mandatory because the shipped continuation shape captures caller-owned `src` across an
asynchronous boundary.

## 29. Test matrix — overlap

For each controlled unresolved operation:

### BufferedWriter

While `flush` unresolved:

```text
write -> ConcurrentOperationError
flush -> ConcurrentOperationError
finish -> ConcurrentOperationError
close -> ConcurrentOperationError
pending -> allowed snapshot
isClosed -> allowed
```

While delayed `write` unresolved, same mutation restrictions apply.

While `finish` unresolved, no write may interleave between its flush and close.

After fulfillment or rejection:

```text
busy cleared
next operation allowed
```

### BufferedReader

While refill unresolved:

```text
second read -> ConcurrentOperationError
close -> ConcurrentOperationError
isClosed -> allowed
```

After fulfillment or rejection, gate is released.

## 30. Test matrix — close ownership

### BufferedReader

```text
close outer -> inner close exactly once
double close outer -> inner not closed again
inner close Err -> outer becomes closed and returns same Err
inner synchronous precondition raise -> outer remains open
```

### BufferedWriter clean close

Same ownership assertions.

### Dirty local close

```text
pending > 0
close raises UnflushedError with exact count
outer remains open
inner remains open
pending unchanged
```

### Nested dirty inner

Outer local buffer clean, inner BufferedWriter dirty.

```text
outer.close
```

must raise inner `UnflushedError`.

Assert:

```text
outer remains open
inner remains open
outer.finish.await can subsequently recover
```

This locks the required inner-before-outer close ordering.

## 31. Test matrix — `finish`

### Dirty writer

```text
finish
-> drains local buffer with short-write tolerance
-> inner.flush
-> inner.close
-> outer close
```

Assert exact ordering with an instrumented inner writer.

### Clean writer

Even with outer `pending == 0`:

```text
inner.flush occurs
then close
```

### Flush failure

Assert:

```text
Future rejects
outer open
inner open
close not called
busy cleared
```

### Inner close `Err`

Assert:

```text
Future fulfills with Err
outer closed
inner consumed
second outer close is Ok
```

### No interleave

With controlled flush pending, a concurrent `write` or `close` raises until finish settles.

## 32. Test matrix — BufferedReader

Required:

```text
serve local buffered data without refill
short positive read allowed
refill once buffer exhausted
zero-length dst does not call inner
inner EOF returns 0
malformed inner count -> StreamProtocolError
inner rejection propagates
busy cleared after rejection
close owns inner
double close
use after close
```

Add a controlled reader that fills a known prefix and reports a short count.

## 33. Stream-protocol documentation amendments

The normative stream protocol should be amended alongside implementation landing to state:

1. `0` means EOF for reads into a non-empty destination; a zero-length destination is a
   no-op returning `0`.
2. a writer may no longer depend on source `Bytes` after `write` returns.
3. stateful stream wrappers reject overlapping unresolved operations in v2.
4. buffered wrapper ownership includes the inner Resource.
5. `BufferedWriter.flush` propagates to `inner.flush`.
6. exact dirty pending-byte count is required at dirty `close`, not at VM shutdown.
7. shutdown reports an open `BufferedWriter` as a resource leak without pretending the
   Resource table can inspect `.ph` `_len`.

This is specification reconciliation, not a new selector surface.

## 34. Implementation ordering

1. Land Resource v2 first.
2. Add the new pure `.ph` error classes.
3. Migrate `BytesReader` and `BytesWriter` to `attach_` / `ensureOpen_`.
4. Add their zero-length/closed-operation regressions.
5. Rewrite `BufferedWriter` internal drain around legal short writes.
6. Add flush propagation and source-snapshot behavior.
7. Add `_busy` gate and overlap tests.
8. Correct BufferedWriter close/finish ownership sequencing.
9. Rewrite BufferedReader gate/count validation/owning close.
10. Add synthetic short/zero/failing/controlled harness streams.
11. Run the full existing stream conformance suite.
12. Run Resource leak/strict shutdown regressions.
13. Only then use these classes as the U-FS stream-conformance target.

## 35. What MUST NOT happen

- No one-write assumption in `BufferedWriter.flush`.
- No clearing `_len` merely because one inner write fulfilled.
- No infinite retry after a non-empty write fulfills `0`.
- No hidden automatic operation queue in v2.
- No `await` inside stream implementation code to fake asynchronous composition.
- No caller-owned `Bytes` retained across a delayed write boundary.
- No `register_`.
- No hand-built `UseAfterCloseError` from stream classes.
- No clean buffered close that performs blocking flush work.
- No dirty close that discards data.
- No outer Resource close before an owned inner synchronous precondition check can fail.
- No assumption that an empty outer buffer means the inner buffering stack is flushed.
- No Resource-table reference to `.ph` stream objects merely to inspect `_len`.
- No new native primitive unless an independently reviewed floor justification proves
  existing `.ph` composition insufficient.
- No `Reader`, `Writer`, or `Seekable` marker class introduced by this unit.

## 36. Downstream U-FS obligations

U-FS may rely on Streams v2 only after these laws are green.

A conforming `File` Writer must:

```text
snapshot source bytes before write returns
allow legal short writes
return validated integer counts
reject IO failures through the Future channel
reject/forbid overlapping stateful operations under the FS ownership model
respond to flush
```

A conforming `File` Reader must:

```text
fill dst before settlement
report 0 EOF for non-empty dst
report only counts within dst capacity
obey the same overlap/lifecycle policy
```

The buffering layer then works over File without special cases.

## 37. Acceptance gates

Streams v2 is complete only when all of the following are true:

1. every stream constructor uses Resource v2 atomic attachment;
2. every live-operation guard delegates to `ensureOpen_`;
3. `BufferedWriter` drains legal short writes without losing or duplicating bytes;
4. zero progress cannot spin forever;
5. partial success followed by failure preserves exactly the unwritten suffix;
6. layered flush always reaches `inner.flush`, including when local pending is zero;
7. delayed writes snapshot caller-owned source bytes before return;
8. overlapping unresolved state mutations are deterministically rejected;
9. the operation gate clears on fulfillment, rejection, and synchronous start failure;
10. clean buffered close owns and closes the inner Resource;
11. dirty close leaves both resources and all pending bytes intact;
12. nested dirty-inner close leaves the outer wrapper recoverably open;
13. `finish` is an atomic flush-then-close lifecycle operation;
14. `BufferedReader` validates inner counts and owns its inner Resource;
15. zero-length read/write semantics are explicitly tested;
16. shutdown leak reporting does not require impossible `.ph` state inspection;
17. the existing stream conformance harness remains green;
18. the new short-write, overlap, nested-buffer, and failure-state harnesses are green;
19. primitive census remains unchanged;
20. U-FS can use the resulting protocol without adding stream-specific exceptions.
