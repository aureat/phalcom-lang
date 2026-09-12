---
id: CONC002.C2
program: CONC002
kind: checkpoint-record
status: IN_PROGRESS
completion: IN_PROGRESS
verification: IN_PROGRESS
---

# CONC002.C2 — VM suspension, control continuations, and coroutine semantics

| Plan | Scope | State |
|---|---|---|
| P1-R1 | native suspension, VM-owned protected/cleanup control, transfer routing, Call failure injection, native-control migration | COMPLETE / IMPLEMENTED |
| P2 | consumer/executor separation, VM-owned queued driving, manual pending await, yield-await-yield, nested GC/failure verification, conditional `Fiber<R>` | READY / IN_PROGRESS |

C2 owns VM control continuation state, coroutine-consumer/executor semantics, and the source-agnostic readiness handoff.

C2 does not own:

- reactor registrations/timers/workers/poller — C3;
- Future library composition/state cleanup — C4;
- cancellation/Task/TaskScope — C5;
- Channel/select — C6.

Sequence:

```text
C1.P4 -> C2.P1-R1 -> C2.P2 -> C3 / C4
```

C3 consumes:

```text
readiness
 -> validate exact wait authority
 -> Parked(generation) -> Queued
 -> enqueue
 -> later executor execution
```

and must not invent a second Fiber resume protocol.
