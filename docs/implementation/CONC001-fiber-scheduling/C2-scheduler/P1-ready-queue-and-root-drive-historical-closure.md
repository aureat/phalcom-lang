---
id: CONC001.C2.P1
program: CONC001
checkpoint: CONC001.C2
kind: historical-implementation-record
status: COMPLETE
completion: IMPLEMENTED
verification: HISTORICAL
---

# CONC001.C2.P1 — ready queue and root drive — historical closure

## Status notice

This is the closure record for the original U-SCHED implementation plan.

The mechanism it introduced exists, but many implementation details in the original plan are no longer current:

- scheduler dequeue is now internal;
- queue ownership is represented explicitly;
- Fiber lifecycle has been split into truthful states;
- Future waiting uses exact park generations;
- unowned scheduler failures have a VM-owned reporting channel;
- C2 replaces scheduler-as-coroutine-resumer driving with VM-owned executor semantics.

For current behavior, use CONC002.C1/C2 and `docs/spec/current/concurrency.md`.

## Historical contribution retained

This plan established:

- `VM::ready_queue`;
- `System.schedule`;
- the initial scheduler drain/root-drive concept;
- the architectural decision not to introduce a second user concurrency primitive.

These are provenance facts, not a complete description of the current scheduler.

## Reactor note

The original plan deliberately deferred timers. Reactor implementation ownership is now CONC002.C3.

No new work should be dispatched from the old CONC001 reactor companion material.
