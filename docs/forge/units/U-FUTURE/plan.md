# U-FUTURE — Work order: `Future` as a library layer over the landed `Fiber`

_Self-contained dispatch plan for **one** implementer. Complements the existing
[specification.md](specification.md) and [implementation-spec.md](implementation-spec.md),
but **re-grounds them against the now-landed `Fiber` substrate** — those two docs were
authored at HEAD `9d3b7e1` when `Fiber` did **not** exist and `Future` was fully DEFERRED.
This plan is authored against **HEAD `0de7496`** (U-FIBER landed on `main`). **Reviewer ON**
(library layer over a deep VM primitive). Green gate: `./scripts/verify.sh` exits 0 +
`cargo doc --workspace --no-deps` clean._

> **Governing sources.** [ADR-0030](../../../adr/0030-fibers-and-futures-cooperative-concurrency.md)
> §1 (`Future` = pure library layer, "no VM mechanism beyond `Fiber` + a queue"); §Consequences
> (floor-amendment convention). [concurrency.md §2](../../../spec/current/concurrency.md) (surface +
> state machine + settle-once). [system.md §2](../../../spec/current/system.md) (the reserved
> `System.schedule(_)`/`sleep(_)` scheduler seam). [scheduler-unit.md](../../../design/experimental/v0.2/scheduler-unit.md)
> (the proposed, **unowned** `U-SCHED`). [open-questions.md §15](../../../spec/current/open-questions.md)
> (still-OPEN: structured concurrency / cancellation, `select`/`race`, fairness).

---

## 1. Mission (one sentence)

Give Phalcom a `Future` value that layers a **settle-once state machine over the landed
`Fiber`** — shipping the **scheduler-free surface** (`value`/`error`/`isReady`/`value` +
settle-once combinators) as **pure `.ph`** now, and scoping `async`/`await`/suspending
combinators behind a single load-bearing decision (**DEC-FUT-SCHED**, §9) because a
suspending `Future` provably needs a native ready-queue home that the object model does not
today provide.

---

## 2. Preconditions — re-verified against landed `Fiber` (HEAD `0de7496`)

| Gate | What v1 needs | Status on HEAD | Anchor |
|---|---|---|---|
| **U-FIBER substrate** | `Object::Fiber` + `Fiber.new`/`call`/`try`/`yield`/`current`/`abort` | **LANDED** | `primitive/fiber.rs` L86–237; registered `universe.rs` L487–496 |
| **`resumer` + result slot kept GENERAL (not generator-specialized)** — `await` suspends through them | **VERIFIED GENERAL.** `resumer: Option<ObjRef>` is a dynamic caller chain set on every `call`/`try`; the resume/yield value crosses through a stack `resume_slot`; nothing is generator-specific. | `heap.rs` `FiberObject` L160–204 (`resumer` L181, `result` L184, `resume_slot` L194, `resume_mode` L204); `fiber_resume` L143–200; `fiber_yield` L212–237 |
| **Unified unwind + fiber-floor capture** (P5 / U-CORE-6 / ADR-0008) — a rejected/failed fiber is capturable, host survives | **LANDED** (was pending U-CORE-6 in U20; now shipped). `RuntimeError::Raise { error, rendered }` propagates via `?`; `Fiber#try` captures it at the fiber floor (`FiberResumeMode::Try`) instead of re-raising. | `error.rs` `RuntimeError::Raise`; `fiber_try` L123–125; floor capture in `vm.rs` `run_until` |
| **`List` + `Error` root** — waiter list storage, error values | **LANDED** (U-ITER/U-STD `List`; U-CORE-6 `Error` + `Error#raise`) | `universe.rs` L204/L482 |

### Gaps found on HEAD that change the old plan (READ THESE — they are load-bearing)

1. **`Fiber#isDone` and `Fiber#error` are NOT landed.** `concurrency.md §1` lists them, but
   U-FIBER shipped only `new/call/try/yield/current/abort` (`universe.rs` L487–496). There is
   **no `.ph`-observable way to tell "the fiber I just resumed terminated" from "it yielded"**
   — the VM knows (`FiberStatus::{Suspended,Done,Failed}`, `heap.rs` L110) but does not expose
   it. A pump-driven `async` cannot detect driver completion/failure without this. **UPDATE
   (2026-07-13): spun out to [U-FIBER-REFLECT](../U-SCHED-FIBER/U-FIBER-REFLECT/plan.md)** — both are pure
   `FiberStatus`/`result`-slot reads with **no scheduler dependency**, so bundling them into
   this unit's own DEC-FUT-SCHED-gated Slice B was incidental coupling, not load-bearing. Slice
   B now **depends on** U-FIBER-REFLECT landing (a precondition, checked at Slice B dispatch)
   rather than implementing the accessors itself. Slice A needs neither.
2. **ADR-0031's `try`/`catch`/`on`/`ensure` surface syntax is Accepted but UNIMPLEMENTED.** No
   `compile_try`/`catch`/`ensure` in the compiler/AST; `.ph` has only `Error#raise` (arity 0,
   `universe.rs` L482). So an `async` driver body **cannot self-`catch`** its function's raise
   to call `settleError`. → The reject path must route failure through **`Fiber#try` + the
   pump** (Slice B), not a `.ph` handler.
3. **The object model has NO class-side / static / module-level mutable state.** No class
   variables in `object-model.md`/`classes.md`; the only `.ph` static is `System.print`'s
   method stub (`core.ph` L293–296) — a method, not a field. → **A ready-queue has no `.ph`
   home.** This is the crux that decides native-vs-`.ph` (§3).

---

## 3. Native-vs-`.ph` verdict (DECIDED, per ADR-0030 §1 + the HEAD gaps)

**The split is not all-or-nothing. It falls exactly on the settled/suspending line:**

- **Scheduler-free `Future` is PURE `.ph`.** `Future.value(_)`/`error(_)` build an
  already-settled instance; `isReady`/`value` read state; settle-once + `then`/`map`/`catch`
  over an *already-settled* future fire synchronously. This needs **zero** native code — only
  the landed `Fiber` is not even required for this slice (settled futures never suspend). It is
  buildable **now**, entirely in `core.ph`. This is **U-FUTURE v1 (Slice A)**.

- **A *suspending* `Future` (`async`/`await`/pending→settle drain) needs a MINIMAL native
  seam**, for three independently-sufficient reasons grounded above: (a) the ready-queue has
  **no `.ph` home** (§2 gap 3); (b) driver-completion/failure detection needs native
  `Fiber#isDone` (§2 gap 1); (c) the reject path needs `Fiber#try` at a pump because `.ph`
  has no `catch` (§2 gap 2). The seam is exactly the one **already reserved** by
  `system.md §2`: **`System.schedule(_)`** (enqueue a resumable fiber onto a runtime-owned
  FIFO) — plus `Fiber#isDone`. The *event-loop policy* (waiter registration, settle-drain,
  pump-until-settled, root-drive) can then be `.ph` over that seam. **This is Slice B, and it
  is the [scheduler-unit.md](../../../design/experimental/v0.2/scheduler-unit.md) `U-SCHED` unit
  — proposed, ratified in spirit by ADR-0030, but with no owner.** Whether U-FUTURE absorbs
  that seam or `U-SCHED` ships it as a prerequisite is **DEC-FUT-SCHED (§9)**.

> **Why not make the ready-queue pure `.ph`?** Every candidate `.ph` home fails on the landed
> object model: no class variables (unspecified), no mutable module globals reachable from an
> arbitrary `settle` site, and threading a queue explicitly through `async`/`await` breaks the
> spec surface. A native FIFO behind `System.schedule(_)` is the smallest correct home and is
> exactly what `system.md §2` reserved — this is a *finding*, not a preference. ADR-0030 §1
> already sanctions "`Fiber` + a queue"; the queue being native is consistent with it.

---

## 4. Surface: in scope for v1 vs. deferred

Spec names are authoritative (`concurrency.md §2`). `resolve`/`reject` in the task brief map to
`Future.value(_)`/`Future.error(_)` respectively — **keep the spec names**; do not introduce
`resolve`/`reject` aliases in v1 (naming is settled by the spec table).

| Signature | Side | Slice | Notes |
|---|---|---|---|
| `construct value(_)` | class | **A (v1, `.ph`)** | already-`fulfilled` future |
| `construct error(_)` | class | **A (v1, `.ph`)** | already-`rejected` future |
| `isReady` | instance | **A (v1, `.ph`)** | `true` once settled; never suspends |
| `value` | instance | **A (v1, `.ph`)** | `Some(v)` if fulfilled else `None`; never suspends |
| `then(_)` / `map(_)` / `catch(_)` | instance | **A (v1, `.ph`) — settled-only** | on an already-settled receiver, fire synchronously and return a settled future. On a `pending` receiver they **register a continuation** — which can only ever fire in Slice B, so v1 documents "pending continuations require the scheduler (Slice B)". |
| `async(_)` | class | **B — LANDED `06432bd`** | run a `Function` on a fresh fiber; uses `System.schedule` + `Fiber#isDone` |
| `await` | instance | **B — LANDED `06432bd`**, fixed `f479189` | suspend current fiber until settled; root fiber degrades to driving the pump. Branch selected by `Fiber#isRoot` — see E004 |
| pending→settle **drain** (enqueue waiters) | — | **B — LANDED `06432bd`** | over the native ready-queue; skips finished fiber waiters (E004(c)) |

> **Status correction, 2026-07-19.** The three rows above read `DEFERRED` for
> five months after Slice B shipped, as did `concurrency.md` and `core.ph`'s own
> `Future` class comment. Slice B landed 2026-07-14 in `06432bd`; `await` did not
> actually suspend anything until [E004](../../../errors/E004-await-cannot-suspend.md)
> was fixed in `f479189`. DEC-FUT-SCHED's Option 1 ruling below (v1 = Slice A
> only) describes what this *unit* scoped, not what the tree contains.

**Explicitly deferred beyond Slice B (OPEN — do not design in):** `select`/`race`, structured
concurrency / cancellation scopes, scheduler **fairness** guarantees, `System.sleep`/timer &
I/O completion sources (open-questions §15; scheduler-unit.md). `System.sleep` in particular
needs a **native clock + timer** completion source and is squarely `U-SCHED`, not U-FUTURE.

---

## 5. Confirmed write-set

### Slice A (U-FUTURE v1 — the primary deliverable of this unit)

| File | Why | Disjoint from U-COLL? |
|---|---|---|
| `phalcom-core/core/core.ph` | `class Future` — state machine, `value`/`error` constructors, `isReady`, `value`, settled-`then`/`map`/`catch` | U-COLL edits `phalcom-ast/parser.rs` → **disjoint file-wise.** BUT `core.ph` is **single-editor** (standing hazard); re-verify no other `core.ph` holder at activation. |
| `phalcom-core/tests/lang/concurrency/` + `tests/lang/MANIFEST.md` | Slice-A goldens (C-FUT-1 partial, C-FUT-3, C-FUT-8); stage C-FUT-2/4/5/6/7 as `pending/` `#[ignore]` | disjoint |
| `docs/forge/units/U-FUTURE/plan.md` (+ status) | this plan | disjoint |

**No `vm.rs`/`bytecode.rs`/`value.rs`/`primitive/*.rs` edit in Slice A** — settled futures never
touch the fiber machinery. This keeps Slice A's write-set tight and parallelizable against any
non-`core.ph` unit.

### Slice B (only if DEC-FUT-SCHED folds the seam into U-FUTURE; otherwise this is `U-SCHED`'s write-set)

| File | Why |
|---|---|
| — | **Precondition, not this unit's write-set:** `Fiber#isDone`/`error` ship via **[U-FIBER-REFLECT](../U-SCHED-FIBER/U-FIBER-REFLECT/plan.md)** (standalone, unblocked, no scheduler dependency) — Slice B depends on it landing but does not implement it. |
| `phalcom-core/src/primitive/system.rs` + `universe.rs` | `System.schedule(_)` enqueue onto a native FIFO; the root-drive/pump hook |
| `phalcom-core/src/vm.rs` | the runtime-owned ready-queue field + the root-drive entry (**SPINE** — conflicts with any `vm.rs` unit; serialize) |
| `phalcom-core/core/core.ph` | `async`/`await`/suspending-`then` in `.ph` over the seam |

> **Adopted-debt note (assume, do not fix — U-FIBER follow-ons, DEFERRED.md):** Slice B's pump
> must respect two landed U-FIBER quirks: (1) `fiber_abort` has **no root-fiber guard**
> (`primitive/fiber.rs` ~L109) — Future's drivers are always **non-root**, so Future never trips
> it; do not rely on aborting the root. (2) `fiber_resume` refuses `call`/`try` whenever
> `native_reentry_depth != 0` (`primitive/fiber.rs` L144), wider than spec §6 — so the pump's
> `driver.call()`/`.try()` **must run at `native_reentry_depth == 0`** (i.e. not underneath a
> `.each`/native `block_call`). `await`/`async` invoked under a native combinator therefore raise
> `CannotYieldAcrossNativeFrame` (C-FUT-7) — this is the *documented, spec-consistent*
> restriction, not a bug. The remaining three DEFERRED.md U-FIBER items (C-FIB-5 golden gap,
> failure-loop parked-frame retention, `fiber_yield` helper dedup) are inert for Future — assume.

---

## 6. Design decisions

### 6.1 `Future` is a plain `InstanceObject` — no new `Value` arm (ADR-0030 §1)
`value.rs` already has `Instance`; `Future` is an ordinary heap class under `Object`. State is
three private slots: `_state` (a small tag / symbol `pending|fulfilled|rejected`), `_value`
(the settled value or captured `Error`), `_waiters` (a `List`, empty in Slice A). Settle-once:
`settleValue`/`settleError` no-op if `isReady`.

### 6.2 Slice A `then`/`map`/`catch` are settled-synchronous
On an already-settled receiver: `then(g)` → `Future.value(g.call(_value))` (fulfilled) or the
rejection propagated (rejected); `map` fires on the fulfilled path only; `catch` on the rejected
path only. This is total and pure `.ph`. On a `pending` receiver they append to `_waiters` and
return a fresh pending future — **which can only ever fire under Slice B's drain** — so v1's
rustdoc + a `pending/` fixture record that pending continuations are Slice B.

### 6.3 Slice B suspension seam (design intent — realize only after DEC-FUT-SCHED)
`await`: `isReady.ifFalse { _waiters.add(Fiber.current); Fiber.yield(None) }; return _state==rejected ? _value.raise() : _value`.
On resume, **re-read `_state`** (do not trust the delivered yield value). `async(fn)`: allocate a
pending `f`; `driver = Fiber.new { f.settleValue(fn.call()) }`; `System.schedule(driver)`; return
`f`. The pump (root-drive) `.try()`s each ready driver; **`driver.isDone && f.isReady.not` ⇒ the
driver failed before settling ⇒ `f.settleError(<the Error `try` delivered>)`**. This is why Slice
B needs `Fiber#isDone` **and** `Fiber#try` (the delivered-Error capture), and why it needs the
native ready-queue for `System.schedule` (§3). Top-level `await` is legal because the root, when
it `await`s, **pumps** (drives ready work until settled) instead of yielding — the root has no
resumer (`fiber_yield` errors "cannot yield the root fiber", `primitive/fiber.rs` L214–216), so
root-await degrades to a pump. This gives the spec's "top-level await is legal" behaviorally in
`.ph`, **without** the not-retrofittable native `main`-as-scheduler change scheduler-unit.md
warns about — provided DEC-FUT-SCHED lets the native FIFO + `Fiber#isDone` land.

### 6.4 Reject / failure path (rides landed U-CORE-6, no new error type)
A rejected future carries an `Error` in `_value`; `await` re-`raise`s it (`Error#raise`, landed),
unwinding the awaiter under the U-CORE-6 unwind. An `async` driver that raises settles rejected
via the pump (§6.3). `Future` introduces **no error class of its own**.

---

## 7. Build order (small, independently-green slices)

1. **Slice A.0 — settled state machine + `value`/`error`/`isReady`/`value`.** Pure `.ph`; green
   on landed tree. Pins C-FUT-1 (the settled half), C-FUT-3, C-FUT-8.
2. **Slice A.1 — settled `then`/`map`/`catch`.** Pure `.ph`; green. Adds settled-combinator
   goldens; stages pending-continuation fixtures as `pending/`.
3. **— DEC-FUT-SCHED gate (§9) —** everything below is blocked until it is ruled.
4. **Slice B.0 — confirm [U-FIBER-REFLECT](../U-SCHED-FIBER/U-FIBER-REFLECT/plan.md) has landed** (external
   precondition, not built by this unit). It has no DEC-FUT-SCHED dependency — dispatch it any
   time, do not let it wait on this ruling. If it hasn't landed by the time Slice B is picked
   up, dispatch it first.
5. **Slice B.1 — native ready-queue + `System.schedule(_)` + root-drive pump.** SPINE (`vm.rs`);
   serialize.
6. **Slice B.2 — `async`/`await` in `.ph`;** graduate C-FUT-2/5/6.
7. **Slice B.3 — pending→settle drain + suspending `then`/`map`/`catch`;** graduate C-FUT-4/7.

Slices A.0–A.1 are the **entire v1 this unit ships**; B.* are the U-SCHED-gated continuation.

---

## 8. Test strategy — `concurrency` corpus label

| ID | Test | Slice / state |
|---|---|---|
| **C-FUT-1** | `Future.value(v).await == v`; `Future.error(e).await` re-raises `e` | A ships the non-`await` half (`value`/`isReady`/`value`); the `.await` assertions graduate in B |
| **C-FUT-3** | settle-once: a second `settleValue`/`settleError` is ignored | **A (ship)** |
| **C-FUT-8** | `value`/`isReady` never suspend | **A (ship)** |
| C-FUT-2 | `Future.async { … }.await` returns the result after suspending | B (`pending/`, `#[ignore]`) |
| C-FUT-4 | `then`/`map`/`catch` fire on settlement with correct value/error path | A ships settled-only; pending-continuation half is B |
| C-FUT-5 | top-level `await` legal (root-drive pump, §6.3) | B |
| C-FUT-6 | `await` yields to the pump; never blocks the OS thread | B |
| C-FUT-7 | `await` under a native `block_call` raises `CannotYieldAcrossNativeFrame` | B (rides `fiber_resume` L144 guard) |

Invariant/regression: Slice A adds `class Future` to the tower — it must pass
`verify_invariants()` (parallel rule, ADR-0002) and appear in the census; all existing
`concurrency` goldens (the C-FIB-* set) stay byte-identical (Slice A touches only `core.ph`).

---

## 9. BLOCKED-ON-DECISION register

- **DEC-FUT-SCHED — the one surviving, load-bearing, spec-unresolvable decision.** *Does
  U-FUTURE absorb the minimal native scheduler seam (Slice B), or does a ratified & owned
  `U-SCHED` ship it as a prerequisite?* It is unresolvable from spec because
  [scheduler-unit.md](../../../design/experimental/v0.2/scheduler-unit.md) proposes `U-SCHED` as a
  **separate, unowned** unit, and open-questions §15 leaves **fairness** OPEN — the architect
  cannot unilaterally (a) fold an unowned unit's scope in, nor (b) decide v0.2 must ship
  suspending futures. Options:
  - **Option 1 (RECOMMENDED, ADOPTED): U-FUTURE v1 = Slice A only (pure `.ph`, zero native).**
    Ship the scheduler-free `Future` now; async/await wait for a ratified, owned
    [`U-SCHED`](../U-SCHED-FIBER/U-SCHED/plan.md) (native FIFO + root-drive pump —
    `Fiber#isDone`/`error` are no longer part of this gate, see U-FIBER-REFLECT above).
    Smallest correct step; unblocks immediately; no `vm.rs` risk; exactly what
    [specification.md §2](specification.md) foresaw ("the scheduler-free set could ship as a
    thin sub-slice before U-SCHED").
  - **Option 2: fold the minimal seam into U-FUTURE** (`System.schedule` + `Fiber#isDone` +
    native FIFO + `.ph` pump). Delivers async/await now, but takes on `U-SCHED`'s
    not-retrofittable root-drive definition, needs the seam design ratified, and inherits the
    OPEN fairness question. Bigger, `vm.rs`-SPINE, pulls unowned scope onto the critical path.
  - **Architect recommendation:** **Option 1.** Ship Slice A as U-FUTURE; spin
    [`U-SCHED`](../U-SCHED-FIBER/U-SCHED/plan.md) as its own unit for Slice B. Slice A does
    **not** depend on this ruling — it is buildable today regardless — so the decision gates
    only B.*, never A.*.

- **DEC-FUT-CLEANUP (secondary, non-blocking for v1).** `ensure`-on-abandoned-fiber is
  **Proposed, unratified** (`fiber-ensure-and-limits.md`): does a `Future` whose driver is
  abandoned run `ensure`? Proposal says **no** (opt-in `Fiber.finish`). Slice A has no drivers →
  unaffected. Flag before Slice B relies on cleanup semantics; do **not** bake guaranteed-cleanup.

**No new ADR required** for either slice — ADR-0030 §1/§Consequences already sanctions the
library layer and the `Fiber`+queue floor extension; `Fiber#isDone`/`error` and `System.schedule`
are ADR-0019 floor amendments authorized by ADR-0030, needing only a census bump. If Option 2 is
chosen and the ready-queue/root-drive design is materially novel, the `documentation-and-adrs`
skill should draft a short `U-SCHED` ADR recording the FIFO + root-drive + fairness posture.

---

## 10. Must-not-preclude

| Hazard | How this plan clears it |
|---|---|
| **A blocking `await` (single-thread deadlock).** | `await` **yields to the pump** (§6.3), never parks the OS thread; the root-drive actively drains ready work. Precluded by construction. |
| **A second concurrency primitive.** | `Future` adds **no VM mechanism beyond `Fiber` + a queue** (ADR-0030 §1). Slice A adds none; Slice B adds only a FIFO + reflection, not a new primitive. |
| **Boxing out the native `U-SCHED`.** | Slice A's `.ph` state machine + Slice B's `.ph` pump both bottom out in `await` = "register waiter, suspend, resume-on-settle, re-check state". A future native `U-SCHED` replaces the FIFO/root-drive **additively** — the `.ph` `await` contract is unchanged. Do not bake queue *policy* into `Future`'s surface. |
| **Generator-specialized `Fiber` breaking `await`.** | Verified: landed `resumer`/`resume_slot`/`result` are **general** (§2), not generator-specific — `await` suspends through exactly them. |
| **`select`/`race`, cancellation.** | Not built (open-questions §15). Keep `_waiters` a plain list and settle-once so a later `select`/cancellation scope layers on. |
| **Resource exhaustion (Future-heavy DoS).** | Frame-depth `StackOverflow` / per-turn `MemoryError` caps are a post-v0.2 robustness dependency (`fiber-ensure-and-limits.md`); flag, do not silently rely on unbounded recursion/allocation. |

---

## 11. Traceability

| Claim | Source |
|---|---|
| `Future` = pure library over `Fiber` + a queue; no new `Value` arm / VM mechanism | ADR-0030 §1/§Consequences; concurrency.md §2 |
| Surface, state machine, settle-once | concurrency.md §2; specification.md §2–§3 |
| Landed `Fiber` surface + general `resumer`/result-slot seam | `primitive/fiber.rs` L86–237; `heap.rs` L110/160–204; `universe.rs` L487–496 |
| Unified unwind + fiber-floor capture (reject path buildable) | `error.rs` `RuntimeError::Raise`; `fiber_try` L123–125; ADR-0008; ADR-0030 §6 |
| `Fiber#isDone`/`error` not landed → ships via standalone U-FIBER-REFLECT, a Slice B precondition | `universe.rs` L487–496 vs concurrency.md §1; `docs/forge/units/U-SCHED-FIBER/U-FIBER-REFLECT/plan.md` |
| ADR-0031 catch syntax unimplemented → reject via `Fiber#try`+pump | ADR-0031 (Accepted, unbuilt); `universe.rs` L482 (`Error#raise` only) |
| No `.ph` class-side state → ready-queue needs native home | object-model.md / classes.md (no class vars); `core.ph` L293 |
| `System.schedule(_)`/`sleep(_)` reserved seam; `U-SCHED` now ratified & planned | system.md §2/§3; scheduler-unit.md; `docs/forge/units/U-SCHED-FIBER/U-SCHED/plan.md` |
| Still-open: structured concurrency, `select`/`race`, fairness, timers | open-questions.md §15; concurrency.md §3 |
| U-FIBER follow-ons Future assumes (abort-root, depth-guard width) | DEFERRED.md (`fiber_abort` ~L109, `fiber_resume` L144) |
