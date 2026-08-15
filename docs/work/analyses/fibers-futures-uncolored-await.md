# Fibers, futures, uncolored await, and Wren-style control transfer

**Status: ANALYSIS — conversation conclusions recorded 2026-07-21.** This file records design direction, costs, hazards, and proposed decision boundaries. It does not amend a specification or ratify a new API. Normative surface changes need a PDR or ADR.

**Baseline:** Phalcom aa29e89cfc54f19d307ec862615e80dc89258920 (2026-07-21).
**External comparison:** vendored Wren source snapshot under [resources/wren/src/vm](../../../resources/wren/src/vm).

Claims tagged **[V]** were verified by opening the cited repository artifact in this session. **[R]** is recalled precedent, not reopened externally. **[O]** is a proposed or still-unratified design position. This follows the repository's [citation discipline](../../theory/00-provenance-and-citation-discipline.md).

Within a subsection, subsequent bullets and table rows inherit the immediately preceding warrant unless they declare a different one.

Related: [Fibers & Futures](../../spec/current/concurrency.md), [ADR-0030](../../adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md), [PDR-0017](../../pdr/0017-future-cancel-is-renunciation.md), and [Wren yield analysis](wren-vs-phalcom-fiber-yield.md).

---

## 1. Executive conclusion

**[O]** Keep fibers and await together. Phalcom should have one suspendable execution unit — Fiber — while Future represents an eventual outcome and its waiters. Await is direct-style suspension over that protocol; then/map/catch are continuation style over it. No second execution engine is justified.

    Fiber      = execution, stack, frame ownership, and suspension
    Scheduler  = readiness, queueing, and external-completion progress
    Future     = settle-once outcome plus waiters
    await      = register current Fiber, park it, later resume it on settlement

**[V]** This extends, rather than contradicts, accepted design: cooperative single-threaded Fiber is sole concurrency primitive; future, async/await, generators, and scheduler derive from it. Future adds no VM mechanism beyond fibers and a ready queue. See [ADR-0030 §1](../../adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md#1-fiber-is-the-sole-concurrency-primitive) and [Consequences](../../adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md#consequences).

| Position | Status in this analysis | Consequence |
|---|---|---|
| No function coloring | Locked user direction | No async function, suspend function, alternate callable kind, or async-only callback universe. |
| No async designation | Locked user direction | Future.async remains an operation starting work, not a declaration changing a method's type. |
| @suspending | **[O]** recommendation | Documentation/API-contract attribute only; no dispatch, scheduler, type, or runtime effect. |
| Future is outcome, not owner | **[O]** recommendation | Cancellation is renunciation; task lifetime belongs in a later scope abstraction. |
| Common callback operations move to .ph | **[O]** direction | More paths may be suspension-transparent, but native-frame guard remains mandatory. |
| CannotYieldAcrossNativeFrame remains | Locked by ADR-0030 | Unsafe control transfer remains a catchable error, never an implementation accident. |

---

## 2. Why uncolored await fits Phalcom

**[V]** Current surface already has the desired shape:

    let f = Future.async { slowComputation() }
    doOtherWork()
    let value = f.await

Future.async starts a fresh fiber and returns a future; Future.await suspends current fiber until settlement, returns a value, or re-raises stored error. [Fibers & Futures §2](../../spec/current/concurrency.md#2-future--pending-asynchronous-result) states that await and continuation APIs bottom out in the same waiter list.

**[V]** A normal Phalcom method or block remains a normal callable. There is no compiler-lowered future value for every caller, no colored signature, and no separate async iterator or async callback protocol implied by the surface.

**[R]** Rust, JavaScript, Kotlin, and C# choose explicit async or suspend functions. That makes suspension statically visible but colors higher-order APIs. Go, BEAM, and Loom make parking a runtime concern instead, avoiding that split at the cost of a scheduler. Phalcom's fiber-backed await belongs to the second family.

| Choice | Benefit | Cost |
|---|---|---|
| Uncolored await | One callable universe; no duplicate standard library; message syntax stays uniform. | A callee may suspend internally, so suspension is an ambient runtime effect. |
| @suspending documentation | Callers see a conservative API warning without creating types/colors. | Dynamic dispatch prevents complete automatic inference or enforcement. |
| Colored declarations | Static visibility and possible compiler checks. | Infects callbacks, iterators, collections, reflection, and public method boundaries. Rejected by direction. |

### @suspending contract

**[O]** @suspending should mean “may suspend this fiber, directly or through a call it makes.” It is conservative: absence does not prove a method cannot suspend until Phalcom has effect analysis, which it does not.

**[O]** It must be metadata only:

- no alternate selector or method identity;
- no alternate invocation or closure representation;
- no implicit scheduler call;
- no inheritance or override enforcement rule yet;
- no promise that the method will actually suspend on a particular execution.

This makes the attribute useful documentation without smuggling function coloring back through attributes.

---

## 3. Real cost: logical reentrancy

**[V]** Cooperative single-threading removes data races, not interleaving. A fiber switch occurs at explicit call, yield, or await points; another fiber may run before the first resumes. [Fibers & Futures §1](../../spec/current/concurrency.md#1-fiber--cooperative-coroutine) documents this model.

    account.debit(amount)
    receipt.await
    account.recordTransfer()

**[O]** Another fiber can observe the account after debit and before recordTransfer. This is not a race or torn write; it is a deliberate logical interleaving point. Public APIs that may cross such a point should carry @suspending once that convention exists.

**[O]** Do not add general atomic blocks, locks, or preemption in response. First response is API design: preserve object invariants before awaiting, or represent multi-step work explicitly. General transactions and locking would overturn the cooperative single-mutator model without solving arbitrary I/O semantics.

---

## 4. Future: outcome without public ownership

### Meaning

**[O]** A Future is a settle-once outcome and waiter list. It is not a task parent, a child supervisor, a cancellation scope, or proof that producer work should still run.

This does not let engine discard work prematurely. Scheduler, reactor registration, running fiber, and future-to-producer relationship retain whatever reachability is necessary to deliver a result safely. The boundary is API/lifecycle: a future handle does not own a task tree.

**[V]** Existing future structure distinguishes state, waiter list, and — for async futures — driving fiber. It settles once and uses fibers plus ready queue as substrate. [Fibers & Futures §2](../../spec/current/concurrency.md#2-future--pending-asynchronous-result)

| Topic | Consequence |
|---|---|
| Future.async | Starts detached, scheduler-driven work and returns outcome handle. |
| Failure | Driver failure settles future rejected; it must not implicitly raise into unrelated fibers. |
| Unobserved failure | **[O]** Needs an explicit supervisor/diagnostic policy later; never silent disappearance. |
| GC/liveness | Parked fiber stacks remain roots while reachable; future/reaction registration must not lose a waiter. |
| Structured lifetime | Requires a separate owner such as future TaskGroup/nursery. It cannot be inferred from Future. |

**[V]** ADR-0030 requires value and frame stacks of reachable parked fibers to be GC roots, not merely current fiber’s stack. [ADR-0030 §7](../../adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md#7-fibers-are-gc-roots-even-when-parked)

### Future structured concurrency

**[O]** If Phalcom later needs task ownership, add a scope abstraction, not ownership behavior to Future:

    TaskGroup.open { group =>
      group.spawn { fetchUsers() }
      group.spawn { fetchOrders() }
    }

Scope, not an arbitrary future, owns children, determines join behavior, and requests cooperative cancellation. This leaves ordinary future shareable by many consumers without inventing ambiguous owner.

---

## 5. Scheduler implications

### One scheduler, distinct parking reasons

**[O]** Adopt FIFO readiness as baseline:

1. scheduling appends to tail;
2. a turn runs until voluntary yield, await, return, or raise;
3. future settlement appends ready waiters/continuations;
4. no preemption and no hidden resumption;
5. CPU-bound or blocking native operation can still starve peers.

**[V]** Current scheduler seam is FIFO VecDeque; it queues fresh fibers and deliberately does not requeue later-suspended fiber. Future settlement owns enqueue-on-ready. See [U-SCHED §2](../pending/scheduling/queue/implementation-spec.md#2-vm-field) and [§5](../pending/scheduling/queue/implementation-spec.md#5-root-drive-vmrun).

Runtime must distinguish these relationships even if public status remains Suspended:

| Park reason | Who may make it runnable? |
|---|---|
| Fiber.yield | Dynamic resumer via call/try. |
| Future.await | Future settlement. |
| System.schedule | Scheduler queue, first run only. |
| External I/O/timer | Reactor completion, then future settlement. |

**[O]** Distinguishing these prevents scheduler from resuming generator whose caller has not asked for next value, and prevents caller from resuming fiber currently parked on a future.

### Root await

**[V]** Root fiber cannot yield because it has no resumer; current design drives ready queue and rechecks awaited future. [Fibers & Futures §2](../../spec/current/concurrency.md#2-future--pending-asynchronous-result)

**[O]** Root await should run one ready turn, poll completions, then recheck awaited future. It must not blindly drain unbounded queue before rechecking; unrelated self-scheduled work must not delay a future already ready. One non-yielding fiber remains able to monopolize cooperative scheduler.

### Fairness

**[O]** FIFO gives deterministic, adequate v0.x behavior. Strong fairness, work stealing, preemption, and multi-thread migration remain outside current model; they add safepoints, memory model, and synchronization obligations the single-mutator design rejects.

---

## 6. Cancellation: renunciation, not interruption

**[V]** PDR-0017 proposes appropriate shallow surface:

    future.cancel()       // Bool: this call won pending -> cancelled
    future.isCancelled

Its contract:

- cancel immediately settles future rejected with CancelledError;
- waiters resume through normal settlement;
- reactor registration releases;
- queued, unstarted work may be suppressed best-effort;
- already-started work is never forcibly interrupted and may still have effects.

See [PDR-0017 §1–4](../../pdr/0017-future-cancel-is-renunciation.md#decision).

**[O]** Adopt that meaning when PDR-0017 is ratified. It matches “future is outcome, not owner”: cancel means stop waiting for result, never undo world or kill named fiber.

| Operation | Meaning | Must not mean |
|---|---|---|
| Future.cancel | Consumer renounces outcome; waiters receive cancellation. | Interrupt, undo, parent/child propagation, target-fiber kill. |
| Fiber.abort(error) | Current fiber raises/unwinds to resumer. | Cancellation of another fiber. |
| Future TaskGroup.cancel | **[O]** Scope requests cooperative stop from owned children. | Retroactive promise to interrupt FFI or worker syscalls. |
| Resource close | Resource disappeared; caller may choose recovery/retry. | Consumer renunciation. |

**[V]** PDR-0017 separates cancelled from closed, keeps cancellation shallow, and defers propagation and fiber cancellation. [PDR-0017 §5–6](../../pdr/0017-future-cancel-is-renunciation.md#5-composition-with-leak-reporting-and-with-closed)

**[O]** Cancel returning Bool cannot later mean “interrupt pending.” Richer scope/token policy must layer above shallow cancel. Effectful APIs must say cancellation may arrive after effect occurred.

---

## 7. Native-frame guard and suspension-transparent libraries

**[V]** Phalcom has restricted re-entrant loop. Yield and resume across live native re-entrant frame raise CannotYieldAcrossNativeFrame, avoiding corrupted suspended native stack. [ADR-0030 §4](../../adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md#4-execution-model--restricted-option-a)

**[V]** This is not a missing Yield opcode. Wren and Phalcom both perform basic handoff in primitive; Phalcom’s stronger native re-entry guard is difference. Recorded Wren study shows that making each { Fiber.yield(_) } work requires flattening/trampolining callback path, not adding an opcode. [Wren comparison](wren-vs-phalcom-fiber-yield.md)

**[O]** Reimplement commonly used callback combinators in .ph where that preserves normal non-re-entrant interpreter path. This can make collection and stream operations suspension-transparent while retaining guard for native boundaries.

**[O]** Use separate documentation contract for higher-order APIs:

    @suspensionTransparent
    each(action) {
      for (item in self) { action.call(item) }
    }

@suspending says operation may park current fiber. @suspensionTransparent says supplied callback may park its fiber. They are distinct claims and need separate golden fixtures.

**[O]** Source language alone does not prove suspension transparency. The complete call path must avoid forbidden native re-entry. This is a documented/tested promise, not static analysis.

---

## 8. Fiber operations: retain, defer, or reject

| Surface | Position | Reason / trade-off |
|---|---|---|
| Fiber.new, call, try, yield | Keep | Asymmetric coroutine protocol. Call installs dynamic resumer; yield/return/failure return through it. Simple control tree. |
| Fiber.current | Keep | Pure reflection. Useful for diagnostics, identity, reentrancy mechanisms, scheduler-aware libraries. |
| isDone, error, isRoot | Keep | Pure status reflection; prevents forbidden-yield probes. |
| Fiber.abort(error) | Keep as current-fiber self-abort | Existing terminating error unwind. Root abort remains illegal. |
| Fiber.transfer | Defer; do not add to core v0.x | Symmetric transfer creates control-flow graph, conflicts with scheduler ownership and structured concurrency. |
| Fiber.suspend | Do not expose as language operation | Wren operation exits to embedding host, not scheduler parking. |

### Fiber.current

**[V]** Fiber.current reads VM current fiber and does not switch, schedule, or inspect native stack. [fiber primitive](../../../phalcom-core/src/primitive/fiber.rs#L217-L220)

**[O]** Keep it class-side and reflective. Do not attach ownership, cancellation, or implicit current-task semantics.

### Fiber.abort(error)

**[V]** Current Phalcom abort raises out of current fiber at fiber floor; it propagates to resumer and rejects root abort because root has no resumer. [fiber primitive](../../../phalcom-core/src/primitive/fiber.rs#L291-L308)

**[O]** Preserve it as error/control-flow. Never grow target-fiber abort API: target cancellation must specify ensure ordering, parked-future deregistration, waiters, resource cleanup, and scheduler behavior.

### Why not Fiber.transfer

**[V]** Wren transfer switches to target without installing current fiber as caller; call does install caller. A transferred-to Wren fiber has no implied route back to source. [Wren source](../../../resources/wren/src/vm/wren_core.c#L79-L138)

**[O]** Cost in Phalcom is larger than primitive:

1. source becomes parked without dynamic resumer result slot;
2. target return, failure, yield, and await need explicit destination;
3. scheduler-owned work can escape into manual transfer;
4. traceback parentage becomes non-tree;
5. TaskGroup can no longer explain ownership.

Capability is expressible only after defining all five semantics. It is not needed for generators, futures, streams, or direct-style await. Defer until concrete feature proves call/yield plus queueing insufficient.

### Why not Fiber.suspend

**[V]** Wren Fiber.suspend sets current VM fiber to null so interpreter exits to host API. [Wren source](../../../resources/wren/src/vm/wren_core.c#L166-L172)

**[O]** That is embedder escape hatch, not language suspension. Exposing it would leave runnable/parked distinction outside scheduler bookkeeping and weaken liveness reasoning. REPL/embedding pause belongs in Rust host control, not Fiber API.

---

## 9. Error, cleanup, non-local return

**[V]** Non-local return is fiber-local. Return whose home frame is on another fiber raises DeadFrameError; errors cross boundaries only through call/await propagation or try/catch capture. [Fibers & Futures §3](../../spec/current/concurrency.md#3-relationship-to-the-rest-of-the-model)

**[O]** Any TaskGroup or fiber-cancellation design must decide before implementation:

- whether child failure cancels siblings or reports at join;
- where cancellation is observed: await, yield, or explicit checkpoints only;
- ensure/cleanup ordering before terminal status;
- removal of parked waiters and reactor registrations;
- cancelled-fiber error and traceback presentation;
- whether cancellation is distinguished Error or scope signal.

Until those rules exist, target-fiber kill is not harmless convenience API.

---

## 10. Representation, GC, performance

**[V]** Fiber state is heap object with value stack and call frames; switch changes fiber VM reads rather than copying stack. [ADR-0030 §2–3](../../adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md#2-fiber-is-a-heap-object-not-a-new-value-arm)

| Choice | Benefit | Cost / constraint |
|---|---|---|
| Heap-held fiber frames | O(1) handoff; parked stacks visible to GC. | VM must preserve parked stacks as roots. |
| Restricted native-frame guard | No unsafe native-stack switch; keeps collector options open. | Some callbacks cannot suspend until paths flatten. |
| .ph suspension-transparent combinators | Ergonomic generators/await in normal library use. | More interpreter/message-send overhead; benchmark needed before hot-path migration. |
| FIFO cooperative scheduler | Deterministic, tiny, no locks. | Runaway fiber starves peers. |
| No transfer | Stack/caller invariants stay simple. | No arbitrary symmetric coroutine graph. |

**[V]** ADR-0030 rejects stackful native coroutines because native stacks constrain future collector and require unsafe stack switching. [ADR-0030 alternatives](../../adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md#alternatives-considered)

---

## 11. Recommended decision order

1. **[O]** Record no function coloring/no async designation in PDR when conversation ruling becomes normative.
2. **[O]** Define @suspending and @suspensionTransparent as documentation attributes only, including scope, override convention, and golden-fixture expectations.
3. **[O]** Define FIFO scheduler and root-await recheck behavior as scheduler policy record; fairness stays explicitly weak/cooperative.
4. **[O]** Ratify or revise PDR-0017 before shipping reactor-facing future cancellation. Keep it shallow.
5. **[O]** Move selected public combinators into .ph only where fixtures prove suspension transparency; preserve native-frame guard elsewhere.
6. **[O]** Design TaskGroup/nursery only when concrete consumer needs child lifetime, join, or propagation. It must not turn Future into owner.
7. **[O]** Revisit transfer only with concrete use case and full answer for return destination, failure, await, cleanup, scheduler ownership, and traceback parentage.

---

## 12. What this direction forecloses

**[O]** Direction rejects or defers:

- colored async/suspend function types and duplicate async APIs;
- hidden compiler-generated coroutine state machines as language model;
- preemptive or multi-threaded fibers in current object model;
- implicit cancellation on GC/drop/future-handle loss;
- interruptive cancellation or claims cancel undoes an effect;
- public arbitrary Fiber.transfer before ownership/unwind semantics exist;
- language-level Fiber.suspend as embedding-control escape hatch;
- removing CannotYieldAcrossNativeFrame merely to make familiar callback shape work.

These are intentional exclusions, not implementation gaps. Model still admits later full-trampoline callback suspension, structured concurrency, race/select, reactor-driven I/O, and richer diagnostics — each as additive design with its own semantics.

---

## 13. Provenance ledger

**Opened first-hand this session ([V]):**

- [docs/spec/current/concurrency.md](../../spec/current/concurrency.md)
- [ADR-0030](../../adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md)
- [PDR-0017](../../pdr/0017-future-cancel-is-renunciation.md)
- [cancellation specification](../../spec/current/stdlib/cancellation.md)
- [U-SCHED implementation spec](../pending/scheduling/queue/implementation-spec.md)
- [Wren vs Phalcom analysis](wren-vs-phalcom-fiber-yield.md)
- [phalcom-core/src/primitive/fiber.rs](../../../phalcom-core/src/primitive/fiber.rs)
- vendored [wren_core.c](../../../resources/wren/src/vm/wren_core.c) and [wren_core.wren](../../../resources/wren/src/vm/wren_core.wren)

**Conversation-derived ([O] until decision record is accepted):** no function coloring; no async designation; Future as outcome rather than owner; preserving guard while expanding suspension-transparent library paths; recommending @suspending and @suspensionTransparent as documentation-only attributes; deferring transfer and language-level suspend.

**Not verified externally ([R]):** language precedent summaries for Go, BEAM, Loom, Rust, JavaScript, Kotlin, and C#. They are explanatory only and carry no normative weight.
