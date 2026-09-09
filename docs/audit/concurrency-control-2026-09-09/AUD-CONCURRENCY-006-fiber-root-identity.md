# AUD-CONCURRENCY-006 — IsRoot misidentifies unstarted Fibers

## Classification

- Severity: Medium
- Category: Concurrency control, runtime correctness
- Confidence: Confirmed by live source and executable probes
- Audited HEAD: `b84da68fededc4b9e5b6e4841f0e1cb74ffcf7ad`
- Runtime impact: Unstarted non-root Fibers report isRoot.
- Status: Reproduced; not fixed. No implementation changes in this audit.

## Contract and implementation

**Reproducer:** first output of [01-coroutine.ph](evidence/probes/01-coroutine.ph).

```phalcom
const f = Fiber.new || { 1 }
System.print(f.isRoot) // true, but f is a fresh non-root Fiber
```

**Expected:** false for every non-root Fiber.

**Path/root cause:** [fiber_is_root 295–298](../../../phalcom-core/src/primitive/fiber.rs#L295) tests `resumer.is_none()`, also true before first resume. Root identity and absence of a caller are different facts.

**Scope:** local predicate defect. It does not currently make an executing ordinary Fiber take await's root branch: first resume has assigned its resumer before executing entry.

## Recommended Direction and Verification Criteria

Root identity is true only for the actual root Fiber. Preserve the demonstrated manual coroutine semantics, Error identity, native-frame ownership guards, and close-before-discard upvalue handling. Recommended direction is bounded to control-transfer correctness; the [executive report](concurrency-control-executive-report.md#minimal-architectural-conclusions) records cross-cutting constraints.

## Evidence and Limitations

Probe stems: `01-coroutine` under [evidence/probes](evidence/probes/). Source and their `.stdout`, `.stderr`, `.exit` files retain actual behavior. A successful diagnostic process means the observation completed, not that the defect is repaired. The focused concurrency corpus passed while these new probes expose missing coverage. No workspace release claim or general VM/exception/GC proof is made.

## Related Findings and Open Questions

See the [control model](concurrency-control-executive-report.md#actual-control-model), [async deep dive](concurrency-control-executive-report.md#futureasync-deep-dive), and [ambiguous semantics](concurrency-control-executive-report.md#ambiguous-semantics). Any repair must agree with the wake and completion contracts across all six findings; a local status check is not sufficient for the completion-observer problem.
