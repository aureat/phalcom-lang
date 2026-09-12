---
id: CONC002.C2
program: CONC002
kind: corrective-implementation
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# CONC002.C2 — Native suspension and reactor groundwork

Make native language-control continuations representable across Fiber suspension,
including protected execution, cleanup and non-local return. Preserve synchronous
leaf primitives and guards on retained host continuations. Reuse ticketed parking
and deferred scheduler admission as the future external-readiness boundary.

Depends on the current C1 ownership/continuation work; does not implement a reactor.

| Plan | Scope | State |
|---|---|---|
| [CONC002.C2.P1](CONC002.C2.P1-native-suspension-and-reactor-groundwork.md) | Source inventory, VM control stack, outcome router, native migrations, GC, readiness and adversarial gates S0–S7 | Proposed; source inspected, runtime unverified |
| [CONC002.C2.P1-R1](CONC002.C2.P1-R1-native-suspension-and-reactor-groundwork.md) | Expanded patch-grade revision: 34 tasks across nine semantic gates, explicit activation outcomes, host escape, parent failure injection, lifetime and delivery evidence | Proposed; P1 preserved unchanged; runtime unverified |
