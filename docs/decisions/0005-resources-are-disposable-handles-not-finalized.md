# PDR-0005 — Native resources are closeable handles with a generation-tagged table; no finalizers

- Status: Accepted
- Date: 2026-07-20
- Revised: 2026-07-20, same day, **before any implementation**, in two passes. Changes:
  `dispose`/`isDisposed` → **`close`/`isClosed`** (§3a); `close` returns **`Result`,
  synchronously** — first revised to `Future`, then reversed on the Java/Python/Lua/Wren evidence
  once it became clear close is not a blocking operation over an unbuffered file (§3b);
  **`File` is unbuffered**, buffering moves to an explicit wrapper, which is what makes the
  synchronous close honest (§3c); **`Resource` is a real root class** while Reader/Writer/Seekable
  stay duck-typed (§3); use-after-close **raises** rather than returning `Err` (§4); **`using` sugar
  withdrawn** (§6); selector surface ratified (§7); the `BufferedWriter#close` trilemma is recorded
  as **unruled** (§7a). **Third revision, 2026-07-20, still before implementation:** §7a is now
  **ruled** — `BufferedWriter` is a `Resource`, `close` never flushes and raises on a non-empty
  buffer, `finish` is the flush-then-close spelling. Amended in place rather than as a separate
  decision because nothing was built against the original.
- Related: [ADR-0050](../adr/accepted/0050-non-moving-mark-sweep-collector.md) (§Context banks
  "no finalizers exist" as a reason the collector is hazard-free — **this decision keeps that
  true**), [ADR-0013](../adr/accepted/0013-closure-upvalues-and-frame-token-return.md)
  (frame tokens — the generation-tag precedent),
  [ADR-0008](../adr/accepted/0008-layered-exceptions-and-result.md) §4 (unified unwind — what
  makes `ensure`-scoping sound), [PDR-0004](0004-io-is-future-shaped-reactor-owned.md)
  (the resources this governs), [ffi.md](../spec/v0.2/drafts/ffi.md) §8 **F-3** (this closes it),
  `docs/logs/2026-07-19-ensure-temp-root-uaf.md`

## Context

ADR-0050 lists "No finalizers exist" among the reasons its collector has no ordering or
resurrection hazards. Native resources — file descriptors, sockets, directory handles — are the
classic reason a language grows finalizers anyway, and [ffi.md](../spec/v0.2/drafts/ffi.md) F-3
recorded the collision without ruling on it.

The precedents are unusually clear. Java deprecated `finalize`. Python's `__del__` still permits
resurrection. C# shipped finalizers *and* still needed `IDisposable`, which is what people
actually use. Every language that took release-on-collect ended up with explicit disposal as
well, having paid for both.

Phalcom also has the mechanism already, twice: frame tokens (ADR-0013) and the generational
`SlotMap` that makes a stale handle a **defined panic rather than undefined behavior**. A
resource table is that same idea a third time.

## Decision

### 1. No finalizers. ADR-0050 §Context stands unamended

Nothing is released because it was collected. Collection frees memory and nothing else.

### 2. Resources are ordinary escapable handles

`File.open(_)` returns a handle that may be stored in a field, returned, and passed around — a
server holds a listening socket for its whole life, and a scope-only design cannot express that.

### 3. `Resource` is a real root class; the other IO axes are duck-typed

`File` needs to be readable, writable, seekable **and** closeable — four axes against one `extends`
slot (U-INH single inheritance; Phalcom has no traits or mixins). Only one axis gets reified, and
it is closeability:

```
Resource                          // kernel root class
Resource#close    -> Future       // settles to Result; idempotent
Resource#isClosed -> Bool
```

`File < Resource`, `TcpStream < Resource`, `Dir < Resource`. Reader / Writer / Seekable stay
**informal protocols** — a type participates by responding to `read(_)` / `write(_)` / `seek(_)`,
with no declaration and no class.

Closeability is the axis that earns a class because two mechanisms need to *ask the type*: leak
reporting (§5) and any generic cleanup path. Nothing needs to ask "is this a Reader?" — it just
sends `read(_)`. This follows the [ADR-0048](../adr/accepted/0048-amend-iteration-bare-cursor-sentinel-and-iterable-root.md)
`Iterable`-as-kernel-root precedent for the one axis that benefits, and declines it for the three
that would collide with single inheritance.

Reifying all four needs **stateless interface-style declarations** — checkable identity, no shared
implementation, no contributed fields, so ADR-0011's frozen slot offsets and ADR-0012's dispatch are
both untouched. That is a real language feature, deliberately **not** decided here — see
[`docs/deferred/io-protocol-axes-need-stateless-interfaces.md`](../deferred/io-protocol-axes-need-stateless-interfaces.md).
Full mixins/traits are the heavier alternative and are **not** what this defers to.

### 3a. The name is `close`, not `dispose`

This protocol exists to serve files and sockets, so it uses their word. `dispose` (this decision's
original spelling) reads as generic-resource ceremony on a `File`.

### 3b. `close` is **synchronous** and returns `Result`

```
Resource#close -> Result        // synchronous; idempotent
```

It returns `Result` and not `None` because `close(2)` can fail, and an `EIO` on close means
**buffered data was lost after the write already reported success**. Discarding that is a
silent-data-loss bug — the one Go linters flag `defer f.Close()` for on writable files.

It is **synchronous**, and does not return a `Future`. Four precedents point the same way:

- **Java** ran cleanup on a separate finalizer thread and retreated: `finalize` deprecated in 9,
  `AutoCloseable.close() throws Exception` + try-with-resources is the survivor, synchronous.
  Suppressed-exception machinery exists because a close during unwind can throw while another
  exception is in flight — the cost of taking close-can-fail seriously.
- **Python** discourages `__del__`; `with`/`__exit__` is the survivor, synchronous. `async with`
  was added *alongside*, not instead. Decisively: **you cannot await in `__del__`**, which is why
  PEP 525 needed `shutdown_asyncgens()` as a whole subsystem, and why aiohttp's "Unclosed client
  session" is a permanent fixture.
- **Lua** has `__gc` finalizers and *still* added to-be-closed variables (`local f <close>`,
  `__close`) in 5.4 — synchronous, scope-exit — because `__gc` timing is unpredictable.
- **Wren**, the nearest sibling: a foreign class's `finalize` runs during GC and may not call back
  into the VM at all. A GC-time hook in a Phalcom-shaped VM likewise could not run `.ph` code,
  allocate, or re-enter the interpreter — so it could never await, settle a `Future`, or reach the
  reactor.

Two cleanups were being conflated. **Safety-net** cleanup cannot be asynchronous in any of these
languages; **explicit** cleanup can. Phalcom has no safety net by §1, so async `close` would break
nothing — but all four made synchronous scoped cleanup the *primary* mechanism, and async variants
are additions bolted alongside.

**This does not carve an exception into [PDR-0004](0004-io-is-future-shaped-reactor-owned.md)
§1.** That rule governs operations that *can block*. Given §3c's unbuffered `File`, `close` has
nothing to flush and `close(2)` on a local descriptor is fast — so it is not a blocking operation
and never was. The earlier `Future` spelling came from assuming it was.

Residual blocking is handled explicitly rather than by making every close async: `File#sync`
(fsync) and `TlsStream#shutdown` (`close_notify`) are separate `Future`-returning selectors, the
shape Java uses for `SSLSocket`. `close(2)` can still block briefly on NFS or FUSE; that is
accepted, as it is by all four languages above.

`close` is idempotent — closing twice is `Ok`, not an error.

### 3c. `File` is unbuffered; buffering is an explicit wrapper

`File#write(_)` is a direct syscall. Buffering lives in `BufferedWriter`, which wraps any writer and
whose `flush` is explicit and asynchronous — Rust's `File`/`BufWriter` and Java's
`FileOutputStream`/`BufferedOutputStream`.

This is what makes §3b honest rather than a trick: a synchronous `close` is only safe if close has
nothing to flush. A buffered `File` would force close to either block on a write syscall or go async
again, and a close that cannot report a failed flush loses data silently.

### 4. A generation-tagged resource table in the VM

The OS handle lives in a VM-side table, not in the Phalcom object. The object holds a
generation-tagged index. Disposal bumps the generation.

Consequences that make this the load-bearing choice:

- Use-after-close **raises**, and never becomes a reused-fd write to the wrong file — the same
  guarantee, by the same mechanism, that `SlotMap` gives the object heap. It raises rather than
  returning `Err` because it is a contract violation, not an expected condition: an `Err` would
  hide a programmer bug in the same channel as a genuine IO error, where it gets ignored. The
  diagnostic names the resource and the site that closed it.
- Collected-without-close is a **leak, not a UAF**. It is detectable and reportable.
- The table is a GC root for nothing: it holds OS handles, not `Value`s.

### 5. Leaks are reported, not silently tolerated

```
System.leakReport -> List        // undisposed resources, with their allocation site
System.strictResources(_)        // Bool; raise on leak instead of warning
```

Undisposed resources at exit produce a diagnostic naming the allocation site. Test lanes should
set `strictResources(true)`.

### 6. ~~`using` — scoped disposal sugar~~ **WITHDRAWN 2026-07-20, before implementation**

No dedicated syntax for now. Scoped cleanup is written with the existing `ensure`:

```phalcom
const f = File.open("a.txt").await
{ f.readAll.await } .ensure { f.close.await }
```

Withdrawn rather than deferred-with-a-design, because sugar should follow evidence of the pain it
removes, and no `File` user exists yet to produce that evidence. Revisit once the surface has real
callers; the lowering was never the hard part.

Note for whoever revisits: `ensure` dropped live values whenever its cleanup block collected until
`cdd2117` (`docs/logs/2026-07-19-ensure-temp-root-uaf.md`), so any `using` lowering onto `ensure`
inherits that fix as a hard dependency. And per §3b, `close` returns a `Future` — so the sugar must
decide whether it awaits, which is the async-`using` problem and the real reason this is not a
five-line desugar.

### 7. The ratified surface

```
Resource#close                 -> Result     // SYNCHRONOUS; idempotent
Resource#isClosed              -> Bool

File.open(_)                   -> Future     // read-only
File.create(_)                 -> Future     // write + truncate
File.openWith(_, mode:)        -> Future     // OpenMode.read/.write/.append/.readWrite
File#read(_)                   -> Future     // fills a Bytes, settles to count; 0 = EOF
File#write(_)                  -> Future     // direct syscall — File is unbuffered (§3c)
File#sync                      -> Future     // fsync; explicit, genuinely blocking
File#seek(_)                   -> Future     // SeekFrom.start(_)/.current(_)/.end(_)
File#position                  -> Future
File#metadata                  -> Future
File#path                      -> Path       // cached; non-blocking, so no Future
File#close                     -> Result     // from Resource

BufferedWriter.new(_)                        // wraps any writer
BufferedWriter#write(_)        -> Future
BufferedWriter#flush           -> Future     // where the blocking work lives
BufferedWriter#close           -> Result     // never flushes; raises if pending > 0 (§7a)
BufferedWriter#finish          -> Future     // flush then close — the recommended spelling
BufferedWriter#pending         -> Number     // buffered bytes not yet handed to the inner writer
BufferedReader.new(_)                        // wraps any reader
BufferedReader#read(_)         -> Future

TlsStream#shutdown             -> Future     // close_notify; explicit, blocking

System.leakReport              -> List       // open resources + allocation site
System.strictResources(_)      -> None       // Bool; raise on leak instead of warn
```

### 7a. `BufferedWriter#close` — ruled: it is a `Resource`, and a dirty close raises

§3b + §3c relocate the data-loss hazard rather than removing it. `File` has nothing to flush, but
**`BufferedWriter` does**, and a synchronous `close` cannot flush it without blocking on a write.
Three shapes were recorded here as an open trilemma. Two are foreclosed by records already
accepted, and the third is Go's known papercut:

1. **Not a `Resource`** — no `close`; the caller flushes then closes the underlying file. This is
   Go's `bufio.Writer`, whose standing failure mode is write-close-forget-flush and an empty file.
   Rust's `BufWriter` takes the adjacent position — flushes on `Drop` but **ignores the error** —
   which is why `into_inner()` exists and is that API's most-cited wart.
2. **`close -> Future`** — defeats the reason §3 reifies closeability at all. That axis earns a
   class because leak reporting and generic cleanup must *ask the type*; if `close` returns
   `Result` on `File` and `Future` here, no generic cleanup path can be written without a type
   test, which is the question reification was supposed to answer.
3. **`close -> Result` flushing synchronously** — unavailable. [PDR-0003](0003-no-user-visible-threads-fibers-and-isolates.md)
   guarantees a single VM thread, so a blocking write syscall inside `close` blocks **every fiber**,
   not just the caller. Java and Python both chose this shape and can afford it because they block
   a thread rather than a scheduler.

**The ruling.** `BufferedWriter` **is** a `Resource` with the uniform synchronous `close -> Result`,
and closing with a non-empty buffer **raises**:

```
BufferedWriter#close   -> Result     // from Resource: synchronous, never blocks, never flushes
BufferedWriter#flush   -> Future     // where the blocking work lives
BufferedWriter#finish  -> Future     // flush, then close — the recommended spelling
BufferedWriter#pending -> Number     // buffered bytes not yet handed to the inner writer
```

- `close` on a non-empty buffer raises with `kind: #unflushed`, naming the pending byte count and
  the site that opened the writer.
- A raising `close` **closes nothing** — the inner writer stays open and the buffer stays intact,
  so the caller can still `flush.await` and close. Nothing is lost and nothing is discarded.
- `close` on an empty buffer is an ordinary `Resource#close`.
- `finish` is what documentation should teach; two selectors rather than one because
  [ADR-0043](../adr/accepted/0043-no-default-arguments-keep-selector-identity-pristine.md) forbids
  default arguments, and a caller who wants to inspect the flush result before closing must be able
  to.

This is not a fourth invented option — it is **§4's rule applied unchanged**: precondition
violations raise, IO failures return `Err`, because an `Err` would hide a programmer bug in the same
channel as a genuine IO error. Writing to a buffer and then closing without flushing is a
programmer error in exactly the same category as use-after-close. §3b's claim that `close` never
blocks survives with no exception carved into it.

The nearest precedent for an explicit fallible finisher is Rust's `BufWriter::into_inner()`, which
returns `Result<W, IntoInnerError<W>>` — it fails when the flush fails and hands the writer back so
nothing is stranded. This is that shape, with recovery expressed as "the resource is untouched"
rather than as a returned value.

**What it costs.** The common case is two calls (`flush.await`, then `close`) unless the caller
uses `finish`. Accepted: the mistake shape 1 permits *silently* now becomes loud at the moment it is
made, rather than at exit — which is strictly better than a leak report alone, since a report
arrives after the data is already gone.

§5's leak reporter stays load-bearing, for a narrower reason: a `BufferedWriter` abandoned with a
non-empty buffer must be reported as a **distinct condition** from an unclosed resource, naming the
pending byte count. Losing buffered writes and leaking a descriptor are different bugs with
different fixes, and a report that conflates them sends the reader to the wrong place.

Full protocol, laws, and conformance harness:
[`stream-protocol.md`](../spec/v0.2/core/stream-protocol.md).

Selector spellings follow [ADR-0012](../adr/accepted/0012-selector-signature-encoding-and-dispatch.md)
(comma form; `openWith(_, mode:)` is one selector with a labelled second parameter) and
[ADR-0043](../adr/accepted/0043-no-default-arguments-keep-selector-identity-pristine.md) (no default
arguments — `open`/`create`/`openWith` are three selectors, not one with defaults). Native
primitives carry the trailing `_` marker (`close_`, `read_`, `write_`) with the `.ph` surface above
them, per U-NATIVE-MARKER.

`File#path` is deliberately **not** a `Future`: it is cached at open time and cannot block. 0068 §1
governs operations that *can* block, not every member.

## Consequences

- ADR-0050 is **not** reopened; its "no finalizers" premise remains literally true.
- Forgetting `dispose` leaks an fd for the process lifetime. That is the accepted cost, and §5 is
  what keeps it observable rather than mysterious. A long-running server that leaks will exhaust
  its fd limit — the leak report is the intended way to find that, and it must be built with the
  table rather than after it.
- The resource table must be drained on VM shutdown, and on isolate teardown if
  [PDR-0003](0003-no-user-visible-threads-fibers-and-isolates.md) §2 is ever built.
- `using`'s desugaring makes `ensure` load-bearing for resource safety, so `ensure`'s own
  correctness is now a resource-safety property. Its GC-rooting regression test
  (`ensure_outcome_survives_collecting_cleanup`) should be read as guarding this decision too.
- **Closes ffi.md F-3.**

## Alternatives rejected

- **Scope-only (`using` with no escapable handle).** Provably leak-free and cheapest, but cannot
  express a socket held in a field. Too tight for the surface 0068 implies.
- **Real finalizers / release-on-collect.** Reopens ADR-0050, imports finalizer ordering and
  object resurrection, and — per §Context — historically arrives *alongside* explicit disposal
  rather than instead of it, so it buys a hazard without removing the API.
- **Refcounted handles.** Would make disposal automatic and deterministic, but reintroduces the
  `Rc` shape ADR-0009 removed by construction, and cycles through a resource-holding object would
  leak silently.
