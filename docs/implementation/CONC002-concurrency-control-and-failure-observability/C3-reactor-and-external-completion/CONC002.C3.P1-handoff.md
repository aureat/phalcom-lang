# CONC002.C3.P1 — Handoff: Reactor core, workers, timers, and executor liveness

## 1. Executive Summary & Predecessor State

- **Predecessor Program**: `CONC002.C2` (*VM suspension, control continuations, and coroutine semantics*) is complete.
  - `CONC002.C2.P1-R1`: VM-owned `ControlStack` and `ControlActivation` records, suspension-safe `on` / `ensure` / `whileTrue` / `Bool` / `Option.match` control combinators, inside-out `Transfer` routing, and Call-mode parent call-site exception injection.
  - `CONC002.C2.P2`: Coroutine consumer / executor separation (`FiberConsumer { fiber: ObjRef, mode: FiberConsumerMode }`), VM-owned executor driving, manual pending await (`yield -> await -> yield`), quiescence detection on empty queues, and strict scheduler admission for fresh work only.
- **Verification Baseline**:
  - `cargo test -p phalcom-core --lib`: 112 passed, 0 failed.
  - `cargo test -p phalcom-core --test language-corpus corpus::concurrency`: 85 passed, 0 failed.
  - `cargo test -p phalcom-core --test language-corpus corpus::concurrency_negative`: 13 passed, 0 failed.
  - `cargo test -p phalcom-core --test language-corpus corpus::streams`: passed.
  - `cargo fmt --all -- --check`: passed.
  - `cargo clippy --workspace --all-targets -- -D warnings`: passed (0 warnings).
  - Clean-`HEAD` baseline blocker: `phalcom-repl --test repl_imports` reflection-export failures exist on clean remote `HEAD` and are documented in `CONC002.C2-CHECKPOINT.md`.

---

## 2. Mission & Ownership Boundary for CONC002.C3.P1

`CONC002.C3.P1` implements the phase-1 reactor on top of the VM-owned executor:
1. **Registration Registry**: Generational, thread-safe token registration (`RegistrationToken(u64)`) mapped to Phalcom completion targets (`ObjRef` / `Future`).
2. **Plain-Data Worker Pool**: Off-thread background execution for blocking tasks (file I/O, compute) transferring owned plain Rust data only (`Vec<u8>`, numeric primitives, strings). **No worker thread ever touches `Value`, `ObjRef`, `Heap`, or `VM`**.
3. **Plain-Data Completion Ingress**: Thread-safe channel (`crossbeam_channel` / `mpsc`) drained by the VM thread at safepoints / executor pump turns.
4. **Monotonic Timers**: Priority-queue / wheel of timer registrations driven by a monotonic clock (`std::time::Instant`), integrated into executor idle-wait without dedicated timer threads.
5. **Executor Liveness & Idle Wait**: When the ready queue is empty, the executor checks if reactor registrations or timers are active; if so, it sleeps until the next timer deadline or completion event, instead of declaring quiescence.
6. **`System.sleep(Int) -> Future<Unit>`**: Public Universe surface for async delays.
7. **Resource / Worker Shutdown & Leak Reporting**: Clean shutdown of background worker pools upon VM exit.

---

## 3. Post-C2 Structural Anchors for C3 Integration

### 3.1 VM Fields & Ownership
In `phalcom-core/src/vm/mod.rs`:
- `pub(crate) current: ObjRef`: The active executing fiber.
- `pub(crate) root_fiber: ObjRef`: The root fiber handle initialized in `bootstrap.rs`.
- `pub(crate) ready_queue: VecDeque<ObjRef>`: FIFO queue of runnable fibers in `FiberStatus::Queued` or `FiberStatus::New`.
- `pub(crate) scheduler_drivers: Vec<ObjRef>`: Stack of blocked driver fibers (e.g. `System.runScheduled` / root pump).
- `pub(crate) unhandled_scheduler_failures: VecDeque<UnhandledSchedulerFailure>`: E010 detached failure queue.

### 3.2 State Transitions & Fiber Model
In `phalcom-core/src/heap/fiber.rs`:
- `FiberStatus`: `New`, `Running`, `BlockedOnChild`, `Yielded`, `Parked(i64)`, `Queued`, `Done`, `Failed`.
- `pub consumer: Option<FiberConsumer>`: Set only during manual `call()` / `try()`. **Preserved** across `Parked(g) -> Queued -> Running`. Cleared only on `Fiber.yield`, `Done`, or terminal `Failed` delivery.

### 3.3 Dispatch Loop & Quiescence Hook
In `phalcom-core/src/vm/dispatch.rs` (`run_until`):
- When `ready_queue` is empty and no guest fiber is running:
  - If a parked leaf blocks root (`find_root_blocking_parked_leaf()`), it currently raises quiescence error: `"await: the future is still pending and the scheduler is empty; nothing can settle it"`.
  - **C3 Hook**: Replace immediate quiescence failure with reactor ingress check / deadline idle wait if reactor registrations or timers are active.

---

## 4. Key Rules for Implementation

1. **Law of Thread Isolation**: Background workers operate exclusively on plain Rust structs (`WorkerJob`, `WorkerCompletion`). Conversion to/from Phalcom `Value` happens exclusively on the main VM thread.
2. **GC Rooting**: Any pending reactor registration referencing a Phalcom object (e.g., `Future` receiver) must be rooted in `VM::collect_roots` (in `phalcom-core/src/vm/gc.rs`).
3. **No Guest Pump Regression**: Do not expose low-level completion channels or manual worker loops to `.ph` guest code. `System.runScheduled` remains a high-level driver.
4. **Monotonic Timing**: Use `Instant::now()` and `Duration` for all delay calculations.

---

## 5. Verification Targets for C3.P1

- `cargo check -p phalcom-core`
- `cargo test -p phalcom-core --lib`
- `cargo test -p phalcom-core --test language-corpus corpus::concurrency`
- `cargo test -p phalcom-core --test language-corpus corpus::system`
- New fixtures for `System.sleep` and worker async I/O completion.
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
