# Typing, Gradual Boundaries, and Proofs

## Type facts seed logic

If checker proves:

```text
x : Int
```

solver can use `x` in integer theory. If `x : Option<Int>`, constructor/tag constraints become available.

Type checking should happen before/with VC generation so ill-formed logical terms are avoided.

## Refinement

Path/proof facts can strengthen a base type:

```text
x : Int
path: x > 0
```

Do not mutate canonical `Int` into a new global type unless refinement types are explicitly part of type representation.

## Dynamic

A dynamic value may not support static proof of selector behavior/value predicates. Options:

- require runtime contract/check;
- return `Unknown(DynamicBoundary)`;
- demand explicit narrowing/cast/protocol evidence.

Never assume arbitrary dynamic operations satisfy desired proposition.

## `Any`/unknown

Top/unknown checker states must map to solver sorts carefully. An unconstrained mathematical value of one sort is not equivalent to a runtime object of arbitrary class.

## Casts/runtime validation

A proven runtime type check refines subsequent path. A cast that can fail adds an exceptional/contract-failure branch.

## Type soundness dependency

If prover assumes checker subtyping/conformance, a bug in those relations can make proofs unsound. Keep type relation tests and proof integration separate.

## Protocol contracts

A protocol can supply callable type contracts without a concrete target. This is ideal for modular proof: prove call through the protocol's guaranteed contract, independent of runtime override.

## Typed runner

If static proof is `Unknown`, typed-runner may validate the obligation on executed paths. Runtime success is evidence for that run, not a static proof for all inputs.

---

## Deep treatment: typing evidence and logical refinement

### Evidence categories

The prover should know why a type fact holds:

```text
Declared
InferredSound
FlowRefined
RuntimeChecked(path-local)
ProtocolGuaranteed
HeuristicIDE
Unknown
```

Only evidence classes with soundness guarantees for the current mode may constrain proof semantics.

### Refinement as proposition, not canonical type mutation

Maintain:

```text
Base type: Int
Path facts: x > 0 ∧ x < 10
```

rather than globally interning a new `IntBetween1And9` unless refinement types are a ratified language concept. This prevents proof-path facts from leaking into canonical type identity and specialization caches.

### Dynamic boundary rule

If a value has dynamic language type or semantically unconstrained runtime object behavior, an operation can still become provable after runtime validation:

```text
if x.is(Int) {
  // success path has evidence x : Int
}
```

The failed type-test path remains separate. A cast that raises must introduce exceptional control.

### `Any` versus unknown analysis

A language-level `Any`/dynamic top may intentionally permit operations checked at runtime. “Not yet inferred because dependency missing” is an analysis state, not a source type. These must not encode to the same proof sort or user diagnostic.

### Subtyping dependency

If proof assumes `T <: U` to obtain protocol contract `U`, the subtype relation is part of the proof TCB/dependency graph. A checker bug can create a false proof. Keep relation tests independent and include type-system revision in proof cache fingerprints when relevant.
