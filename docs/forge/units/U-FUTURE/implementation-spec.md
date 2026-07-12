# U-FUTURE — Implementation Spec: `Future` as a pure library layer over `Fiber`

> **Status: DEFERRED — post-v0.2. DO NOT DISPATCH until unblocked.** This is a
> ready-to-activate work order, not a live one. `Future` derives entirely from
> [[U-FIBER]](../U-FIBER/implementation-spec.md) and adds **no new VM mechanism** beyond
> `Fiber` + a ready-queue ([ADR-0030](../../../adr/0030-fibers-and-futures-cooperative-concurrency.md)
> §1) — the library layer that ADR already sanctions. **No plan.md exists and no new ADR
> is needed.** Realizes [specification.md](specification.md) (deepening
> [`concurrency.md`](../../../spec/v0.2/concurrency.md) §2).
>
> **Baseline: none yet.** All anchors below are **provisional** and must be
> **re-grounded against HEAD at activation** — the whole substrate ([[U-FIBER]](../U-FIBER/implementation-spec.md))
> does not exist at HEAD `9d3b7e1`, so file:line targets for the seam (`FiberObject`,
> `resumer`, result slot) are named by role, not verified line.
>
> **Two hard gates before any edit:**
> 1. **[[U-FIBER]](../U-FIBER/implementation-spec.md) landed** — the `Fiber` substrate,
>    with the `resumer` link + result slot kept **general** (its spec §2.1/§7.2). Re-read
>    it on HEAD and confirm.
> 2. **U-SCHED decided and landed** — the scheduler (ready-queue + timer source + root
>    scheduler fiber). **Proposed only** today (`scheduler-unit.md`); it owns what `main`
>    is and is **not retrofittable**. Everything past `value`/`error`/`isReady`/`value`
>    (§ specification §2) is blocked on it.

---

## §0. Prerequisites + scope gate

### BLOCKED — activation preconditions (verify on HEAD)

| Gate | What it must provide | Status at HEAD `9d3b7e1` |
|---|---|---|
| **[[U-FIBER]](../U-FIBER/implementation-spec.md)** | `Object::Fiber(FiberObject)`; `Fiber.new`/`call`/`try`/`yield`/`current`/`abort`; the O(1) switch + typed `ControlFlow`; the fiber-floor failure capture; **general `resumer` + result slot**. | **NOT landed** — v0.2 scope is bare `Fiber`, and even that is planned, not built. |
| **U-SCHED** | ready-queue of resumable fibers; timer/I-O completion source via [`System`](../../../spec/v0.2/system.md); **top-level runs inside the root scheduler fiber**. | **Proposed only** (`scheduler-unit.md`) — no owner. |
| **U-CORE-6 unwind** | the unified `RuntimeError::Raise` unwind a rejected `Future` re-raises through; `Fiber` failure-capture. | **Landed** (`error.rs:86-94`). |
| **`ensure`/limits ruling** | whether abandoned-`Future` fibers run `ensure` (proposal: **no**; opt-in `Fiber.finish`). | **Proposed only** (`fiber-ensure-and-limits.md`). |

**Do not begin until gates 1 and 2 are green.** If only gate 1 is green, the **maximum
buildable slice** is the scheduler-free `Future.value`/`error`/`isReady`/`value` sub-slice
(§2 below) — flag and confirm with the user before shipping even that.

### Explicitly OUT of scope

- Any **new VM mechanism** — `Future` is pure `.ph`/library over `Fiber` + a queue
  (ADR-0030 §1). No opcode, no `Value` arm, no floor primitive **except** the scheduler
  hooks, which are **U-SCHED's**, not U-FUTURE's.
- `select`/`race`, cancellation scopes, structured concurrency, fairness — open
  ([`open-questions.md`](../../../spec/v0.2/open-questions.md) §15).

---

## §1. What exists vs what is missing (provisional — re-ground at activation)

### Will exist once the gates are green

- **`Future` as a plain `InstanceObject`** — `value.rs` already has `Instance`
  ([ADR-0030](../../../adr/0030-fibers-and-futures-cooperative-concurrency.md) §1); **no
  new `Value` arm.**
- **The `Fiber` seam** — `FiberObject.resumer` (dynamic caller chain) + the result slot,
  kept general by [[U-FIBER]](../U-FIBER/implementation-spec.md) (its §2.1). `await`
  suspends through `Fiber.yield` + `Fiber.current`.
- **U-SCHED** — the ready-queue + timer source + root scheduler fiber.

### Missing (this unit adds, once unblocked)

| Missing | Add in |
|---|---|
| `class Future` — state (`pending`/`fulfilled`/`rejected`), waiters list, optional driving `Fiber` | `core.ph` (mostly `.ph`) |
| `value(_)`/`error(_)` constructors; `isReady`; `value` (scheduler-free sub-slice) | `core.ph` |
| `async(_)`/`await`/`then(_)`/`map(_)`/`catch(_)` over the scheduler | `core.ph` + U-SCHED hooks |
| settlement → enqueue waiters + `then`s | `core.ph` + U-SCHED ready-queue |
| `concurrency` corpus goldens (all PENDING until gates green) | `tests/lang/concurrency/` |

**No `vm.rs`/`bytecode.rs`/`value.rs` edit** — the mechanism is `Fiber` + U-SCHED's queue.

---

## §2. Native/`.ph` split + insertion points (provisional)

**Decision: `Future` is almost entirely `.ph`.** The only native surface is whatever
U-SCHED exposes (ready-queue enqueue, timer registration via `System`) — and **that is
U-SCHED's floor, not U-FUTURE's.** `Future` itself is a library class.

| Concern | Owner | Native / `.ph` |
|---|---|---|
| `Future` state machine, waiters, constructors, `then`/`map`/`catch`, `isReady`, `value` | **U-FUTURE** | `.ph` (`core.ph`) |
| `async`/`await` suspension via `Fiber.yield`/`current` | **U-FUTURE** over **[[U-FIBER]]** | `.ph` over `Fiber` primitives |
| ready-queue, timer completion, root scheduler fiber, `System.sleep` | **U-SCHED** | native (U-SCHED's floor bump) |

**Insertion points (once unblocked):** a `class Future` block in `core.ph`, serialized
against every other `core.ph` editor; U-SCHED's `System` hooks are consumed, not defined,
here. Re-ground all of this at activation — the substrate does not exist yet.

---

## §3. Concrete bodies / pseudocode (design intent — re-verify against the landed `Fiber` surface)

```phalcom
// Future — a state machine over Fiber (ADR-0030 §1, concurrency.md §2). Pure library.
class Future {
  construct value(v) { _state = fulfilled; _value = v }
  construct error(e) { _state = rejected;  _value = e }

  isReady => _state != pending
  value   => _state == fulfilled ? Some(_value) : None    // never suspends

  // Requires U-SCHED: run fn on a fresh fiber, settle this future with its result.
  static async(fn) {
    let f = Future.new                                   // pending, waiters = []
    let driver = Fiber.new {
      // try/catch here rides the U-CORE-6 unwind + U-FIBER failure capture
      f.settleValue(fn.call())                           // or settleError on raise
    }
    System.enqueue(driver)                               // U-SCHED ready-queue
    return f
  }

  // Requires U-SCHED: suspend the current fiber until settled (direct style).
  await {
    self.isReady.ifFalse {
      _waiters.add(Fiber.current)                        // the resumer/result-slot seam
      Fiber.yield(None)                                  // yield to the scheduler
    }
    return _state == rejected ? _value.raise() : _value
  }

  then(g)  { /* register g; return a Future for g's result (CPS) */ }
  map(g)   { /* then on the fulfilled path only */ }
  catch(h) { /* handler on the rejected path; recovered future */ }

  // settle-once: ignore further completions; enqueue every waiter + then continuation.
  settleValue(v) { self.isReady.ifFalse { _state = fulfilled; _value = v; self.drain } }
  settleError(e) { self.isReady.ifFalse { _state = rejected;  _value = e; self.drain } }
  drain { /* System.enqueue each waiter fiber + then continuation (U-SCHED) */ }
}
```

> **⚠ All spellings above are design intent.** Re-verify `Fiber.new`/`yield`/`current`,
> `Option`/`ifFalse`, and the U-SCHED `System.enqueue`/timer API against the **landed**
> surface at activation. The `await`-under-native-frame restriction
> ([[U-FIBER §3.2]](../U-FIBER/specification.md#32-the-restriction-adr-0030-4)) applies:
> an `await` under a `block_call` raises `CannotYieldAcrossNativeFrame`.

---

## §4. Test strategy — all PENDING until the gates are green

| ID | Test | Gated on |
|---|---|---|
| **C-FUT-1** | `Future.value(v).await == v`; `Future.error(e).await` re-raises `e`. | U-FIBER |
| **C-FUT-2** | `Future.async { … }.await` returns the result after suspending. | U-FIBER + U-SCHED |
| **C-FUT-3** | settle-once: further completions ignored. | U-FIBER |
| **C-FUT-4** | `then`/`map`/`catch` fire on settlement with correct value/error path. | U-FIBER + U-SCHED |
| **C-FUT-5** | top-level `await` is legal (root scheduler fiber). | U-SCHED |
| **C-FUT-6** | `await` yields to the scheduler; **never blocks the OS thread** (no single-thread deadlock). | U-SCHED |
| **C-FUT-7** | `await` under a native `block_call` raises `CannotYieldAcrossNativeFrame`. | U-FIBER |
| **C-FUT-8** | `value`/`isReady` never suspend. | U-FIBER |

Stage them under `tests/lang/concurrency/pending/` with `#[ignore]` and a `DEFERRED.md`
pointer until activation.

---

## §5. Must-not-preclude

| Hazard | How this design clears it |
|---|---|
| **A blocking `await`** (single-thread deadlock). | `await` **yields to the scheduler** (§3), never parks the OS thread — precluded by construction (`scheduler-unit.md`). |
| **A second concurrency primitive.** | `Future` adds **no VM mechanism** beyond `Fiber` + a queue (ADR-0030 §1) — it is pure library; the primitive stays singular. |
| **Generator-specialized `Fiber`.** | Relies on [[U-FIBER]](../U-FIBER/specification.md) keeping the `resumer` + result slot **general** (§6 seam); if U-FIBER specialized them, `await` breaks — verify at gate 1. |
| **`select`/`race`, cancellation scopes.** | Not built; keep the waiter list + ready-queue shaped so a later cancellation scope / `select` layers on (§8 spec, open-questions §15). |
| **Abandoned-fiber cleanup.** | Follows the `fiber-ensure-and-limits.md` ruling (abandoned fibers do **not** run `ensure`; opt-in `Fiber.finish`) — **not yet ratified**; do not bake guaranteed-cleanup semantics. |
| **Resource exhaustion.** | Frame-depth `StackOverflow` / per-turn `MemoryError` caps are a robustness dependency (post-v0.2); a `Future`-heavy program is a DoS surface without them — flag, don't silently rely on unbounded recursion/allocation. |

---

## §6. Open items, sequencing, traceability

### Open / BLOCKED items (do not guess — resolve before building)

- **U-SCHED ownership + ratification** — the single largest blocker (§0). Until it lands,
  `async`/`await`/`then`/`map`/`catch` cannot be built. Only the scheduler-free
  `value`/`error`/`isReady`/`value` sub-slice is reachable, and only after U-FIBER.
- **`ensure`-on-abandoned-fiber + resource caps** — `fiber-ensure-and-limits.md` proposals,
  unratified; they shape `Future` cleanup + robustness (§5). Confirm the ruling first.
- **Structured concurrency / cancellation / `select`/`race` / fairness** — open-questions
  §15; out of scope, keep layerable.

### Sequencing (once unblocked)

```
[[U-FIBER]] (bare Fiber, landed) ──▶ U-SCHED (ready-queue + timers + root fiber) ──▶ U-FUTURE
       │                                                                              ▲
       └── keeps resumer + result slot general (the seam) ────────────────────────────┘
```

1. *(optional, after U-FIBER only)* the scheduler-free `Future.value`/`error`/`isReady`/
   `value` sub-slice — flag with the user first.
2. after U-SCHED: `async`/`await`; graduate C-FUT-2/5/6.
3. `then`/`map`/`catch` + settlement drain; graduate C-FUT-3/4.
4. the `await`-under-native-frame restriction test (C-FUT-7) + `value`/`isReady`
   non-suspension (C-FUT-8).

### Traceability

| Claim | Source |
|---|---|
| `Future` = pure library over `Fiber` + ready-queue; no new VM mechanism / `Value` arm | [ADR-0030](../../../adr/0030-fibers-and-futures-cooperative-concurrency.md) §1; [concurrency.md](../../../spec/v0.2/concurrency.md) §2 |
| Surface (`value`/`error`/`async`/`await`/`then`/`map`/`catch`/`isReady`/`value`); state machine; settle-once | [concurrency.md](../../../spec/v0.2/concurrency.md) §2; [specification.md](specification.md) §2–§3 |
| Scheduler = U-SCHED (ready-queue + timers + root fiber); blocking-await precluded; not retrofittable | [scheduler-unit.md](../../../spec/v0.2/experimental/scheduler-unit.md) |
| The `Fiber` seam (resumer + result slot; `await` = waiters + `Fiber.yield`) | [[U-FIBER §7.2]](../U-FIBER/specification.md#72-fiber--future--the-resumerresult-slot-seam); [concurrency.md](../../../spec/v0.2/concurrency.md) §2 |
| `ensure`-on-abandoned-fiber (no; opt-in `Fiber.finish`); caps (`StackOverflow`/`MemoryError`) | [fiber-ensure-and-limits.md](../../../spec/v0.2/experimental/fiber-ensure-and-limits.md) |
| Still-open: structured concurrency, `select`/`race`, fairness | [open-questions.md](../../../spec/v0.2/open-questions.md) §15 |
| No new ADR needed (ADR-0030 already sanctions the library layer) | [ADR-0030](../../../adr/0030-fibers-and-futures-cooperative-concurrency.md) §1/§Consequences |
