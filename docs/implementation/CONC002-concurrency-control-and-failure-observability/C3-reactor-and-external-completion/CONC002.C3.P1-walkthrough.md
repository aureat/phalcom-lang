# CONC002.C3.P1 Walkthrough — Reactor Core, Workers, Timers, and Executor Liveness

This document records the architectural changes, implementation details, defect resolutions, and verification evidence for **`CONC002.C3.P1`** (*Reactor Core, Workers, Timers, and Executor Liveness*).

---

## 1. Executive Summary

- **Checkpoint**: `CONC002.C3` (*Reactor and External Completion Runtime*)
- **Plan**: `CONC002.C3.P1` (*Reactor Core, Workers, Timers, and Executor Liveness*)
- **Status**: **COMPLETE / IMPLEMENTED**
- **Objective**: Implement the phase-1 reactor runtime integrated with the post-C2 VM-owned executor, introducing a thread-safe generational registration table, a plain-data background worker pool, a monotonic timer priority queue, safepoint ingress, executor idle wait/liveness, and the public `System.sleep(Int) -> Future<Unit>` primitive.

---

## 2. Key Architecture & Implementation Changes

### 2.1 Generational Reactor Token Registry (`phalcom-core/src/reactor/registry.rs`)
Created the core data structures for generational reactor registrations:
- **`ReactorToken`**: `(slot: u32, generation: u32)` identifying an allocated registration. Distinct from Fiber park generations and scheduler admission tokens.
- **`ReactorSource`**: Identifies registration origins (`Timer`, `Worker`).
- **`RegistrationState`**: Lifecycle states (`Active`, `Completed`, `Cancelled`).
- **`ReactorRegistration`**: Holds target `ObjRef` (e.g. `Future`), state, generation, and source.
- **`ReactorRegistry`**: Generational slot table with O(1) lookups, free-list recycling, settle-once semantics, stale completion discard, and active registration target GC root tracing (`trace_roots`).

### 2.2 Thread-Isolated Worker Pool & Plain-Data Transport (`phalcom-core/src/reactor/worker.rs`)
- **Law of Thread Isolation**: Background workers operate exclusively on owned plain Rust structs (`Send + 'static`). Conversion to/from Phalcom `Value` or `ObjRef` happens exclusively on the main VM thread. No worker thread accesses `Value`, `ObjRef`, `Heap`, or `VM`.
- **`WorkerJob` & `WorkerCompletion`**: Owned plain-data payloads (`WorkerOutcome`, `WorkerResult`). Verified via compile-time static assertions (`_assert_send_static`).
- **`WorkerPool`**: Bounded worker thread pool (default 4 threads) receiving jobs via synchronized channel and returning completions via MPSC channel to the VM.

### 2.3 Monotonic Timer Priority Queue (`phalcom-core/src/reactor/timer.rs`)
- **`TimerEntry`**: `(deadline: Instant, sequence: u64, token: ReactorToken)` implementing min-heap ordering with sequence tie-breaking for deterministic ordering of identical deadlines.
- **`TimerQueue`**: Priority queue polled at safepoints and bounding executor idle wait, operating without dedicated timer threads.

### 2.4 Reactor Coordinator & VM Settlement (`phalcom-core/src/reactor/mod.rs`, `phalcom-core/src/vm/mod.rs`)
- **`Reactor`**: Coordinates registry, worker pool, timer queue, and completion channels.
- **VM Helpers**:
  - `VM::create_pending_future`: Creates a new pending `Future` instance by invoking `@constructor new()`.
  - `VM::settle_future`: Materializes plain `WorkerOutcome` into a Phalcom `Value` and invokes `Future#settleValue` or `Future#settleError` via VM dynamic send.
  - `VM::drain_and_deliver_reactor_completions`: Drains due timers and worker completions up to batch bounds and settles their target futures on the VM thread.
  - `VM::reactor_idle_wait`: Bounded sleep until the next timer deadline or incoming worker completion.

### 2.5 Dispatch Loop & Safepoint Ingress (`phalcom-core/src/vm/dispatch.rs`, `gc.rs`)
- **`VM::run_until`**: Integrated reactor completion drain and idle wait at `base_frames == 0`. When `ready_queue` is empty and pending reactor registrations/timers exist, the executor enters idle wait rather than declaring quiescence.
- **`VM::service_gc_safepoint`**: Non-blockingly polls cross-thread ingress into internal buffers without allocating heap objects.
- **`VM::collect_roots`**: Exhaustively traces active reactor registration targets.

### 2.6 Public Async Delay Surface (`System.sleep`)
- Declared `@class @native sleep(_ milliseconds: Int) -> Future<Unit>` in `core/universe/src/concurrency/fiber.ph`.
- Implemented native `system_sleep` in `phalcom-core/src/primitive/system.rs`.
- Validated non-negative duration checking, 0ms deferred next-turn completion, and root / child fiber awaiting.

### 2.7 Leak Diagnostics (`phalcom-core/src/primitive/resource.rs`)
- Integrated unclosed/active reactor registrations into `System._$leakReport`.

---

## 3. Defect Resolutions & Hardening

1. **Root Fiber `Future.await` Settlement on Empty Queue**:
   - In `fiber.ph`, when `System._$nextScheduled` drains the timer/worker completion, the future settles directly without queuing a child fiber.
   - Updated `Future.await` to check `if (self.isReady) { break }` when `next.isNone`, preventing premature empty-scheduler error raises when the future settled during the poll turn.

2. **Clippy & Code Formatting**:
   - Used `std::mem::take` for draining buffered worker completions.
   - Formatted all files with `cargo fmt --all` to meet workspace CI standards.

---

## 4. Verification Evidence

All test gates were executed serially with clean compiler flags (`RUSTFLAGS='' RUSTC_WRAPPER=''`):

| Gate / Suite | Command | Result | Notes |
|---|---|---|---|
| **Reactor Unit Tests** | `cargo test -p phalcom-core --lib reactor` | **PASS** | 5 passed (registry, GC roots, timer ordering, worker round trip, shutdown & leaks) |
| **Core Lib Tests** | `cargo test -p phalcom-core --lib` | **PASS** | 117 passed, 0 failed |
| **Concurrency Positive Corpus** | `cargo test -p phalcom-core --test language-corpus corpus::concurrency` | **PASS** | 88 passed, 0 failed (includes sleep liveness, sleep zero, child fiber) |
| **Concurrency Negative Corpus** | `cargo test -p phalcom-core --test language-corpus corpus::concurrency_negative` | **PASS** | 14/14 passed (includes negative sleep error) |
| **Memory GC Suite** | `cargo test -p phalcom-core --test core memory_gc` | **PASS** | 18 passed, 0 failed |
| **Native Surface Contracts** | `cargo test -p phalcom-core --test core native_surface_contracts` | **PASS** | 3 passed, 0 failed |
| **Code Formatting** | `cargo fmt --all -- --check` | **PASS** | Clean across workspace |
| **Workspace Clippy** | `cargo clippy --workspace --all-targets -- -D warnings` | **PASS** | 0 warnings |

---

## 5. Next Steps

- Proceed to **`CONC002.C3.P2`** (*Poller-Backed External Readiness*) upon resolution and acceptance of PDR-0016.
