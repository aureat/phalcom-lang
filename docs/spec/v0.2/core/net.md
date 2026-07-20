# Specification — Network (`TcpStream`, `TcpListener`, `Dns`, `IpAddr`, `Shutdown`)

> **Status:** **Proposed — normative upon ratification of
> [PDR-0015](../../../decisions/0015-network-surface-tcp-dns-endpoints.md)** (rule 5:
> never design on an unratified record; no unit builds this until that record is
> Accepted). The already-Accepted inputs it composes:
> [PDR-0004](../../../decisions/0004-io-is-future-shaped-reactor-owned.md) §1/§3
> (`Future` for can-block; poller-vs-pool split),
> [PDR-0005](../../../decisions/0005-resources-are-disposable-handles-not-finalized.md)
> §3/§4/§7 (`TcpStream < Resource`; the resource table; `TlsStream#shutdown -> Future`),
> [`stream-protocol.md`](stream-protocol.md) (Reader/Writer protocols, `close` laws),
> [`reactor.md`](reactor.md) §3/§7 (completion lifecycle, cancellation substrate).
> **Floor delta: nonzero, enumerated in [`../impl/net.md`](../impl/net.md)** (PDR-0015
> ruling 13), censused at impl time under
> [PDR-0012](../../../decisions/0012-numeric-tower-implementation-and-floor-amendment.md)
> ruling 21's rebase discipline. Selector spellings follow ADR-0012 and ADR-0043
> (no default arguments, no flags, no options bags — every variant its own selector).
> **Build order:** requires reactor phase 2 (the poller — [`../impl/net.md`](../impl/net.md));
> phase 1 ([`../impl/reactor.md`](../impl/reactor.md)) deliberately shipped without it.
>
> **Owner:** unassigned.

## 1. Scope and the shape of the surface

Split by what can block (PDR-0004 §1), stated per kind exactly as
[`filesystem.md`](filesystem.md) §1 does:

| Kind | Blocks? | Returns |
|---|---|---|
| `IpAddr` / `Shutdown` operations (§2, §3) | never — value manipulation | plain values |
| `TcpListener.bind` (§5) | never — `bind(2)`/`listen(2)` fail or succeed immediately | `Result` |
| `TcpStream` establishment + IO (§4) | yes | `Future`; `close` is the synchronous `Result` from `Resource` |
| `TcpListener#accept` (§5) | yes | `Future` |
| `Dns.resolve` (§6) | yes — blocking `getaddrinfo`, no pollable form | `Future` |
| endpoint accessors (`peerAddr` etc.) | never — cached at establishment | plain values |

Mechanism assignment (PDR-0015 ruling 3): sockets ride the **poller**; DNS rides the
**worker pool**. Both settle through the phase-1 completion pipeline unchanged.

## 2. `IpAddr`

An immutable `.ph` value class over the 4 (IPv4) or 16 (IPv6) octets, owned exclusively —
the `Path` pattern, PDR-0013 rulings 1/2 applied verbatim: construction copies in,
`bytes` copies out, structural `==`, content `hash` cached at construction. Immutable +
value-hashed ⇒ a valid `Map`/`Set` key (collection-protocol law 4).

| Selector | Returns | Meaning |
|---|---|---|
| `IpAddr.parse(_)` | `IpAddr` \| `None` | from text (`"127.0.0.1"`, `"::1"`); `None` on anything else — parse failure is data, not a contract violation |
| `IpAddr.ofBytes(_)` | `IpAddr` | from a `Bytes` of length 4 or 16 (defensive copy); any other length **raises** — a wrong-shaped program, not wrong-shaped data |
| `IpAddr.loopbackV4` / `IpAddr.anyV4` | `IpAddr` | `127.0.0.1` / `0.0.0.0` |
| `IpAddr.loopbackV6` / `IpAddr.anyV6` | `IpAddr` | `::1` / `::` |
| `isV4` / `isV6` | `Bool` | by octet length |
| `bytes` | `Bytes` | defensive copy of the octets |
| `toString` | `String` | canonical text — dotted quad; RFC 5952 lowercase-compressed for v6. Total: every `IpAddr` has a valid text form (unlike `Path`, nothing lossy here) |
| `==(_)` / `!=(_)` / `hash` | | value semantics |

**Laws:** no selector touches the network; parse/display are pure `.ph`; the octets are
the wire form and the only thing that crosses a native boundary (PDR-0015 ruling 9 —
the PDR-0013 ruling-4 posture). No aliasing in either direction.

## 3. `Shutdown`

Three singleton objects, the `OpenMode` pattern (PDR-0013 ruling 5; no numbers, no
flags): `Shutdown.read`, `Shutdown.write`, `Shutdown.both`. A plain `.ph` class with
three static instances and `toString`.

## 4. `TcpStream`

`TcpStream < Resource` (PDR-0005 §3). [`stream-protocol.md`](stream-protocol.md) §3 laws
apply verbatim: synchronous idempotent fallible `close`, use-after-close raises
`kind: #useAfterClose`. The descriptor lives in the VM-side generation-tagged resource
table (PDR-0005 §4), never in the object. It is a conformant Reader and Writer (§2 of
that spec) and must pass its harness.

| Selector | Returns | Meaning |
|---|---|---|
| `TcpStream.connect(_, port:)` | `Future` | `String` hostname; resolves, then attempts each address sequentially in resolver order; first success settles `Ok(TcpStream)`; all-fail settles `Err` carrying the last failure (PDR-0015 ruling 4) |
| `TcpStream.connectAddr(_, port:)` | `Future` | `IpAddr`; no DNS anywhere near it |
| `read(_)` | `Future` | fill the given `Bytes`, settle to count; **0 = peer's FIN = EOF**, never an error (stream law 1; PDR-0015 ruling 6) |
| `write(_)` | `Future` | settle to count accepted; short writes reported, not hidden (stream law 2) |
| `flush` | `Future` | settles `Ok` immediately — the socket is kernel-buffered, there is no user-space buffer; kept for Writer-protocol totality (stream law 3) |
| `shutdown` | `Future` | end-of-writes: sends FIN, settles `Ok` at the next pump. The **protocol** selector — uniform with `TlsStream#shutdown` (PDR-0015 ruling 5) |
| `shutdown(_)` | `Future` | directional, TCP-only: takes a `Shutdown` singleton |
| `peerAddr` / `localAddr` | `IpAddr` | cached at establishment; never a syscall, never a `Future` |
| `peerPort` / `localPort` | `Number` | same |
| `close` | `Result` | from `Resource`; pending operations settle `Err(#closed)` — §7 |

Ports: integral `Number` in `1..65535`; out-of-range or fractional **raises** (PDR-0015
ruling 9). Hostname: `String`; the resolver gets its UTF-8 bytes as-is, IDNA not
performed (Q-N3); resolution failure settles `Err(#dnsFailure)`.

**One pending read and one pending write per stream** (PDR-0015 ruling 7): a second
concurrent `read(_)` — or `write(_)` — while one is pending raises
`kind: #concurrentOperation`. Read-concurrent-with-write is legal; every echo/proxy
needs it. This is the stream-protocol §3.1 law-6 split: program structure error, so a
raise, not an `Err`.

Buffering is a wrapper, never a parameter (stream-protocol §4): `BufferedWriter.new(s)` /
`BufferedReader.new(s)` work over a `TcpStream` unchanged, including the `#unflushed`
dirty-close raise and `finish`.

## 5. `TcpListener`

`TcpListener < Resource` (PDR-0015 ruling 1). Same close/use-after-close laws.

| Selector | Returns | Meaning |
|---|---|---|
| `TcpListener.bind(_, port:)` | `Result` of `TcpListener` | **synchronous** — `bind(2)`/`listen(2)` do not block, and blocking-visibility cuts both ways (PDR-0015 ruling 2). Takes `IpAddr` only, never a hostname. Port `0..65535`; `0` = OS-assigned |
| `accept` | `Future` | settles `Ok(TcpStream)` for the next connection. Concurrent `accept`s from several fibers are legal; they settle in call order (FIFO) |
| `localAddr` | `IpAddr` | cached at bind |
| `localPort` | `Number` | cached at bind; reveals an OS-assigned port after `bind(addr, port: 0)` |
| `close` | `Result` | from `Resource`; pending `accept`s settle `Err(#closed)` — §7 |

In-use bind settles nothing — it *returns* `Err(#addrInUse)` synchronously. The listen
backlog is an implementation constant with a doc comment (PDR-0015 ruling 12; the Q-R2
posture), never a knob.

## 6. `Dns`

A static namespace, never instantiated.

| Selector | Settles to | Meaning |
|---|---|---|
| `Dns.resolve(_)` | `Result` of `List` of `IpAddr` | `getaddrinfo` on the worker pool; resolver order preserved (RFC 6724 ordering is the resolver's job, not re-sorted here). Empty result is `Err(#dnsFailure)`, not `Ok([])` — a name that resolves to nothing did not resolve |

Reverse lookup, record-type queries (TXT/MX/SRV), and resolver configuration are out of
scope (§11).

## 7. `close` with operations in flight

PDR-0015 ruling 8. `close` on a stream or listener deregisters its pending registrations
(token generation bump — reactor.md §7.1's mechanism, first user-visible consumer) and
each pending `Future` settles `Err(#closed)` at the next drain:

1. **Never a hang** — a fiber parked on a closed stream's `read` resumes with the `Err`.
2. **Never a leak report** — deregistered is not leaked (reactor.md §7.2's posture).
3. **Never a native settlement** — the `Err` rides a synthetic completion through the
   ordinary pump ([`../impl/reactor.md`](../impl/reactor.md) §1 architecture untouched).
4. **Not in tension with reactor.md §8** ("pending futures never settle at shutdown"):
   that rule is VM exit, where no observer remains; explicit `close` has a live caller.

## 8. TLS — deferred, obligations registered

Binds nothing beyond PDR-0005 §7's ratified `TlsStream#shutdown -> Future`. Whichever
record specs TLS inherits three obligations (PDR-0015 ruling 11): `TlsStream < Resource`;
conformant Reader/Writer; no-arg `shutdown -> Future` as §4's protocol selector. The TLS
dependency decision is that record's own — not smuggled in with the poller's.

## 9. Laws, consolidated

1. **Blocking is visible in the type, both ways** (PDR-0004 §1): can-block ⇒ `Future`;
   cannot-block ⇒ plain value or `Result` — which is why `bind` is synchronous and
   `peerAddr` is an `IpAddr`, and the *only* deliberate uniformity exception is the
   no-arg `shutdown`, grounded in the `flush`-totality precedent (stream law 3).
2. **EOF is not an error; a reset is.** `read` settles `0` on orderly FIN; abnormal
   teardown settles `Err(#connectionReset)`.
3. **`Err` for the world, raise for the caller** (filesystem law 3): refused/reset/
   unreachable/in-use/dns-failure settle `Err`; wrong types, bad ports, wrong-length
   octets, use-after-close, and concurrent pending operations raise.
4. **Addresses are octets end to end** — text is display and input convenience; the wire
   form is `bytes`, and it is what crosses the native boundary.
5. **One selector, one operation** (ADR-0043; filesystem law 5): resolve-then-connect is
   the *documented composition* of `connect`, available separately as
   `Dns.resolve` + `connectAddr`; no selector takes an options bag or a mode flag.
6. **Close is prompt and observable**: §7's three-way contract; a closed handle never
   holds a fiber hostage.

## 10. Conformance harness

Runs against loopback; no external network. Every row a `.ph` golden test once reactor
phase 2 exists. Absorbs reactor.md §10's socket-echo row (its phase-1 plan moved it here).

| Check | Asserts |
|---|---|
| socket echo | two fibers over loopback, interleaved readiness: client writes, server echoes, client reads back — the reactor.md §10 row, end to end through the poller |
| connect refused | `connectAddr` to a closed port settles `Err(#connectionRefused)`, never raises |
| EOF on FIN | peer `shutdown` ⇒ `read` settles `0`, repeatedly (stream law 1 re-run against sockets) |
| shutdown uniformity | no-arg `shutdown.await` on a plain `TcpStream` settles `Ok`; peer observes EOF |
| directional shutdown | `shutdown(Shutdown.write)` stops writes but reads still work; `.read` inverse |
| concurrent-op raise | second `read(_)` while one pending raises `#concurrentOperation`; read+write concurrently is legal and completes |
| close with pending | `close` while a `read` is parked: the read settles `Err(#closed)` promptly; no leak-report row; the fiber resumes |
| close contract | double-close `Ok`; use-after-close raises `#useAfterClose` (stream harness rows re-run against `TcpStream`) |
| accept FIFO | three fibers `accept`; three connections; settle in call order |
| bind errors | second `bind` on the same port returns `Err(#addrInUse)` synchronously |
| port 0 | `bind(loopbackV4, port: 0)` succeeds; `localPort` is nonzero and connectable |
| port validation | `connect(_, port: 0)`, `port: 70000`, `port: 80.5` raise |
| `IpAddr` value semantics | parse/`toString` round-trips canonical forms; `==`/`hash` agree; keys a `Map`; `ofBytes` wrong length raises; `parse("bogus")` is `None` |
| resolve | `Dns.resolve("localhost")` settles `Ok` containing a loopback; a guaranteed-NXDOMAIN name settles `Err(#dnsFailure)` |
| buffered wrappers | `BufferedWriter.new(stream)` passes the stream-protocol §8 rows unchanged |
| leak report | an unclosed `TcpStream` at exit appears in `System.leakReport` naming its open site (PDR-0005 §5) |

## 11. Open questions

| # | Question | Notes |
|---|---|---|
| Q-N1 | UDP | datagrams break the Reader/Writer stream shape (message boundaries); own surface, own record; nothing here precludes it |
| Q-N2 | Unix domain sockets | pollable, fit the reactor unchanged; surface question is path-vs-address spelling (`Path` from PDR-0013 is the obvious carrier) |
| Q-N3 | IDNA / punycode hostnames | deliberately not performed (PDR-0015 ruling 9); doing it half-right is worse than a documented absence |
| Q-N4 | Socket options (`TCP_NODELAY`, keepalive) | excluded from v0.2 (PDR-0015 ruling 12); each arrives as its own selector per ADR-0043 when evidence demands it — the Nagle cost is the named pressure |
| Q-N5 | Connect timeout + Happy Eyeballs | both need cancellation (Q-R4); sequential-with-system-timeouts until then, dependency named in PDR-0015 ruling 4 |
| Q-N6 | `SocketAddr` / SRV-style endpoint resolution | the only sanctioned door for an endpoint type (PDR-0015 ruling 1 / Consequences); opens only if endpoint-returning resolution lands |

## 12. What this document does not cover

- **The poller and its dependency decision (Q-R3)** — [`../impl/net.md`](../impl/net.md)
  and its accompanying record.
- **TLS internals** — §8's obligations only.
- **UDP, Unix sockets, raw sockets, multicast** — no design, no owner (Q-N1/Q-N2).
- **DNS beyond `resolve`** — reverse lookup, record types, resolver config.
- **`Future#cancel`** — Q-R4, its own unit; §7's `#closed` settlement is *not*
  cancellation, it is close semantics.
