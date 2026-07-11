# Fibers & Futures

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

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
| `construct new(_)` | class | wrap a `Function` as a not-yet-started fiber |
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

Unrealized today: no `Fiber` value, no per-fiber stack, no `Yield` opcode
([`bytecode.rs`](../../phalcom-core/src/bytecode.rs)). The VM currently owns a
single value stack and a single `Vec<CallFrame>`
([`frame.rs`](../../phalcom-core/src/frame.rs), driven by `vm.rs`). Fibers
require:

1. a `FiberObject` holding its own `stack: Vec<Value>` and `frames:
   Vec<CallFrame>`, a `status`, a `resumer: Option<PhRef<FiberObject>>`, and the
   entry closure; `Value::Fiber(PhRef<FiberObject>)`;
2. relocating the VM's "current stack / current frames" into a `current:
   PhRef<FiberObject>` pointer, so `call`/`yield` become a swap of which fiber's
   stacks the interpreter loop reads — no data copying, O(1) switch;
3. `call`/`yield` implemented as primitives that (a) set statuses, (b) move the
   transferred value across the boundary, (c) repoint `current`, then return to
   the dispatch loop, which resumes at the new fiber's saved `ip`;
4. failure = the entry's error unwinds its own frame stack to empty, sets
   `failed`, stores the `Error`, and resumes the resumer as if `try` had caught
   it (or re-raises under `call`).

Because there is no preemption, no synchronization primitives are needed: a fiber
switch happens only at an explicit `call`/`yield`/`await` point.

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

| Signature | Side | Meaning |
|-----------|------|---------|
| `construct value(_)` | class | an already-`fulfilled` future |
| `construct error(_)` | class | an already-`rejected` future |
| `async(_)` | class | run a `Function` on a fresh fiber, returning a future for its result |
| `await` | instance | suspend the current fiber until settled; return the value or re-raise the error |
| `then(_)` | instance | register a continuation; returns a future for the continuation's result |
| `map(_)` | instance | `then` for the non-error path only |
| `catch(_)` | instance | register an error handler; returns a recovered future |
| `isReady` | instance | `true` once `fulfilled` or `rejected` |
| `value` | instance | the settled value as `Option` (never blocks) |

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

`Future` is a library-level `InstanceObject` — no new `Value` arm required
([`value.rs`](../../phalcom-core/src/value.rs) already has `Instance`). It needs:

1. `Fiber` (§1) as the substrate — `await` = "add `current` to the future's
   waiters, then `Fiber.yield` to the scheduler";
2. a **scheduler**: a ready-queue of resumable fibers plus a source of external
   completions (timers, I/O) exposed through [`System`](system.md). The top-level
   program runs inside the scheduler's root fiber, so `await` at top level is
   legal;
3. settlement moves the future to `fulfilled`/`rejected` and enqueues every waiter
   fiber and `then` continuation onto the ready-queue.

`Future` deliberately owns **no** new VM mechanism beyond `Fiber` + a queue —
keeping the concurrency primitive singular ([ADR to follow](../adr/README.md)).

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

See [Open Questions](open-questions.md) for undecided points (structured
concurrency / cancellation scopes, whether `Future` gets `select`/`race`
combinators, and the scheduler's fairness guarantees).
