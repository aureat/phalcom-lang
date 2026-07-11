# Scheduler unit (proposed — closes the Future bootstrap gap)

- Status: Proposed · resolves the "scheduler has no owner" gap
- Hazard: **primitive/library boundary ⊗ bootstrap order**

## Problem

`Future.await`/`async` and `System.sleep` bottom out in a scheduler run-loop that
exists only in prose (concurrency.md §2, system.md §Scheduler) — no unit builds
it, like DEC-A's kernel `List`. `Future` is "just a library class" sitting on a
runtime that must exist *first*.

## Decision

Add an explicit unit **U-SCHED**, sequenced **after `Fiber`, before any `Future`
method beyond `value`/`isReady`**:

- a **ready-queue** of resumable fibers;
- a **timer completion source** driving `System.sleep`;
- settlement enqueues every waiter fiber + `then` continuation.

**Top-level runs inside the root scheduler fiber** — so top-level `await` is legal
(as system.md implies). This is *not* retrofittable: it defines what `main` is.
Fix it here, not after `Future` ships.

## Dependency DAG

```
Fiber ──▶ U-SCHED (ready-queue + timers) ──▶ Future.await/async/then
                                        └──▶ System.sleep/schedule
```

## Precludes

A blocking `await`. `await` must yield to U-SCHED, never park the OS thread —
otherwise the single thread deadlocks. Enforced by construction.
