# Migration from the Original Monolith

The 0.1.0 release removes the historical `legacy/hypothesis-monolith.ph` source and both temporary compatibility modules. The package now has one implementation for each feature slice and one root façade.

## Import the package façade

Replace copied monolith declarations or private-module imports with root imports:

```phalcom
import { Given, Gen, PropertySuite, PropertyRunner, Settings } from "hypothesis"
```

Do not import `_internal/legacy_adapter` or `_internal/phase01_surface`; those modules no longer exist.

## Configuration metadata

The preferred release spelling is:

```phalcom
@WithSettings(Settings.standard.examples(500))
```

The prototype spellings remain source-compatible:

- `CheckConfig` is an alias of `Settings`.
- `Check` is an alias of `WithSettings`.

No adapter object is involved.

## Reporting

Prefer `ConsoleReporter.standard` and pass it to `PropertyRunner.run(..., reporter:)`.

`PropertyReporter.console` remains as a compatibility factory and delegates directly to the authoritative `ConsoleReporter` implementation.

## Stateful testing

Prefer:

- `StateMachine` instead of `RuleBasedStateMachine`;
- `StateInvariant` instead of the stateful `Invariant` alias.

`RuleBasedStateMachine` and `Invariant` remain direct aliases for broad-v1 source compatibility. Stateful `StateInvariant` remains semantically distinct from the language-level contract decorator `@invariant`.

## Choice and example terminology

The prototype's tape-like representation is now the immutable `Example` model. Primitive generation and replay flow through `DrawData`; extensions should implement `Strategy`, `ChoiceProvider`, `ShrinkPass`, `ExampleDatabase`, or `Reporter` rather than copying engine internals.

## Constructor and control-flow syntax

Active release source uses `@constructor`, unparenthesized `if`/`while`/`for` conditions, and compound updates. Historical code using the retired `construct` form must be migrated before use with the current language.
