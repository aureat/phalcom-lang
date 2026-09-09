# AUD-CONCURRENCY-001 — Async settles on suspension

## Classification

- Severity: High
- Category: Concurrency control, runtime correctness
- Confidence: Confirmed by live source and executable probes
- Audited HEAD: `b84da68fededc4b9e5b6e4841f0e1cb74ffcf7ad`
- Runtime impact: Incorrect early settlement and lost terminal observation.
- Status: Reproduced; not fixed. No implementation changes in this audit.

## Contract and implementation

**Reproducer:** [03-async.ph](evidence/probes/03-async.ph), [stdout](evidence/probes/03-async.stdout), [trace](evidence/probes/03-async.stderr). The first part extends the supplied reproducer to three awaits; the second fails after a later wake.

Minimal executable form:

```phalcom
const input = Future.new()
const output = Future.async || { input.await; 42 }
System.runScheduled()
System.print(output.isReady) // actual true; required false
input.settleValue(1)
System.runScheduled()
System.print(output.value)   // actual Some(None); required Some(42)
```

**Actual:** first pump fulfills with None. Three sequential settles do execute A/B/C and reach terminal return, but output remains Some(None). A later uncaught failure also leaves a successful Some(None).

**Expected:** pending through every nonterminal suspension; settle with actual terminal success or failure.

**Path/root cause:** [async lines 258–271](../../../phalcom-core/core/universe/src/concurrency/fiber.ph#L258) -> action.try -> await's bare yield -> driver's error-only test -> settleValue. “No captured failure” is incorrectly treated as “completed successfully.” `resumer` is the only terminal route, and scheduler wake replaces it.

**Scope:** local bad discriminator plus architectural completion-observation gap; not fixed by a single conditional. See deep dive.

## Recommended Direction and Verification Criteria

The outer Future stays pending through every suspension and receives exactly one terminal result or error. Preserve the demonstrated manual coroutine semantics, Error identity, native-frame ownership guards, and close-before-discard upvalue handling. Recommended direction is bounded to control-transfer correctness; the [executive report](concurrency-control-executive-report.md#minimal-architectural-conclusions) records cross-cutting constraints.

## Evidence and Limitations

Probe stems: `03-async, 09-guard-only` under [evidence/probes](evidence/probes/). Source and their `.stdout`, `.stderr`, `.exit` files retain actual behavior. A successful diagnostic process means the observation completed, not that the defect is repaired. The focused concurrency corpus passed while these new probes expose missing coverage. No workspace release claim or general VM/exception/GC proof is made.

## Related Findings and Open Questions

See the [control model](concurrency-control-executive-report.md#actual-control-model), [async deep dive](concurrency-control-executive-report.md#futureasync-deep-dive), and [ambiguous semantics](concurrency-control-executive-report.md#ambiguous-semantics). Any repair must agree with the wake and completion contracts across all six findings; a local status check is not sufficient for the completion-observer problem.
