# CONC002 PROGRAM.md — C3/C4 amendment

Apply this to the scopes 1–3 replacement `PROGRAM.md` once the new plans are committed.

Replace transitional C3/C4 placeholders with:

```text
C3 — Reactor and external completion runtime
  P1 — reactor core, workers, timers, executor liveness
  P2 — poller-backed external readiness

C4 — Concurrency standard-library completion
  P1 — Future state, readiness registrations, CompletionSource
  P2 — aggregation, timeout, Backoff time integration
```

Dependency:

```text
C2.P2 -> C3.P1
C3.P1 -> C3.P2
C2.P2 -> C4.P1
C4.P1 -> C4.P2
C3.P1 -> C4.P2 time-dependent portions
```

C5 and C6 remain roadmap-only until their own plans are authored.

Do not describe C3 as library polish or C4 as reactor work.
