# Reactor

> Sources: reactor and cancellation specifications; CONC001
> Raw: [concurrency specification and program snapshot](../raw/concurrency/2026-09-08-concurrency-sources.md)
> Updated: 2026-09-08

The reactor is the host-event boundary for readiness and timers. It schedules
the continuation of a fiber without making host I/O semantics part of fiber
identity or duplicating native resource ownership.

Pending operations need explicit completion, cancellation, and cleanup paths.
[Host capabilities](../native/host-capabilities.md) provides the effects and
handle contracts; [futures](futures.md) composes their outcomes. CONC001 is in
progress and partial.
