---
id: CONC002
category: CONC
kind: implementation
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
---

# CONC002 — Cooperative concurrency correctness and evolution

CONC002 owns Phalcom's staged concurrency correction and evolution.

## Checkpoints

### C1 — Corrected concurrency foundation

- P1 — lifecycle, scheduler admission, ticketed park/wake, completion observers — COMPLETE.
- P2 — detached scheduler-failure observability — COMPLETE.
- P3 — current-main Future/type architecture closure — COMPLETE.
- P4 — pre-C2 stabilization/type prerequisites — PROPOSED.

### C2 — VM suspension and coroutine semantics

- P1-R1 — native suspension and VM-owned control continuations — IN_PROGRESS.
- P2 — consumer/executor separation, pending manual await, yield-await-yield, final honest Fiber generic decision — PROPOSED / PUBLISHED.

C2 owns no reactor backend.

### C3 — Reactor and external completion runtime

- P1 — registration registry, GC roots, worker pool, completions, timers, executor liveness, `System.sleep` — PROPOSED.
- P2 — poller-backed external readiness — BLOCKED on PDR-0016 acceptance or replacement ruling.

### C4 — Concurrency standard-library completion

- P1 — typed Future state, settlement, explicit/detachable readiness registrations, CompletionSource, pure Backoff policy — PROPOSED.
- P2 — `all`, `allSettled`, `race`, `timeout`, timed Backoff — PROPOSED.

C4 does not own Fiber runtime semantics, reactor machinery, cancellation, channels, or select.

### C5 — Cancellation and structured concurrency

Roadmap only. Owns cancellation semantics, task/scope lifetime, revocation and structured child policy.

### C6 — Channels and select

Roadmap only. Owns channels, communication waiters, select arbitration and communication-specific fairness/backpressure.

## Dependency graph

```text
C1
 ↓
C2.P1-R1
 ↓
C2.P2
 ├──────────→ C3.P1 → C3.P2
 └──────────→ C4.P1 → C4.P2
                    ↘ time features also require C3.P1
                         ↓
                        C5
                         ↓
                        C6
```

PDR-0016 blocks C3.P2, not C3.P1 or C4.P1.

Legacy monolithic C3 library plans are superseded.
