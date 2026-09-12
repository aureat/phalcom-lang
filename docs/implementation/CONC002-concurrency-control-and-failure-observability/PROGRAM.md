---
id: CONC002
category: CONC
kind: implementation
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
---

# CONC002 — Cooperative concurrency correctness and evolution

CONC002 owns the staged correction and evolution of Phalcom's single-threaded cooperative concurrency model. The program begins with truthful Fiber/scheduler/Future ownership, then makes VM continuation state suspension-safe, then adds external readiness, library composition, structured cancellation and communication primitives in dependency order.

## Program invariants

Across all checkpoints:

- a user value is never used as a hidden lifecycle sentinel;
- each nonterminal stop has one truthful execution reason;
- each wake/admission episode has one explicit authority;
- user `yield` belongs to a coroutine consumer, while parking belongs to an executor/readiness owner;
- terminal completion is durable and independent of the latest dynamic coroutine caller;
- wake makes work runnable; it does not recursively execute arbitrary guest code;
- an active ancestor is never made independently runnable merely because its descendant parks;
- host/native stack state may be crossed only when its continuation is representable or the boundary remains explicitly guarded;
- direct-style suspension remains uncolored at the source-language level;
- later cancellation/reactor/thread work consumes these ownership contracts rather than replacing them.

## Checkpoint graph

### C1 — Corrected concurrency foundation

Owns the landed Fiber/scheduler/Future remediation and its final current-main verification/type prerequisites.

- P1: lifecycle, scheduler admission, ticketed park/wake, completion observers, terminal Future driving — **complete**.
- P2: detached scheduler-failure observability (E010) — **complete**.
- P3: recertify the later `Future<T>`/Unit/callback/adoption delta together with P1/P2 — **complete**.
- P4: pre-C2 callable-domain/applied-type/Fiber-typing prerequisites — **proposed**.

### C2 — VM suspension and coroutine semantics

Owns execution-state representation that must survive suspension independently of Rust host frames and later separates coroutine-consumer ownership from executor driving.

- P1-R1: native suspension and VM control continuations; `on`/`ensure`, non-local return, host-floor escape, Call child-failure injection, native callback fallbacks, lifetime/trace integration and executor readiness handoff — **proposed**.
- P2: manual coroutine + executor composition (`yield → await → yield`) and final `Fiber<R>` typing if C1.P4 prerequisites are available — **next plan, intentionally not yet published in this restructuring bundle**.

C2 does not implement a reactor backend.

### C3 — Reactor and external completion execution

Planned checkpoint. It will own external registration lifetime, idle-with-registrations versus complete executor state, readiness polling/injection, timer substrate and the execution boundary that turns external readiness into ordinary validated queue admission.

The current older C3 plans are superseded as execution authorities because they mix this work with later library/cancellation/channel concerns. New executable C3 plans must be produced before implementation.

### C4 — Concurrency standard library

Planned checkpoint. It will own bounded library-level completion/composition/time utilities on top of C1–C3. Existing Future behavior already shipped by C1 is input, not reimplemented here.

The previous monolithic C3 library plans are source material only until split/reissued under this checkpoint.

### C5 — Cancellation and structured concurrency

Roadmap checkpoint. Owns cancellation as a distinct terminal/control request, queue/wait revocation identity, scope/child lifetime, cleanup-before-publication and structured task policy. It must not reuse `Fiber.abort`, ordinary Error-as-data, or the dynamic coroutine `resumer` as lifetime ownership.

### C6 — Channels and select

Roadmap checkpoint. Owns channels, multi-wait registration/select and associated fairness/backpressure semantics after cancellation/readiness ownership is stable.

## Dependency order

```text
C1 truthful ownership and type prerequisites
  ↓
C2 VM-owned suspension/control + consumer/executor semantics
  ↓
C3 reactor/external readiness
  ↓
C4 library composition/time
  ↓
C5 cancellation/structured lifetimes
  ↓
C6 channels/select
```

A later checkpoint may investigate future needs early, but it may not make itself an implementation dependency of an earlier one by smuggling policy into the lower layer.

## Current program state

C1 P1/P2/P3 are implemented. C1 remains open for pre-C2 type prerequisites (P4). C2 P1-R1 is the next runtime architecture implementation plan. C2 P2 follows P1-R1. C3–C6 require rewritten plans before execution.

Historical focused evidence is retained in C1's implementation-state record. The program remains `BASELINE_BLOCKED` at the broadest verification level because the original final workspace-test gate encountered independently diagnosed REPL export failures; this does not reopen completed concurrency implementation.
