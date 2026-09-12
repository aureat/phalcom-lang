---
id: CONC002.C4
program: CONC002
kind: implementation
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
requires:
  - CONC002.C2.P2 COMPLETE
---

# CONC002.C4 — Concurrency standard-library completion

C4 completes the Future-oriented standard-library layer after C2 fixes runtime ownership.

It owns:

- Future internal state cleanup;
- explicit/detachable readiness registrations;
- CompletionSource;
- aggregation;
- timeout;
- Backoff policy/time integration;
- packaging/bootstrap/tests for those surfaces.

It does not own reactor machinery, Fiber runtime semantics, cancellation/TaskScope, channels, or select.

## Plans

| Plan | Scope | Dependencies |
|---|---|---|
| C4.P1 | Future state, readiness registrations, CompletionSource, pure Backoff policy | C2.P2 |
| C4.P2 | Future.all/allSettled/race, timeout, timed Backoff | C4.P1; timeout/time Backoff additionally require C3.P1 |

## Downstream exclusions

- C5: cancellation + structured concurrency;
- C6: channels + select.

Tracer/OffBehavior are decorator/observability support types and are not redesign targets of this concurrency checkpoint.
