---
id: CONC002.C1
program: CONC002
kind: corrective-implementation
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
---

# CONC002.C1 — concurrency control and failure observability

This checkpoint groups the ordered concurrency-control remediation and the
follow-on E010 scheduler-failure observability patch.

## Plans

| Plan | Scope | State |
|---|---|---|
| `CONC002.C1.P1` | Fiber lifecycle, scheduler admission, ticketed await, durable completion, and Future adoption | In progress; focused evidence retained |
| `CONC002.C1.P2` | REPL/E010 plan, Patch B: unowned scheduler-failure observability | Proposed companion plan |

## State

The implementation record is
[`concurrency-control-implementation-state.md`](concurrency-control-implementation-state.md).
