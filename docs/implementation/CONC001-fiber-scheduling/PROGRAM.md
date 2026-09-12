---
id: CONC001
category: CONC
kind: implementation
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
---

# CONC001 — original Fiber scheduling program

CONC001 records the original implementation lineage for:

- cooperative Fiber execution;
- the first ready-queue/root-drive scheduler;
- Fiber terminal reflection;
- the original Future/await library.

It is no longer the owner of current concurrency hardening or reactor evolution.

## Current ownership handoff

| Concern | Current owner |
|---|---|
| Fiber/scheduler ownership hardening, native suspension, coroutine/executor semantics | CONC002 C1/C2 |
| reactor, worker completions, timers, external readiness | CONC002.C3 |
| concurrency standard-library completion | CONC002.C4 |
| cancellation/structured concurrency | CONC002.C5 |
| channels/select | CONC002.C6 |

The historical CONC001 plans remain useful provenance. They must not be treated as the current implementation authority where CONC002 supersedes them.
