---
id: CONC002
category: CONC
kind: implementation
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
---

# CONC002 — concurrency control and failure observability

This program owns the corrective concurrency work that establishes truthful
Fiber control transfer, exclusive scheduler/await ownership, durable terminal
completion, and reporting for unowned scheduler failures.

## Native suspension follow-on

[C2 — Native suspension and reactor groundwork](C2-native-suspension-and-reactor-groundwork/CHECKPOINT.md)
plans VM-visible protected execution, cleanup and native control continuations,
with preserved parking identity and a future external-readiness boundary.
The checkpoint is proposed and runtime-unverified; it does not implement a reactor.

## Library polish and composition follow-on

[C3 — Concurrency library polish and composition primitives](C3-concurrency-library-polish-and-composition-primitives/CHECKPOINT.md)
splits existing Fiber/Future and helper polish (P1) from additional completion,
composition, time, task and channel primitives (P2). It is a bounded enhancement
checkpoint, not a full standard-library implementation. Runtime prerequisites and
cross-plan integration gates remain explicit; both plans are proposed.
