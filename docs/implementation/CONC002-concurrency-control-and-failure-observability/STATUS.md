---
id: CONC002
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
---

# CONC002 status

| Checkpoint | State | Next action |
|---|---|---|
| C1 | IN_PROGRESS / PARTIAL | finish/disposition P4 |
| C2 | IN_PROGRESS | execute published P2 (P1-R1 COMPLETE) |
| C3 | PROPOSED | P1 after C2.P2; P2 blocked on PDR-0016 |
| C4 | PROPOSED | P1 after C2.P2; P2 after P1, time portions after C3.P1 |
| C5 | roadmap | no implementation plan yet |
| C6 | roadmap | no implementation plan yet |

## Ownership corrections

- C2.P2 is already published.
- C3/C4 executable plans are already published.
- C5 owns cancellation/structured concurrency.
- C6 owns channels/select.
- the legacy `C4-concurrency-library-polish-and-composition-primitives` directory contains old `CONC002.C3.*` records and must be SUPERSEDED.
- reactor implementation ownership is CONC002.C3, not CONC001.

Publishing these records claims no new runtime verification.
