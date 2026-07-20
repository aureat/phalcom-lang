# Specification — The Reactor (completion machinery for `Future`-shaped IO)

> **Status:** **Normative machinery contract.** Encodes
> [PDR-0004](../../../decisions/0004-io-is-future-shaped-reactor-owned.md) §1–§5 and
> [PDR-0003](../../../decisions/0003-no-user-visible-threads-fibers-and-isolates.md) §3 —
> both **Accepted**, so rule 5 does not block this document. This is a *machinery* spec:
> its consumer is the implementer, not the `.ph` programmer; the only user-visible selector
> it adds is `System.sleep(_)` (§6, ruled in substance by PDR-0004 §5).
> **Floor delta: +3** (`System.sleep_(_,_)` and the two pump seams
> `System.nextCompletion_` / `System.parkForCompletion_(_)` — the U-SCHED
> `schedule_`/`nextScheduled` seam precedent; amended from "+1" by
> [`../impl/reactor.md`](../impl/reactor.md), which also rules phase 1 std-only:
> worker pool + timers, no poller, no sockets, no new dependency); census arithmetic follows
> [PDR-0012](../../../decisions/0012-numeric-tower-implementation-and-floor-amendment.md)
> ruling 21's rebase discipline alongside the other pending amendments.
> **Build order is ruled:** this machinery lands **before** any `File`/`Fs`/socket surface
> (PDR-0004 §2 — a stubbed always-settled `Future` keeps the types and breaks the
> programs).
>
> **Owner:** unassigned. Precondition met: E004 fixed (`f479189`) — fibers genuinely park
> (`tests/lang/concurrency/concurrency_future_await_suspends.ph`).

## 1. Role and the two mechanisms

The reactor is the thing that makes every `Future` in
[`filesystem.md`](filesystem.md) / [`stream-protocol.md`](stream-protocol.md) settle.
Split by what the kernel can actually poll (PDR-0004 §3):

| Source | Mechanism | Why |
|---|---|---|
| Sockets, pipes, TTYs, timers, signals | one poller (`epoll`/`kqueue`/IOCP), single-threaded | genuinely pollable; a waiting fiber costs nothing |
| Filesystem operations | bounded worker pool running blocking syscalls | `epoll` reports regular files always-ready; async file IO does not exist at the OS level — libuv's thread pool is the proof, not a shortcut |

Both are invisible from `.ph`: user code sees only `Future`s.

## 2. Thread discipline — the absolute law

PDR-0003 §3 / PDR-0004 §4, restated as the invariant every line of this subsystem is
reviewed against:

1. **Workers receive owned plain data** (`PathBuf` built from `Path` bytes, `Vec<u8>`,
   scalars) **and return owned plain data.** No `Value`, no `ObjRef`, no heap access, no
   allocation into the Phalcom heap — enforced structurally: the job and completion enums
   contain only plain-data types, so a violation is a compile error, not a review catch.
2. **Completions cross back on one MPSC channel**, plain data only.
3. **Only the VM thread mints handles, settles `Future`s, and touches
   `VM::ready_queue`** — which therefore stays the unsynchronized single-threaded
   `VecDeque<ObjRef>` it is today (`vm/mod.rs:221`), with no atomics added anywhere
   (PDR-0003's single-VM-thread guarantee doing its work).

## 3. Completion lifecycle

```
submit -> park -> complete -> drain (safepoint) -> settle -> ready
```

- **Submit.** A native IO primitive builds the plain-data job, registers it (poller
  interest or pool queue) under a fresh **generation-tagged token** (the ADR-0013
  frame-token / PDR-0005 §4 resource-table idea, third use), creates the un-settled
  `Future`, and returns it. No syscall has happened on the VM thread.
- **Park.** The caller `await`s; the fiber yields to its floor and is now owned by the
  pending token, reachable via the registration — **the registration is a GC root for its
  fiber and its `Future`** (a parked fiber with no other reference must not be collected
  out from under a pending completion).
- **Complete.** The poller reports readiness, or a worker finishes and pushes the
  completion (token + plain result) onto the channel. Nothing else happens off-thread.
- **Drain, only at the dispatch safepoint.** The VM thread drains the channel at the same
  back-edge site that services the GC latch (`service_gc_safepoint`,
  `vm/dispatch.rs:540`) — not at arbitrary points. Draining there is what keeps handle
  minting single-threaded and composes with the `temp_roots` / Invariant L discipline
  (PDR-0004 Consequences). A completion whose token generation is stale (cancelled,
  §7) is dropped on the floor here, by design.
- **Settle.** `Future` settles once, to `Ok(value)` or `Err(error)` — one settlement
  channel, never `Future<Result>` nesting (PDR-0004 §1).
- **Ready.** The parked fiber is pushed onto `ready_queue` and runs when the scheduler
  reaches it — never immediately, never preempting the current fiber.

## 4. The pump and the liveness law

The scheduler currently has one completion source: the ready queue, drained by
`System.runScheduled` (`core.ph:1317`) over the `system_schedule`/`system_next_scheduled`
seam (`primitive/system.rs:56`/`:70`), with the VM's root-drive pump behind it. This
machinery adds a second source, and PDR-0004 names the resulting failure mode the
sharpest in the decision:

> **Liveness law.** A program whose every fiber is parked on IO makes progress. The pump
> must therefore treat "ready queue empty" as *"block in the poller until the next
> completion or timer deadline"*, not as *"exit"*.

Exit condition, exactly: no runnable fiber **and** no pending registration **and** no
undrained completion. Anything less exits silently mid-IO; anything more never exits.
This law is the **first test written** (§10), before any consumer exists.

## 5. Fairness between the two sources

Live question, not ruled (`open-questions.md` §15; PDR-0004 Consequences). This spec
states a **proposed default** and marks it open rather than deciding silently:

- At each safepoint drain, take the completions available **at entry** (a bounded batch —
  later arrivals wait a round); resumed fibers join the **back** of `ready_queue`; a
  completion never preempts a running fiber.

That is starvation-free in both directions under a finite batch, and it is Q-R1 until a
real workload confirms or refutes it.

## 6. Timers — `System.sleep(_)`

Ruled by PDR-0004 §5 (closes `system.md`'s open question): timers are a reactor
completion source, and

```
System.sleep(_) -> Future     // settles Ok(None) after >= the given duration
```

- Duration is a `Number` of **integral milliseconds**, the [`filesystem.md`](filesystem.md)
  ruling-8 wording (representation-independent; exact in f64). Negative or non-integral
  raises; `0` is a valid yield-until-next-pump.
- Clock is **monotonic** — wall-clock changes never fire or starve a timer.
- `>=`, never `==`: settlement happens at the first pump after the deadline, and the spec
  promises no tighter bound (single VM thread; a fiber that never yields delays every
  timer — that is PDR-0003's documented cost, not a reactor defect).
- Native: `System.sleep_(_,_)` — duration plus the pending future to register
  (impl ruling: every reactor-registering native takes the future as its last
  argument, because `Future` is pure `.ph` and natives never settle one). The `.ph`
  `sleep` validates, creates the `Future`, registers, returns it.

## 7. Cancellation — deferred, with its obligations recorded now

Own unit (PDR-0004 Consequences: "it cannot be deferred indefinitely — a leaked
registration is an fd leak"). What this spec binds now so the deferral stays honest:

1. Every registration carries the generation-tagged token from §3; **deregistration is
   generation bump + poller removal**, and a stale completion is dropped at the drain.
   The mechanism ships with the reactor even though no user-facing `cancel` exists yet.
2. A discarded parked fiber (unreachable but registered) is **not collectable** (§3's
   root rule) — it is a *leak*, and it must appear in `System.leakReport` as a distinct
   condition ("fiber parked on a registration nothing can complete"), the PDR-0005 §5
   posture applied to registrations.
3. `Future` gets no `cancel` selector in this spec. When it does, it composes with the
   token mechanism above rather than adding a second one.

## 8. Shutdown

On VM exit: stop intake, drain the completion channel once, drop poller registrations,
signal and join pool workers (bounded wait — a worker stuck in an uninterruptible syscall
is abandoned, documented), then run the PDR-0005 resource-table drain and the leak
report. Pending `Future`s never settle at shutdown — they are reported (§7.2), not
force-settled with a synthetic error (a synthetic `Err` looks like an IO failure and gets
retried by well-meaning code).

## 9. Laws, consolidated

1. **No `Value` crosses a thread boundary, ever** (§2 — structural, not reviewed-for).
2. **Settlement happens only at the safepoint drain, on the VM thread** (§3).
3. **A `Future` settles at most once, to one channel** (`Ok`/`Err`, PDR-0004 §1).
4. **Liveness**: parked-on-IO-only programs progress; the exit condition is §4's
   three-way conjunction, exactly.
5. **A pending registration roots its fiber and its `Future`** (§3); an unreachable
   registered fiber is a reported leak, not a collection (§7).
6. **Timers are monotonic and lower-bounded only** (§6).
7. **Stale tokens complete into the void** (§3/§7) — a cancelled or superseded
   completion is dropped at drain, never settled.

## 10. Conformance — written before the surfaces exist

PDR-0004's mitigation for "specified against no real usage" is that these are the
*first consumers*, preceding `File`/`Socket`:

| Check | Asserts |
|---|---|
| liveness | law 4: a program that only `sleep(50).await`s completes; one that awaits a never-completing registration with `strictResources` reports rather than spinning |
| exit exactness | §4: exits iff the three-way conjunction; does not exit with a timer outstanding |
| file-read integration | worker-pool path end to end: submit → park → drain → settle `Ok(count)`, with the fiber resumed on the queue, not inline |
| socket echo | poller path end to end, two fibers, interleaved readiness |
| settle-once | law 3: double completion for one token settles once, second is dropped |
| stale token | law 7: deregistered token's completion is dropped at drain |
| back-of-queue resume | §5 default: a resumed fiber runs after already-ready fibers, never preempts |
| timer monotonicity | law 6: `sleep` unaffected by wall-clock jumps (where the harness can fake them); `sleep(0)` settles on the next pump |
| plain-data boundary | §2.1: the job/completion enums contain no `Value`/`ObjRef` — a compile-time assertion (e.g. a `static_assert`-style impl-Send check), not a runtime test |
| GC ⊗ parked fiber | law 5: a parked fiber with no `.ph` reference survives a forced `System.gc` and completes correctly |

## 11. Open questions

| # | Question | Notes |
|---|---|---|
| Q-R1 | Fairness policy | §5's bounded-batch back-of-queue default, pending a real workload (`open-questions.md` §15) |
| Q-R2 | Worker-pool size | Bounded, but bounded at what? libuv defaults to 4; measure, don't copy. Whatever it is, it is a constant with a doc comment, never a user-visible knob in v0.2 |
| Q-R3 | Poller backend on macOS-first development | `kqueue` now with `epoll` later, or an abstraction (`mio`-style) from day one? A dependency decision with workspace-pinning consequences |
| Q-R4 | `Future` cancellation surface | deferred unit (§7); the token mechanism is its fixed substrate |

## 12. What this document does not cover

- **Any IO selector surface.** `File`/`Fs` are [`filesystem.md`](filesystem.md); sockets,
  DNS, TLS, process-wait have no spec yet — they plug into §3's lifecycle unchanged.
- **Isolates.** PDR-0003 §2; if ever built, each isolate owns a reactor — nothing here
  assumes a process singleton beyond `System.sleep`'s binding.
- **Streaming/backpressure abstractions.** Q-3 of PDR-0013 and beyond; the reactor
  settles single completions only.
