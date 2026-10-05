# Concepts

The package has one locked search model: strategies consume typed primitive choices through `DrawData`; generation records a semantic example; replay supplies the example again; shrinking transforms the example and reruns the complete test.

## Core execution values

- `Settings` is an immutable execution policy with an ordered phase list.
- `Phase` is a sealed search-stage value.
- `ExampleStatus` distinguishes valid, invalid, overrun, and interesting evaluations.
- `PropertyResult` distinguishes passed, falsified, inconclusive, and errored runs.
- `FailureOrigin` identifies the source site of a failure independently of its error class.
- `Failure` carries the origin, error, semantic example, arguments, and notes.
- `Statistics` is an immutable snapshot produced by a private mutable collector.

## Semantic examples

- `ChoiceRequest` describes a typed primitive domain and shrink target.
- `Choice` records an integer, Boolean, index, or bytes decision.
- `ChoiceBuffer` is a mutable construction worker.
- `Span` marks a nested half-open range of choices.
- `Example` is the immutable frozen sequence of normalized choices, spans, and generation size.
- `ChoiceProvider` separates primitive choice supply from strategy semantics.
- `ChoiceProviderFactory` creates a fresh provider for each generated example; the seeded system-random factory remains the default and scripted factories provide deterministic extension input.
- `DrawData` records generation and deterministic replay through the same interface.

Replay exhaustion, primitive-kind changes, current-bound violations, choice-budget exhaustion, and unclosed spans are overruns/cache-invalidating conditions, not counterexamples.

## Strategies

- `Strategy<out T>` is the structural generation protocol.
- `StrategyBase<T>` is an optional reusable implementation of the standard combinators; structural conformance never requires inheritance.
- `Gen` constructs primitive, collection, sum-type, composite, deferred, and recursive strategies.
- `map`, `filter`, `flatMap`, and `named` are ordinary strategy messages.
- `Draw` makes dependent generation explicit inside `Gen.build`.
- `StrategyRegistry` maps exact reflected built-in types to strategies.
- collection/text spans identify structural deletion units;
- recursive strategies consume scoped generation size and expose expanded payload spans.

## Search and shrinking

- `PropertySpec<T...>` is an immutable engine input.
- `SearchEngine` runs explicit, reuse, generation, shrink, and final verification phases.
- `_Evaluator` invokes the complete target through generation and replay.
- `ExampleComplexity` defines a strict deterministic ordering.
- `ShrinkPass` proposes immutable candidates.
- `Shrinker` suppresses duplicate semantic candidates, then accepts only strictly smaller candidates that preserve the failure origin.
- `find` is a value-producing search mode over the same evaluator, passes, and ordering.

No active core, choice, strategy, or engine state uses free-form semantic string tags. Invalid examples, overruns, flaky runs, and engine errors are not represented as counterexamples.
