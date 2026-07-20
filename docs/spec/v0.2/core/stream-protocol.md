# Specification — Stream Protocol (readers, writers, and the close contract)

> **Status:** **Normative contract.** The `Resource` class, the `close` contract, and the informal
> Reader/Writer/Seekable protocols encode
> [PDR-0005](../../../decisions/0005-resources-are-disposable-handles-not-finalized.md) §3/§3a/§3b/§3c
> and [PDR-0004](../../../decisions/0004-io-is-future-shaped-reactor-owned.md) §1. **§5 is the
> protocol-level statement of PDR-0005 §7a**, ruled in that record's third revision (2026-07-20);
> this document holds the laws and the conformance harness, §7a holds the ruling and its rationale.
> Adds **zero** floor primitives — no [ADR-0019](../../adr/accepted/0019-freeze-vm-blessed-primitive-floor.md)
> amendment. Selector spellings follow [ADR-0012](../../adr/accepted/0012-selector-signature-encoding-and-dispatch.md)
> and [ADR-0043](../../adr/accepted/0043-no-default-arguments-keep-selector-identity-pristine.md).
>
> **Owner:** unassigned. Normative for any `BufferedWriter` implementation.

## 1. Why one class and three informal protocols

`File` must be readable, writable, seekable **and** closeable — four axes against one `extends`
slot, since Phalcom is single-inheritance with no traits or mixins
([ADR-0041](../../adr/accepted/0041-hierarchy-stability-policy.md) DEC-U13b).

PDR-0005 §3 reifies exactly one axis, with a stated test: **an axis earns a class only when some
mechanism needs to *ask the type*, rather than just send to it.** Two mechanisms need to ask about
closeability — leak reporting (§7) and any generic cleanup path. Nothing ever needs to ask "is
this a Reader?"; it just sends `read(_)`.

So:

| Axis | Form | Why |
|---|---|---|
| Closeable | **`Resource`, a kernel root class** | leak reporting and generic cleanup must ask |
| Readable | informal protocol | callers only ever send `read(_)` |
| Writable | informal protocol | callers only ever send `write(_)` |
| Seekable | informal protocol | callers only ever send `seek(_)` |

This mirrors [ADR-0048](../../adr/accepted/0048-amend-iteration-bare-cursor-sentinel-and-iterable-root.md)'s
`Iterable`-as-kernel-root for the one axis that benefits, and declines it for the three that would
collide with the single `extends` slot. Reifying all four needs stateless interface-style
declarations, deferred to
[`io-protocol-axes-need-stateless-interfaces.md`](../../../deferred/io-protocol-axes-need-stateless-interfaces.md).

Conformance to the three informal protocols is a **documented contract plus a harness**, the shape
[`collection-protocol.md`](collection-protocol.md) already uses — not a declaration and not a class.

## 2. The informal protocols

Every selector that can block returns a `Future`, per PDR-0004 §1. Selectors that cannot block
return their value directly.

### Reader

| Selector | Returns | Meaning |
|---|---|---|
| `read(_)` | `Future` | fill the given `Bytes`; settles to a count. **0 means EOF**, never an error |

### Writer

| Selector | Returns | Meaning |
|---|---|---|
| `write(_)` | `Future` | write the given `Bytes`; settles to the count accepted |
| `flush` | `Future` | make previously-accepted bytes durable at the next layer down |

`flush` is on Writer, not on `BufferedWriter` alone: an unbuffered writer's `flush` settles
immediately with `Ok`, so generic code can always call it. This is what makes §5's discipline
expressible without type tests.

### Seekable

| Selector | Returns | Meaning |
|---|---|---|
| `seek(_)` | `Future` | reposition; `SeekFrom.start(_)` / `.current(_)` / `.end(_)` |
| `position` | `Future` | current offset |

## 3. `Resource` — the reified axis

```
Resource#close    -> Result      // SYNCHRONOUS; idempotent
Resource#isClosed -> Bool
```

`File < Resource`, `TcpStream < Resource`, `Dir < Resource`, and — by §5 — `BufferedWriter < Resource`.

### 3.1 Laws

1. **Synchronous.** `close` returns `Result`, never a `Future`. PDR-0005 §3b: with an unbuffered
   `File` (§3c) there is nothing to flush, and `close(2)` on a local descriptor is not a blocking
   operation. This is a law about the *contract*, so §5 must not break it.
2. **Never blocks.** No implementor of `close` may perform a write syscall. Residual blocking work
   is a separate, explicitly `Future`-returning selector — `File#sync` (fsync),
   `TlsStream#shutdown` (`close_notify`), and §5's `finish`.
3. **Fallible.** `close` returns `Result` because `close(2)` can fail, and an `EIO` on close means
   buffered data was lost *after* the write already reported success. Discarding that is the
   silent-data-loss bug Go linters flag `defer f.Close()` for.
4. **Idempotent.** Closing an already-closed resource is `Ok`, not an error.
5. **Use-after-close raises.** A send to a closed resource is a contract violation, not an expected
   condition, so it raises rather than returning `Err` — an `Err` would hide a programmer bug in
   the same channel as a genuine IO error, where it gets ignored (PDR-0005 §4). The diagnostic
   names the resource and the site that closed it. Carries `kind: #useAfterClose`.
6. **Precondition violations raise; IO failures return `Err`.** The general form of law 5, and what
   §5 rests on.

## 4. Buffering is a wrapper, never a parameter

```
BufferedWriter.new(_)          // wraps any writer
BufferedReader.new(_)          // wraps any reader
```

There is no `buffered:` argument on `File.open`. A flag makes buffering a property of the file
*type*; a wrapper makes it a property of the *use*, which is what it actually is. C's `setvbuf`
mutates the stream in place and is the counterexample; Java's `BufferedOutputStream`-wraps-
`FileOutputStream` and Rust's `BufWriter`-wraps-`File` are both the wrapper form, and both are
considered correct.

PDR-0005 §3c makes this load-bearing rather than stylistic: a synchronous `close` (§3.1 law 1) is
only honest if close has nothing to flush. A buffered `File` would force close to either block on a
write syscall or become asynchronous again.

## 5. `BufferedWriter#close`

> Ruled in [PDR-0005](../../../decisions/0005-resources-are-disposable-handles-not-finalized.md)
> §7a, third revision, 2026-07-20. This section states the resulting contract; §7a states the
> reasoning and the two shapes it forecloses.

### 5.1 The problem restated

§3c relocates the data-loss hazard rather than removing it. `File` has nothing to flush;
`BufferedWriter` does. PDR-0005 §7a names three shapes and rules none:

1. `BufferedWriter` is not a `Resource` — no `close`. Honest, and easy to forget the flush.
2. `BufferedWriter#close -> Future` — flushes then closes. Breaks the uniform contract.
3. `BufferedWriter#close -> Result` flushing synchronously. Uniform, but blocks on a write.

### 5.2 Two of the three are foreclosed by records already accepted

**Shape 3 is unavailable.** PDR-0003 guarantees a single VM thread. A synchronous flush that blocks
on a write syscall blocks *every fiber*, not just the caller. It also contradicts §3.1 law 2
directly.

**Shape 2 defeats the reason `Resource` is a class.** §1's test is that closeability is reified so
leak reporting and generic cleanup can ask the type. If `close` returns `Result` on `File` and
`Future` on `BufferedWriter`, no generic cleanup path can be written — the caller must type-test
before closing, which is precisely the question reification was supposed to answer.

**Shape 1 is Go's `bufio.Writer`.** Go has no `Close` on a buffered writer; you must call `Flush`.
The resulting bug — write, close the file, never flush, get an empty file — is a standing Go
papercut. Rust's `BufWriter` takes the adjacent position: it flushes on `Drop` but **ignores the
error**, which is why `into_inner()` exists to recover it, and is the most-cited wart in that API.
Java and Python both chose shape 3, which they can afford because they block a thread rather than
a scheduler.

### 5.3 Ruling: `BufferedWriter` **is** a `Resource`; a dirty `close` raises

```
BufferedWriter#close   -> Result     // from Resource: synchronous, never blocks
BufferedWriter#flush   -> Future     // where the blocking work lives
BufferedWriter#finish  -> Future     // flush, then close — the recommended spelling
BufferedWriter#pending -> Number     // buffered bytes not yet handed to the inner writer
```

1. **`close` never flushes and never blocks.** §3.1 laws 1 and 2 hold unamended; no exception is
   carved into PDR-0005 §3b.
2. **`close` on a non-empty buffer raises**, with `kind: #unflushed`. The diagnostic names the
   pending byte count and the site that opened the writer. This is §3.1 law 6: writing to a buffer
   and then closing without flushing is a programmer error, in the same category as use-after-close
   — not an IO failure, so not an `Err`.
3. **A raising `close` closes nothing.** The inner writer stays open and the buffer stays intact,
   so the caller can still `flush.await` and close. Nothing is lost and nothing is silently
   discarded. If the caller instead abandons the writer, §7's leak report catches it.
4. **`close` on an empty buffer is an ordinary `Resource#close`** — synchronous, `Result`,
   idempotent — and closes the inner writer.
5. **`finish` is the recommended spelling** and the one documentation should teach:
   `w.finish.await` is flush-then-close as a single awaitable call. Two selectors rather than one
   because ADR-0043 forbids default arguments, and because a caller who wants to inspect the flush
   result before closing must be able to.

### 5.4 Why this dissolves the trilemma rather than picking a corner

| Cost | Shape 1 | Shape 2 | Shape 3 | §5.3 |
|---|---|---|---|---|
| Silent data loss | **yes** | no | no | no |
| Blocks the scheduler | no | no | **yes** | no |
| Non-uniform `close` | **yes** (absent) | **yes** (type varies) | no | no |
| Easy to get wrong | **yes** | no | no | no — it raises |

The mistake shape 1 permits silently becomes loud **at the moment it is made**, not at exit. That
is strictly better than a leak report alone, which tells you after the data is already gone.

Precedent for an explicit fallible finisher: Rust's `BufWriter::into_inner()` returns
`Result<W, IntoInnerError<W>>` — it fails when the flush fails, and the error hands the writer back
so nothing is stranded. §5.3's raising `close` is that shape, with the recovery expressed as "the
resource is untouched" instead of as a returned value.

### 5.5 `BufferedReader` has no equivalent problem

A reader's buffer holds bytes already taken from the source that the caller has not consumed.
Discarding them loses nothing durable. `BufferedReader#close` is an ordinary `Resource#close`:
synchronous, idempotent, closes the inner reader, no precondition.

## 6. Laws, consolidated

A conformant stream type satisfies:

1. **EOF is not an error.** `read(_)` settling to `0` is end-of-input. Readers never raise at EOF.
2. **Short writes are reported, not hidden.** `write(_)` settles to the count accepted, which may be
   less than requested. Writing everything is the caller's loop, or `writeAll(_)` above it.
3. **`flush` is total.** Every writer responds to `flush`; an unbuffered one settles `Ok`
   immediately.
4. **`close` is synchronous, fallible, idempotent, and never blocks** (§3.1).
5. **Precondition violations raise, IO failures return `Err`** (§3.1 law 6). Use-after-close and
   dirty-close are the two instances in this document.
6. **Blocking work is always visible in the selector's type.** If it can block, it returns a
   `Future` (PDR-0004 §1). `File#path` is deliberately not a `Future` — it is cached at open.

## 7. Leak-reporting obligations

PDR-0005 §5 reports resources open at exit. This document adds one obligation:

- A `BufferedWriter` abandoned with a non-empty buffer is reported **as a distinct condition** from
  an unclosed resource, naming the pending byte count and the allocation site. Losing buffered
  writes and leaking a descriptor are different bugs with different fixes, and a report that
  conflates them sends the reader to the wrong place.

Both conditions are reported by default and escalated to a raise by `System.strictResources(true)`.
Test lanes set it.

## 8. Conformance harness

A type is a conformant Reader / Writer / Seekable **iff it passes the harness**, not by inspection —
the [`collection-protocol.md`](collection-protocol.md) §1 rule. The harness must cover, per type:

| Check | Asserts |
|---|---|
| read-to-EOF | law 1; repeated `read(_)` at EOF keeps settling `0`, never raises |
| short write | law 2; a writer that accepts fewer bytes reports the real count |
| `flush` totality | law 3; an unbuffered writer's `flush` settles `Ok` |
| double `close` | §3.1 law 4; second `close` is `Ok` |
| use-after-close | §3.1 law 5; raises, `kind: #useAfterClose` |
| dirty `close` | §5.3.2; raises, `kind: #unflushed` |
| dirty `close` leaves state intact | §5.3.3; after the raise, `pending` is unchanged and a subsequent `finish.await` succeeds |
| `finish` on a clean writer | §5.3.5; equivalent to `close` |

`BytesReader` / `BytesWriter` (in-memory, no syscall) are the reference implementations — the role
kernel `List` plays for the collection protocol — so the harness runs with no filesystem.

## 9. What this document does not cover

- **`Path`, `Fs`, `File.open` modes.** PDR-0005 §7 ratifies the selector surface; the filesystem
  spec is separate and unwritten.
- **The reactor.** PDR-0004 rules IO reactor-owned and requires the reactor before the IO surface.
  This document says what the surface *is*, not how a `Future` settles.
- **Reifying the other three axes.** Deferred to
  [`io-protocol-axes-need-stateless-interfaces.md`](../../../deferred/io-protocol-axes-need-stateless-interfaces.md).
  Full mixins and traits are the heavier alternative and are **not** what that defers to.
- **`Bytes`.** Every selector here takes or fills a `Bytes`, which does not exist in the tree
  yet. Its spec is [`bytes.md`](bytes.md), normative upon ratification of
  [PDR-0011](../../../decisions/0011-admit-bytes-native-octet-buffer.md) (Proposed
  2026-07-20). Still a hard dependency for any implementation.
