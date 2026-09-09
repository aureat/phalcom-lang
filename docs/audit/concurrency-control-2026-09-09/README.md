# Concurrency-control audit — 2026-09-09

**Finding: the manual coroutine substrate works, but Fiber/Future/Scheduler composition is not internally sound.** Suspension is treated as completion, wake ownership is unenforced, and changing resumers loses terminal observers.

Start with the [executive report](concurrency-control-executive-report.md), which includes the actual state machine, confirmed semantics, failure cascade, native boundaries, async deep dive, invariant verdicts, and prioritized next work.

| Finding | Severity / confidence | Confirmed defect |
| --- | --- | --- |
| [AUD-CONCURRENCY-001](AUD-CONCURRENCY-001-async-completion-observer.md) | High / executed | Incorrect early settlement and lost terminal observation |
| [AUD-CONCURRENCY-002](AUD-CONCURRENCY-002-continuation-suspension.md) | High / executed | Pending then/map/catch fulfill on callback suspension |
| [AUD-CONCURRENCY-003](AUD-CONCURRENCY-003-waiter-wake-ownership.md) | High / executed | Unauthorized wakes and stale registrations cross wait boundaries |
| [AUD-CONCURRENCY-004](AUD-CONCURRENCY-004-stale-ready-queue.md) | High / executed | Duplicate/stale entries abort unrelated scheduled work |
| [AUD-CONCURRENCY-005](AUD-CONCURRENCY-005-fiber-running-state.md) | Medium / executed | Executing and active-ancestor waiting share Running status |
| [AUD-CONCURRENCY-006](AUD-CONCURRENCY-006-fiber-root-identity.md) | Medium / executed | Unstarted non-root Fibers report isRoot |

[Evidence index and reproduction](evidence/README.md) contains executable probes, actual stdout/stderr/exit codes, and the focused corpus log. Findings are reproduced, not fixed. No runtime implementation changes or release certification.
