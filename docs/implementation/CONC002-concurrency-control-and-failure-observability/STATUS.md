---
id: CONC002
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
---

# CONC002 status

## Current position

CONC002's foundational control-transfer remediation is implemented, but the program is not complete.

| Checkpoint | State | Next action |
|---|---|---|
| C1 — corrected concurrency foundation | **IN_PROGRESS / PARTIAL** | P3 recertified; execute or disposition P4 type prerequisites |
| C2 — VM suspension and coroutine semantics | **PROPOSED / NOT_STARTED** | Execute P1-R1 after C1 handoff; produce P2 afterward |
| C3 — reactor/external readiness | **planned restructuring** | Produce new plans after C2 contracts are fixed |
| C4 — concurrency standard library | **planned restructuring** | Extract/reissue valid library material from superseded old C3 plans |
| C5 — cancellation/structured concurrency | roadmap | Plan only after C2/C3 ownership is stable |
| C6 — channels/select | roadmap | Plan only after cancellation/multi-wait prerequisites are stable |

## C1 status detail

- P1 Fiber/scheduler/Future ownership remediation: **COMPLETE / IMPLEMENTED / BASELINE_BLOCKED**.
- P2 E010 detached scheduler-failure observability: **COMPLETE / IMPLEMENTED / FOCUSED_TESTED**.
- P3 current-main Future/type closure: **COMPLETE / IMPLEMENTED / BASELINE_BLOCKED**; freshly certified on current revision.
- P4 pre-C2 type prerequisites: proposed.

The old wording that E010 is merely “tracked as a companion plan” is obsolete; it is implemented.

## C2 ownership

`CONC002.C2.P1-R1` is the sole executable native-control plan. The predecessor `CONC002.C2.P1` is removed after link migration. C2 retains the validated park-generation/queued-admission boundary as an executor readiness contract but owns **no reactor backend**.

C2.P2 is intentionally deferred to the next planning turn. Its investigated scope is consumer/executor separation for manual coroutine pending-await plus final `Fiber<R>` typing when the type-system prerequisites are sound.

## Verification classification

No fresh full repository test run is claimed by this documentation restructuring. Historical P1/P2 evidence remains valid for the revisions on which it was recorded. P3 is the mechanism for certifying the combined current-main concurrency/type state.
