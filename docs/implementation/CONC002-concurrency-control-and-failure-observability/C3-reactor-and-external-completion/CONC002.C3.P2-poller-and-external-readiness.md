---
id: CONC002.C3.P2
program: CONC002
checkpoint: CONC002.C3
kind: implementation-plan
status: COMPLETE
completion: COMPLETE
verification: VERIFIED
prepared: 2026-09-12
requires:
  - CONC002.C3.P1 COMPLETE
blocked_on: []
---

# CONC002.C3.P2 — Poller-backed external readiness

## 0. Mission

Complete the reactor for pollable descriptors without introducing a second executor.

P1 supplies:

```text
registration identity
GC rooting
timer source
worker completion ingress
executor idle wait
VM-thread settlement
```

P2 adds:

```text
pollable descriptor readiness
one unified blocking wait
cross-source wake
```

This plan is implementation-blocked while PDR-0016 remains Proposed. Do not treat its `mio` decision as accepted architecture merely because this plan describes the expected implementation if that PDR is ratified unchanged.

## 1. Hard decision gate

At execution start:

1. read current PDR-0016 status;
2. if Accepted unchanged, use the `mio` design below;
3. if superseded/changed, rebase backend-specific tasks;
4. if still Proposed, stop before dependency/backend implementation.

The P1 reactor remains valid independently.

## 2. Backend-independent requirements

Regardless of backend:

- one reactor registration vocabulary;
- poller token index maps into C3 registry;
- generation validity remains C3's, not the poller's;
- no poller type leaks outside the reactor module;
- no poller thread;
- readiness processing occurs on VM thread;
- completion/wake feeds the same executor as workers/timers;
- ready event never executes Fiber inline;
- stale/deregistered readiness is harmless;
- timer/worker/poller sources share one idle wait;
- IO selector surfaces are not implemented here.

## 3. Expected PDR-0016 design if accepted

Use `mio` with:

```text
features = ["os-poll", "net"]
```

and confine every `mio` type to `phalcom-core/src/reactor.rs` or the single canonical reactor module chosen in P1.

Components:

```text
Poll
Events
Waker
mio::Token(index)
```

Generation remains in the C3 registry.

Worker threads may call `Waker::wake()` after pushing plain-data completion; they still never receive VM/heap state.

## 4. Unified wait

Replace P1's phase-1 worker-channel timeout wait with:

```text
poll(timeout = min(next_timer_deadline, executor_cap))
```

On wake:

1. process poll events;
2. non-blockingly drain worker MPSC;
3. collect expired timers;
4. validate every reactor token/generation;
5. enqueue/deliver VM-owned completion work;
6. return to ordinary executor selection.

No source gets priority by preemption.

## 5. Readiness correctness

Expected accepted-PDR rule:

```text
try syscall first
if WouldBlock:
    register/rearm readiness
```

Do not register first under an edge-triggered backend because data already available before registration can otherwise produce no new edge.

On readiness:

- retry operation on VM thread;
- success -> produce reactor completion;
- WouldBlock -> rearm;
- terminal OS error -> produce error completion;
- stale token -> drop.

The reactor mechanism may expose an internal operation trait/enum for host-surface programs, but it must not embed Phalcom `Value` or callbacks into poller state.

## 6. One-pending-operation assumptions

If PDR-0015 remains the consuming network design, preserve its one-pending-read / one-pending-write discipline.

Do not independently build a multi-waiter socket arbitration policy in C3.

If future host surfaces require more complex multiplexing, that is a consumer design over the same registration substrate.

## 7. Fairness

Implementation default:

- running guest is never preempted;
- process bounded event batch;
- completions/woken Fibers join executor queue tail;
- later events wait to a later executor round.

Treat this as executor policy unless separately ratified as language semantics.

Add starvation/adversarial tests with a hot ready queue plus repeated external readiness.

## 8. Tests

Backend-independent:

- stale readiness after deregistration;
- token slot reuse;
- timer + worker + readiness wake in same executor;
- worker Waker wakes blocked poll;
- completion does not run waiter inline;
- GC root survives descriptor wait;
- no descriptor registration -> clean exit.

If `mio` accepted:

- socketpair/loopback raw test without public network API;
- data buffered before interest registration does not deadlock;
- WouldBlock then readiness then success;
- rearm correctness;
- read/write interests independent where consuming design permits;
- macOS/Linux supported under repository platform gates;
- assert no `mio::` usage outside reactor module.

## 9. Non-goals

- `TcpStream`/DNS/TLS APIs;
- filesystem user API;
- process user API;
- cancellation semantics;
- select;
- parallel executor;
- a second Rust async runtime;
- tokio/futures integration.

## 10. Completion criteria

P2 is complete when:

- a ratified poller decision has been implemented;
- poller is confined;
- generation validation remains reactor-owned;
- timer/worker/poller wake one executor;
- try-first correctness tests pass;
- stale events cannot settle reused registrations;
- no external event resumes guest code directly;
- normative reactor/poller links match implementation;
- C3 checkpoint may become COMPLETE.
