# AUD-CONCURRENCY-004 — Stale ready entries abort the pump

## Classification

- Severity: High
- Category: Concurrency control, runtime correctness
- Confidence: Confirmed by live source and executable probes
- Audited HEAD: `b84da68fededc4b9e5b6e4841f0e1cb74ffcf7ad`
- Runtime impact: Duplicate/stale entries abort unrelated scheduled work.
- Status: Reproduced; not fixed. No implementation changes in this audit.

## Contract and implementation

**Reproducers:** [06-duplicate-queue.ph](evidence/probes/06-duplicate-queue.ph) uses manual scheduling plus Future settlement; [11-root-drive-stale.ph](evidence/probes/11-root-drive-stale.ph) uses automatic root drive. Both exit 70 before a healthy later task runs. See their `.stdout`, `.stderr`, and `.exit` siblings.

```phalcom
const f = Fiber.new || { System.print("once"); 1 }
System.schedule(f)
System.schedule(f)
System.schedule(|| { System.print("must still run") })
System.runScheduled()
```

**Actual:** body executes once, then the second resume of Done fails; healthy later work is not executed in that run. `System.schedule` also accepts a currently Running Fiber; probe 12 queues itself and later retrieves its now-Done handle.

**Expected:** duplicate/stale wake requests must not abort unrelated scheduled work or resume a different suspension than they originally authorized. A clear rejection or stale-entry policy is needed.

**Paths:** [enqueue 51–63](../../../phalcom-core/src/primitive/system.rs#L51), [dequeue 74–79](../../../phalcom-core/src/primitive/system.rs#L74), [manual pump 55–63](../../../phalcom-core/core/universe/src/concurrency/fiber.ph#L55), [automatic pump 700–703](../../../phalcom-core/src/vm/dispatch.rs#L700). None owns a queued-state reservation. `try` captures failure *inside a validly resumed Fiber*; it does not capture an error validating its own invocation. Root-await has the same dequeue/try sequence.

**Scope:** queue validation and wake-ownership policy; terminal filtering alone misses duplicates that encounter another Suspended state.

## Recommended Direction and Verification Criteria

Stale queue entries cannot abort healthy later work or consume another suspension. Preserve the demonstrated manual coroutine semantics, Error identity, native-frame ownership guards, and close-before-discard upvalue handling. Recommended direction is bounded to control-transfer correctness; the [executive report](concurrency-control-executive-report.md#minimal-architectural-conclusions) records cross-cutting constraints.

## Evidence and Limitations

Probe stems: `06-duplicate-queue, 11-root-drive-stale, 12-resume-validation` under [evidence/probes](evidence/probes/). Source and their `.stdout`, `.stderr`, `.exit` files retain actual behavior. A successful diagnostic process means the observation completed, not that the defect is repaired. The focused concurrency corpus passed while these new probes expose missing coverage. No workspace release claim or general VM/exception/GC proof is made.

## Related Findings and Open Questions

See the [control model](concurrency-control-executive-report.md#actual-control-model), [async deep dive](concurrency-control-executive-report.md#futureasync-deep-dive), and [ambiguous semantics](concurrency-control-executive-report.md#ambiguous-semantics). Any repair must agree with the wake and completion contracts across all six findings; a local status check is not sufficient for the completion-observer problem.
