# Raw source snapshot: concurrency

> Captured: 2026-09-08
> Source area: concurrency specifications and fiber-program boundary
> This immutable snapshot preserves the source excerpts used by the compiled concept articles. Program metadata is lifecycle evidence, not proof that proposed work is implemented.

--- docs/spec/current/concurrency.md ---
# Fibers & Futures

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1. The
surface and execution model are ratified by
[ADR-0030](../../adr/0030-fibers-and-futures-cooperative-concurrency.md).

Concurrency in Phalcom is **cooperative and single-threaded**, built on one
primitive — the `Fiber` — with `Future` as the ergonomic layer over it. Both are
ordinary heap classes ([Object Model §4](object-model.md)); both take a
[`Function`](functions.md) as their unit of suspendable work.

Invariant: **there is no preemption and no shared-memory data race.** A running
fiber runs until it yields, awaits, returns, or raises. This keeps the object
model free of locks and keeps message send atomic.

---

## 1. `Fiber` — cooperative coroutine

A `Fiber` is an independently suspendable call stack. It is the *only* concurrency
primitive; `Future`, `async`/`await`, generators, and the scheduler are all built
from it.

### Structure

- an **entry** — the [`Function`](functions.md) the fiber runs when first resumed;
- its **own value stack** and **own `CallFrame` stack** — a fiber does not share
  the caller's stack, which is what makes suspension possible;
- a **status** — one of `suspended` (created or yielded, resumable),
  `running` (currently on the CPU), `done` (entry returned), `failed`
  (entry raised, error captured);
- a **resumer link** — the fiber to hand control back to on `yield` / return /
  failure (forming a dynamic caller chain, not a fixed parent);
- a **result slot** — the last yielded/returned value, or the captured `Error`.

The **root fiber** is the main program; it is `suspended` only while a callee
fiber runs.

### Interface

| Signature | Side | Meaning |
|-----------|------|---------|
| `@constructor new(_)` | class | wrap a `Function` as a not-yet-started fiber |
| `call` / `call(_)` | instance | resume; the argument becomes the value of the suspended `yield` (or the entry's parameter on first resume). Returns the next yielded/returned value |
| `try` / `try(_)` | instance | like `call`, but a failure yields `None`/an `Error` value instead of propagating |
| `isDone` | instance | `true` once `done` or `failed` |
| `error` | instance | the captured `Error` as `Option`, if `failed` |
| `yield(_)` | **class** | suspend the *current* fiber, handing the value to its resumer. Returns the value passed to the next `call` |
| `current` | **class** | the fiber now running |
| `abort(_)` | **class** | raise an `Error` out of the current fiber to its resumer |

`Fiber.yield(_)` is class-side because it always acts on the running fiber, never
a named one — you cannot yield another fiber. This mirrors the receiver-less
nature of "suspend me."

```phalcom
let counter = Fiber.new {
  let n = 0
  while (true) { Fiber.yield(n); n = n + 1 }
}
counter.call()   // 0
counter.call()   // 1
counter.call()   // 2
```

Control transfer is symmetric and explicit: `call` pushes onto the resumer chain,
`yield`/return pops it. There is no implicit scheduler at this layer — that is
`Future`'s job (§2).

### Implementation

**Landed** (U-FIBER, [ADR-0030](../../adr/0030-fibers-and-futures-cooperative-concurrency.md)).
No `Value::Fiber` arm — `Object::Fiber(FiberObject)` is a heap arena variant
reached through `Value::Obj(ObjRef)`, exactly as native `List` is
([`heap.rs`](../../../phalcom-core/src/heap.rs) `FiberObject`/`Object::Fiber`).
There is **no `Yield` opcode** — `Fiber.yield` is an ordinary class-side
primitive send (a deliberate, sanctioned deviation from the original design
sketch, D-FIB-7; see [`primitive/fiber.rs`](../../../phalcom-core/src/primitive/fiber.rs)).
The four points below are realized:

1. `FiberObject` owns `stack: Vec<Value>`, `frames: Vec<CallFrame>`,
   `open_upvalues: BTreeMap<usize, ObjRef>`, a `status`, a
   `resumer: Option<ObjRef>`, a `result` slot, and the entry closure
   (`heap.rs`);
2. the VM's "current stack / current frames" live behind
   `VM::current: ObjRef` — `call`/`try`/`yield` swap which fiber's stacks the
   interpreter loop reads via `mem::take`, an O(1) pointer-free handoff, never
   a copy (`primitive/fiber.rs` `store_live_into`/`load_live_from`);
   `CallFrame.stack_offset` stays frame-relative, so per-fiber stacks need no
   rebasing;
3. `call`/`try`/`yield` are primitives that set statuses, move the
   transferred value across the boundary (`resume_slot`), repoint `current`,
   and set a **typed switch signal** (`VM::switch_pending`, not a
   frame-count heuristic — D5) so `VM::call_method`'s `Primitive` arm skips
   ordinary post-call stack reconciliation and the dispatch loop transparently

--- docs/spec/current/stdlib/reactor.md ---
# Specification — The Reactor (completion machinery for `Future`-shaped IO)

> **Status:** **Normative machinery contract.** Encodes
> [PDR-0004](../../../pdr/0004-io-is-future-shaped-reactor-owned.md) §1–§5 and
> [PDR-0003](../../../pdr/0003-no-user-visible-threads-fibers-and-isolates.md) §3 —
> both **Accepted**, so rule 5 does not block this document. This is a *machinery* spec:
> its consumer is the implementer, not the `.ph` programmer; the only user-visible selector
> it adds is `System.sleep(_)` (§6, ruled in substance by PDR-0004 §5).
> **Floor delta: +3** (`System.sleep_(_,_)` and the two pump seams
> `System.nextCompletion_` / `System.parkForCompletion_(_)` — the U-SCHED
> `schedule_`/`nextScheduled` seam precedent; amended from "+1" by
> [`../../forge/units/U-REACTOR/implementation-spec.md`](../../forge/units/U-REACTOR/implementation-spec.md), which also rules phase 1 std-only:
> worker pool + timers, no poller, no sockets, no new dependency); census arithmetic follows
> [PDR-0012](../../../pdr/0012-numeric-tower-implementation-and-floor-amendment.md)
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


--- docs/spec/current/stdlib/cancellation.md ---
# Specification — Cancellation (`Future#cancel`, `Future#isCancelled`, `CancelledError`)

> **Status:** **Proposed — normative upon ratification of
> [PDR-0017](../../../pdr/0017-future-cancel-is-renunciation.md)** (rule 5: no
> unit builds this until it flips). Discharges [`reactor.md`](reactor.md) §11 **Q-R4**
> on the substrate §7 already binds (token generations, deregistration-as-bump,
> stale-drop at drain). Already-Accepted inputs:
> [PDR-0004](../../../pdr/0004-io-is-future-shaped-reactor-owned.md)
> (Consequences — cancellation named unavoidable),
> [PDR-0005](../../../pdr/0005-resources-are-disposable-handles-not-finalized.md)
> §5 (leak-report composition), ADR-0030 / C-FUT-3 (settle-once no-op, the load-bearing
> shipped fact — `core.ph` `class Future`, `settleValue`'s settle-once comment).
> **Floor delta: +1** (`System.cancelRegistration_(_)`, `NEW_CANCEL`) — the U-SCHED
> seam-precedent shape; census at impl time under
> [PDR-0012](../../../pdr/0012-numeric-tower-implementation-and-floor-amendment.md)
> ruling 21's rebase discipline.
> **Build order:** needs U-REACTOR (phase 1) only — `System.sleep` is a sufficient test
> substrate; independent of and parallel-safe with U-NET.
>
> **Owner:** unassigned.

## 1. What `cancel` means — the three-clause contract

PDR-0017 ruling 1, restated as the contract every selector below serves:

| Clause | Strength | Content |
|---|---|---|
| settle | **guaranteed** | the future settles rejected with `CancelledError` (`kind: #cancelled`) at the `cancel` call; waiters reschedule through the ordinary drain |
| release | **guaranteed** | any reactor registration is released: generation bumped, poller/timer interest dropped, GC root removed, pump exit-count decremented |
| suppress | best-effort | unstarted work is skipped (queued pool job dropped at dequeue; unfired poller op never syscalls) |
| interrupt | **never** | started work completes on its worker; its completion arrives stale and drops (reactor law 7). A cancelled effectful op may still have effected |

"Stop waiting", never "undo".

## 2. Surface

```
Future#cancel      -> Bool    // true iff this call moved pending -> cancelled
Future#isCancelled -> Bool
```

- `cancel` on a settled or already-cancelled future: `false`, no effect, no error —
  idempotent. The `Bool` is the race outcome (did *I* cancel it, or had it settled?),
  which timeout patterns branch on.
- `isCancelled` distinguishes rejected-by-cancel from rejected-by-failure without
  inspecting the error; producers use it for cooperative early exit (§4).
- Awaiting a cancelled future **raises** the `CancelledError` — the existing rejected
  arm of `await` (`core.ph`); `catch(_)`/`on` observe it like any error. No new
  channel, no third state.
- `CancelledError < Error`, pure `.ph`, no natives — the dedicated-class kind mechanism
  until traceback T3/T6 land the `kind` carrier.

## 3. The cancel algorithm — ordinary `.ph`, one native

```phalcom
cancel() {
  if (self.isReady) { return false }
  System.cancelRegistration_(self)      // release; Bool ignored here — no-registration futures cancel too
  self.settleError(CancelledError.new)  // ordinary .ph settlement; waiters drain normally
  _cancelled = true
  return true
}
```

Order is normative: **release before settle** (a waiter resumed by the settlement must
not observe a still-armed registration), settle before flagging (nothing between them
can run — single VM thread, no re-entry in this sequence). Natives never settle
anything: `cancelRegistration_` only releases; the settlement above is `.ph` — the
[`../../forge/units/U-REACTOR/implementation-spec.md`](../../forge/units/U-REACTOR/implementation-spec.md) §1 architecture holds with zero exceptions.

## 4. Late settlement, and non-IO futures

Two defenses, **both mandatory** (PDR-0017 ruling 4):

1. **Drain-side**: a completion whose token generation is stale drops at the safepoint
   fill (reactor law 7) — the normal path, no minting wasted.
2. **Settle-side**: a completion that entered the pending buffer *before* the
   generation bump is past defense 1; its `.ph` settlement lands on the settled future
   and hits C-FUT-3's shipped no-op. Benign by construction, not by review.

Corollary: cancelling a `Future.async` result (or any `.ph`-settled future) stops
nothing — the driver fiber runs to completion, its `settleValue` drops. Renunciation is
uniform across IO and non-IO futures. A producer that wants to stop early polls
`future.isCancelled` at its own checkpoints; that is the entire cooperative story, and
it is opt-in.

## 5. Composition laws

1. **Cancelled ⇒ not a leak.** Release removes the registration from the pump's
   pending set and the leak surface in one motion (PDR-0005 §5 posture; reactor.md
   §7.2's parked-fiber condition is *remedied* by cancel — the fiber resumes with the
   raise — never triggered by it).
2. **`#cancelled` ≠ `#closed`.** Same spine, different initiator: resource-went-away
   vs consumer-renounced (PDR-0015 ruling 8 / PDR-0017 ruling 5). Handlers may retry
   after `#closed`; retrying `#cancelled` re-does what the canceller renounced.

--- docs/implementation/CONC001-fiber-scheduling/PROGRAM.md ---
---
id: CONC001
category: CONC
kind: implementation
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
---

# CONC001 — fiber scheduling

This program owns the concurrency runtime's fiber substrate, scheduler and
reactor, fiber reflection, and Future library.
