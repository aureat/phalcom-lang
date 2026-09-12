---
id: CONC001.C2
program: CONC001
kind: implementation
status: COMPLETE
completion: IMPLEMENTED
verification: HISTORICAL
---

# CONC001.C2 — original ready-queue scheduler

This checkpoint records the original ready-queue and root-drive implementation.

## Historical plan

| Plan | Scope | State |
|---|---|---|
| `CONC001.C2.P1` | initial native ready queue, `System.schedule`, root-drive pump | IMPLEMENTED HISTORICAL |

## Current authority

The original scheduler has since been hardened by CONC002:

- truthful Fiber lifecycle;
- queued ownership;
- exact park generations;
- stale wake protection;
- detached scheduler-failure observability;
- VM-owned executor semantics.

Current scheduler semantics are governed by CONC002.C1/C2, not this historical plan.

## Reactor ownership

Reactor/worker/timer implementation no longer belongs to CONC001.C2.

Current owner:

```text
CONC002.C3 — Reactor and external completion runtime
```

The historical reactor implementation spec is removed after the CONC002.C3 migration ledger confirms complete requirement transfer.
