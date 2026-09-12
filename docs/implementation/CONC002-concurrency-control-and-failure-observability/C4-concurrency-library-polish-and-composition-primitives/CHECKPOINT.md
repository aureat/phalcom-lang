---
id: CONC002.C3
program: CONC002
kind: implementation
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# CONC002.C3 — Concurrency library polish and composition primitives

Polish the existing Fiber/Future package and its shipped helpers, then add a
bounded set of completion, scheduling, time, task and communication primitives.
This checkpoint does not claim a complete concurrency standard library.

| Plan | Scope | Start / completion boundary |
|---|---|---|
| [CONC002.C3.P1](CONC002.C3.P1-existing-library-typing-and-behavior-polish.md) | Existing state, registration, typing, callback/scheduler behavior, Tracer, OffBehavior, Backoff, packaging and bootstrap | Start E0–E3 now; full completion includes C1.P4 Fiber typing and P2.N3-backed Backoff |
| [CONC002.C3.P2](CONC002.C3.P2-completion-composition-time-and-task-primitives.md) | CompletionSource, aggregation, checkpoint, timers/timeout, structured tasks/cancellation, channels, bounded map/retry and atomic select | N0–N1 consume early P1 work; later features have explicit runtime/design gates |

Both plans are proposed, not implemented or runtime-verified. Ship their units
individually and retain partial status until all acceptance gates are satisfied.
C2 supplies native continuation/readiness groundwork; it supplies no timer backend
or public cancellation contract by itself. Backoff waits depend on P2.N3, while
P2.N3 depends only on P1's early registration work; the dependency is not cyclic.

Renamed from `C3-concurrency-standard-library`; the previous single P1 plan is
restructured into these two plans. The checkpoint ID remains CONC002.C3. No source
implementation or specification change is included in this documentation revision.
