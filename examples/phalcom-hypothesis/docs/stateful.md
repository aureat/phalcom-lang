# Rule-based stateful testing

Stateful testing is a structured client of the same semantic-choice search kernel used by ordinary properties, `find`, replay, shrinking, reporting, and persistent example reuse. It does not have a second generator or a stateful-only shrink engine.

## Public surface

```phalcom
class StateMachine
class Stateful

@Rule(...)
@Initialize(...)
@StateInvariant
@When(...)
@Teardown

class Bundle<T>
```

`RuleBasedStateMachine` aliases `StateMachine`, and `Invariant` aliases `StateInvariant`, for broad-v1 compatibility. The aliases delegate to the same implementation.

## Passive metadata and discovery

Stateful attributes are retained metadata. They do not wrap methods or change ordinary dispatch. `Stateful.check` reflects the machine class, sorts selectors deterministically, validates metadata, and constructs immutable `StateMachineMetadata`.

Discovery rejects:

- a machine with no normal `@Rule` methods;
- multiple or contradictory stateful attributes on one method;
- `@When` on anything other than a normal rule;
- non-Symbol `@When` predicates, because stateful fingerprints must be stable;
- argument-source counts which do not match reflected parameters;
- duplicate bundle targets;
- bundle targets on methods without a reflected return annotation;
- parameterized invariants or teardown methods;
- more than one teardown method.

`@StateInvariant` is independent of the language-level `@invariant` contract decorator. `@When` is normal generated-program applicability, while `@requires` remains a caller/programmer contract.

## Bundles and result references

A `Bundle<T>` is a typed descriptor, not storage:

```phalcom
const Keys = Bundle<Bytes>.new(#key)
```

Each evaluation creates fresh bundle storage. Passing the bundle as a rule argument selects a reference without removing it; `Keys.consume` selects and removes it. `Keys.publish` marks a rule return value for publication:

```phalcom
@Rule(Gen.bytes, Keys.publish)
createKey(value: Bytes) -> Bytes {
  return value
}

@Rule(Keys)
read(key: Bytes) {
  ...
}

@Rule(Keys.consume)
delete(key: Bytes) {
  ...
}
```

Scenarios retain `ResultReference` values rather than copying produced values. Later actions resolve references against the current execution. Deleting a producer during shrinking therefore makes a dependent replay invalid; the candidate is treated as an overrun/cache miss and ignored.

## Generation and execution

`_StatefulScenarioStrategy` consumes `DrawData` directly. Every draw creates a fresh machine, context, result table, and bundle table. Execution is incremental:

1. Run initializers in stable selector order.
2. Run all state invariants once after initialization completes.
3. Draw the normal-rule step budget.
4. Evaluate `@When` predicates and bundle availability.
5. Choose only among applicable rules.
6. Draw strategy and reference arguments.
7. Record and invoke the action.
8. Publish its result and run every invariant.
9. Stop successfully when no rule is applicable.

Unavailable rules do not reject examples. Ordinary strategy rejection while drawing a selected rule argument remains an invalid example.

## Teardown and failures

Execution and teardown are captured structurally, so teardown is attempted exactly once after execution begins on pass, assertion failure, invariant failure, rejection, replay invalidation, choice overrun, initializer failure, rule failure, and engine error.

When execution and teardown both fail, the execution error remains primary and the teardown error is retained as secondary context. When only teardown fails, it becomes primary.

Stateful wrappers retain the partial immutable `StateScenario`, but `failureOrigin` delegates to the underlying assertion or invariant. The ordinary shrinker therefore continues to preserve the original source-aware failure site.

## Structural shrinking

The scenario has one non-discardable parent span. Every normal action has a discardable `#stateAction` span containing:

- rule selection;
- strategy argument choices;
- bundle-reference choices;
- action-local structural choices.

The normal-rule count uses the shared `#length` choice convention. `_DeleteDiscardableSpans` can therefore delete an irrelevant middle action and decrement the count while retaining later independent actions. No stateful-specific greedy tree or prefix-only shrinker exists.

## Reproduction

A scenario renders as an executable-style program with stable result names:

```text
state = DatabaseMachine.new()
key1 = state.createKey(value: b"")
value1 = state.createValue(value: b"")
state.save(key: key1, value: value1)
state.delete(key: key1)
```

Literal arguments render their values; reference arguments render their stable names. Consuming a reference does not alter its historical name in the trace. Console and JSON reporters use the existing `Failure` and `ReportEvent` stream.

## Persistence

`Stateful.check` uses `Settings.database`, `DatabaseKey`, `ExampleCodec`, and `SearchEngine`. The strategy fingerprint includes the stateful format version, machine name, stable rule selectors, argument-source fingerprints, target bundles, applicability selectors, invariant selectors, and teardown selector. Stale or corrupt examples are cache misses, never counterexamples.
