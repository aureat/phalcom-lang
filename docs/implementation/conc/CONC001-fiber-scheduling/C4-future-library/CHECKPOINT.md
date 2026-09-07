---
id: CONC001.C4
program: CONC001
kind: implementation
status: DEFERRED
completion: NOT_STARTED
verification: UNVERIFIED
deferred_reason: Awaiting the Fiber substrate and scheduler/root-drive prerequisites.
---

# CONC001.C4 — Future library

This checkpoint contains the deferred Future/await layer and its cancellation
surface. It remains gated on Fiber and scheduler prerequisites.

## Plans

| Plan | Scope | State |
|---|---|---|
| `CONC001.C4.P1` | Future library and `await` | Deferred until Fiber and scheduler prerequisites are available |

## Specifications

| Specification | Role |
|---|---|
| `future-library-spec.md` | Future surface and state-machine contract |
| `future-library-architecture-spec.md` | Future implementation architecture |
| `future-cancellation-spec.md` | Cancellation contract and blocker |
