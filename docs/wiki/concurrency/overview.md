# Concurrency

> Sources: concurrency/reactor/cancellation specifications; CONC001
> Raw: [concurrency specification and program snapshot](../raw/concurrency/2026-09-08-concurrency-sources.md)
> Updated: 2026-09-08

The concurrency domain owns cooperative fibers, scheduling, reactor integration, reflection, cancellation, and future-library contracts. It neighbors [runtime](../runtime/overview.md) for lifecycle, [native](../native/host-capabilities.md) for I/O, [semantic](../semantic/capability-and-flow.md) for effects, and [performance](../performance/overview.md) for workload evidence.

## Reading order

- [Fibers and scheduling](fibers-and-scheduling.md)
- [Reactor](reactor.md)
- [Fiber reflection](fiber-reflection.md)
- [Futures](futures.md)

CONC001 is in progress and partial; proposed library surfaces remain explicitly unverified.
