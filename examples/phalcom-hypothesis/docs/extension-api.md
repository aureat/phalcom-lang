# Extension API

Phase 11 stabilizes five structural extension boundaries:

```text
ChoiceProvider
Strategy<out T>
ShrinkPass
ExampleDatabase
Reporter
```

Extensions participate in the existing `DrawData`, `SearchEngine`, `Shrinker`, persistence, and reporting pipelines. They do not replace those pipelines.

## Provider contract

```phalcom
protocol ChoiceProvider {
  choose(request: ChoiceRequest) -> Choice
  consumedChoices -> Int
}
```

A provider returns exactly one choice for each request. The returned value must be normalized to the current request: current bounds, size domain, shrink target, and label replace stale metadata from any source choice. Providers never shrink, open spans, interpret strategies, or classify property outcomes.

`SystemRandomChoiceProvider` supplies values from `Random`. `ScriptedChoiceProvider` consumes a deterministic list of choices and uses the same `_ChoiceNormalization` path as replay. Script exhaustion, type mismatch, and out-of-domain values are engine overruns rather than counterexamples.

The engine integration helper is:

```phalcom
protocol ChoiceProviderFactory {
  create(exampleIndex: Int, generationSize: Int) -> ChoiceProvider
}
```

`Settings.choiceProvider(factory:)` installs a factory. A fresh provider is created for each generated example. Without an override, `Settings` creates `SystemRandomProviderFactory` from the resolved seed, preserving ordinary deterministic seed behavior.

## Strategy contract

```phalcom
protocol Strategy<out T> {
  draw(data: DrawData) -> T
  map<U>(transform: [T] -> U) -> Strategy<U>
  filter(predicate: [T] -> Bool) -> Strategy<T>
  flatMap<U>(transform: [T] -> Strategy<U>) -> Strategy<U>
  named(label: Symbol) -> Strategy<T>
  label -> Option<Symbol>
  fingerprint -> String
}
```

Structural conformance does not require inheritance. `StrategyBase<T>` is the supported reusable base for extensions that want the standard `map`, `filter`, `flatMap`, `named`, default label, and default fingerprint behavior.

A strategy may only consume primitive choices through `DrawData`. It must not own randomness, replay cursors, database access, reporting, or shrink acceptance policy. Fingerprints must be deterministic and include every configuration value that can change generated examples.

## Shrink-pass contract

```phalcom
protocol ShrinkPass {
  name -> Symbol
  candidates(current: Example) -> List<Example>
}
```

A shrink pass proposes immutable candidate examples in deterministic order. It does not replay them and does not decide whether they remain interesting. `Shrinker` suppresses duplicate semantic candidates, rejects equal-or-greater complexity, reruns the complete target, ignores invalid or overrun candidates, and preserves the original `FailureOrigin`.

Custom passes are installed with `Shrinker.new(passes:)`. Pass order is observable because the first accepted candidate restarts the ordered pipeline.

## Database contract

```phalcom
protocol ExampleDatabase {
  fetch(key: DatabaseKey) -> List<Example>
  save(
    key: DatabaseKey,
    example: Example,
    failureOrigin: Option<FailureOrigin>
  ) -> ExampleDatabase
  delete(key: DatabaseKey, example: Example) -> ExampleDatabase
}
```

Adapters return isolated list containers, deduplicate semantic examples, preserve newest-first order, and enforce their declared retention bound. Missing data, stale formats, corruption, oversized payloads, and recoverable storage failures are cache misses. They never become property failures.

`MemoryDatabase` and `DirectoryDatabase` are reference implementations. Directory writes use bounded encoding, flush-before-close, atomic replacement, a shared process-local path lock, and merge-on-write from the latest visible record set. Overlapping same-process writes fail closed rather than replacing a concurrent operation. True cross-process exclusion still requires a standardized runtime file-lock primitive; atomic replacement remains the portable integrity guarantee.

The codec is an implementation detail, not the extension boundary. Custom adapters exchange `Example` values, not encoded strings.

## Reporter contract

```phalcom
protocol Reporter {
  handle(event: ReportEvent) -> None
}
```

Delivery is synchronous, deterministic, and ordered. Each event is delivered exactly once to each child of `CompositeReporter`, in child-list order. Composite delivery is fail-fast: when a child throws, later children do not receive that event and the checked boundary classifies the exception immediately. Reporters may retain state but must not mutate immutable events.

Property execution wraps reporters in `_CheckedReporter`. The first extension exception becomes public `ReporterFailure`, the property result is `Errored`, and later events for that failed reporter are suppressed. A suite-lifecycle failure before property execution propagates as `ReporterFailure` because no property result exists yet. Reporter exceptions are never captured as falsifying user examples.

## Conformance matrix

| Boundary | Reference implementations | Required evidence |
|---|---|---|
| Provider | system-random, scripted, replay | request normalization, consumption count, exhaustion, seeded/scripted overlap |
| Strategy | standard strategies, `StrategyBase` custom strategy | deterministic draw behavior, combinators, fingerprint |
| Shrink pass | seven standard passes, custom fixture | immutable candidates, deterministic order, duplicate suppression, strict complexity |
| Database | memory, directory | copy isolation, deduplication, retention, deletion, corruption/cache-miss policy |
| Reporter | null, recording, composite, console, JSON | ordered exactly-once delivery and extension-failure classification |

The focused fixtures live under `tests/conformance/`. `scripts/verify_phase11_mutations.py` removes one guarantee at a time in temporary tree copies and requires the Phase 11 verifier to reject every mutation.

## Performance boundary

Phase 11 removes four structural hot spots without changing observable semantics:

- nested label, generation-size, and span stacks pop their tail in place;
- closed spans are stored by identifier, making freeze ordering linear;
- example and database signatures use part lists plus `join` rather than repeated concatenation;
- the shrinker replays each unique candidate signature at most once per current example.

Deterministic benchmark workloads cover primitive generation, nested lists, replay, integer shrinking, and stateful shrinking under `benchmarks/`. They expose workloads but do not fabricate timing results. Runtime measurements are valid only when run by a real Phalcom toolchain.
