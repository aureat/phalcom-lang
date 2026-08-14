# Review Checklist and Validation Scenarios

## Review checklist

- Concrete property named.
- May/must/sound/advisory contract named.
- Abstract order documented.
- Top and bottom distinct.
- Join laws tested.
- Transfer functions monotone or justified.
- Branch edge refinement modeled.
- Loop fixed-point strategy defined.
- Recursion/SCC strategy defined.
- Unknown calls/effects conservatively handled.
- Reflection/FFI/fibers have explicit havoc/effect policy.
- Widening reason is visible.
- Cache dependency/invalidation ownership defined.
- Incremental result equals full rebuild.
- Performance limits have conservative fallback.
- Consumer does not promote heuristic fact into proof.

## Scenario 1 — one-pass loop

Pressure: analyze loop body once and use resulting state after loop.

Expected: identify need for zero iterations + back-edge fixed point and exit-state merge.

## Scenario 2 — unknown means no effect

Pressure: unresolved dynamic call; keep all local/field facts unchanged for precision.

Expected: unknown call must conservatively affect state according to possible dynamic effects; precision cannot trump soundness.

## Scenario 3 — union cap used by checker

Pressure: reuse LSP bounded union and widen to Unknown; then checker allows all sends on Unknown.

Expected: reject. Advisory widening cannot become correctness acceptance; define checker dynamic/unknown policy separately.

## Scenario 4 — branch latest value

Pressure: after `if`, whichever branch is visited last in AST walker determines variable type.

Expected: join both reachable branch states independent of traversal order.

## Scenario 5 — recursive summary stack overflow

Pressure: call analyzer recursively enters callee and cycles.

Expected: call graph/SCC fixed-point summaries with seeds and convergence.

## Scenario 6 — path explosion

Pressure: preserve every branch path for perfect precision.

Expected: bound partitions/select predicates; state precision/cost policy.

## Scenario 7 — stale cache

Pressure: cache field type keyed only by `ClassId`, no revision.

Expected: dependency/revision invalidation; otherwise editor/checker lies after edit.

## Scenario 8 — may vs must confusion

Pressure: one predecessor initializes `x`, one does not; mark definitely initialized because initialization is possible.

Expected: must-property merge requires fact on every reachable predecessor.

## Scenario 9 — native trust

Pressure: assume Rust primitive has no side effects because code unavailable to analyzer.

Expected: use explicit trusted native summary or conservative effects.

## Scenario 10 — heuristic optimizer

Pressure: LSP confidence says receiver is probably String, devirtualize send.

Expected: optimizer requires a sound guard/proof/runtime check; heuristic editor fact is insufficient.
