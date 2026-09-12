---
id: CONC002.C4.MIGRATION
program: CONC002
checkpoint: CONC002.C4
kind: migration-ledger
status: PROPOSED
---

# Old C3 -> new C4/C5/C6 migration map

The old `CONC002.C3` mixed Future library work with reactor, cancellation and synchronization runtime work.

This ledger prevents those units from being lost or duplicated.

| Old unit | New owner |
|---|---|
| P1.E0 signature/type lock | C4.P1 |
| P1.E1 typed Future state / one settlement | C4.P1 |
| P1.E2 explicit/detachable registrations | C4.P1 |
| P1.E3 callback/scheduler consistency | mostly already landed; remaining registration integration in C4.P1 |
| P1.E4 Fiber typing | C2.P2 |
| P1.E4 Tracer | decorator/observability program |
| P1.E4 OffBehavior | decorator/feature-flag program |
| P1.E5 Backoff pure policy | C4.P1 |
| P1.E5 timed wait integration | C4.P2 consuming C3.P1 |
| P1.E6 packaging/bootstrap | C4.P1/P2 by changed surface |
| P2.N0 CompletionSource | C4.P1 |
| P2.N1 all/allSettled/race | C4.P2 |
| P2.N2 checkpoint/revocable admission | C5 or separate scheduler policy project; not C4 |
| P2.N3 reactor registration/timer/pump | C3 |
| P2.N3 Future.timeout | C4.P2 |
| P2.N4 TaskScope/Task/cancellation | C5 |
| P2.N5 Channel | C6 |
| P2.N6 mapConcurrent / structured retry | C5 if retained after structured-concurrency design |
| P2.N7 select | C6 |

## Supersession rule

After new C3/C4 plans are committed, the old C3 P1/P2 must no longer remain `PROPOSED` executable plans.

Either:

- retain as `SUPERSEDED` historical records with this mapping; or
- delete after repository policy permits.

No agent should implement reactor, TaskScope, Channel or select from the old C3 plans.
