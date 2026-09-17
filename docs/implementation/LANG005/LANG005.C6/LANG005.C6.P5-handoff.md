---
id: LANG005.C6.P5
kind: implementation-handoff
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
revision: c0ede6acceaae541d3ba161a2dad980b2a1d7d1d
prepared: 2026-09-17
---

# LANG005.C6.P5 handoff

This handoff includes the advisory corrective continuation on top of
`c0ede6acceaae541d3ba161a2dad980b2a1d7d1d`; the implementation remains
uncommitted in the working tree.

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
- Built-in stateful slice: `BytesIterator`, `StringByteIterator`,
  `StringCodePointIterator`, `SetIterator<T>`, `MapIterator<K,V>`, and Map
  view iterator classes own traversal position. Direct Set and Map runtime
  tests pass, including fresh/sticky observation and Map views.
- Getter transport: source-backed `Expr::GetProperty`, setter, and static
  subscript paths preserve semantic call-entry recipes through `emit_invoke`;
  concrete Option getter lowering is `Closed`, while generic owner forwarding
  remains `Template`.
- Metadata pools: canonical Universe metadata is installed before user
  bundles, and materialized user lowering products are rebased to their actual
  runtime pool identity. A focused lowering test verifies the user recipe is
  not left at pool 0.
- Range slice: `RangeIterator` is now source-authored with iterator-owned
  state and first-`next` validation. Its runtime probe reaches the inherited
  `Option<Number>` call-entry baseline.
- Applied type-form repair: `ModuleLoweringSemantics.type_form_values` projects
  an accepted applied nominal form such as `Set<Int>` to its ordinary class
  object; the compiler consumes that product and emits the existing linked
  class-object load. The call specialization remains the owner-generic channel.
- Native constructor contract: class-side native `Set.new()` and `Map.new()`
  publish receiver-flow `Self` with constructor semantics, so contextual
  `Set<Int>`/`Set<String>` results are inferred by the semantic layer.

## Remaining work

- Continue Part II T6/T7/T8 after resolving the remaining generic runtime
  call-entry / `Option<Number>` boundary and obtaining the semantic decision
  for Tuple's publishable `Iterator::Item` type. Applied Set/Map construction,
  direct Set/Map iteration, getter transport, and metadata rebasing are
  focused-tested, but G3 is not passed.
- Do not begin T9–T11: the canonical built-in source gate is incomplete.
- Retire the compatibility cursor lowering and runtime superclass only after
  all supported sources and the relevant corpus lanes are migrated.
- Re-run the positive iteration corpus after the inherited
  `Option.unwrapOr` generic call-entry baseline is repaired or separately
  dispositioned.

The advisory decision was implemented without adding VM type inference or a
permissive template fallback. The remaining blocked Range/legacy probes
reflect an open `Option<Number>` recipe at the inline Universe-source/call-entry
boundary; preserve this as a baseline blocker rather than hiding it with raw
generic constructor syntax or selector-specific exceptions.

Do not move proof search into the compiler or VM, add a second projection
normalizer, or treat the existing cursor fallback as canonical semantic
authority.
