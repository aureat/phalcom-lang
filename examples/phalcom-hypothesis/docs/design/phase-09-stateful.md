# Phase 09 Design — Rule-based stateful testing

## Decision

Stateful testing is implemented as one strategy and target pair over the existing property search kernel. It does not own randomness, replay, persistence, failure identity, or shrinking.

```text
StateMachineMetadata
        │
        ▼
_StatefulScenarioStrategy.draw(DrawData)
        │ incrementally creates and executes
        ▼
StateScenario + shared semantic Example
        │
        ▼
SearchEngine → shared Shrinker → Reporter → ExampleDatabase
```

## Immutable descriptions

The authoritative descriptive model consists of:

- `Bundle<T>` descriptors;
- sealed `RuleArgument` variants for strategy draws, selections, and consuming selections;
- `RuleDefinition` and `StateMachineMetadata`;
- `ResultReference`, `LiteralArgument`, and `ReferenceArgument`;
- `StateAction` and `StateScenario`.

These values are copied or immutable. Rule kinds and argument kinds are variants, not string tags.

## Mutable evaluation workers

Every draw constructs a fresh machine and `_StatefulContext`. The context owns:

- actual values indexed by result-reference id;
- per-bundle lists of references;
- consumed-reference removal;
- stable result-name counters;
- the partial action trace.

No state is shared across examples.

## Discovery

`_StatefulDiscovery` sorts reflected selectors before building metadata. It validates duplicate roles, `@When` placement, predicate existence/arity/reflected `Bool` results, reflected rule arity, bundle sources, typed bundle/parameter relationships, duplicate targets, typed target/result relationships, invariant arity, teardown arity, and the existence of at least one normal rule.

Attributes remain passive. Discovery never rewrites a method or changes dispatch.

## Execution order

One evaluation performs:

1. initializers in stable selector order;
2. all invariants once;
3. a bounded normal-step loop;
4. applicability calculation from `@When` and sequential bundle cardinality;
5. rule selection only from the applicable list;
6. argument draws and reference resolution;
7. action recording before invocation;
8. result publication;
9. all invariants after each normal rule;
10. one teardown attempt after execution completes or fails.

No applicable rule ends the scenario successfully. It is not a rejected example.

## Teardown and failures

Execution and teardown are captured as two `Result` values. This guarantees one teardown attempt for every evaluation which begins. Stateful wrappers retain:

- the primary error;
- the partial immutable scenario;
- an optional secondary teardown error.

User failures delegate `failureOrigin` to the primary assertion or invariant. Overrun and rejection wrappers retain their search-control classification.

## Structural shrinking

The semantic span tree is:

```text
#stateScenario (not discardable)
├── #stateInitializer* (not discardable)
└── #stateSteps (not discardable)
    ├── #length choice
    └── #stateAction* (discardable)
```

The dedicated `#stateSteps` parent prevents initializer-local collection lengths from being mistaken for the normal action count. Deleting an action uses the existing `_DeleteDiscardableSpans` pass and decrements the shared `#length` choice. Dependency-breaking candidates become invalid replay overruns and are ignored by the shared shrinker.

## Reproduction and persistence

Scenarios render stable assignment names such as `key1`. Literal arguments print values; reference arguments print names. Console and JSON reporters inspect the stateful wrapper through its scenario protocol.

`Stateful.check` derives a `DatabaseKey` using the stateful metadata fingerprint and `ExampleCodec.engineFormatVersion`, then applies the same reuse/save/stale-delete policy as property runs.
