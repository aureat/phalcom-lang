# N. Fibers & Futures: cooperative concurrency (proposed ADR)

> **Promoted → [ADR-0030](../../../adr/0030-fibers-and-futures-cooperative-concurrency.md)**
> (Accepted 2026-07-12). The ratified ADR adds the execution-model decision
> (Option A — restricted re-entrant loop) and the forward-compat §7 corrections
> (heap `Object::Fiber`, not a `Value` arm). This draft is retained for history.

- Status: Superseded by ADR-0030 · promotes concurrency.md to a ratified ADR
- Related: ADR-0009 (handle heap), ADR-0013 (closures/non-local return), concurrency.md

## Context

`Fiber`/`Future` is the only major subsystem fully specified but ADR-less. Two
interactions are unrecorded and load-bearing.

## Decision

1. **`Fiber` is the sole concurrency primitive.** Cooperative, single-threaded,
   no preemption; `Future`, `async`/`await`, generators, scheduler all derive from
   it. No data races by construction; no locks in the object model.
2. **Stackful fibers ⊗ moving-ready heap (ADR-0009).** Each `FiberObject` owns its
   own `Vec<Value>` value stack and `Vec<CallFrame>` frame stack holding `ObjRef`
   handles. **New invariant:** *a `FiberObject`'s value stack and frame stack are
   GC roots for as long as the fiber is reachable and not `done`* — not only the
   `current` fiber's. A collector that scans only `current` frees objects held
   solely by a parked fiber. This must hold before any tracing/compacting GC lands.
3. **Fiber switch is an O(1) pointer swap** of `current`, not a stack copy; the
   dispatch loop reads `current.stack`/`current.frames`.
4. **Non-local return is fiber-local by construction.** A block's home-frame token
   (ADR-0013) lives on one fiber's frame stack; a `return` across a fiber boundary
   fails the generation check → `DeadFrameError`, never a wrong-stack unwind.

## Precludes

Shared-memory multithreading without a new memory model. Accepted — the singular
cooperative primitive is the whole point.
