---
id: CONC002.C3
category: CONC
program: CONC002
checkpoint: CONC002.C3
kind: checkpoint-record
status: COMPLETE
completion: COMPLETE
verification: VERIFIED
requires:
  - CONC002.C2.P1-R1 COMPLETE
  - CONC002.C2.P2 COMPLETE
---

# CONC002.C3 — Reactor and external completion runtime

C3 gives the post-C2 VM executor a real external progress source.

It owns reactor registrations, worker completions, timers, executor idle/liveness integration, shutdown/leak mechanics, and the poller-backed readiness phase. It does not own Future composition policy, cancellation/structured concurrency, channels/select, or IO selector surfaces.

`CONC002.C3` is fully completed and verified across both phases (`C3.P1` and `C3.P2`).

---

## 1. Plans Ledger

| Plan | Scope | Status | Verification | Summary / Outcome |
|---|---|---|---|---|
| `CONC002.C3.P1` | Reactor core, generational token registry, plain-data worker pool, monotonic timers, safepoint ingress, executor idle wait/liveness, `System.sleep` | **COMPLETE** | `VERIFIED` | Generational token table implemented; worker pool and channel transport strictly isolate threads from `Value`/`ObjRef`/`Heap`; timer min-heap integrated into executor idle wait; `System.sleep(Int) -> Future<Unit>` added; unit and language corpus tests green. |
| `CONC002.C3.P2` | Poller-backed descriptor readiness and unified external wait | **COMPLETE** | `VERIFIED` | PDR-0016 ratified (`mio` confined to `reactor/`); `Poller` and cross-thread `Waker` integrated; unified wait in `idle_wait`; try-first ET correctness verified; unit and corpus tests green. |

---

## 2. Ownership Boundary

C3 owns:
- Generational token registration table (`ReactorToken`, `ReactorRegistration`, `ReactorRegistry`);
- Plain-data background worker pool (`WorkerPool`, `WorkerJob`, `WorkerCompletion`, `WorkerOutcome`, `WorkerResult`);
- Cross-thread completion transport and safepoint ingress;
- Monotonic timer priority queue (`TimerQueue`, `TimerEntry`);
- Executor liveness and idle wait integration (`VM::reactor_idle_wait`, `System._$nextScheduled`);
- Public async sleep surface (`System.sleep(Int) -> Future<Unit>`);
- Reactor shutdown and resource leak reporting.

C3 does **not** own:
- Future composition policy, state representation changes, or callbacks — `CONC002.C4`;
- Public cancellation, `TaskScope`, structured concurrency — `CONC002.C5`;
- Channels and select — `CONC002.C6`;
- High-level socket or filesystem user-facing APIs.

---

## 3. Established Invariants & Architectural Rules

1. **Law of Thread Isolation**: Background workers operate exclusively on owned plain Rust structs (`Send + 'static`). Conversion to/from Phalcom `Value` or `ObjRef` happens exclusively on the main VM thread.
2. **Generational Registration Identity**: Reactor registration tokens (`ReactorToken { slot, generation }`) are generation-tagged. Stale completions from previous registrations in recycled slots are discarded as harmless no-ops.
3. **Distinct Identity Types**: `ReactorToken` is strictly distinct from Fiber park generations, scheduler admission identities, and C2 control frame tokens.
4. **GC Root Tracing**: While a reactor registration is in `Active` state, its target (`ObjRef`, e.g. `Future`) is rooted in `VM::collect_roots`. Upon completion, cancellation, or shutdown, the root is released.
5. **Wake Is Not Execution**: Reactor completion materializes the plain outcome and invokes `Future#settleValue` or `Future#settleError` on the VM thread. This drains ticketed waiters into `ready_queue` without executing guest bytecode inline.
6. **Monotonic Timers Without Threads**: Timers use `std::time::Instant` in a min-heap priority queue, polled during safepoints and bounded in executor idle wait without dedicated timer threads.
7. **Liveness & Quiescence**: Ready-queue emptiness is not quiescence when active reactor registrations or undrained completions exist. The executor sleeps until the next deadline or completion event before applying C2 no-progress rules.

---

## 4. Root & Lifetime Ledger

- `ReactorRegistry` is owned by `Reactor`, which is owned by `VM`.
- `VM::collect_roots` exhaustively traverses all active registration targets via `reactor.trace_roots`.
- Completed, cancelled, or shut down registrations release their target handles immediately, enabling garbage collection.
- Safepoint ingress drains cross-thread completion items non-blockingly into internal buffers without triggering allocations or user-code execution during GC safepoints.

---

## 5. Verification Ledgers

### 5.1 C3.P1 Evidence Ledger

All verification commands executed serially on `main` with cleared compiler flags:

| Gate | Scope | Command / Target | Result | Duration / Details |
|---|---|---|---|---|
| **C0** | Baseline rebase | `cargo test -p phalcom-core --lib`, baseline concurrency suites | **PASS** | 112 passed, 0 failed; clean baseline established |
| **C1** | Reactor registry | `cargo test -p phalcom-core --lib reactor::tests::test_token_registry_lifecycle_and_stale_drop` | **PASS** | Generational invalidation, slot reuse, and stale drop verified |
| **C2** | GC Root lifecycle | `cargo test -p phalcom-core --lib reactor::tests::test_registry_gc_root_tracing` | **PASS** | Active targets rooted, completed/cancelled targets released |
| **C3** | Worker pool & transport | `cargo test -p phalcom-core --lib reactor::tests::test_worker_job_round_trip` | **PASS** | Plain-data compile-time check + async execution verified |
| **C4** | Monotonic timers | `cargo test -p phalcom-core --lib reactor::tests::test_timer_min_heap_ordering_and_tie_breaking` | **PASS** | Min-heap ordering, sequence tie-breaking, deadline calculation |
| **C5** | Shutdown & leaks | `cargo test -p phalcom-core --lib reactor::tests::test_shutdown_and_leak_reporting` | **PASS** | Registration cleanup, thread join, and leak diagnostics verified |
| **C6** | Language acceptance | `cargo test -p phalcom-core --test language-corpus corpus::concurrency` | **PASS** | 88 passed, 0 failed (includes sleep liveness, sleep zero, child fiber) |
| **C7** | Negative acceptance | `cargo test -p phalcom-core --test language-corpus corpus::concurrency_negative` | **PASS** | 14/14 passed (includes negative sleep duration error) |
| **C8** | GC & Surface suites | `cargo test -p phalcom-core --test core memory_gc`, `native_surface_contracts` | **PASS** | 18/18 GC tests, 3/3 surface contract tests green |
| **C9** | Workspace lint & fmt | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` | **PASS** | Clean across workspace (0 warnings) |

---

## 6. Next Actions and Handoff

- **Active Plan**: `CONC002.C3.P1` is completed and certified.
- **Next Step**: `CONC002.C3.P2` (*Poller-backed descriptor readiness*) remains gated on PDR-0016 acceptance.
