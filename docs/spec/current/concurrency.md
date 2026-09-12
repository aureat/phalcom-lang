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
- a **status** — one of `new`, `running`, `blockedOnChild`, `yielded`,
  `parked(generation)`, `queued`, `done`, or `failed`. `running` identifies
  exactly `VM.current`; a child-blocked, Future-parked, or queued fiber is not
  manually resumable;
- a **root marker** — root identity is stable and is not inferred from the
  dynamic caller chain;
- a **resumer link** — the fiber to hand control back to on `yield` / return /
  failure (forming a dynamic immediate-control-transfer chain, not a fixed
  parent or durable completion owner);
- a **result slot** — the last yielded/returned value, or the captured `Error`;
- for Future parking, a monotonic **park generation** and a single completion
  observer handle, both retained with the Fiber object.

The **root fiber** is the main program and remains identifiable even when its
dynamic resumer link is empty. A non-root Fiber reaches `done` or `failed` only
once; terminal state is never resumed.

### Interface

| Signature | Side | Meaning |
|-----------|------|---------|
| `@constructor new(_)` | class | wrap a `Function` as a not-yet-started fiber |
| `call` / `call(_)` | instance | resume; the argument becomes the value of the suspended `yield` (or the entry's parameter on first resume). Returns the next yielded/returned value |
| `try` / `try(_)` | instance | like `call`, but terminal failure delivers an `Error` value; use `error`/`isDone` to distinguish it from Error data |
| `isDone` | instance | `true` once `done` or `failed` |
| `error` | instance | the captured `Error` as `Option`, if `failed` |
| `yield(_)` | **class** | suspend the *current* fiber, handing the value to its resumer. Returns the value passed to the next `call` |
| `current` | **class** | the fiber now running |
| `abort(_)` | **class** | raise in the current non-root fiber; has no normal return (`Never`); this is not cancellation |

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
`yield`/return pops it. The link describes immediate control transfer only; it is
not the ownership channel for an asynchronous completion. Scheduler admission,
Future parking/wake, and terminal completion use the explicit internal seams
described in §2.

A scheduler-owned Fiber MUST NOT use user `yield`: its queue driver has no
coroutine consumer to receive that value. Such an attempt raises before any
transfer. Manual coroutine yield remains available, including None and Error
as ordinary data. Await parking uses a separate, ticketed operation.

`call` currently has linked terminal failure semantics: uncaught child failure
terminally fails Call-linked parents until a Try/Scheduler boundary or root.
It does not inject an ordinary catchable exception at the call expression.

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

### Suspension through calls

Ordinary Function calls and source-language collection callbacks use flat VM
activations and permit suspension. A stored closure or `each` callback can yield
when driven by a manual Fiber; an executor-driven callback can await.

Native helpers that recursively drive user code on the Rust stack, including
`on` and `ensure`, remain non-suspendable. Attempting to switch beneath such a
frame MUST raise `CannotYieldAcrossNativeFrame` before changing execution
ownership. This guard protects host continuation state; it is not an async
modifier requirement on ordinary functions.

---

## 2. `Future` — pending asynchronous result

A `Future` represents a value that may not exist yet. It is a thin state machine
over `Fiber`: `await` suspends the current fiber until the future is settled, and
a **scheduler** (a run loop over ready work) drives settlement.

### Structure

- a **state** — `pending`, `fulfilled(value)`, or `rejected(error)`;
- a **waiters** list — exact `(Fiber, generation)` tickets for suspended `await`
  operations and closures for pending `then`/`map`/`catch` callbacks;
- (for `async` futures) an action Fiber whose terminal observer owns settlement.

A `Future` settles **exactly once**; further completions are ignored.

### Interface

**Both slices are landed.** The status column below records which slice a member
came from, not whether it exists: **A** = the scheduler-free state-machine
surface; **B** = the scheduler-backed async/await and pending-continuation
surface. The original yield-probe implementation was replaced by the stable
`Fiber#isRoot` predicate when E004 was fixed.

| Signature | Side | Status | Meaning |
|-----------|------|--------|---------|
| `@constructor value(_)` | class | **A** | an already-`fulfilled` future |
| `@constructor error(_)` | class | **A** | an already-`rejected` future |
| `async(_)` | class | **B** | run a `Function` on a fresh fiber, returning a future for its result |
| `await` | instance | **B** | suspend the current fiber until settled; return the value or re-raise the error |
| `then(_)` | instance | **A** (settled-only); pending continuation is **B** | chain a callback returning Future<U>; return Future<U> |
| `map(_)` | instance | **A** (settled-only); pending continuation is **B** | map a fulfilled value through (T) -> U; return Future<U> |
| `catch(_)` | instance | **A** (settled-only); pending continuation is **B** | recover an error with (Error) -> T; return Future<T> |
| `isReady` | instance | **A** | `true` once `fulfilled` or `rejected` |
| `value` | instance | **A** | the settled value as `Option` (never blocks) |

Matching `then`/`map`/`catch` callbacks MUST run on scheduler-owned Fibers,
independent of receiver readiness. Registration MUST return before invoking user
code. The derived Future settles from terminal callback success or failure;
parking does not settle it. A thrown callback Error rejects the derived Future;
a returned Error or None fulfills it as data. An unmatched already-settled
outcome may pass through immediately without invoking a callback.

`Future<T>` is invariant because it exposes `settleValue(T)`. Its `await`
result is T and its `value` is Option<T>. `async<U>(() -> U)` returns Future<U>.
A pending constructor requires sufficient contextual type evidence.

`map<U>((T) -> U)` returns Future<U> and preserves every callback result as data,
including Future values. `then<U>((T) -> Future<U>)` returns Future<U> by adopting
the returned Future. `catch((Error) -> T)` recovers with a value of the original
payload type; `recoverWith((Error) -> Future<T>)` recovers by adopting asynchronous
work. `flatten<U>(Future<Future<U>>)` removes exactly one Future layer. These
operations MUST NOT infer value mapping versus chaining from runtime payload type.
Direct self-adoption MUST reject the result. Indirect dependency cycles are not
detected. Dynamic callers whose chaining callback returns a non-Future receive a
rejected result, rather than losing the completion observer.

```phalcom
let f = Future.async { slowComputation() }
doOtherWork()
let result = f.await          // suspends this fiber until f settles
```

`await` is **sugar-free suspension**, not blocking. A root Fiber pumps the
internal scheduler dequeue/resume path while the Future is pending. A
scheduler-owned non-root Fiber obtains a park generation, registers the exact
`(Fiber, generation)` ticket, and parks; settlement wakes it only when the
generation still matches. Manual Fiber call-chains cannot await a pending Future
under this contract (D-07), and native-boundary refusal is checked before a
waiter is registered.

### Implementation

`Future` is a library-level `InstanceObject` — no new `Value` arm
([`value.rs`](../../../phalcom-core/src/value.rs) already has `Instance`).

**Slice A — landed** ([`core.ph`](../../../phalcom-core/core/core.ph) `class
Future`, [U-FUTURE](../../work/pending/fiber-schedule/future/plan.md)): a pure-`.ph`
settle-once state machine over three private fields (`_state`/`_value`/
`_waiters`). `value(_)`/`error(_)` construct an already-settled future;
`isReady`/`value` read state. Reading or awaiting an already-settled Future
does not suspend; matching continuation callbacks use the observed execution
bridge even when registration occurs after settlement.

**Slice B — landed** over `Fiber` (§1) and the native ready-queue. Public code
admits work with `System.schedule(_)` and drains it with `System.runScheduled`;
raw dequeue, scheduler resume, park/wake, and completion-observer operations are
internal runtime seams. The public raw `System.nextScheduled` getter was removed
because it released queue ownership and allowed a queued Fiber to be stolen by
manual `try`.

The root-drive pump in `VM::run` and the `.ph` pump both dequeue FIFO work and
resume it through the scheduler mode. A scheduled failure is isolated from
sibling work according to the Call/Try/Scheduler policy. Detached failures are
reported at safe scheduler/root boundaries; completion-owned failures instead
reach their Future. Admission MUST reject an unstarted entry that cannot accept
zero arguments before reserving queue ownership, so malformed work cannot
poison a later drain.

Future settlement drains two waiter forms: exact `(Fiber, generation)` tickets
for parked awaiters, and closures for pending `then`/`map`/`catch`. Each callback
is run on a fresh Fiber with a durable terminal observer. Terminal success or
failure, rather than a first yield or the dynamic `resumer`, settles the derived
Future. Observer handles are GC-traced and detached once at `Done`/`Failed`.

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
