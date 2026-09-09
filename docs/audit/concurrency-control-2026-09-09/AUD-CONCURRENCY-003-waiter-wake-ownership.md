# AUD-CONCURRENCY-003 — No exclusive wake authority or wait validation

## Classification

- Severity: High
- Category: Concurrency control, runtime correctness
- Confidence: Confirmed by live source and executable probes
- Audited HEAD: `b84da68fededc4b9e5b6e4841f0e1cb74ffcf7ad`
- Runtime impact: Unauthorized wakes and stale registrations cross wait boundaries.
- Status: Reproduced; not fixed. No implementation changes in this audit.

## Contract and implementation

**Reproducer:** [05-wrong-authority.ph](evidence/probes/05-wrong-authority.ph), [stdout](evidence/probes/05-wrong-authority.stdout). Covers manual Try, manual Call, scheduling while still pending, old waiter waking a newer await, and later waiter advancing a coroutine yield.

```phalcom
const a = Future.new()
const b = Future.new()
const f = Fiber.new || {
  System.print(a.await)
  System.print(b.await)
  System.print("past both waits")
}
f.call()
f.try()              // illegally passes pending a.await, returning None
System.print(a.isReady) // false
a.settleValue(1)
System.runScheduled()  // stale a waiter passes pending b.await
System.print(b.isReady) // false
```

**Actual:** all three resume APIs are allowed. A manual value is delivered to the yield expression but ignored by await; `_value` is returned even while still pending, hence None. The old Future retains the Fiber, later schedules it, and wakes whichever suspension it now occupies. A second stale waiter can advance it again. The body is not restarted from entry; unrelated suspension points are consumed prematurely.

**Expected:** pending await must not produce a value; only its current wait's authorized wake may resume it. Cancelled/superseded registrations must not act on another wait.

**Path/root cause:** [await 227–254](../../../phalcom-core/core/universe/src/concurrency/fiber.ph#L227) registers raw current Fiber, yields once, then falls through without a loop/revalidation. [resume](../../../phalcom-core/src/primitive/fiber.rs#L387) checks only status/native depth. [drain](../../../phalcom-core/core/universe/src/concurrency/fiber.ph#L186) checks only terminal status. No wait identity, queued reservation, or authority token exists.

**Scope:** architectural ownership gap with local missing post-wake readiness check. A loop alone would stop premature reads but would not remove stale registrations or duplicate wake sources.

## Recommended Direction and Verification Criteria

Only the current wait owner may wake a parked Fiber; stale entries cannot act on a newer suspension. Preserve the demonstrated manual coroutine semantics, Error identity, native-frame ownership guards, and close-before-discard upvalue handling. Recommended direction is bounded to control-transfer correctness; the [executive report](concurrency-control-executive-report.md#minimal-architectural-conclusions) records cross-cutting constraints.

## Evidence and Limitations

Probe stems: `05-wrong-authority` under [evidence/probes](evidence/probes/). Source and their `.stdout`, `.stderr`, `.exit` files retain actual behavior. A successful diagnostic process means the observation completed, not that the defect is repaired. The focused concurrency corpus passed while these new probes expose missing coverage. No workspace release claim or general VM/exception/GC proof is made.

## Related Findings and Open Questions

See the [control model](concurrency-control-executive-report.md#actual-control-model), [async deep dive](concurrency-control-executive-report.md#futureasync-deep-dive), and [ambiguous semantics](concurrency-control-executive-report.md#ambiguous-semantics). Any repair must agree with the wake and completion contracts across all six findings; a local status check is not sufficient for the completion-observer problem.
