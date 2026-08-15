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
   resumes at the new fiber's saved position;
4. failure = the entry's error unwinds via the unified unwind
   (U-CORE-6, [ADR-0008](../../adr/0008-layered-exceptions-and-result.md)) to the
   fiber's own top-level activation; `VM::run_until`'s fiber-floor capture
   marks it `failed`, stores the captured `Error`, and resumes the resumer —
   re-raising under `call`'s cascade, delivering the `Error` as a value under
   `try` (`vm.rs` `run_until`).

`isDone`/`error` (the two reflective accessors in the Interface table above)
are **landed** — pure reads over `FiberObject::status`/`result`, added by
[U-FIBER-REFLECT](../../work/pending/fiber-schedule/reflect/plan.md)
alongside U-FIBER's own `new`/`call`/`try`/`yield`/`current`/`abort`. They
needed no scheduler and no new state.

Because there is no preemption, no synchronization primitives are needed: a fiber
switch happens only at an explicit `call`/`yield`/`await` point.

### Execution model — restricted yield (Option A)

The suspension mechanism is the **restricted re-entrant loop** ratified by
[ADR-0030](../../adr/0030-fibers-and-futures-cooperative-concurrency.md) §4. The
VM dispatch loop is *not* a flat trampoline: pure Phalcom→Phalcom sends are
trampolined, but any primitive that calls a block back into Phalcom (`block_call`,
`perform`, `doesNotUnderstand` forwarding, and every collection combinator built
on them — `each`/`map`/`reduce`) **re-enters the loop on the native Rust stack**.

`Fiber.yield` therefore integrates with the **top-level** loop only. Yielding
while such a native frame sits between the fiber's entry and the yield site raises
a catchable **`CannotYieldAcrossNativeFrame`** error rather than corrupting the
suspended position:

- **Suspends freely** — bodies using pure sends and *inlined* control flow
  (`while`/`ifTrue:` lower to `Jump`/`Loop` in one chunk,
  [ADR-0018](../../adr/0018-sacred-selector-inliner-and-override-guard.md)); the
  `counter` generator above is exactly this shape.
- **Foreclosed** — the *callback generator*
  `Fiber.new { list.each { x => Fiber.yield(x) } }`, where `yield` sits under
  `each`'s native `block_call`, raises `CannotYieldAcrossNativeFrame`. Write the
  generator with a **`for` loop** instead, which lowers to an inlined `while` over
  the cursor protocol — no `block_call`, so it suspends freely
  ([iteration.md](iteration.md) §2/§6, [ADR-0035](../../adr/0035-iteration-protocol-cursor.md)):

  ```phalcom
  Fiber.new { for (x in list) { Fiber.yield(x) } }   // ✅ inlined while — suspends
  ```

  This is the idiomatic v0.2 form and supersedes the older "rewrite with index
  iteration" advice; `for` also gives `break`/`continue`, which a block handed to
  `each` cannot express.

This restriction is a **guard, not a wall**: lifting it for the residue that `for`
cannot express (`.each { yield }`, a stored-block generator, a user-defined native
combinator that yields) is the deferred general lift
[ADR-0033](../../adr/0033-amend-fiber-execution-trampolined-block-callsite.md) —
de-recursing the block call-site (audit Option B), purely additive, breaking no
program that ran under A, to land with the typed fiber-switch signal below. The
switch is signalled to the loop as a typed control-flow value, never inferred from
a frame-count change (which a fiber swap and a non-local return would both trip).

---

## 2. `Future` — pending asynchronous result

A `Future` represents a value that may not exist yet. It is a thin state machine
over `Fiber`: `await` suspends the current fiber until the future is settled, and
a **scheduler** (a run loop over ready work) drives settlement.

### Structure

- a **state** — `pending`, `fulfilled(value)`, or `rejected(error)`;
- a **waiters** list — fibers suspended in `await` on this future, plus `then`
  continuations, resumed/queued when it settles;
- (for `async` futures) the driving `Fiber`.

A `Future` settles **exactly once**; further completions are ignored.

### Interface

**Both slices are landed.** The status column below records which slice a member
came from, not whether it exists: **A** = U-FUTURE Slice A (pure `.ph`, no
`Fiber`/scheduler involvement); **B** = Slice B, landed in `06432bd`
(2026-07-14) over the native ready-queue. Slice B's `await` did not actually
*work* until [E004](../../errors/E004-await-cannot-suspend.md) was fixed — it
probed its own permission to yield with a wrapper that made the yield illegal,
so it never suspended a fiber; `Fiber#isRoot` replaced the probe
([U-SCHED](../../forge/units/U-SCHED-FIBER/U-SCHED/plan.md), ratified per
[DEC-FUT-SCHED](../../forge/units/U-FUTURE/plan.md#9-blocked-on-decision-register)
Option 1).

| Signature | Side | Status | Meaning |
|-----------|------|--------|---------|
| `@constructor value(_)` | class | **A** | an already-`fulfilled` future |
| `@constructor error(_)` | class | **A** | an already-`rejected` future |
| `async(_)` | class | **B** | run a `Function` on a fresh fiber, returning a future for its result |
| `await` | instance | **B** | suspend the current fiber until settled; return the value or re-raise the error |
| `then(_)` | instance | **A** (settled-only); pending continuation is **B** | register a continuation; returns a future for the continuation's result |
| `map(_)` | instance | **A** (settled-only); pending continuation is **B** | `then` for the non-error path only |
| `catch(_)` | instance | **A** (settled-only); pending continuation is **B** | register an error handler; returns a recovered future |
| `isReady` | instance | **A** | `true` once `fulfilled` or `rejected` |
| `value` | instance | **A** | the settled value as `Option` (never blocks) |

On an already-settled receiver, `then`/`map`/`catch` fire synchronously today
(Slice A, pure `.ph`) — no suspension involved. On a `pending` receiver they
currently **raise** rather than register a continuation and hang, since Slice
A has no drain to ever fire it later; registering an actual continuation is
Slice B.

```phalcom
let f = Future.async { slowComputation() }
doOtherWork()
let result = f.await          // suspends this fiber until f settles
```

`await` is **sugar-free suspension**, not blocking: the fiber yields to the
scheduler, which runs other ready fibers until `f` settles, then resumes this one.
`then`/`map`/`catch` are the non-suspending, continuation-passing form; `await` is
the direct-style form. They are interconvertible because both bottom out in the
same waiter list.

### Implementation

`Future` is a library-level `InstanceObject` — no new `Value` arm
([`value.rs`](../../../phalcom-core/src/value.rs) already has `Instance`).

**Slice A — landed** ([`core.ph`](../../../phalcom-core/core/core.ph) `class
Future`, [U-FUTURE](../../work/pending/fiber-schedule/future/plan.md)): a pure-`.ph`
settle-once state machine over three private fields (`_state`/`_value`/
`_waiters`). `value(_)`/`error(_)` construct an already-settled future;
`isReady`/`value` read state; settled-receiver `then`/`map`/`catch` fire
synchronously. **Zero native code, zero `Fiber` involvement** — a settled
future never suspends.

**Slice B — landed** in `06432bd` (2026-07-14), over `Fiber` (§1) as the
substrate and the native ready-queue below. What it took:

1. `await` = "add `current` to the future's waiters, then `Fiber.yield` to
   the scheduler" — plus, on the **root** fiber, which has no resumer and so
   cannot yield at all, a degrade to driving the queue in place. Choosing
   between those two branches turned out to need a *predicate*
   (`Fiber#isRoot`): the shipped implementation originally chose by attempting
   a yield inside `{ … }.attempt()` and inspecting the failure, and since
   `.attempt()` is itself two native re-entrant frames, the probe tripped the
   restricted-yield guard (§4) it was probing for — so `await` never suspended
   any fiber until [E004](../../errors/E004-await-cannot-suspend.md) was fixed.
   Also uses `Fiber#isDone`/`error`
   ([U-FIBER-REFLECT](../../work/pending/fiber-schedule/reflect/plan.md))
   to detect an `async` driver's completion/failure;
2. a **scheduler**: a ready-queue of resumable fibers plus a source of
   external completions (timers, I/O) exposed through [`System`](system.md).
   No `.ph`-reachable class-side/module mutable state exists today
   ([object-model.md](object-model.md)/[classes.md](classes.md)), so the
   ready-queue needed a native home — **landed** as `System.schedule(_)`/
   `System.nextScheduled`/`System.runScheduled` (`system.md` §2, `VM::ready_queue`)
   — not a "top level runs inside the scheduler's root fiber" retrofit as
   originally sketched, but a **root-drive pump**: `VM::run` drains the
   ready-queue once the top-level program's own activation ends, so `await`
   degrades to "drain, re-check" rather than requiring `main` itself to be a
   scheduler fiber;
3. settlement moves the future to `fulfilled`/`rejected` and enqueues every
   waiter fiber and `then` continuation onto the ready-queue — the
   enqueue-on-settle half is `Future`'s own job (`drain`), since U-SCHED's
   `runScheduled` only ever *pops*, never pushes a resumed-later fiber back
   on. `drain` skips a waiter fiber that has already finished: one can fail
   *after* registering, and resuming it would abort the run and take the
   future's healthy waiters with it (E004(c)).

The native ready-queue seam is landed
([U-SCHED](../../forge/units/U-SCHED-FIBER/U-SCHED/plan.md),
ratified — [DEC-FUT-SCHED](../../work/pending/fiber-schedule/future/plan.md#9-blocked-on-decision-register)
Option 1); `Future` deliberately owns **no** new VM mechanism beyond
`Fiber` + a queue — keeping the concurrency primitive singular
([ADR-0030](../../adr/0030-fibers-and-futures-cooperative-concurrency.md)).
`Future`'s own waiter-list/settlement wiring over that substrate is what Slice
B added, and it is now landed too. `_waiters` holds two kinds of thing —
`Fiber`s registered by `await`, `Block`s registered by `then`/`map`/`catch` —
unified by `System.schedule(_)`, which enqueues a fiber as-is and wraps anything
else.

---

## 3. Relationship to the rest of the model

- A [`Function`](functions.md) is the unit of work for both classes: `Fiber.new`
  and `Future.async` each take one.
- **Non-local `return`** ([Blocks §5](blocks.md)) is frame-local and therefore
  fiber-local: a block's home frame lives on one fiber's frame stack, so a
  `return` across a fiber boundary raises `DeadFrameError` rather than silently
  unwinding the wrong stack.
- Errors ([Object Model §4](object-model.md)) cross fiber boundaries only through
  `call`/`await` (propagate) or `try`/`catch` (capture) — never implicitly.

The **execution model** (restricted re-entrant loop, Option A) is decided —
[ADR-0030](../../adr/0030-fibers-and-futures-cooperative-concurrency.md), recorded
as open-question 15 in [Open Questions](open-questions.md). Still open there:
structured concurrency / cancellation scopes, whether `Future` gets `select`/`race`
combinators, and the scheduler's fairness guarantees.
