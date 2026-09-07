# U-FUTURE — Specification: `Future` as a pure library layer over `Fiber`

> **Status: DEFERRED — post-v0.2.** `Future` is **not** part of the v0.2 bare-`Fiber`
> ship (user decision, 2026-07-12). **Every section of this document is gated on
> [[U-FIBER]](../../../../forge/units/U-FIBER/specification.md) landing** — `Future` derives entirely from
> `Fiber` and adds **no new VM mechanism** beyond `Fiber` + a ready-queue
> ([ADR-0030](../../../adr/0030-fibers-and-futures-cooperative-concurrency.md) §1). This
> is the library layer [ADR-0030](../../../adr/0030-fibers-and-futures-cooperative-concurrency.md)
> already sanctions; **no new ADR is needed.**
>
> **This is authored fresh** from [`concurrency.md`](../../../../spec/current/concurrency.md) §2
> (the surface) + [`experimental/scheduler-unit.md`](../../../design/experimental/v0.2/scheduler-unit.md)
> (the scheduler that closes the bootstrap gap) +
> [`experimental/fiber-ensure-and-limits.md`](../../../design/experimental/v0.2/fiber-ensure-and-limits.md)
> (abandoned-fiber `ensure` + resource caps). It deepens `concurrency.md` §2 by reference;
> that document stays the surface index.
>
> **Governing sources.** [ADR-0030](../../../adr/0030-fibers-and-futures-cooperative-concurrency.md)
> §1 (`Future` = pure library layer); [`concurrency.md`](../../../../spec/current/concurrency.md)
> §2 (surface + implementation notes); [`system.md`](../../../../spec/current/system.md)
> (the scheduler's external-completion source); [`open-questions.md`](../../../../spec/current/open-questions.md)
> §15 (still-open: structured concurrency / cancellation, `select`/`race`, scheduler
> fairness).
>
> **BLOCKED / verify-on-HEAD — do not build until resolved:**
> - **U-FIBER must have landed** (the whole substrate). Re-read
>   [[U-FIBER]](../../../../forge/units/U-FIBER/specification.md) on HEAD; confirm the `resumer` link + result
>   slot are general (not generator-specialized) — `await` suspends through them (§6).
> - **The scheduler has no owner yet.** `scheduler-unit.md` proposes a unit **U-SCHED**
>   sequenced after `Fiber`, before any `Future` method beyond `value`/`isReady`. **U-SCHED
>   must be ratified and landed** before `async`/`await`/`then` (§4). This is a genuine open
>   dependency, flagged throughout.
> - **`ensure`-on-abandoned-fiber** is a proposal (`fiber-ensure-and-limits.md`), not
>   ratified. Its ruling ("abandoned fibers do NOT run `ensure`; opt-in `Fiber.finish`")
>   affects `Future` cleanup semantics (§8).

---

## 1. Overview and dependency

A `Future` represents a value that may not exist yet — a thin **state machine over
`Fiber`**. It is a **library-level `InstanceObject`** — **no new `Value` arm** required
([`value.rs`](../../../../phalcom-core/src/value.rs) already has `Instance`;
[ADR-0030](../../../adr/0030-fibers-and-futures-cooperative-concurrency.md) §1). It needs
exactly three things, **all of which depend on [[U-FIBER]](../../../../forge/units/U-FIBER/specification.md)**:

1. **`Fiber`** (§6) as the substrate — `await` = "add `current` to the future's waiters,
   then `Fiber.yield` to the scheduler";
2. **a scheduler** (§4, the proposed **U-SCHED**) — a ready-queue of resumable fibers plus
   an external-completion source (timers, I/O) exposed through
   [`System`](../../../../spec/current/system.md);
3. **settlement** (§5) moves the future to `fulfilled`/`rejected` and enqueues every waiter
   fiber and `then` continuation onto the ready-queue.

```
[[U-FIBER]] ──▶ U-SCHED (ready-queue + timers) ──▶ Future.await/async/then
                                              └──▶ System.sleep/schedule
```

`Future` deliberately owns **no** new VM mechanism beyond `Fiber` + a queue — the
concurrency primitive stays singular (ADR-0030).

---

## 2. Surface

*Deepens [`concurrency.md`](../../../../spec/current/concurrency.md) §2.*

| Signature | Side | Meaning | Needs scheduler? |
|---|---|---|---|
| `@constructor
value(_)` | class | an already-`fulfilled` future | no |
| `@constructor
error(_)` | class | an already-`rejected` future | no |
| `async(_)` | class | run a `Function` on a fresh fiber, returning a future for its result | **yes** |
| `await` | instance | suspend the current fiber until settled; return the value or re-raise the error | **yes** |
| `then(_)` | instance | register a continuation; returns a future for the continuation's result | **yes** |
| `map(_)` | instance | `then` for the non-error path only | **yes** |
| `catch(_)` | instance | register an error handler; returns a recovered future | **yes** |
| `isReady` | instance | `true` once `fulfilled` or `rejected` | no |
| `value` | instance | the settled value as `Option` (never blocks) | no |

The `value`/`error`/`isReady`/`value` set is **scheduler-free** and could ship as a thin
sub-slice before U-SCHED; everything else is gated on the scheduler (§4).

```phalcom
let f = Future.async { slowComputation() }
doOtherWork()
let result = f.await          // suspends this fiber until f settles
```

---

## 3. State machine

- **state** — `pending`, `fulfilled(value)`, or `rejected(error)`;
- **waiters** — fibers suspended in `await` on this future, plus `then` continuations,
  resumed/queued when it settles;
- (for `async` futures) **the driving `Fiber`**.

```
        Future.async(fn) / new pending future
                    │
                    ▼
              ┌──────────┐   settle-value(v)   ┌──────────────┐
              │ pending  │ ──────────────────▶ │ fulfilled(v) │
              │ waiters: │                     └──────────────┘
              │  [fibers,│   settle-error(e)   ┌──────────────┐
              │   thens] │ ──────────────────▶ │ rejected(e)  │
              └──────────┘                     └──────────────┘
                    │  (settle enqueues every waiter + then onto the ready-queue)
                    ▼
        further completions are IGNORED — a Future settles exactly ONCE.
```

`isReady` is `true` in `fulfilled`/`rejected`; `value` returns `Some(v)` when `fulfilled`,
else `None` (never blocks — it does not suspend).

---

## 4. The scheduler (proposed U-SCHED) — BLOCKED, no owner yet

*From [`scheduler-unit.md`](../../../design/experimental/v0.2/scheduler-unit.md); hazard:
**primitive/library boundary ⊗ bootstrap order**.*

`Future.await`/`async` and `System.sleep` bottom out in a **scheduler run-loop** that
today exists only in prose. `scheduler-unit.md` proposes an explicit unit **U-SCHED**,
sequenced **after [[U-FIBER]](../../../../forge/units/U-FIBER/specification.md), before any `Future` method
beyond `value`/`isReady`**:

- a **ready-queue** of resumable fibers;
- a **timer completion source** driving `System.sleep`;
- **settlement enqueues** every waiter fiber + `then` continuation.

**Top-level runs inside the root scheduler fiber** — so top-level `await` is legal. This
is **not retrofittable**: it defines what `main` is, and must be fixed in U-SCHED, not
after `Future` ships.

> **Precludes a blocking `await`.** `await` must **yield to the scheduler**, never park the
> OS thread — otherwise the single thread deadlocks. Enforced by construction (§6).

> **⚠ BLOCKED / verify-on-HEAD.** U-SCHED is **Proposed, not ratified, and has no landed
> owner.** U-FUTURE's scheduler-dependent surface (`async`/`await`/`then`/`map`/`catch`)
> **cannot be built until U-SCHED is decided and landed.** Confirm its status on HEAD
> before dispatch; if still open, U-FUTURE ships at most the scheduler-free
> `value`/`error`/`isReady`/`value` sub-slice (§2), or waits.

---

## 5. Operational semantics (gated on U-SCHED)

- **`Future.async(fn)`** — allocate a `pending` future `f`; `Fiber.new(fn')` where `fn'`
  runs `fn`, then `settle`s `f` with the result (or the raised error); enqueue that fiber
  onto the ready-queue; return `f`.
- **`f.await`** — if `f.isReady`, return/re-raise immediately; else add `Fiber.current` to
  `f.waiters` and `Fiber.yield` to the scheduler. When `f` settles, the scheduler resumes
  this fiber with the value (or re-raises the error). **Direct-style** suspension.
- **`f.then(g)`** — register `g` as a continuation on `f`; return a new future for `g`'s
  result. **Continuation-passing** style; non-suspending.
- **`f.map(g)`** — `then` on the fulfilled path only; propagate rejection unchanged.
- **`f.catch(h)`** — register `h` on the rejected path; return a recovered future.
- **settlement** — moves `f` to `fulfilled`/`rejected` (exactly once) and enqueues every
  waiter fiber + `then` continuation onto the ready-queue.

`await` and `then`/`map`/`catch` are **interconvertible** because both bottom out in the
same waiter list — `await` is the direct-style face, `then` the CPS face.

---

## 6. The `Fiber` seam {#the-fiber-seam}

*Cross-links [[U-FIBER §7.2]](../../../../forge/units/U-FIBER/specification.md#future-seam).*

`Future` layers over `Fiber` at exactly two [[U-FIBER]](../../../../forge/units/U-FIBER/specification.md)
features, which U-FIBER was required to keep **general**:

1. **`Fiber.yield` + the resumer link** — `await` suspends the current fiber to the
   scheduler by `Fiber.yield`; the scheduler is the resumer. The `resumer` link must be
   the dynamic caller chain U-FIBER built, not a generator-specific parent.
2. **the result slot** — a settled value / captured `Error` moves across the fiber
   boundary through the same result slot U-FIBER uses for `yield`/return/failure.

Because U-FIBER keeps both general (its spec §2.1, §7.2), `Future` adds only the waiter
list + ready-queue on top — **no VM change**. `await`ing inside a fiber body is subject to
the same **restricted-yield guard** as any `Fiber.yield`: an `await` under a native
`block_call` raises `CannotYieldAcrossNativeFrame` ([[U-FIBER §3.2]](../../../../forge/units/U-FIBER/specification.md#restricted-yield)).

---

## 7. Error surface

| Situation | Result |
|---|---|
| the driving fiber of an `async` future raises | the future settles `rejected(error)`; `await` re-raises it, `catch` handles it, `then`'s error path propagates it |
| `await` on a `rejected` future | re-raises the captured `Error` into the awaiter's unwind (U-CORE-6 unwind) |
| `await` under a native frame (a fiber restriction) | `CannotYieldAcrossNativeFrame` (§6) |
| double settlement | ignored — settle-once (§3) |
| a blocking `await` implementation | **precluded by construction** (§4) — would deadlock the single thread |

`Future` introduces no error type of its own; it rides the U-CORE-6 unified unwind and the
U-FIBER failure-capture path.

---

## 8. Cross-feature interactions & open items — BLOCKED

- **`ensure`-on-abandoned-fiber** (`fiber-ensure-and-limits.md`, Proposed). A `Future`
  whose driving fiber is abandoned (suspended forever, then collected) **does not run its
  `ensure` blocks** (matches Lua, not Python) — cleanup that must be guaranteed belongs in
  the awaiter (`try`/`ensure` around `await`), or via the opt-in **`Fiber.finish`**. **⚠
  This ruling is not ratified; confirm before relying on it** for `Future` cleanup.
- **Resource caps** (`fiber-ensure-and-limits.md`): max frame-stack depth → `StackOverflow`
  (an `Error`, never a Rust `panic!`); per-turn allocation ceiling → `MemoryError`. A
  `Future`-heavy program is a DoS surface without these; **flag as a robustness dependency**
  (post-v0.2).
- **Structured concurrency / cancellation scopes, `select`/`race`, scheduler fairness** —
  **still open** ([`open-questions.md`](../../../../spec/current/open-questions.md) §15). U-FUTURE
  ships none of them; leave the waiter list + ready-queue shaped so a later cancellation
  scope / `select` can layer on.

---

## 9. Conformance points (ALL PENDING — graduate only after U-FIBER + U-SCHED land)

| ID | Requirement | Gated on |
|---|---|---|
| **C-FUT-1** | `Future.value(v).await` returns `v`; `Future.error(e).await` re-raises `e`. | U-FIBER |
| **C-FUT-2** | `Future.async { … }.await` returns the function's result after suspending. | U-FIBER + U-SCHED |
| **C-FUT-3** | a settled future ignores further completions (settle-once). | U-FIBER |
| **C-FUT-4** | `then`/`map`/`catch` fire on settlement with the right value/error path. | U-FIBER + U-SCHED |
| **C-FUT-5** | top-level `await` is legal (root scheduler fiber). | U-SCHED |
| **C-FUT-6** | `await` yields to the scheduler and **never blocks the OS thread**. | U-SCHED |
| **C-FUT-7** | `await` under a native `block_call` raises `CannotYieldAcrossNativeFrame`. | U-FIBER |
| **C-FUT-8** | `value`/`isReady` never suspend. | U-FIBER |

---

## 10. Non-goals

- **Any v0.2 delivery** — U-FUTURE is post-v0.2 and Deferred; it does not ship with bare
  `Fiber`.
- **A new VM mechanism** — `Future` is pure library over `Fiber` + a ready-queue
  (ADR-0030 §1); it adds no opcode, no `Value` arm, no floor primitive beyond the
  scheduler hooks U-SCHED owns.
- **Preemption / multithreading** — inherited from `Fiber`: cooperative, single-threaded.
- **`select`/`race`, cancellation scopes, fairness guarantees** — open (§8); not shipped.
