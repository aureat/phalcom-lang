---
id: LANG005.C6.P5
kind: implementation-handoff
status: IN_PROGRESS
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
revision: 06e37b6830cd9114d03f2a794d0b9a973f8cdc82
prepared: 2026-09-17
---

# LANG005.C6.P5 handoff

Part I establishes the canonical evidence-driven iteration seam and a focused
production vertical for `List`. It does not complete collection migration or
cursor retirement.

## Stable interfaces

- Semantic product: `CallableAnalysis.for_iteration_lanes`, using
  `ForIterationLaneResolution` and
  `ForIterationRequirementResolution` in
  `phalcom-semantic/src/checker/analysis.rs`.
- Lowering product: `ModuleLoweringSemantics.for_iteration_lanes`, using
  `ForIterationLoweringSpec` and `ForIterationRequirementSpec` in
  `phalcom-core/src/modules/semantic_lowering.rs`.
- Compiler boundary: `emit_for_iteration_requirement` emits the existing
  selected-trait or requirement-slot VM operations; it does not prove,
  resolve, or normalize semantic facts.
- Universe vertical: `ListIterator<T>` in
  `phalcom-core/core/universe/src/collections/list.ph` has an independent
  index, fresh construction, and sticky `None` exhaustion.
- Strict zip error: `ZipLengthMismatchError` is registered under
  `universe.errors.iteration` and used by canonical parallel lowering.
- Built-in stateful slice: `BytesIterator`, `StringByteIterator`, and
  `StringCodePointIterator` own traversal position and pass the focused
  canonical surface test. `SetIterator<T>`, `MapIterator<K,V>`, and Map view
  iterator classes are source-authored but their runtime constructor probes
  remain blocked by the generic call-entry recipe failure.

## Remaining work

- Continue Part II T6/T7/T8 only after resolving the generic runtime
  call-entry / `Option<Number>` boundary and obtaining guidance for Tuple's
  publishable `Iterator::Item` type. G3 is not passed.
- Do not begin T9–T11: the canonical built-in source gate is incomplete.
- Retire the compatibility cursor lowering and runtime superclass only after
  all supported sources and the relevant corpus lanes are migrated.
- Re-run the positive iteration corpus after the inherited
  `Option.unwrapOr` generic call-entry baseline is repaired or separately
  dispositioned.

Do not move proof search into the compiler or VM, add a second projection
normalizer, or treat the existing cursor fallback as canonical semantic
authority.
