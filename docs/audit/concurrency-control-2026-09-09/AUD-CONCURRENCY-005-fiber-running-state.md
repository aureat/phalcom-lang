# AUD-CONCURRENCY-005 — Running conflates executing and waiting for a child

## Classification

- Severity: Medium
- Category: Concurrency control, runtime correctness
- Confidence: Confirmed by live source and executable probes
- Audited HEAD: `b84da68fededc4b9e5b6e4841f0e1cb74ffcf7ad`
- Runtime impact: Executing and active-ancestor waiting share Running status.
- Status: Reproduced; not fixed. No implementation changes in this audit.

## Contract and implementation

**Executable reproducer:** [10-state-inspection.rs](evidence/probes/10-state-inspection.rs), [output](evidence/probes/10-state-inspection.stdout). It installs a read-only inspection primitive into a probe-only class, executes it inside a child Fiber, and checks heap status and buffers.

**Actual:** `Running statuses: 2; current buffers empty; parent buffers parked`.

**Expected:** either Running uniquely denotes CPU execution, or a distinct active/waiting state explicitly represents the ancestor's ownership and non-resumability. Current field documentation and candidate single-Running invariant are false.

**Path/root cause:** `fiber_resume` at [440–466](../../../phalcom-core/src/primitive/fiber.rs#L440) stores the caller but never changes its status; `store_live_into` only transfers buffers. This preserves ancestor non-reentrancy incidentally through Running rejection.

**Scope:** runtime state representation. **Do not repair by blindly setting caller Suspended:** that would permit resuming an active ancestor and replacing its child-call continuation while the child still owns it. Single execution itself is preserved by `VM.current`.

## Recommended Direction and Verification Criteria

Executing ownership and active-ancestor non-resumability are represented coherently. Preserve the demonstrated manual coroutine semantics, Error identity, native-frame ownership guards, and close-before-discard upvalue handling. Recommended direction is bounded to control-transfer correctness; the [executive report](concurrency-control-executive-report.md#minimal-architectural-conclusions) records cross-cutting constraints.

## Evidence and Limitations

Probe stems: `10-state-inspection` under [evidence/probes](evidence/probes/). Source and their `.stdout`, `.stderr`, `.exit` files retain actual behavior. A successful diagnostic process means the observation completed, not that the defect is repaired. The focused concurrency corpus passed while these new probes expose missing coverage. No workspace release claim or general VM/exception/GC proof is made.

## Related Findings and Open Questions

See the [control model](concurrency-control-executive-report.md#actual-control-model), [async deep dive](concurrency-control-executive-report.md#futureasync-deep-dive), and [ambiguous semantics](concurrency-control-executive-report.md#ambiguous-semantics). Any repair must agree with the wake and completion contracts across all six findings; a local status check is not sufficient for the completion-observer problem.
