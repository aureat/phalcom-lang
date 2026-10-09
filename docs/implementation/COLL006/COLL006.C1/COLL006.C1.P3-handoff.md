---
id: COLL006.C1.P3
program: COLL006
checkpoint: COLL006.C1
kind: handoff
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# COLL006.C1.P3 — Handoff to P4

P3 is complete for its bounded implementation surface. Continue with
`COLL006.C1.P4` in the same worktree.

## Stable interfaces

- `SequenceLiteralResolution` remains the sole checked family/cardinality product.
- `SequencePublication` owns query-local staged construction products and never
  exports pending owner state.
- Vector uses canonical `Vector<T,N>` identity with exact static `Int` extent.
- Vector runtime storage is immutable native dense storage; Array runtime storage
  is fixed extent with mutable ordinary elements and frozen runtime defense.
- Array source operations now expose typed index/replacement/default contracts.
- Bare brackets still default to List until P4 performs the atomic semantic
  cutover.

## Required P4 inheritance

1. Do not re-open P3 publication or Vector runtime architecture.
2. Reuse `StaticValueStore`, static constant owner/reference paths, and semantic
   family products; do not add a second solver or static-value authority.
3. Treat D1/D2 broad metadata/reflection records as deferred baseline evidence.
4. Begin with P4 T0 and verify the current branch/worktree before editing.

## Verification anchor

The P3 focused gates are recorded in the checkpoint and walkthrough. The current
worktree contains the implementation changes and is not committed or pushed.
