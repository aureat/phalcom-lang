---
id: CONC001.C2
program: CONC001
kind: implementation
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# CONC001.C2 — scheduler and reactor

This checkpoint owns the ready queue, root-drive pump, worker completions, and
timers that provide the runtime substrate for deferred Future work.

## Plans

| Plan | Scope | State |
|---|---|---|
| `CONC001.C2.P1` | Ready queue and root-drive pump | Proposed scheduler substrate |

## Specifications

| Specification | Role |
|---|---|
| `ready-queue-and-root-drive-spec.md` | Queue and VM root-drive contract |
| `reactor-worker-pool-and-timers-spec.md` | Worker completion and timer contract |
| `scheduler-and-reactor-scope-spec.md` | Scheduler/reflection boundary overview |
