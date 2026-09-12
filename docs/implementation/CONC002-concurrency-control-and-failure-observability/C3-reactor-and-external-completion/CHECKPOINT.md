---
id: CONC002.C3
program: CONC002
kind: implementation
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
requires:
  - CONC002.C2.P1-R1 COMPLETE
  - CONC002.C2.P2 COMPLETE
---

# CONC002.C3 — Reactor and external completion runtime

C3 gives the post-C2 VM executor a real external progress source.

It owns reactor registrations, worker completions, timers, executor idle/liveness integration, shutdown/leak mechanics, and the poller-backed readiness phase. It does not own Future composition policy, cancellation/structured concurrency, channels/select, or IO selector surfaces.

## Plans

| Plan | Scope | State |
|---|---|---|
| `CONC002.C3.P1` | reactor core, registration registry, worker pool, completion ingress, timers, executor liveness, `System.sleep` | PROPOSED |
| `CONC002.C3.P2` | poller-backed descriptor readiness and unified external wait | BLOCKED on PDR-0016 acceptance |

## Authority

- PDR-0003: accepted worker/VM-thread discipline.
- PDR-0004: accepted Future-shaped reactor/worker split and timer ownership.
- `docs/spec/current/stdlib/reactor.md`: normative machinery contract, amended for post-C2 executor architecture.
- PDR-0016: proposed poller backend decision; P2 must not implement it as settled policy until accepted.

## C2 handoff consumed here

C3 assumes C2 provides:

- VM-owned executor control between guest Fiber turns;
- source-agnostic `Parked(g) -> Queued` wake authorization;
- wake-is-not-execution;
- consumer/executor separation;
- safe VM-visible user-code activation;
- truthful no-progress detection when no external progress source exists.

C3 extends the executor's no-progress decision. It does not create another Fiber resume path.

## Completion boundary

C3 is complete when:

- P1 is complete;
- P2 is complete or explicitly remains blocked by an unresolved PDR-0016 decision while the checkpoint is marked PARTIAL/BLOCKED accordingly;
- normative reactor links point to the current implementation plans;
- no old CONC001 record still claims current reactor implementation ownership.
