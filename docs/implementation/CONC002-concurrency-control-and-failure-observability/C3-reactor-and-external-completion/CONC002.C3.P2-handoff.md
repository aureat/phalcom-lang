# CONC002.C3.P2 Implementation Handoff & Execution Guide

## 1. Context and Current State

- **Current Revision**: `main` (clean working tree).
- **Prerequisites Completed**:
  - `CONC002.C2` (**VM Suspension, Control Continuations, and Coroutine Semantics**) complete.
  - `CONC002.C3.P1` (**Reactor Core, Workers, Timers, and Executor Liveness**) complete and verified across all test gates.
- **Active Plan**: `CONC002.C3.P2-poller-and-external-readiness.md`.
- **Status**: **COMPLETE and VERIFIED**.

---

## 2. P1 Accomplishments Inherited by P2

1. **Generational Token Registry**:
   - `ReactorToken(slot, generation)` in `phalcom-core/src/reactor/registry.rs` provides generational slot allocation, O(1) lookup, stale completion discard, and active target GC root tracing.
   - Distinct from Fiber park generations and scheduler admission tokens.
2. **Thread-Isolated Worker Pool**:
   - `WorkerPool` in `phalcom-core/src/reactor/worker.rs` executes background jobs transferring only owned `Send + 'static` plain Rust structs (`WorkerJob`, `WorkerCompletion`, `WorkerOutcome`, `WorkerResult`).
   - Conversion to/from Phalcom `Value`/`ObjRef` occurs exclusively on the main VM thread.
3. **Monotonic Timers & Public Sleep**:
   - `TimerQueue` in `phalcom-core/src/reactor/timer.rs` manages monotonic deadlines (`std::time::Instant`) with sequence tie-breaking.
   - `System.sleep(Int) -> Future<Unit>` is active, tested, and integrated with executor idle wait.
4. **VM-Thread Settlement & Safepoint Ingress**:
   - `VM::drain_and_deliver_reactor_completions` and `VM::settle_future` invoke canonical `Future#settleValue` / `Future#settleError` methods, waking ticketed waiters into `ready_queue` without inline execution.
   - Non-blocking safepoint ingress polls cross-thread completions into internal buffers.
5. **Executor Liveness & Idle Wait**:
   - `VM::run_until` enters bounded idle wait when `ready_queue` is empty but reactor progress is pending.

---

## 3. P2 Objectives and Core Mission

P2 completes the reactor for descriptor readiness without introducing a second executor or exposing poller internals to guest code.

### Key Architectural Invariants to Realize in P2:
1. **Decision Gate (PDR-0016)**:
   - Verify PDR-0016 status before introducing `mio` dependencies or backend-specific code.
   - If accepted unchanged, use `mio` (`features = ["os-poll", "net"]`) strictly confined to `phalcom-core/src/reactor/`.
2. **Unified Wait**:
   - Replace phase-1 channel timeout wait with a unified descriptor/timer polling wait: `poll(timeout = min(next_timer_deadline, executor_cap))`.
3. **Cross-Source Wake**:
   - Integrate a thread-safe `Waker` so background worker completions awaken the poller immediately.
4. **Poller Registration Mapping**:
   - Map poller tokens into C3 `ReactorRegistry` slot/generation records.
   - Ensure stale or deregistered descriptor readiness events are harmless no-ops.
5. **Thread Discipline**:
   - Readiness event processing and descriptor completion settlement occur exclusively on the main VM thread.

---

## 4. Immediate Starting Steps for P2

1. **Verify Baseline State**:
   ```sh
   git status --short
   RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib reactor
   RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency
   ```
2. **Check Hard Decision Gate**:
   - Inspect `docs/decisions/0016-poller-backend-and-external-readiness.md` (or relevant PDR).
   - If accepted, proceed to Phase 1 backend integration.
   - If still proposed, maintain `BLOCKED` status on `CONC002.C3.P2`.
3. **Task Breakdown**:
   - Phase 1: Poller backend integration and `Waker` cross-thread wake.
   - Phase 2: Unified blocking wait integration in `Reactor::idle_wait`.
   - Phase 3: Descriptor registration lifecycle and hostile unit tests.
   - Phase 4: Full workspace verification and checkpoint certification.
