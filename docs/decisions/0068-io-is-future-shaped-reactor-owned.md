# 68. IO is `Future`-shaped and reactor-owned; the reactor is built before the IO surface

- Status: Accepted
- Date: 2026-07-20
- Related: [decision 0067](0067-no-user-visible-threads-fibers-and-isolates.md) (no user-visible
  threads — constrains how the filesystem half may be implemented),
  [ADR-0030](../adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md) (`Fiber` +
  `Future`, the parking mechanism this rests on),
  [E004](../errors/E004-await-cannot-suspend.md) (**FIXED** `f479189` — the precondition),
  [system.md](../spec/v0.2/system.md) (`System.sleep(_)` was the open timer question this closes),
  [ffi.md](../spec/v0.2/drafts/ffi.md) §4 (the object-graph boundary the worker pool must respect)

## Context

Every potentially-blocking operation in the planned IO surface — `File.read`, `TcpStream.connect`,
`Dns.resolve`, `Timer.after` — has to answer one question before a single selector is written:
**does it return `Result` or `Future`?** The answer is not stylistic. It is baked into every
selector, and changing it later breaks the entire surface at once.

Two facts settle the shape.

**Fibers can genuinely park.** `Future#await` could not suspend under any circumstance until
`f479189`: its own `.attempt()` probe re-entered the interpreter twice, so the restricted-yield
guard compared `floor_depth + 2` against `floor_depth` and refused unconditionally, for every
fiber, in every program ([E004](../errors/E004-await-cannot-suspend.md)). That is fixed and
covered by `tests/lang/concurrency/concurrency_future_await_suspends.ph`. Had it not been, this
decision would have been forced to blocking-only regardless of preference.

**Regular files are not pollable.** `epoll`/`kqueue` report a regular file as always ready, so an
event loop cannot give asynchronous file IO. Genuine async file operations require blocking
syscalls on worker threads. This is why libuv ships a thread pool alongside its event loop, and it
is not an implementation detail that can be designed away — it is a property of the operating
systems.

## Decision

### 1. Every potentially-blocking operation returns `Future`

Not `Result`. Applies to filesystem, sockets, DNS, TLS, timers, and process wait. Operations that
cannot block (`Path#join`, `Metadata#size`, `Fs.exists` on a cached stat) return ordinary values or
`Result`.

Errors surface as a `Future` settling to `Err` — `Future<Result<T>>` collapses to one settlement
channel, not two nested ones.

### 2. The reactor is built **before** the IO surface, not staged behind it

The alternative considered and rejected was shipping `Future`-shaped signatures over a synchronous
blocking implementation, then swapping a reactor in later. Ruled against: an always-already-settled
`Future` is observationally different from a real one in ordering, and code written against the
stub would encode assumptions that the real reactor then breaks. The type would survive the swap;
the programs would not.

So the machinery lands first, with no IO consumers, and the surface is written on top of a reactor
that already works.

### 3. Two mechanisms, split by what the kernel can actually poll

- **Pollable descriptors** — sockets, pipes, TTYs, timers, signals: a real reactor
  (`epoll`/`kqueue`/IOCP). Single-threaded, no worker involved. A fiber awaiting one parks; the
  reactor resumes it on readiness.
- **The filesystem** — a bounded worker pool running blocking syscalls, because §Context's second
  fact leaves no alternative.

### 4. The worker pool obeys decision 0067 §3 absolutely

Workers receive owned plain data (`PathBuf`, `Vec<u8>`, scalars) and return owned plain data. They
never see a `Value`, an `ObjRef`, or the heap.

Completions cross back on an MPSC channel carrying **plain data only**. The VM thread drains that
channel **at a dispatch safepoint**, and only there does it mint handles, settle the `Future`, and
push the waiting fiber onto `VM::ready_queue`. `ready_queue` therefore stays a single-threaded
`VecDeque<ObjRef>` and needs no synchronization.

### 5. `System.sleep(_)` is a timer completion source on this reactor

[system.md](../spec/v0.2/system.md) records `sleep` as "still open — U-SCHED deliberately splits
the ready-queue from timers." This decision closes it: timers are reactor-owned, and `sleep`
returns a `Future` settling after the interval.

## Consequences

- The scheduler grows a second completion source. `System.runScheduled`'s drain loop and `VM::run`'s
  root-drive pump must now also service the reactor, or a program that awaits only IO makes no
  progress and exits silently. **This is the sharpest failure mode in this decision** and belongs
  in the first test written.
- Fairness between the ready queue and reactor completions is now a live question rather than a
  deferred one (`open-questions.md` §15).
- The safepoint gains a second job. Draining completions there — not at an arbitrary point — is
  what keeps handle-minting single-threaded, and it is the same discipline that makes `temp_roots`
  and Invariant L sound.
- Building the reactor with zero consumers means it is specified against no real usage. Mitigation:
  write the socket-echo and file-read integration tests as the *first* consumers, before the `File`
  and `Socket` surfaces exist.
- Cancellation is now unavoidable: a parked fiber whose fiber is discarded must deregister from the
  reactor. Deferred to its own unit, but it cannot be deferred indefinitely — a leaked registration
  is a fd leak.

## Alternatives rejected

- **Blocking + `Result`.** Simplest, zero machinery, and adequate for a CLI language — but one
  fiber blocking on `read` stalls every other fiber, and retrofitting async breaks every selector
  shipped.
- **`Future` signatures over a blocking implementation, reactor later.** Rejected per §2: it keeps
  the types and breaks the programs.
- **Thread pool for everything, no reactor.** Wastes a thread per idle socket and scales with
  connections rather than with activity — the problem event loops exist to solve.
