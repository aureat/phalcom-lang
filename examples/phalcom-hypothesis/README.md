# Hypothesis for Phalcom 0.1.0

Hypothesis for Phalcom is a typed, reflective property-based and rule-based stateful testing package. Strategies consume primitive choices through `DrawData`; generation freezes those choices into an immutable semantic `Example`; replay supplies the same example again; structural shrinking transforms the example and reruns the complete property.

The 0.1.0 release is a real multi-file package. It contains no historical monolith and no compatibility implementation adapter. Broad-v1 names that remain supported are direct aliases to authoritative release modules.

## Install and import

The package manifest exposes `src/hypothesis.ph` as the root module:

```phalcom
import { Given, Gen, PropertySuite, PropertyRunner, Settings } from "hypothesis"
```

See `docs/public-api.md` for the exact export inventory and `docs/migration-from-monolith.md` for prototype migration.

## Property testing

Bare `@Given` derives strategies from retained parameter annotations:

```phalcom
import { Given, PropertySuite } from "hypothesis"

class ArithmeticProperties is PropertySuite {
  @Given
  additionIsCommutative(left: Int, right: Int) {
    self.assertEqual(left + right, right + left)
  }
}
```

Explicit strategies, named overrides, settings, and mandatory cases remain available:

```phalcom
@WithSettings(Settings.standard.examples(500).seed(20260723))
@Case([], 0)
@Given(
  GivenArgs.new()
    .for(#offset, use: Gen.int(min: -10, max: 10))
)
preservesSize(values: List<Int>, offset: Int) {
  ...
}
```

The explicit builder uses the same search engine:

```phalcom
const result = Property
  .given(Gen.int, Gen.int)
  .using(Settings.standard.examples(500))
  .check { left, right =>
    Assert.equal(left + right, right + left)
  }
```

## Strategies and derivation

Built-in strategies cover primitive values, collections, tuples, options, results, dependent composite draws, and size-bounded recursion. All draws pass through `DrawData`, so generation and replay execute the same strategy code.

Typed domain models may opt into automatic derivation:

```phalcom
@arbitrary
@data
@immutable
class Point {
  const _x: Int
  const _y: Int
}

class PointProperties is PropertySuite {
  @Given
  generated(point: Point) {
    Assert.true(point.isA(Point))
  }
}
```

Sealed hierarchies derive stable variant selection. Recursive sealed models use `Gen.recursive` and require at least one terminal variant. Exact registrations and `@strategy(Type)` providers override derivation. Constructors with arbitrary contracts are rejected rather than translated into hidden rejection filters.

## Structural shrinking

The shrinker transforms immutable semantic examples rather than invoking per-strategy shrink trees. Ordered passes can:

- delete discardable collection elements and state-machine actions;
- collapse recursive payload spans;
- shorten choice sequences;
- minimize branches, integers, bytes, and text;
- preserve the original source-aware `FailureOrigin`.

Every accepted candidate must be strictly smaller by `ExampleComplexity` and reproduce the complete target. Invalid, stale, or overrun candidates are ignored.

## Observations and reporting

Properties may attach notes and aggregate events or classifications:

```phalcom
@Given
codecRoundTrips(payload: Bytes) {
  Property.note("payload size=" + payload.size.toString)
  Property.event(#codecAttempt)
  Property.classify(payload.size == 0, as: #empty)

  Assert.equal(payload, Codec.decode(Codec.encode(payload)))
}
```

The engine emits immutable `ReportEvent` values through the structural `Reporter` protocol. The release provides `NullReporter`, `RecordingReporter`, `CompositeReporter`, `ConsoleReporter`, and `JsonReporter`. Reporter exceptions cross a checked boundary as `ReporterFailure`; they never become user counterexamples.

```phalcom
const reporter = CompositeReporter.new(
  const [ConsoleReporter.standard, JsonReporter.new()]
)

const suite = PropertyRunner.run(
  const [ArithmeticProperties],
  with: Settings.standard,
  reporter: reporter
)
```

`Property.target` is intentionally absent from 0.1.0 because the engine has no evidence-backed target optimizer. `Phase.Target` remains reserved and disabled by default.

## Persistent example reuse

The package includes `MemoryDatabase` and `DirectoryDatabase`:

```phalcom
const database = DirectoryDatabase.new(
  root: ".phalcom-hypothesis/examples",
  maxEntries: 16,
  maxFileBytes: 1048576
)

const suite = PropertyRunner.run(
  const [ArithmeticProperties],
  with: Settings.standard.database(database)
)
```

`DatabaseKey` combines package, module, suite, selector, ordered strategy fingerprints, and engine format version. Directory writes use bounded records, temporary-file flush, atomic replacement, process-local path exclusion, and merge-on-write from the latest visible bucket. Corrupt, stale, oversized, or invalid records are cache misses.

## Stateful testing

Stateful tests use the same choice stream, examples, search engine, shrinker, reporters, and database:

```phalcom
const Keys = Bundle<Bytes>.new(#key)

class DatabaseMachine is StateMachine {
  @Rule(Gen.bytes, Keys.publish)
  createKey(value: Bytes) -> Bytes {
    return value
  }

  @When(#hasKeys)
  @Rule(Keys.consume)
  delete(key: Bytes) {
    ...
  }

  @StateInvariant
  modelAgrees() {
    ...
  }

  @Teardown
  close() {
    ...
  }
}

const result = Stateful.check(
  DatabaseMachine,
  with: Settings.standard.statefulSteps(50)
)
```

Initializers run before normal rules. Invariants run after initialization and after every normal action. Applicability and bundle availability remove unavailable rules before selection. Result references preserve dependency identity, consumed references cannot be reused, and teardown is attempted exactly once after execution begins.

## Extension API

Stable structural extension boundaries are:

- `ChoiceProvider` and `ChoiceProviderFactory`;
- `Strategy<out T>` and optional `StrategyBase<T>`;
- `ShrinkPass`;
- `ExampleDatabase`;
- `Reporter`.

`SystemRandomChoiceProvider` and `ScriptedChoiceProvider` share canonical request normalization with replay. Custom shrink passes propose candidates only; the authoritative `Shrinker` retains duplicate suppression, complexity ordering, replay, and failure-origin policy.

See `docs/extension-api.md` and `tests/conformance/`.

## Compatibility aliases

The following broad-v1 names remain direct aliases:

- `Check` → `WithSettings`;
- `CheckConfig` → `Settings`;
- `RuleBasedStateMachine` → `StateMachine`;
- stateful `Invariant` → `StateInvariant`;
- `PropertyReporter.console` → the authoritative console reporter factory.

There is no compatibility adapter in the release source.

## Examples

Executable documentation lives in `examples/`:

- `arithmetic.ph`;
- `codec.ph`;
- `collections.ph`;
- `derived_data.ph`;
- `parser_roundtrip.ph`;
- `recursive_expression.ph`;
- `stateful_database.ph`.

## Verification

With a Phalcom toolchain:

```sh
phalcom test --all
```

Source/static release verification:

```sh
PYTHONPYCACHEPREFIX=/tmp/phalcom-hypothesis-pycache \
python3 -m py_compile scripts/verify_phase*.py scripts/verify_release.py

python3 scripts/verify_release.py
sha256sum -c SHA256SUMS
```

`verify_release.py` runs the Phase 11 mutation suite and every Phase 01–12 verifier twice. The final archive is separately extracted and subjected to the same gate.

The checkpoint environment used to build this archive did not provide a `phalcom` executable. Consequently, `.ph` compilation, execution, real cross-process persistence, example execution, and benchmark timings were not observed there. Those limitations are recorded in `TEST-RESULTS.md` and `CHECKPOINT.md` rather than represented as simulated results.

## Documentation

- `docs/concepts.md` — search model and outcomes;
- `docs/strategies.md` — strategy contract and built-ins;
- `docs/shrinking.md` — semantic spans and passes;
- `docs/inference.md` — reflected strategy resolution;
- `docs/stateful.md` — rule-based stateful testing;
- `docs/database.md` — persistence format and recovery;
- `docs/extension-api.md` — supported extension boundaries;
- `docs/public-api.md` — exact root exports;
- `docs/migration-from-monolith.md` — broad-v1 migration.

Licensed under MIT.
