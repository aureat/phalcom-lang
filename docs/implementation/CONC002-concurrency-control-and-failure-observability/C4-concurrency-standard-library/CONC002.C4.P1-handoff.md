# CONC002.C4.P1 Implementation Handoff & Execution Guide

## 1. Context and Current State

- **Current Revision**: `main` (clean working tree).
- **Prerequisites Completed**:
  - `CONC002.C2` (**VM Suspension, Control Continuations, and Coroutine Semantics**) complete and certified.
  - `CONC002.C3` (**Reactor and External Completion Runtime** — P1 & P2) complete and certified.
- **Active Plan**: [`CONC002.C4.P1-future-state-registrations-and-completion-source.md`](CONC002.C4.P1-future-state-registrations-and-completion-source.md).
- **Status**: **DISPATCHABLE / READY FOR IMPLEMENTATION**.

---

## 2. Architectural Foundation Inherited by C4.P1

1. **C2 Coroutine & Suspension Engine**:
   - Ticketed Fiber park/wake mechanism (`Fiber._$preparePark()`, `_$park(_)`, `_$onComplete(_)`, `_$terminalValue`, `System._$wake(_,_)`).
   - Ready queue FIFO scheduler (`ready_queue: VecDeque<ObjRef>`).
   - Non-inline waiter wake: waking an authorization queues the Fiber for subsequent executor rounds without immediate inline execution.

2. **C3 Reactor & Progress Runtime**:
   - Generational token registry (`ReactorToken(slot, generation)` in `phalcom-core/src/reactor/registry.rs`).
   - Thread-isolated `WorkerPool` and monotonic `TimerQueue`.
   - `Poller` (`mio`-backed) with cross-thread `Waker` and unified blocking wait in `Reactor::idle_wait`.
   - VM settlement helpers: `VM::settle_future` calls canonical `Future#settleValue` / `Future#settleError` methods on the VM thread.

3. **Current `Future<T>` State in Universe (`core/universe/src/concurrency/fiber.ph`)**:
   - Invariant generic `Future<T>`.
   - Public surface: `async`, `map`, `then`, `catch`, `recoverWith`, `flatten`, `await`, `settleValue`, `settleError`.
   - Current debt: Internal string comparisons (`"pending"`, `"fulfilled"`, `"rejected"`), mixed waiter storage (closures vs tuples), lack of detachable subscriptions, lack of producer capability wrapper (`CompletionSource<T>`), and unvalidated `Backoff` policy.

---

## 3. C4.P1 Core Mission & Objectives

Transform `Future<T>` into a precise, typed, maintainable library substrate without altering C2/C3 runtime ownership.

### Target Subsystems & Deliverables:
1. **F1: Typed Future State & Single Settlement Transition**:
   - Replace string-based state with typed representation:
     ```text
     FutureState<T> = Pending | Fulfilled(T) | Rejected(Error)
     ```
     (or `Option<Result<T, Error>>` if private ADT enum support requires it).
   - Single internal settlement transition:
     ```phalcom
     _$trySettle(_ outcome: Result<T, Error>) -> Bool
     ```
   - Settle-once invariant: first settlement installs outcome and detaches registrations; subsequent calls return `false` / no-op.

2. **F2: Explicit Readiness Registration Model**:
   - Replace dynamic tuple/closure sniffing with explicit registration variants:
     ```text
     ReadinessRegistration = ParkedFiber { id, fiber, parkGeneration, state } | Callback { id, callback, state }
     ```
   - Preserve generation checks so stale park generations are harmlessly ignored.

3. **F3: Detachable Subscriptions & Bounded Retention**:
   - Provide package-internal `subscribeReady` returning a detachable handle.
   - Detachment is idempotent and releases callback captures.
   - Bounded retention after repeated attach/detach cycles (no unbounded tombstone leak).

4. **F4: Combinator Integration**:
   - Verify `map`, `then`, `catch`, `recoverWith`, `flatten` integrate seamlessly with the new readiness model.
   - User callbacks execute exclusively under scheduler/executor turns (never inline on settlement).

5. **F5: `CompletionSource<T>`**:
   - Introduce `CompletionSource<T>` with:
     ```phalcom
     class CompletionSource<T> {
       future -> Future<T>
       tryResolve(_ value: T) -> Bool
       tryReject(_ error: Error) -> Bool
     }
     ```
   - Enforces producer-side settlement encapsulation.

6. **F6: Pure `Backoff` Policy Validation**:
   - Validate `none`, `fixed(ms >= 0)`, `exponential(base >= 0, max >= base)`.
   - Pure overflow-safe calculation without host sleeping (`delay = min(max, base * 2^retryIndex)`).

7. **Non-Goals in C4.P1**:
   - No reactor/timer changes (owned by C3).
   - No cancellation / `TaskScope` (owned by C5).
   - No channels / select (owned by C6).
   - No Tracer / OffBehavior redesign (separate decorator / observability plans).

---

## 4. Immediate Starting Steps for C4.P1

1. **Verify Clean Baseline**:
   ```sh
   git status --short
   RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib reactor
   RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency
   ```

2. **Source Inspection & Rebase (F0)**:
   - Review `core/universe/src/concurrency/fiber.ph` (`Future<T>`, `Backoff`).
   - Check `phalcom-core/src/vm/mod.rs` (`VM::settle_future`, `VM::create_pending_future`).

3. **Execution Sequence**:
   - Step 1 (F1): Refactor `Future` state representation to typed `Result`/ADT and implement `_$trySettle`.
   - Step 2 (F2/F3): Implement explicit readiness registration and detachable subscriptions.
   - Step 3 (F4): Re-verify combinators (`map`, `then`, `catch`, `recoverWith`, `flatten`).
   - Step 4 (F5): Implement `CompletionSource<T>`.
   - Step 5 (F6): Implement and validate pure `Backoff` calculation.
   - Step 6 (F7): Run full semantic, GC, corpus, and workspace verification gates.
