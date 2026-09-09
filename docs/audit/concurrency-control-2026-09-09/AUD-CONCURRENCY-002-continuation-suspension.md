# AUD-CONCURRENCY-002 — Pending continuations settle on suspension

## Classification

- Severity: High
- Category: Concurrency control, runtime correctness
- Confidence: Confirmed by live source and executable probes
- Audited HEAD: `b84da68fededc4b9e5b6e4841f0e1cb74ffcf7ad`
- Runtime impact: Pending then/map/catch fulfill on callback suspension.
- Status: Reproduced; not fixed. No implementation changes in this audit.

## Contract and implementation

**Reproducer:** [08-continuations.ph](evidence/probes/08-continuations.ph), [stdout](evidence/probes/08-continuations.stdout). It registers pending then/map and a pending rejected-path catch, each awaiting a shared gate. All three print Some(None) before and after the gate settles.

```phalcom
const source = Future.new()
const gate = Future.new()
const output = source.then |v| { gate.await; 42 }
source.settleValue(1)
System.runScheduled()
System.print(output.value) // actual Some(None)
gate.settleValue(2)
System.runScheduled()
System.print(output.value) // still Some(None)
```

**Expected:** the continuation Future must remain pending while its callback is suspended, then adopt its terminal result/error.

**Exact paths:** [then 298–308](../../../phalcom-core/core/universe/src/concurrency/fiber.ph#L298), [map 327–337](../../../phalcom-core/core/universe/src/concurrency/fiber.ph#L327), [catch 357–367](../../../phalcom-core/core/universe/src/concurrency/fiber.ph#L357). Each constructs a callback Fiber, receives one try outcome, tests only `error`, then flattens it. `flatten(None)` becomes a fulfilled Future; it has no lifecycle information.

**Scope:** same control-observation gap as D1, replicated at three library sites. The already-settled branch directly calls the callback on its caller's stack; it has no one-shot Fiber outcome discriminator. Ordinary direct block calls can suspend there; actual native boundaries still prohibit it. `flatten` alone merely returns a Future unchanged or wraps a plain value and does not itself switch Fibers.

## Recommended Direction and Verification Criteria

The derived Future settles only after its callback reaches terminal completion, including adoption of a returned Future. Preserve the demonstrated manual coroutine semantics, Error identity, native-frame ownership guards, and close-before-discard upvalue handling. Recommended direction is bounded to control-transfer correctness; the [executive report](concurrency-control-executive-report.md#minimal-architectural-conclusions) records cross-cutting constraints.

## Evidence and Limitations

Probe stems: `08-continuations` under [evidence/probes](evidence/probes/). Source and their `.stdout`, `.stderr`, `.exit` files retain actual behavior. A successful diagnostic process means the observation completed, not that the defect is repaired. The focused concurrency corpus passed while these new probes expose missing coverage. No workspace release claim or general VM/exception/GC proof is made.

## Related Findings and Open Questions

See the [control model](concurrency-control-executive-report.md#actual-control-model), [async deep dive](concurrency-control-executive-report.md#futureasync-deep-dive), and [ambiguous semantics](concurrency-control-executive-report.md#ambiguous-semantics). Any repair must agree with the wake and completion contracts across all six findings; a local status check is not sufficient for the completion-observer problem.
