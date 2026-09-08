# Futures

> Sources: cancellation/reactor specifications; CONC001
> Raw: [concurrency specification and program snapshot](../raw/concurrency/2026-09-08-concurrency-sources.md)
> Updated: 2026-09-08

Future values compose pending work, completion, failure, and cancellation above the fiber/reactor substrate. They must preserve exactly one completion outcome and release the associated resource or waiter on cancellation.

Future combinators are library behavior; scheduling and readiness remain owned by [fibers and scheduling](fibers-and-scheduling.md) and the [reactor boundary](reactor.md). CONC001 is in progress and partial.
