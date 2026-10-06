# Current API

This inventory describes the active source under `src/`. The package currently
exports modules from `src/package.ph`; it does not provide the historical
monolithic `hypothesis` root facade.

## Choices

`choices.choice` exports `Choice`. `choices.request` exports `ChoiceRequest`.
`choices.example` exports `Example`, and `choices.buffer` exports
`ChoiceBuffer`. `choices.provider` exports:

- `ChoiceProvider`, the generic `ChoiceProviderFactory<P>` class, and
  provider-specific factory builders;
- `ScriptedChoiceProvider` and `ScriptedChoiceProviderFactory`;
- `ReplayChoiceProvider`;
- `SystemRandomChoiceProvider` and `SystemRandomChoiceProviderFactory`.

`ChoiceProviderFactory<P>` carries the concrete provider type through the
pipeline and stores a typed creator function. `ScriptedChoiceProviderFactory`
and `SystemRandomChoiceProviderFactory` expose a `providerFactory` property
that creates independent providers for each example. The system-random factory
derives a repeatable per-example seed from its configured seed.

`choices.data` exports `DrawData<P>`, constrained by `P |= ChoiceProvider`.
Strategies consume this typed context through a generic method and do not store
an existential provider value.

`choices.span` exports `Span` as scaffolding. The active strategy, example, and
shrinking paths do not create or consume spans.

## Strategies and property execution

`strategies.strategy` exports `Strategy<T>`. Its `draw<P>` method accepts
`DrawData<P>` for any `P` conforming to `ChoiceProvider`.
`strategies.primitives` exports `IntStrategy` and `BoolStrategy`.

`property.specification` exports `PropertySpec<T, S>`, which stores the
concrete strategy `S`, a `(T) -> Bool` predicate, and an identity `Symbol`.
`property.property` exports `Property`; `Property.check` accepts a
`PropertySpec<T, S>`, `Settings`, and a `ChoiceProviderFactory<P>` whose
concrete provider `P` conforms to `ChoiceProvider`.

`core.settings` exports `Settings(maxExamples, maxShrinkSteps)`. The shrink
limit counts accepted improvements. Seed selection belongs to the injected
provider factory. `core.result` exports `PropertyResult<T>` with `Passed`,
`Falsified`, `Flaky`, `GaveUp`, and `Errored` outcomes.

## Outcomes and counters

`Falsified` carries the minimized value, failure identity, and replay tape.
`Passed` carries only statistics. `Errored` represents provider or strategy
execution failures; a property predicate that throws is recorded as a
property failure with a distinct origin kind.

`core.statistics` exports `Statistics` with stable, separate counters:

- `generatedExamples` — examples evaluated during the search;
- `shrinkEvaluations` — candidate replay attempts;
- `acceptedShrinks` — improvements accepted by the shrinker;
- `discards` — evaluations classified as discarded;
- `verificationReplays` — final replay checks.

The current evaluator does not yet create `Discarded` outcomes, so that status
and counter are reserved for a later assumption/rejection API.

## Other active modules

`core.phase` and `core.failure` expose phase and failure identity data.
`core.status` exposes `ExampleStatus<T>`. `engine.complexity`,
`engine.shrink_pass`, and `engine.shrinker` expose the current primitive
complexity and shrinking machinery.

Historical 0.1.0 exports and release aliases are recorded in
[`../archive/legacy-0.1.0/docs/public-api.md`](../archive/legacy-0.1.0/docs/public-api.md)
when available; they are not part of this API.
