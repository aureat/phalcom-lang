# PDR-0016 — The poller backend is `mio`, wrapped once at the reactor seam; syscalls try first and register second

- Status: Proposed
- Date: 2026-07-20
- Related: [PDR-0004](0004-io-is-future-shaped-reactor-owned.md) §3 (pollable descriptors
  get "a real reactor (`epoll`/`kqueue`/IOCP)"),
  [PDR-0003](0003-no-user-visible-threads-fibers-and-isolates.md) §3 (single VM thread —
  the poller must not add a second),
  [PDR-0015](0015-network-surface-tcp-dns-endpoints.md) (the surface this serves; its
  ruling 7's one-pending-op rule is load-bearing below),
  [`reactor.md`](../spec/v0.2/core/reactor.md) §11 **Q-R3** (the question this closes),
  [`impl/reactor.md`](../spec/v0.2/impl/reactor.md) (phase 1's std-only ruling scoped the
  deferral: "epoll/kqueue arrives with a network unit"),
  [`impl/net.md`](../spec/v0.2/impl/net.md) (the consuming unit).

## Context

Q-R3 asked: "`kqueue` now with `epoll` later, or an abstraction (`mio`-style) from day
one? A dependency decision with workspace-pinning consequences." Phase 1 answered *not
yet* — worker pool and timers need no poller, and the `recv_timeout` park was enough.
The network unit ends the deferral: sockets are pollable descriptors, and PDR-0004 §3
assigned them a real poller by name.

Facts that bound the choice:

- Development is macOS-first (`kqueue`), but the tree is *unix*-first, not Darwin-first —
  [`impl/filesystem.md`](../spec/v0.2/impl/filesystem.md) §2.1 gates on
  `compile_error!` for non-unix, which includes Linux as a first-class target. A
  Darwin-only poller would make the net unit unbuildable for any Linux contributor or CI
  lane on day one.
- The workspace pins seven small dependencies (`Cargo.toml:10-18`) and PDR-0014 is
  removing one (miette). The bar for a new pin is real but not prohibitive; phase 1's
  "no new dependency" was a phase-scope ruling, not a policy.
- The park seam must wake for **two** event sources at once: socket readiness *and*
  worker-pool completions arriving on the mpsc channel. A hand-rolled `kqueue` needs
  `EVFILT_USER` for the second; `epoll` needs an `eventfd`; both are per-OS wake
  machinery that has to be written and tested twice.

## Decision

### 1. `mio`, workspace-pinned, confined to `reactor.rs`

One new workspace dependency: `mio` (features `os-poll`, `net`), version pinned in
`[workspace.dependencies]` like the existing seven. `mio` is the readiness substrate
under tokio with none of tokio's runtime — no executor, no futures, no threads: a safe
wrapper over `kqueue`/`epoll` (and a wepoll-shaped backend on Windows, which keeps
PDR-0013 Q-1's door open without paying for it now). Its `Waker` is exactly the
cross-source wake primitive the park seam needs; pool workers call `waker.wake()` after
pushing a completion, and the VM thread parks in one `poll(timeout)` for both sources.

**Confinement rule:** no `mio` type appears outside `phalcom-core/src/reactor.rs`. The
rest of the tree sees the reactor's own token/registration vocabulary. This is what
keeps the decision reversible at the cost of one file — the check that stops a
dependency becoming load-bearing beyond its job.

### 2. Tokens map, generations stay ours

`mio::Token(usize)` carries the registration **index**; the **generation** lives in the
reactor's registry as phase 1 built it. Staleness is checked at the safepoint drain
(reactor.md law 7), never trusted to the poller — `mio` deregistration has no generation
concept and gets none. One discipline, third mechanism, same as PDR-0005 §4.

### 3. Try first, register second

Every poller-backed operation attempts its non-blocking syscall **at submission**;
only on `EWOULDBLOCK` does it register interest. On readiness, the syscall runs on the
VM thread at the drain seam; `EWOULDBLOCK` there re-arms rather than settles.

This is not an optimization posture, it is a **correctness rule under edge-triggered
readiness**: bytes already buffered in the kernel when interest is registered produce no
edge, so register-first deadlocks on data that arrived early. Try-first empties the
buffer before arming, making the next arrival a genuine edge. It works precisely because
PDR-0015 ruling 7 caps each stream at one pending read and one pending write — there is
never a second waiter whose syscall could steal the readiness the first was armed on.
The hot-socket fast path (data already there ⇒ settle without a poll round) falls out
for free.

### 4. Phase 1's architecture is not renegotiated

The poller adds an event source, not a settlement path. Registration registry (a GC root
for its futures), plain-data completions, safepoint-only drain, `.ph`-pump-only
settlement, `System.nextCompletion_` / `System.parkForCompletion_(_)` seams — all
unchanged. `parkForCompletion_`'s blocking call becomes `poll(min(next timer, cap))`
instead of `recv_timeout(..)`, and the mpsc is drained non-blockingly after every wake;
that is the entire diff to phase 1's contract. Readiness becomes a `Completion` on the
VM thread; no worker ever touches a socket.

## Consequences

- **The cost, named plainly:** a new supply-chain surface (mio plus its per-OS
  transitive deps — `libc` on unix), pinned and audited like any other; edge-triggered
  discipline is subtle, and §3's rule must be enforced in review and by the impl spec's
  ET-regression test (buffered-data-before-registration), because its failure mode is a
  silent hang, not an error; and `mio`'s MSRV now joins the workspace's Rust-version
  constraints.
- **What this precludes:** io_uring-style *completion*-based submission — `mio` is
  readiness-only, and the reactor's readiness-to-completion adapter (impl/net.md) bakes
  that shape in. If io_uring ever matters, it arrives as a new backend behind the §1
  confinement seam, which is the seam's whole job. Similarly, nothing outside
  `reactor.rs` may grow a `mio` type without reopening this record.
- Linux and macOS are both first-class from the first commit of the net unit; the
  socket-echo conformance row runs identically on both.
- PDR-0013 Q-1 (Windows) gets cheaper, not decided: mio's Windows backend exists, but
  the tree's `compile_error!` gates stay until Q-1 is actually ruled.

## Alternatives rejected

- **Raw `kqueue` now, `epoll` later behind our own trait.** Zero dependencies, and the
  dev box is Darwin — but it makes the net unit Darwin-only until someone writes and
  tests a second backend plus per-OS wake machinery (`EVFILT_USER` / `eventfd`), which
  is a whole unit of work `mio` has already done and had audited by a decade of
  production tokio. The trait we'd design today would have one implementor — 
  speculative generality with none of the second data point that makes abstractions
  honest.
- **The `polling` crate.** The genuinely close call, named as such: smaller than `mio`,
  and its oneshot re-arm model matches the one-pending-op rule naturally, sidestepping
  §3's ET subtlety. Rejected on ecosystem weight — `mio` carries more production
  scrutiny, a maturer `Waker`, and the tokio-adjacent audit surface; and §3's rule is
  needed anyway for the try-first fast path, so the ET subtlety is paid regardless.
  If `mio` proves heavier than its job, `polling` slots behind the §1 seam.
- **tokio.** A runtime, an executor, and a futures model — Phalcom has its own all
  three (PDR-0003/0004; fibers park on *Phalcom* futures). Importing a second scheduler
  to use one syscall wrapper is the tail wagging two dogs.
- **Sockets on the worker pool (no poller).** PDR-0004 already rejected it: a thread
  per idle connection "scales with connections rather than with activity — the problem
  event loops exist to solve."
- **Deferring again.** Phase 1 could defer because timers and files don't poll. Sockets
  are the poller's raison d'être; deferring Q-R3 now is deferring the network unit
  itself, with PDR-0015 already Proposed against it.
