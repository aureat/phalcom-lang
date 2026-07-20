# Implementation spec — streams: `BytesReader`/`BytesWriter`, `BufferedReader`/`BufferedWriter` (U-STREAMS)

> **Status:** dispatch-ready. Governing records **Accepted**:
> [PDR-0005](../../../decisions/0005-resources-are-disposable-handles-not-finalized.md)
> §3c/§7/§7a; surface contract [`../core/stream-protocol.md`](../core/stream-protocol.md)
> (all of it — this unit is that spec's first implementation and runs its conformance
> harness §8 with **no filesystem and no reactor**).
> **Needs U-BYTES (✅ shipped) and U-RESOURCE (spec:
> [`resource-table.md`](resource-table.md)) — including its `Resource.register_(_)` seam
> and field-stamp harness row.**
> **Floor delta: zero.** Four `.ph` classes over the shipped `Bytes` floor and
> U-RESOURCE's primitives; no new natives, no ADR-0019 traffic.
> Read [`bytes.md`](bytes.md) §7 first. Anchors as of `e33e8e5`.

## 1. Shape

Four `.ph` classes in core.ph after `class Path` (U-PATH) or `class Bytes` if U-PATH has
not landed — the two units are order-independent:

- `BytesReader < Resource` — reads from an immutable snapshot of a `Bytes`.
- `BytesWriter < Resource` — accumulates writes in memory; `toBytes` extracts.
- `BufferedReader < Resource` / `BufferedWriter < Resource` — wrap **any** reader/writer
  (stream-protocol §4: buffering is a wrapper, never a parameter).

Plus one `.ph` error class and the harness. These are the reference implementations the
protocol names (§8: "the role kernel `List` plays for the collection protocol").

## 2. `Future` discipline for in-memory streams

Every can-block selector returns a `Future` (PDR-0004 §1) — for in-memory streams the
work is synchronous, so they return **already-settled** futures via `Future.value(_)` /
`Future.error(_)` (`core.ph`'s `class Future`; both route through the settle-once path).
This is *not* the stubbed-reactor anti-pattern PDR-0004 §2 forbids: that rule bans
faking a *blocking* operation's future; an in-memory copy genuinely cannot block, and
`Future.value` is its honest type. State this in a comment at the first use — the next
reader will otherwise "fix" it.

`await` on an already-settled future works on the root fiber today
(`concurrency_future_slice_b.ph` C-FUT-2 exercises exactly this).

## 3. The classes

### 3.1 `BytesReader`

```phalcom
class BytesReader extends Resource {
  construct new(source) {
    // snapshot: source is a Bytes, copied — the reader's contents never
    // change under it (slice copies, bytes.md law 6)
    _data = source.slice(0, source.size)
    _pos = 0
    _handle = Resource.register_("BytesReader")
  }
  read(dst) { ... }   // -> Future
}
```

- `read(dst)`: `dst.isA(Bytes)` guard (raise `ArgumentError` otherwise); raise
  `UseAfterCloseError` if `self.isClosed` (stream-protocol §3.1 law 5 — precondition
  raise, not `Err`); copy `n = min(dst.size, remaining)` octets via
  `_data.slice(_pos, _pos + n).copyInto(dst, 0)` (two native bulk ops, zero per-byte
  loops — bytes.md §3.1); advance `_pos`; return `Future.value(n)`. **At exhaustion
  `n == 0`: EOF is the settled value `0`, never an error, repeatably** (law 1).
- `close` is the inherited `Resource#close` — a reader's buffer holds nothing durable
  (stream-protocol §5.5), no override.

### 3.2 `BytesWriter`

- Fields: `_chunks` (a `List` of `Bytes` copies), `_handle`.
- `write(src)`: guards as above; append `src.slice(0, src.size)` (snapshot — caller may
  mutate their buffer after the call, law: short writes are *reported*, and an
  in-memory sink accepts everything, so this settles to `src.size`); return
  `Future.value(src.size)`.
- `flush`: `Future.value(None)` — total, an unbuffered writer settles `Ok` immediately
  (stream-protocol law 3; this is the law's reference case).
- `toBytes`: sum sizes, `Bytes.new(total)`, `copyInto` each chunk at its offset —
  bulk-op derivation, no per-byte loop.
- `close`: inherited.

### 3.3 `BufferedWriter` — the §5 contract, exactly

- `construct new(inner)`: accepts any object responding to `write(_)`/`flush` — the
  informal Writer protocol, so **no type test** (stream-protocol §1's whole point).
  Fields: `_inner`, `_buf` (a `Bytes` of capacity `8192` — a plain literal, spelled in
  exactly one place with a name comment; no capacity parameter, ADR-0043), `_len`
  (fill level), `_handle`.
- `write(src)`: if `src.size >= _buf.size`, flush then delegate straight to
  `_inner.write(src)` (large writes bypass the buffer — Rust `BufWriter`'s rule);
  else if it doesn't fit, flush first; then `src.copyInto(_buf, _len)`... `copyInto`
  copies the **whole** receiver, so partial-fit writes need
  `src.slice(a, b).copyInto(_buf, _len)` composition; settle to `src.size`.
- `pending => _len` (stream-protocol §5.3).
- `flush`: hand `_buf.slice(0, _len)` to `_inner.write(_)`, chain on its future
  (`then`/`map`, `core.ph` `Future` combinators), zero `_len` **after** the inner
  write settles `Ok`; settles to `Ok(None)`. On inner `Err`, the buffer is **kept**
  (nothing discarded — §5.3.3's spirit at the flush level too).
- `close` (overrides `Resource#close`): if `_len > 0`, **raise** `UnflushedError` with
  the pending count in the message — and **close nothing** (§5.3.2-3: inner stays
  open, buffer intact, a subsequent `finish` succeeds). Else inherited close of self;
  the inner writer is NOT closed by the wrapper's close — PDR-0005 §7's surface has no
  owns-inner rule; the caller closes what it opened. Document this explicitly; Java's
  close-cascades-and-swallows is the counterexample we are not copying.
- `finish`: `self.flush.then { _ => Future.value(self.close) }` shaped so it settles to
  the close's `Result` after a successful flush, and to the flush's `Err` without
  closing on a failed one (§5.3.5 — "a caller who wants to inspect the flush result
  before closing must be able to": `finish` is the one-call spelling, `flush`+`close`
  the inspectable one).
- `class UnflushedError extends Error {}` — pure `.ph`, next to `ArgumentError`
  (`core.ph:80`); PDR-0010 is Proposed, so the kind is carried by the class, the
  U-RESOURCE §2.4 pattern.

### 3.4 `BufferedReader`

`construct new(inner)`, fixed `_buf`, refill-on-empty from `_inner.read(_buf)` chaining
on its future, serve from the buffer; EOF propagates as settled `0`s (law 1). `close`
inherited, **no precondition** (§5.5) — discarding read-ahead loses nothing durable.

## 4. Leak-report addition (stream-protocol §7)

A `BufferedWriter` abandoned with `_len > 0` reports as a **distinct condition** from an
unclosed resource, naming the pending byte count. Mechanism: U-RESOURCE's exit reporter
walks table rows; a row's kind string is all it has. Cheapest honest implementation:
`BufferedWriter` re-registers its row's kind... it cannot — the table is write-once per
row. Instead: register as kind `"BufferedWriter"`, and the exit report, for rows of that
kind still open, prints the generic unclosed line **plus** a note that pending bytes may
be lost. The *exact* pending count at exit requires a native query seam the table does
not have; **defer the count** to the follow-up filed in §6 rather than adding a floor
primitive ad hoc. The distinct-condition wording ships; the count is best-effort absent.

## 5. Test plan

The conformance harness IS stream-protocol §8's table, run against these four types —
one golden `.ph` per row, lanes `streams/` + `streams/negative/`:

| Row (from §8) | Fixture asserts |
|---|---|
| read-to-EOF | repeated `read` at EOF settles `0`, never raises |
| short write | `BytesWriter` reports the real accepted count |
| `flush` totality | unbuffered `flush` settles `Ok` immediately |
| double `close` | second `close` is `Ok` (via U-RESOURCE idempotence) |
| use-after-close | `read`/`write` on closed raises `UseAfterCloseError` |
| dirty `close` | write, then `close` → raises `UnflushedError` naming the count |
| dirty `close` leaves state intact | after the raise: `pending` unchanged, `finish.await` then succeeds, inner received the bytes |
| `finish` on clean writer | equivalent to `close` |
| round-trip | `BytesWriter` → `toBytes` → `BytesReader` reproduces the payload across chunked writes and partial reads |
| buffer boundary | writes of size `cap-1`, `cap`, `cap+1`, `2cap+3` land correctly (the §3.3 bypass and partial-fit branches) |
| wrapper neutrality | `BufferedWriter` over `BytesWriter`: bytes arrive only after `flush`/`finish`; never before capacity or flush |

Plus: yield-inside-`then`-callback smoke (futures' combinator blocks are ordinary
flat-entry calls now — bytes.md §7.3; assert it stays true here).

## 6. Not in this unit — file these as DEFERRED entries on landing

- `writeAll(_)` loop helper (stream-protocol law 2 mentions it "above" the protocol).
- The exit-report pending-byte **count** (§4 — needs a table query seam; do not add it
  silently).
- `File`-backed streams (U-FS), `SeekFrom` (U-FS), any timeout/cancellation.
