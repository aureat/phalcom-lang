# Implementation spec — the network unit: reactor phase 2 (poller) + TCP/DNS surface (U-NET)

> **Status:** **blocked on ratification** of
> [PDR-0015](../../../decisions/0015-network-surface-tcp-dns-endpoints.md) (surface) and
> [PDR-0016](../../../decisions/0016-poller-backend-is-mio.md) (poller backend) — both
> **Proposed**; rule 5 forbids building until they flip. Dispatch-ready in every other
> respect. Governing Accepted records:
> [PDR-0004](../../../decisions/0004-io-is-future-shaped-reactor-owned.md) §3/§4,
> [PDR-0005](../../../decisions/0005-resources-are-disposable-handles-not-finalized.md)
> §3/§4; surface contract [`../core/net.md`](../core/net.md); machinery contract
> [`../core/reactor.md`](../core/reactor.md).
> **Needs shipped: U-BYTES ✅, U-RESOURCE, U-REACTOR** (phase 1 — this unit is phase 2).
> Independent of U-PATH/U-FS/U-STREAMS; parallel-safe with U-FS (disjoint `Job`/`Payload`
> variants, disjoint primitive files; the shared `nextCompletion_` match gains disjoint
> arms — coordinate that one file if truly concurrent).
> **Floor delta: 7** (`NEW_NET`) — enumerated §2.5; registering natives take the pending
> future **last** ([`reactor.md`](reactor.md) §2.3's rule). Census arithmetic against the
> live `floor_census_matches_installed_bindings` under
> [PDR-0012](../../../decisions/0012-numeric-tower-implementation-and-floor-amendment.md)
> ruling 21's rebase discipline — never against a number quoted here.
> Read [`bytes.md`](bytes.md) §7 first; obligation 1 (`add_class!`) applies to
> `TcpStream` and `TcpListener`.

## 1. Shape

The pollable half of PDR-0004 §3, built as phase 2 of the reactor: `mio` (PDR-0016)
confined to `reactor.rs`, readiness converted to phase-1 `Completion`s **on the VM
thread**, settlement unchanged (`.ph` pump only). Sockets live in U-RESOURCE's table;
DNS rides the phase-1 worker pool (std's blocking `to_socket_addrs` — **no** resolver
dependency). `.ph` composes `connect`'s resolve-then-attempt loop; natives only try
syscalls and register.

## 2. File-by-file

### 2.1 `reactor.rs` — the poller (phase 2 core)

- **`Poller`**: `mio::Poll` + `Events` + `Waker`, owned by the reactor next to the
  phase-1 channels. Pool workers call `waker.wake()` after every mpsc push (worker
  completions must interrupt a park that is now inside `poll`, not `recv_timeout`).
- **Registration entries grow an op.** Phase 1's registry maps `token -> Future ObjRef`
  (GC-rooted). Phase 2 extends the entry: `{ future: ObjRef, op: NetOp }` where

  ```rust
  /// Poller-op state. Plain data + the fd; buffers are Vec<u8>, never Bytes —
  /// the GC root stays "the future", exactly as phase 1 wired it.
  enum NetOp {
      Connect { fd: RawFd, site: SourceRange },        // armed writable; SO_ERROR on ready
      Read    { fd: RawFd, len: usize },               // armed readable; scratch read at drain
      Write   { fd: RawFd, buf: Vec<u8>, off: usize }, // armed writable; may re-arm on short write
      Accept  { fd: RawFd, site: SourceRange },        // armed readable on the listener
  }
  ```

  No new root kinds: buffers are plain `Vec` (write snapshots in, read mints a payload
  out — the U-FS §2.4 compose pattern; the user's dst `Bytes` is held by the `.ph`
  closure, not the registry).
- **Token mapping** (PDR-0016 §2): `mio::Token(index)`; generation checked only at the
  drain. Plus a side index `resource_index -> SmallVec<token>` so close can find a
  stream's pending ops (§2.4).
- **Try-then-register** (PDR-0016 §3, binding): every native attempts the non-blocking
  syscall at submission; `EWOULDBLOCK` ⇒ register interest (`reregister` per op — one
  pending op per direction is PDR-0015 ruling 7's guarantee that this is sound);
  success/failure ⇒ push a **synthetic completion** onto the pending buffer directly.
  Natives never settle anything either way.
- **Readiness → completion, at the drain seam.** The §2.2-of-phase-1 safepoint fill
  gains: after the mpsc `try_recv` drain, if the poller has events pending (non-blocking
  `poll(0)`), run each ready token's syscall — `SO_ERROR`+`getpeername`/`getsockname`
  for `Connect`, scratch-`Vec` `read(2)` sized to `len` for `Read`, `write(2)` from
  `buf[off..]` for `Write` (short write re-arms with advanced `off` — surface `write`
  settles the *total* accepted only when the buf empties or errors… **no**: stream law 2
  says short writes are *reported*; settle with the count from the **first** successful
  syscall and never loop internally), `accept(2)` for `Accept` — and convert to
  `Completion`s. `EWOULDBLOCK` re-arms. All on the VM thread; workers never see an fd.
- **`parkForCompletion_`'s block** becomes `poll(min(next timer deadline, caller cap))`;
  wake sources: readiness, `Waker` (worker completion), timeout (timer due). Exit-idle
  condition (reactor.md §4's third clause) now also counts poller registrations.
- **New `Job`/`Payload` variants** (plain data, structural boundary):
  `Job::DnsResolve { host: Vec<u8> }`;
  `Payload::AddrList(Vec<Vec<u8>>)` (4- or 16-byte octet vectors),
  `Payload::Conn { fd: RawFd, peer: (Vec<u8>, u16), local: (Vec<u8>, u16), site: SourceRange }`
  (also produced by `Accept`). Socket error names extend the one `ErrorKind` map in
  `reactor.rs` (`"connectionRefused"`, `"connectionReset"`, `"addrInUse"`,
  `"addrNotAvailable"`, `"brokenPipe"`, `"networkUnreachable"`, `"hostUnreachable"`,
  `"timedOut"`, `"dnsFailure"`, `"closed"` — core/net.md §9 law 3's `Err` kinds).

### 2.2 `System.nextCompletion_` — minting arms

`AddrList` → `List` of `Bytes`; `Conn` → open a `ResourceKind::TcpStream(fd)` row
(site from the payload) and mint a `Tuple` `(packedHandle, peerBytes, peerPort,
localBytes, localPort)`. Minting stays dumb; `.ph` shapes `TcpStream`/`IpAddr`
instances (the U-FS §2.2 posture).

### 2.3 `ResourceKind` extensions — U-RESOURCE

`TcpStream(RawFd)`, `TcpListener(RawFd)`. Close branch: `close(2)`, synchronous —
sockets have nothing to flush (kernel-buffered; PDR-0005 §3b's logic transfers intact).

### 2.4 Close-with-pending (core/net.md §7)

Inside the kind-specific close branch, *before* `close(2)`: look up the resource's
pending tokens (§2.1's side index), deregister each from the poller, bump its
generation, and push a synthetic `Completion { token, Err(IoErrorData { name: "closed", .. }) }`
per op. The parked fibers settle `Err(#closed)` through the ordinary pump at the next
drain — never a hang, never a leak row, never a native settlement. (Ordering matters:
deregister before `close(2)`, or kqueue/epoll auto-removal races the registry.)

### 2.5 `primitive/net.rs` — 7 natives (`NEW_NET`)

Conventions: `primitive/bytes.rs` (bare-or-`None`, contract violations raise, `.ph`
lifts). Registering natives take the future **last**.

| Native | Serves | Notes |
|---|---|---|
| `TcpStream.connectAddr_(_,_,_)` static | `.ph` `connectAddr` and (via `.ph` loop) `connect` | addr-`Bytes` (len 4/16), integral port `1..65535` (validate, raise), future. Non-blocking `socket`+`connect`; `EINPROGRESS` ⇒ arm writable; instant success/failure ⇒ synthetic completion |
| `TcpListener.bind_(_,_)` static | `.ph` `TcpListener.bind` | **synchronous, no future** (PDR-0015 ruling 2): socket/bind/listen inline, backlog a doc-commented `const` (Q-R2 posture). Success ⇒ open a `TcpListener` row, return `Tuple` `(packedHandle, localBytes, localPort)`; OS failure ⇒ return an `IoError` instance (the §2.2 minting pattern applied synchronously); `.ph` lifts to `Result` |
| `accept_(_)` | `TcpListener#accept` | resolve handle (stale/closed raises `UseAfterCloseError` **before** registration); try `accept(2)` then arm readable. FIFO across concurrent accepts = registration order — one token each |
| `read_(_,_)` | `TcpStream#read(_)` | dst-`Bytes` sizes the op; payload `Bytes` + count settle; `.ph` `copyInto` dst (U-FS `read_` shape). Second pending read on the stream raises `ConcurrentOperationError` at submission |
| `write_(_,_)` | `TcpStream#write(_)` | snapshots src `Bytes` into the op's `Vec` (caller may mutate after); settles first-syscall count (stream law 2) |
| `shutdown_(_,_,_)` | `.ph` `shutdown` / `shutdown(_)` | direction-name `String` (`"read"`/`"write"`/`"both"` — one native, two surface selectors; ADR-0043 governs selectors, not plumbing). `shutdown(2)` inline (non-blocking), synthetic completion ⇒ settles next pump, satisfying ruling 5's uniform `Future` |
| `Dns.resolve_(_,_)` static | `.ph` `Dns.resolve(_)` | host-`Bytes` (the `String`'s UTF-8), future; `Job::DnsResolve` on the **worker pool** — std `(host, 0).to_socket_addrs()` is the blocking `getaddrinfo`; dedupe-preserving-order to `AddrList`. Empty ⇒ `IoErrorData("dnsFailure")` (core/net.md §6) |

### 2.6 `.ph` layer — core.ph

- **`TcpStream` / `TcpListener`** — bootstrapped kernel classes `< Resource`
  (`make_core_class` + `CoreClasses` field + verify row + **`add_class!`** — all four
  sites; bytes.md §7 obligation 1). Field stamps in `VM::new`:
  `TcpStream` 5 (`_handle`, `_peerAddr`, `_peerPort`, `_localAddr`, `_localPort`),
  `TcpListener` 3 (`_handle`, `_localAddr`, `_localPort`) — the U-RESOURCE §2.2
  handle-in-slot-0 packing, and its riskiest-seam warning applies: the
  subclass-field-offset harness row runs against both.
- **`IpAddr`, `Shutdown`** — pure `.ph` (parse/RFC 5952 display, value `==`/cached
  `hash`, singletons); zero natives, zero census. `IpAddr` construction copies its
  `Bytes` in; `bytes` copies out (PDR-0013 rulings 1/2 pattern).
- **`Dns`** — `.ph` `class Dns {}` with the static native, the U-FS `Fs` shape.
  `resolve(_)` validates `String`, passes UTF-8 bytes, shapes `List<Bytes>` →
  `List<IpAddr>`.
- **`connect(host, port:)`** — `.ph` composition: `Dns.resolve(host)` then-chain a
  sequential `connectAddr` attempt loop (resolver order, first success, last failure —
  PDR-0015 ruling 4). Policy lives in `.ph` where cancellation (Q-R4) can later reshape
  it without touching a native.
- **`ConcurrentOperationError < Error`** — the dedicated-class kind mechanism
  (U-RESOURCE §2.4; `CannotYieldAcrossNativeFrame` precedent) for
  `#concurrentOperation`; `IoError` arrives with whichever of U-FS/U-NET lands first —
  **shared class, single definition** (coordinate; it is one `.ph` class + one minting
  arm).

### 2.7 Census

`NEW_NET: usize = 7` + class rows for `TcpStream`/`TcpListener` (bootstrapped).
`IpAddr`/`Shutdown`/`Dns` are `.ph`-declared, not censused. Verify the live baseline
first — U-RESOURCE, U-REACTOR, possibly U-FS and PDR-0012's tower land before this.

## 3. Ordering

1. `Cargo.toml`: pin `mio` (workspace + `phalcom-core`) — PDR-0016 §1's feature set.
2. `reactor.rs` phase-2 core: `Poller`, `NetOp`, token side-index, try-then-register,
   drain-seam adapter, park rewrite. Pure-Rust tests with raw fds (loopback pair), no VM.
3. `ResourceKind` socket variants + close-with-pending (§2.4) + its Rust test.
4. Minting arms (§2.2). Boot green — behavior-neutral while nothing registers.
5. `primitive/net.rs` + bootstrapped classes (all four sites) + census. Boot green.
6. `.ph` layer (§2.6).
7. Golden lanes (§4) — **ET-regression and echo first**. Clean-worktree verify at the SHA.

## 4. Test plan

core/net.md §10's harness row-for-row in golden lanes `net/` + `net/negative/`
(loopback only, `strictResources(true)`), plus the unit-level rows this file owns:

| Check | Asserts |
|---|---|
| ET regression (**first**) | peer writes *before* `read_` is called: the read settles anyway (try-first drained it; register-first would hang) — PDR-0016 §3's silent-hang failure mode |
| waker cross-source | a fiber parked in `poll` (no timers due) is woken by a worker-pool completion (`Dns.resolve` while a socket read is pending) |
| echo | core/net.md §10's two-fiber loopback echo through the real poller |
| close-with-pending | §2.4: parked read settles `Err(#closed)`; no leak row; deregister-before-close ordering exercised with a second live stream on the same drain |
| short write | a full send buffer settles a partial count, `.ph` sees stream law 2; the re-armed remainder path exercised via a draining peer |
| concurrent-op | second `read` raises `ConcurrentOperationError` at submission, before any registration exists to leak |
| bind sync | `bind` returns without a pump running (provably synchronous: no `await` in the lane); `#addrInUse` on rebind |
| accept FIFO / port 0 / refused / EOF-on-FIN / shutdown rows | core/net.md §10 verbatim |
| resolve | localhost `Ok`-with-loopback; NXDOMAIN `Err(#dnsFailure)`; order preserved (mock-free: assert non-empty + membership, not exact order beyond dedupe stability) |
| plain-data boundary | `Job::DnsResolve`/new payloads join the existing compile-time Send/no-`Value` assertions |
| GC ⊗ armed op | registered `Read` op's future + parked fiber survive forced `System.gc` (phase-1 M-row re-run against a poller token) |

## 5. What must NOT happen

- No `mio` type outside `reactor.rs` (PDR-0016 §1's confinement rule — reviewed-for
  *and* greppable: `grep -r "mio::" --include="*.rs"` hits one file).
- No register-without-try (PDR-0016 §3 — the ET hang).
- No syscall on a worker for any socket op; no fd in any `Job` except via §2.1's
  VM-thread-only `NetOp`s. DNS is the only pooled job here.
- No internal write-until-done loop in `write` (stream law 2: report the short write).
- No native settling a future, ever — synthetic completions ride the pending buffer
  (phase 1 §1's architecture).
- No hostname anywhere near a native: `connect`'s resolve loop is `.ph`; `connectAddr_`
  and `bind_` take octets only (PDR-0015 ruling 9).
- No options, timeouts, backlog knobs (PDR-0015 ruling 12); no TLS types.

## 6. Not in this unit — file as DEFERRED on landing

TLS (own record; obligations in core/net.md §8), UDP (Q-N1), Unix sockets (Q-N2), IDNA
(Q-N3), socket options (Q-N4), connect timeout / Happy Eyeballs (Q-N5, blocked on
Q-R4's cancellation surface — whose token-generation substrate this unit exercises but
does not surface), `SocketAddr`/SRV (Q-N6), fairness tuning (Q-R1).
