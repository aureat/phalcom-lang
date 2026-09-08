# Fibers and scheduling

> Sources: concurrency specification; CONC001
> Raw: [concurrency specification and program snapshot](../raw/concurrency/2026-09-08-concurrency-sources.md)
> Updated: 2026-09-08

A fiber is a cooperatively scheduled unit of execution with explicit suspension/resumption state. The scheduler owns runnable ordering, handoff, and cancellation observation; the VM owns frame and value lifetime within a fiber.

## Boundaries

The reactor integrates external readiness without making host I/O semantics part of scheduler identity. Reflection exposes fiber state only through an authorized runtime surface. Futures are library-level composition over the substrate, not a second scheduler.

CONC001 is in progress and partial. [Lifecycle and memory](../runtime/lifecycle-and-memory.md) owns cleanup when a fiber exits or is cancelled.
