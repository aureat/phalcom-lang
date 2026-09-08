# Fiber reflection

> Sources: concurrency specification; CONC001
> Raw: [concurrency specification and program snapshot](../raw/concurrency/2026-09-08-concurrency-sources.md)
> Updated: 2026-09-08

Fiber reflection exposes authorized state such as identity, readiness, and lifecycle without handing callers scheduler internals. Reflection is a capability-bearing runtime surface, not a second scheduler or semantic authority.

The VM owns live frame/value state; the scheduler owns runnable state; native contracts own host handles. CONC001 is in progress and partial. See [fibers and scheduling](fibers-and-scheduling.md) and [semantic capabilities](../semantic/capability-and-flow.md).
