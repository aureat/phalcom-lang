# PDR-0015 — The network surface: TCP is poller-backed and `Future`-shaped, DNS rides the pool, endpoints are address-plus-port

- Status: Proposed
- Date: 2026-07-20
- Related: [PDR-0004](0004-io-is-future-shaped-reactor-owned.md) (IO is `Future`-shaped;
  reactor before surface — every ruling here presupposes it),
  [PDR-0005](0005-resources-are-disposable-handles-not-finalized.md) §3/§4/§7
  (`TcpStream < Resource` is *already ruled there*; `TlsStream#shutdown -> Future` is
  *already ratified there* — this record cites both and redesigns neither),
  [PDR-0013](0013-path-is-bytes-backed-filesystem-surface.md) rulings 1/2/5 (the
  bytes-backed value-class pattern and the singleton-object pattern, reused),
  [`stream-protocol.md`](../spec/v0.2/core/stream-protocol.md) (Reader/Writer protocols and
  the `close` laws, applied verbatim),
  [`reactor.md`](../spec/v0.2/core/reactor.md) §3/§7 (completion lifecycle and the
  cancellation substrate), [`impl/reactor.md`](../spec/v0.2/impl/reactor.md) (phase 1
  std-only ruling: "epoll/kqueue arrives with a network unit" — this is that unit's
  surface), ADR-0012 (selector encoding), ADR-0043 (no default arguments, no flags).

## Context

The reactor's phase 1 shipped worker pool + timers and deliberately deferred the poller:
"`epoll`/`kqueue` arrives with a network unit" ([`impl/reactor.md`](../spec/v0.2/impl/reactor.md)
header ruling). PDR-0004's own mitigation for "machinery specified against no real usage"
names the **socket echo** as a first-class consumer (reactor.md §10). So sockets are not
one more surface — they are the thing the poller half of the reactor is built against, and
nothing rules their shape yet beyond two lines ratified in PDR-0005 §7:
`TcpStream < Resource` (§3) and `TlsStream#shutdown -> Future`.

What is genuinely open, and what this record rules: the class set, how endpoints are
spelled, whether `connect` resolves names, the shutdown protocol across plain and TLS
streams, EOF-versus-reset, what a `close` does to operations in flight, and which
mechanism (poller or pool) each operation rides.

## Decision

### 1. The class set: two `Resource` classes, one namespace, two value types — and no `SocketAddr`

```
TcpStream   < Resource      // PDR-0005 §3, cited not re-ruled
TcpListener < Resource
Dns                          // static namespace, never instantiated
IpAddr                       // immutable .ph value class over 4 or 16 octets
Shutdown                     // three singletons: .read / .write / .both
```

`IpAddr` reuses the PDR-0013 ruling-1/2 pattern exactly: backed by a `Bytes` it owns
exclusively (construction copies in, `bytes` copies out), structural `==`, content `hash`
cached at construction — immutable + value-hashed ⇒ a valid `Map`/`Set` key. `Shutdown`
is the `OpenMode` singleton pattern (PDR-0013 ruling 5): no numbers, no flags.

**There is no `SocketAddr` class.** An endpoint travels as an `IpAddr` plus a labelled
`port:` argument, and comes back as paired accessors (`peerAddr`/`peerPort`). DNS resolves
*addresses*, not endpoints — `Dns.resolve` settling to address-port pairs would be a lie
about what `getaddrinfo` returns for a hostname. Go's `net.SplitHostPort` exists because
"host:port" strings force every caller to re-parse what the API glued together, with IPv6
colons as the standing ambiguity; a pair class avoids the string but still buys a kernel
class for what is one address and one integer. Revisit only if SRV-style resolution (which
genuinely returns endpoints) ever lands — recorded as open question Q-N6 in
[`core/net.md`](../spec/v0.2/core/net.md).

### 2. Blocking shape, applied honestly in both directions

Per PDR-0004 §1: `connect` / `accept` / `read` / `write` / `resolve` / `shutdown` can all
block ⇒ every one returns a `Future`.

**`TcpListener.bind` is synchronous and returns `Result`.** `bind(2)` + `listen(2)` do not
block — they fail or succeed immediately. The rule that makes blocking visible in the type
cuts both ways: wrapping a non-blocking operation in a `Future` would claim a suspension
that never happens, the same honesty that keeps `File#path` a plain `Path` and
`Resource#close` a synchronous `Result` (PDR-0004 §1 scope note). `bind` takes an
`IpAddr`, never a hostname, so no DNS can hide inside it.

### 3. Two mechanisms, assigned per operation

- **TCP sockets ride the poller** — the pollable-descriptor mechanism PDR-0004 §3 named
  and phase 1 deferred. Readiness, not completion: the poller reports readable/writable,
  and the non-blocking syscall runs on the VM thread at the drain seam (it owns the heap,
  so it may fill a `Bytes` directly — the plain-data law of reactor.md §2 is a *worker*
  law and workers never see sockets).
- **DNS rides the worker pool.** `getaddrinfo` is a blocking libc call with no pollable
  form; libuv resolves on its thread pool for the same reason it does file IO there. The
  phase-1 completion pipeline (job → mpsc → safepoint drain → `.ph` pump settle) carries
  it unchanged.

Both reuse phase 1's registration registry, token generations, and pump seams. The network
unit adds no third mechanism and no second settlement path.

### 4. `connect` resolves; `connectAddr` does not; both exist

```
TcpStream.connect(_, port:)       -> Future   // String hostname; resolve, then attempt
TcpStream.connectAddr(_, port:)   -> Future   // IpAddr; no DNS anywhere near it
```

Two selectors per ADR-0043, not one selector over a `String`-or-`IpAddr` union — a
union-typed argument is a flag wearing a type's clothes. `connect` is the ergonomic
spelling every mainstream API converged on (Rust's `ToSocketAddrs`, Go's `net.Dial`,
Node's `net.connect` all resolve implicitly); `connectAddr` is the explicit path for
callers who already resolved, and composes with `Dns.resolve` without double lookups.

Attempt policy for `connect`: try each resolved address **sequentially, in resolver
order**; first success wins; if all fail, settle `Err` carrying the *last* failure.
Happy Eyeballs (RFC 8305, parallel staggered attempts) requires cancelling the losing
attempt, which is Q-R4's cancellation surface — deferred with that dependency named, not
silently dropped (Q-N5).

### 5. `shutdown` is a uniform `Future` protocol; the no-arg form is end-of-writes

```
#shutdown        -> Future    // "I am done sending": FIN on TCP, close_notify on TLS
TcpStream#shutdown(_) -> Future    // directional: Shutdown.read / .write / .both
```

PDR-0005 §7 already ratifies `TlsStream#shutdown -> Future` — close_notify is a write and
genuinely blocks. TCP's `shutdown(2)` does *not* block; the syscall queues the FIN and
returns. It returns a `Future` anyway, settling on the next pump, **because the no-arg
`shutdown` is a protocol selector**: the plain-or-TLS polymorphic caller (every HTTP
client with an optional TLS layer) must write `stream.shutdown.await` without a type
test. This is not an exception carved into PDR-0004 §1 — it is stream-protocol §2's
`flush` precedent applied: `flush` is on every Writer and "an unbuffered writer's `flush`
settles immediately with `Ok`", uniform protocol position outranking per-type blocking
visibility. The semantic pairing is exact: no-arg `shutdown` means "no more bytes from
me", which is what a FIN and a close_notify both say.

The directional `shutdown(_)` is TCP-only (TLS has no read-shutdown concept in the
protocol), takes a `Shutdown` singleton, and is not part of the informal protocol.

### 6. EOF is a FIN; a reset is an `Err`

`read(_)` settling `0` means the peer shut down its write half — orderly EOF, stream-protocol
law 1, never an error. `ECONNRESET` settles `Err` with kind `#connectionReset`. The two
are different events on the wire and stay different in the surface; conflating them (as
raw BSD sockets invite) is how "connection closed" bugs get retried forever.

### 7. One pending read and one pending write per stream

A second `read(_)` while one is pending **raises** (kind `#concurrentOperation`); same
for `write(_)`. Read-concurrent-with-write is legal — the halves are independent, and
half-duplex-only would foreclose every echo/proxy shape. Two interleaved reads on one
socket have no defined answer for *which bytes land in which buffer*; queuing them
manufactures an ordering the caller never asked for and cannot observe. Raising follows
the stream-protocol §3.1 law-6 split: this is a program structure error, not weather from
the world. (Tokio reaches the same posture by construction — `split()` yields exactly one
read half and one write half.)

### 8. `close` with operations in flight: they settle `Err(#closed)`, promptly

`Resource#close` on a stream or listener with pending registrations deregisters them
(token generation bump, reactor.md §7.1) and each pending `Future` settles
`Err` with kind `#closed` at the next drain. Never a hang, never a leak-report entry —
a deregistered registration is not a leak, and the parked fiber resumes with an error it
can handle. Go's `"use of closed network connection"` is this exact contract. The
settlement rides the normal pump (a synthetic completion, not a native settling anything
— the impl/reactor.md §1 architecture is untouched). This does **not** contradict
reactor.md §8's "pending futures never settle at shutdown": that rule is about VM exit,
where no code remains to observe a settlement; an explicit `close` has a live caller.

### 9. Hostnames are `String`; addresses are bytes; ports are validated integers

- A hostname is a `String` — DNS names are ASCII on the wire; the resolver receives the
  UTF-8 bytes as-is, and a non-ASCII name fails at the resolver and surfaces as
  `Err(#dnsFailure)`. IDNA/punycode is deliberately not performed (Q-N3): doing it
  half-right (lowercase-only, no mapping tables) is worse than a documented absence.
- An `IpAddr` crosses the native boundary as its 4-or-16 octets (the PDR-0013 ruling-4
  posture: the wire form is bytes; text is display).
- A port is an integral `Number`. `connect`/`connectAddr` accept `1..65535`;
  `bind` accepts `0..65535`, where `0` means OS-assigned and `TcpListener#localPort`
  reveals the assignment. Out-of-range or fractional raises — contract, not IO.

### 10. Error kinds, and their mechanism until the traceback units land

IO failures settle `Err`; contract violations raise (filesystem.md law 3, applied
unchanged). The `Err` kinds this surface adds: `#connectionRefused`, `#connectionReset`,
`#addrInUse`, `#addrNotAvailable`, `#brokenPipe`, `#networkUnreachable`,
`#hostUnreachable`, `#timedOut`, `#dnsFailure`, `#closed`. The raise kinds:
`#useAfterClose` (stream-protocol §3.1), `#concurrentOperation` (ruling 7), plus ordinary
type/range contract raises. Until traceback plan units T3/T6 land PDR-0010's `kind`
carrier, these are dedicated `Error` classes; the symbols above are the names the `kind`
field takes when that carrier exists — additions to the normative table in
`docs/spec/traceback/implementation-spec.md` §8.1, filed with that plan's lane rather
than duplicated here.

### 11. TLS is deferred wholesale

This record binds nothing about TLS except what PDR-0005 §7 already ratified. Obligations
registered for whichever record specs it: `TlsStream < Resource`; it satisfies the Reader/
Writer informal protocols; its no-arg `shutdown -> Future` is ruling 5's protocol selector.
Handshake configuration, certificate verification, and the TLS dependency decision are that
record's problem — a TLS stack is a bigger dependency commitment than a poller, and
bundling it here would let the small decision smuggle in the large one.

### 12. No socket options, no timeouts, no backlog knob in v0.2

No `setNodelay`, no keepalive, no read timeouts, no `backlog:` on `bind`. Timeouts
compose with the cancellation surface (Q-R4) and are ruled there or after it, never as
per-selector duplicates. The listen backlog is an implementation constant with a doc
comment, the Q-R2 worker-pool-size posture. Options that later prove necessary arrive as
their own selectors per ADR-0043 (never an options bag). This is a real cost — see
Consequences.

### 13. Floor and registration discipline

Floor delta is nonzero and **enumerated in [`impl/net.md`](../spec/v0.2/impl/net.md)**,
censused at impl time under PDR-0012 ruling 21's rebase discipline against the live
`floor_census_matches_installed_bindings`. Every registering native takes the pending
`Future` as its **last** argument and never settles it (impl/reactor.md §2.3's rule,
binding on this unit).

## Consequences

- **The poller dependency decision is now unavoidable** — reactor phase 2 cannot be built
  without answering Q-R3 (kqueue-vs-abstraction). That is a workspace-pinning dependency
  decision and gets its own record with the impl spec, not a line here.
- **The cost, named plainly:** no options means no `TCP_NODELAY` in v0.2, so
  latency-sensitive small-write workloads eat Nagle delays with no recourse but a
  buffering pattern; sequential `connect` means dual-stack hosts with a dead IPv6 route
  pay a full timeout before the IPv4 attempt (the exact pain Happy Eyeballs exists for,
  deferred behind cancellation); one-pending-read-per-stream forces explicitly structured
  concurrent readers.
- **What this precludes:** a future `SocketAddr` cannot be introduced compatibly without
  re-speccing `Dns.resolve` and the accessor pairs — Q-N6 is the only sanctioned door.
  `connect`'s String argument can never grow implicit IP-literal fast-paths that bypass
  the resolver *observably* (an impl may short-circuit literals, but the surface promises
  resolver semantics). The `#closed` settlement contract forecloses close-blocks-until-
  drained semantics later.
- The socket-echo conformance row (reactor.md §10) finally has the surface it was written
  against; it moves into [`core/net.md`](../spec/v0.2/core/net.md) §10's harness.
- `System.leakReport` gains stream/listener rows for free via `Resource` (PDR-0005 §5) —
  no new reporting machinery.

## Alternatives rejected

- **String endpoints (`"host:port"`)** — Go/Node's shape. Forces every consumer to
  re-parse, is ambiguous under IPv6 (`[::1]:80` bracket convention exists solely to patch
  this), and couples DNS into every API that accepts an endpoint.
- **A `SocketAddr` class** — one more kernel value class to carry one address and one
  integer; `Dns.resolve` cannot honestly produce them (no port in an A/AAAA record); and
  the accessor-pair spelling loses nothing measurable.
- **`connect` over a union argument type** (`String`-or-`IpAddr`) — a runtime type test
  selecting between two behaviors is ADR-0043's flag in disguise; two selectors keep both
  paths honest and separately teachable.
- **Blocking `Result` sockets** — foreclosed by PDR-0004 §1/§2 before this record existed.
- **Callback-based `accept`** (`onConnection` handler) — a second concurrency model beside
  fibers+Futures; PDR-0003/0004 shaped the whole IO story around the latter, and one
  `accept -> Future` loop in a fiber expresses the same server with no new machinery.
- **Queuing concurrent reads** — manufactures an unobservable byte-interleaving order;
  raising makes the structural error loud at the site that made it (ruling 7).
- **Deferring the ruling** — the poller cannot be built consumer-first without a socket
  surface to consume it (PDR-0004 §2's own build-order logic), so deferral here silently
  defers reactor phase 2 too.
